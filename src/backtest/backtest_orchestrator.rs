use crate::metrics::BacktestReport;
use crate::position::{PositionBook, PositionManager};
use crate::risk::RiskManager;
use crate::strategy::base::Strategy;
use crate::strategy::context::StrategyContext;
use crate::strategy::types::StrategyDecision;

use crate::metrics::BacktestAnalytics;

use super::{
    constants, helpers, BacktestError, ConditionEvaluator, EquityCalculator, FeedManager,
    IndicatorEngine, SessionManager,
};

pub struct BacktestOrchestrator;

impl BacktestOrchestrator {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize_context(
        feed_manager: &mut FeedManager,
        timeframe_order: &[crate::data_model::types::TimeFrame],
    ) -> StrategyContext {
        feed_manager.reset();
        let mut context = feed_manager.initialize_context_ordered(timeframe_order);
        for timeframe in feed_manager.frames().keys() {
            if let Ok(data) = context.timeframe_mut(timeframe) {
                data.set_index(constants::INITIAL_INDEX);
            } else if let Some(frame) = feed_manager.get_frame(timeframe) {
                helpers::ensure_timeframe_in_context(&mut context, timeframe, frame);
            }
        }
        context
    }

    pub fn populate_indicators_and_conditions(
        indicator_engine: &mut IndicatorEngine,
        condition_evaluator: &ConditionEvaluator,
        strategy: &dyn Strategy,
        frames: &std::collections::HashMap<
            crate::data_model::types::TimeFrame,
            std::sync::Arc<crate::data_model::quote_frame::QuoteFrame>,
        >,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError> {
        indicator_engine.populate_indicators(strategy, frames, context)?;
        indicator_engine.populate_auxiliary_indicators(strategy, frames, context)?;
        indicator_engine.populate_custom_data(strategy, frames, context)?;
        condition_evaluator.populate_conditions(strategy, frames, context)?;
        Ok(())
    }

    pub fn process_bar(
        feed_manager: &mut FeedManager,
        context: &mut StrategyContext,
        processed_bars: &mut usize,
        warmup_bars: usize,
        initial_capital: f64,
        metrics_collector: &mut BacktestAnalytics,
    ) -> bool {
        if !feed_manager.step(context) {
            return false;
        }
        *processed_bars += 1;

        if *processed_bars < warmup_bars {
            metrics_collector.push_equity_point(initial_capital);
            return true;
        }
        true
    }

    pub fn check_session(
        session_manager: &SessionManager,
        feed_manager: &FeedManager,
        context: &mut StrategyContext,
        processed_bars: usize,
        needs_session_check: &mut bool,
    ) {
        if !*needs_session_check {
            if processed_bars % constants::SESSION_CHECK_INTERVAL == 0 {
                *needs_session_check = true;
            }
            return;
        }

        let primary_tf = match feed_manager.primary_timeframe() {
            Some(tf) => tf,
            None => return,
        };

        let frame = match feed_manager.get_frame(primary_tf) {
            Some(f) => f,
            None => return,
        };

        let session_state = session_manager.session_state(primary_tf, frame, context);
        if let Some(state) = session_state {
            session_manager.update_metadata(context, Some(state));
            *needs_session_check = false;
        }
    }

    pub fn process_decision(
        decision: StrategyDecision,
        position_manager: &mut PositionManager,
        risk_manager: &mut RiskManager,
        context: &mut StrategyContext,
        metrics_collector: &mut BacktestAnalytics,
        equity_calculator: &mut EquityCalculator,
        buffers: &mut crate::backtest::engine::BacktestBuffers,
    ) -> Result<bool, BacktestError> {
        if !decision.exits.is_empty() && !decision.entries.is_empty() {
            return Ok(false);
        }

        let equity_changed = !decision.is_empty();
        let had_new_entries = !decision.entries.is_empty();

        if !equity_changed {
            return Ok(false);
        }

        let mut validated_decision = decision;

        if !validated_decision.entries.is_empty() {
            buffers.filtered_entries.clear();
            buffers
                .filtered_entries
                .reserve(validated_decision.entries.len());
            for entry in &validated_decision.entries {
                let entry_price = context
                    .timeframe(&entry.timeframe)
                    .ok()
                    .and_then(|tf| {
                        tf.price_series_slice(&crate::strategy::types::PriceField::Close)
                            .and_then(|series| series.get(tf.index()).copied())
                            .map(|p| p as f64)
                    })
                    .unwrap_or(0.0);

                if entry_price <= 0.0 {
                    buffers.filtered_entries.push(entry.clone());
                    continue;
                }

                if risk_manager
                    .validate_before_entry(
                        context,
                        &entry.direction,
                        entry_price,
                        &entry.timeframe,
                        crate::strategy::types::PriceField::Close,
                    )
                    .is_some()
                {
                    continue;
                }

                buffers.filtered_entries.push(entry.clone());
            }
            std::mem::swap(
                &mut validated_decision.entries,
                &mut buffers.filtered_entries,
            );
        }

        let mut report = position_manager
            .process_decision(context, &validated_decision)
            .map_err(BacktestError::Position)?;

        for trade in &mut report.closed_trades {
            let history = risk_manager.take_stop_history(&trade.position_id);
            trade.stop_history = history
                .into_iter()
                .map(|h| crate::position::StopHistoryEntry {
                    bar_index: h.bar_index,
                    stop_level: h.stop_level,
                    max_high: h.max_high,
                    min_low: h.min_low,
                })
                .collect();
        }

        metrics_collector.absorb_execution_report(&report);
        equity_calculator.reset();

        let has_positions = position_manager.open_position_count() > 0;
        if had_new_entries && has_positions {
            risk_manager.sync_with_positions(context);
            risk_manager.on_new_bar(context);
        }

        Ok(true)
    }

    pub fn process_stop_checks(
        risk_manager: &mut RiskManager,
        position_manager: &mut PositionManager,
        context: &mut StrategyContext,
        metrics_collector: &mut BacktestAnalytics,
        equity_calculator: &mut EquityCalculator,
        buffers: &mut crate::backtest::engine::BacktestBuffers,
    ) -> Result<(), BacktestError> {
        loop {
            let stop_signals = risk_manager.check_stops(context);
            if stop_signals.is_empty() {
                break;
            }

            buffers.stop_decision.stop_signals.clear();
            buffers.stop_decision.stop_signals.extend(stop_signals);

            let mut report = position_manager
                .process_decision(context, &buffers.stop_decision)
                .map_err(BacktestError::Position)?;

            for trade in &mut report.closed_trades {
                let history = risk_manager.take_stop_history(&trade.position_id);
                trade.stop_history = history
                    .into_iter()
                    .map(|h| crate::position::StopHistoryEntry {
                        bar_index: h.bar_index,
                        stop_level: h.stop_level,
                        max_high: h.max_high,
                        min_low: h.min_low,
                    })
                    .collect();
            }

            metrics_collector.absorb_execution_report(&report);
            equity_calculator.reset();
        }

        Ok(())
    }
}

impl Default for BacktestOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
