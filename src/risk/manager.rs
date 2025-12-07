use std::collections::HashMap;
use std::sync::Arc;

use crate::data_model::types::TimeFrame;
use crate::position::view::ActivePosition;
use crate::strategy::context::{StrategyContext, TimeframeData};
use crate::strategy::types::{
    PositionDirection, PriceField, StopSignal, StopSignalKind, StrategySignal, StrategySignalType,
};

use super::context::{StopEvaluationContext, StopValidationContext};
use super::state::{PositionRiskState, RiskStateBook};
use super::traits::{StopHandler, StopOutcome};
use super::utils::{calculate_stop_exit_price, get_price_at_index, is_stop_triggered};

pub struct StopHandlerEntry {
    pub handler: Arc<dyn StopHandler>,
    pub timeframe: TimeFrame,
    pub price_field: PriceField,
    pub direction: PositionDirection,
    pub priority: i32,
}

pub struct RiskManager {
    stop_handlers: Vec<StopHandlerEntry>,
    state_book: RiskStateBook,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            stop_handlers: Vec::new(),
            state_book: RiskStateBook::new(),
        }
    }

    pub fn with_handlers(handlers: Vec<StopHandlerEntry>) -> Self {
        Self {
            stop_handlers: handlers,
            state_book: RiskStateBook::new(),
        }
    }

    pub fn add_handler(&mut self, entry: StopHandlerEntry) {
        self.stop_handlers.push(entry);
    }

    pub fn reset(&mut self) {
        self.state_book.clear();
    }

    /// Валидация стоп-хендлеров перед открытием позиции
    /// Возвращает None если все валидации прошли успешно, иначе возвращает причину отказа
    pub fn validate_before_entry(
        &self,
        context: &StrategyContext,
        direction: &PositionDirection,
        entry_price: f64,
        timeframe: &TimeFrame,
        price_field: PriceField,
    ) -> Option<String> {
        let timeframe_data = match context.timeframe(timeframe) {
            Ok(data) => data,
            Err(_) => return Some(format!("Timeframe {:?} not found in context", timeframe)),
        };

        let index = timeframe_data.index();
        let current_price = entry_price;

        let validation_ctx = StopValidationContext {
            direction: direction.clone(),
            entry_price,
            timeframe_data,
            price_field,
            index,
            current_price,
        };

        for handler_entry in &self.stop_handlers {
            // Проверяем только стоп-хендлеры для соответствующего направления и таймфрейма
            if handler_entry.timeframe != *timeframe {
                continue;
            }

            if handler_entry.direction != *direction
                && handler_entry.direction != PositionDirection::Both
            {
                continue;
            }

            if let Some(validation_result) =
                handler_entry.handler.validate_before_entry(&validation_ctx)
            {
                if !validation_result.is_valid {
                    return validation_result.reason;
                }
            }
        }

        None
    }

    pub fn on_position_opened(&mut self, position: &ActivePosition, context: &StrategyContext) {
        if self.state_book.contains(&position.id) {
            return;
        }

        let (high, low) = self.get_current_high_low(context, &position.timeframe);

        let mut state = PositionRiskState::new(
            position.id.clone(),
            position.direction.clone(),
            position.entry_price,
            high.unwrap_or(position.entry_price),
            low.unwrap_or(position.entry_price),
        );

        if let Some(stop_level) = self.compute_initial_stop(&state, position, context) {
            state.update_stop(stop_level);
        }

        self.state_book.insert(state);
    }

    pub fn on_position_closed(&mut self, position_id: &str) {
        self.state_book.remove(position_id);
    }

    pub fn take_stop_history(&mut self, position_id: &str) -> Vec<super::state::StopHistoryRecord> {
        self.state_book
            .get_mut(position_id)
            .map(|state| state.take_stop_history())
            .unwrap_or_default()
    }

    pub fn on_new_bar(&mut self, context: &StrategyContext) {
        self.update_price_extremes(context);
        self.update_trailing_stops(context);
    }

    pub fn sync_with_positions(&mut self, context: &StrategyContext) {
        let active_ids: Vec<String> = context
            .active_positions()
            .values()
            .map(|p| p.id.clone())
            .collect();

        let state_ids = self.state_book.position_ids();
        for state_id in state_ids {
            if !active_ids.contains(&state_id) {
                self.state_book.remove(&state_id);
            }
        }

        for position in context.active_positions().values() {
            if !self.state_book.contains(&position.id) {
                self.on_position_opened(position, context);
            }
        }
    }

    fn update_price_extremes(&mut self, context: &StrategyContext) {
        let updates: Vec<(String, f64, f64)> = context
            .active_positions()
            .values()
            .filter_map(|position| {
                if !self.state_book.contains(&position.id) {
                    return None;
                }
                let (high, low) = self.get_current_high_low(context, &position.timeframe);
                match (high, low) {
                    (Some(h), Some(l)) => Some((position.id.clone(), h, l)),
                    _ => None,
                }
            })
            .collect();

        for (position_id, high, low) in updates {
            if let Some(state) = self.state_book.get_mut(&position_id) {
                state.update_price_extremes(high, low);
            }
        }
    }

    fn update_trailing_stops(&mut self, context: &StrategyContext) {
        let updates: Vec<(String, f64, usize)> = context
            .active_positions()
            .values()
            .filter_map(|position| {
                let state = self.state_book.get(&position.id)?;
                let new_stop = self.compute_stop_level(state, position, context)?;
                let bar_index = context
                    .timeframe(&position.timeframe)
                    .ok()
                    .map(|tf| tf.index())
                    .unwrap_or(0);
                Some((position.id.clone(), new_stop, bar_index))
            })
            .collect();

        for (position_id, new_stop, bar_index) in updates {
            if let Some(state) = self.state_book.get_mut(&position_id) {
                state.update_stop(new_stop);
                state.record_stop_history(bar_index);
            }
        }
    }

    fn compute_initial_stop(
        &self,
        state: &PositionRiskState,
        position: &ActivePosition,
        context: &StrategyContext,
    ) -> Option<f64> {
        self.compute_stop_level(state, position, context)
    }

    fn compute_stop_level(
        &self,
        state: &PositionRiskState,
        position: &ActivePosition,
        context: &StrategyContext,
    ) -> Option<f64> {
        let mut best_stop: Option<f64> = state.current_stop;

        for handler_entry in &self.stop_handlers {
            if !self.handler_matches_position(handler_entry, position) {
                continue;
            }

            let tf_data = match context.timeframe(&handler_entry.timeframe) {
                Ok(data) => data,
                Err(_) => continue,
            };

            let eval_ctx =
                self.build_eval_context(state, position, tf_data, &handler_entry.price_field);

            if let Some(new_level) = handler_entry.handler.compute_stop_level(&eval_ctx) {
                best_stop = Some(match position.direction {
                    PositionDirection::Long => best_stop
                        .map(|prev| prev.max(new_level))
                        .unwrap_or(new_level),
                    PositionDirection::Short => best_stop
                        .map(|prev| prev.min(new_level))
                        .unwrap_or(new_level),
                    _ => new_level,
                });
            }
        }

        best_stop
    }

    pub fn check_stops(&self, context: &StrategyContext) -> Vec<StopSignal> {
        let mut signals = Vec::new();

        for position in context.active_positions().values() {
            let state = match self.state_book.get(&position.id) {
                Some(s) => s,
                None => continue,
            };

            for handler_entry in &self.stop_handlers {
                if !self.handler_matches_position(handler_entry, position) {
                    continue;
                }

                let tf_data = match context.timeframe(&handler_entry.timeframe) {
                    Ok(data) => data,
                    Err(_) => continue,
                };

                let eval_ctx =
                    self.build_eval_context(state, position, tf_data, &handler_entry.price_field);

                if let Some(outcome) = self.evaluate_stop(&eval_ctx, handler_entry, state) {
                    let signal = self.build_stop_signal(position, handler_entry, outcome);
                    signals.push(signal);
                    break;
                }
            }
        }

        signals
    }

    fn evaluate_stop(
        &self,
        ctx: &StopEvaluationContext<'_>,
        handler_entry: &StopHandlerEntry,
        state: &PositionRiskState,
    ) -> Option<StopOutcome> {
        let effective_stop = state.current_stop?;

        let low_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::Low,
            ctx.index,
            ctx.current_price,
        );
        let high_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::High,
            ctx.index,
            ctx.current_price,
        );

        if !is_stop_triggered(
            &ctx.position.direction,
            low_price,
            high_price,
            effective_stop,
        ) {
            return None;
        }

        let open_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::Open,
            ctx.index,
            ctx.current_price,
        );

        let exit_price = calculate_stop_exit_price(
            &ctx.position.direction,
            effective_stop,
            open_price,
            ctx.current_price,
        );

        let mut metadata = HashMap::new();
        metadata.insert("level".to_string(), effective_stop.to_string());
        metadata.insert("triggered_price".to_string(), exit_price.to_string());
        metadata.insert(
            "handler".to_string(),
            handler_entry.handler.name().to_string(),
        );

        Some(StopOutcome {
            exit_price,
            kind: StopSignalKind::StopLoss,
            metadata,
        })
    }

    fn build_eval_context<'a>(
        &self,
        state: &PositionRiskState,
        position: &'a ActivePosition,
        tf_data: &'a TimeframeData,
        price_field: &PriceField,
    ) -> StopEvaluationContext<'a> {
        let series = tf_data.price_series_slice(price_field);
        let index = tf_data
            .index()
            .min(series.map(|s| s.len().saturating_sub(1)).unwrap_or(0));
        let current_price = series.and_then(|s| s.get(index).copied()).unwrap_or(0.0) as f64;

        StopEvaluationContext {
            position,
            timeframe_data: tf_data,
            price_field: price_field.clone(),
            index,
            current_price,
            max_high_since_entry: state.max_high_since_entry,
            min_low_since_entry: state.min_low_since_entry,
            current_stop: state.current_stop,
        }
    }

    fn build_stop_signal(
        &self,
        position: &ActivePosition,
        handler_entry: &StopHandlerEntry,
        outcome: StopOutcome,
    ) -> StopSignal {
        let signal = StrategySignal {
            rule_id: format!("stop_{}_{}", handler_entry.handler.name(), position.id),
            signal_type: StrategySignalType::Exit,
            direction: position.direction.clone(),
            timeframe: position.timeframe.clone(),
            strength: crate::condition::types::SignalStrength::Strong,
            quantity: Some(position.quantity),
            entry_rule_id: position.entry_rule_id.clone(),
            tags: vec!["stop".to_string()],
            position_group: position.position_group.clone(),
            target_entry_ids: position
                .entry_rule_id
                .as_ref()
                .map(|id| vec![id.clone()])
                .unwrap_or_default(),
        };

        StopSignal {
            handler_id: handler_entry.handler.name().to_string(),
            signal,
            exit_price: outcome.exit_price,
            kind: outcome.kind,
            priority: handler_entry.priority,
            metadata: outcome.metadata,
        }
    }

    fn handler_matches_position(
        &self,
        handler: &StopHandlerEntry,
        position: &ActivePosition,
    ) -> bool {
        if position.timeframe != handler.timeframe {
            return false;
        }

        match &handler.direction {
            PositionDirection::Both => true,
            dir => dir == &position.direction,
        }
    }

    fn get_current_high_low(
        &self,
        context: &StrategyContext,
        timeframe: &TimeFrame,
    ) -> (Option<f64>, Option<f64>) {
        let tf_data = match context.timeframe(timeframe) {
            Ok(data) => data,
            Err(_) => return (None, None),
        };

        let index = tf_data.index();

        let high = tf_data
            .price_series_slice(&PriceField::High)
            .and_then(|s| s.get(index).copied())
            .map(|v| v as f64);

        let low = tf_data
            .price_series_slice(&PriceField::Low)
            .and_then(|s| s.get(index).copied())
            .map(|v| v as f64);

        (high, low)
    }

    pub fn get_state(&self, position_id: &str) -> Option<&PositionRiskState> {
        self.state_book.get(position_id)
    }

    pub fn get_current_stop(&self, position_id: &str) -> Option<f64> {
        self.state_book
            .get(position_id)
            .and_then(|s| s.current_stop)
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        Self::new()
    }
}
