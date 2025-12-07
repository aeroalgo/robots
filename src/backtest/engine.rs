use std::collections::HashMap;
use std::sync::Arc;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::di::ServiceContainer;
use crate::metrics::{BacktestAnalytics, BacktestReport};
use crate::position::{PositionBook, PositionManager};
use crate::risk::RiskManager;
use crate::strategy::base::Strategy;
use crate::strategy::context::{StrategyContext, TimeframeData};
use crate::strategy::types::StrategyDecision;

use super::{
    BacktestConfig, BacktestError, ConditionEvaluator, FeedManager, IndicatorEngine,
};
use crate::strategy::executor::BacktestConfig as ExecutorBacktestConfig;

pub struct BacktestEngine {
    feed_manager: FeedManager,
    indicator_engine: IndicatorEngine,
    condition_evaluator: ConditionEvaluator,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    metrics_collector: BacktestAnalytics,
    strategy: Box<dyn Strategy>,
    context: StrategyContext,
    warmup_bars: usize,
    cached_session_duration: Option<chrono::Duration>,
    cached_equity: Option<f64>,
    last_equity_bar: usize,
    initial_capital: f64,
    config: BacktestConfig,
}

#[derive(Clone, Copy, Debug, Default)]
struct SessionState {
    is_session_start: bool,
    is_session_end: bool,
}

impl BacktestEngine {
    pub fn new(
        strategy: Box<dyn Strategy>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, BacktestError> {
        if frames.is_empty() {
            return Err(BacktestError::Feed("frames collection is empty".to_string()));
        }

        let mut required_timeframes: std::collections::HashSet<TimeFrame> =
            std::collections::HashSet::new();

        for binding in strategy.indicator_bindings() {
            required_timeframes.insert(binding.timeframe.clone());
        }

        for condition in strategy.conditions() {
            required_timeframes.insert(condition.timeframe.clone());
        }

        for requirement in strategy.timeframe_requirements() {
            required_timeframes.insert(requirement.timeframe.clone());
        }

        let required_timeframes: Vec<TimeFrame> = required_timeframes.into_iter().collect();

        let mut arc_frames: HashMap<TimeFrame, Arc<QuoteFrame>> =
            HashMap::with_capacity(frames.len());
        for (tf, frame) in frames {
            arc_frames.insert(tf, Arc::new(frame));
        }

        let mut feed_manager = FeedManager::with_frames(arc_frames);

        let mut timeframe_order: Vec<TimeFrame> = feed_manager.frames().keys().cloned().collect();
        timeframe_order.sort_by(|a, b| {
            let a_min = FeedManager::timeframe_to_minutes(a).unwrap_or(0);
            let b_min = FeedManager::timeframe_to_minutes(b).unwrap_or(0);
            a_min.cmp(&b_min)
        });

        if let Some(base_tf) = timeframe_order.first() {
            feed_manager.set_primary_timeframe(base_tf.clone());
        }

        let context = feed_manager.initialize_context_ordered(&timeframe_order);

        let config = BacktestConfig::default();
        let initial_capital = config.initial_capital;
        
        // Разрешаем зависимости через DI или создаем напрямую
        let position_manager = if let Some(container) = &container {
            container.resolve::<PositionManager>()
                .and_then(|pm_arc| Arc::try_unwrap(pm_arc).ok())
                .unwrap_or_else(|| PositionManager::new(initial_capital))
        } else {
            PositionManager::new(initial_capital)
        };
        
        let risk_manager = if let Some(container) = &container {
            container.resolve::<RiskManager>()
                .and_then(|rm_arc| Arc::try_unwrap(rm_arc).ok())
                .unwrap_or_else(|| Self::build_risk_manager(strategy.as_ref()))
        } else {
            Self::build_risk_manager(strategy.as_ref())
        };

        Ok(Self {
            feed_manager,
            indicator_engine: IndicatorEngine::new(),
            condition_evaluator: ConditionEvaluator::new(),
            position_manager,
            risk_manager,
            metrics_collector: BacktestAnalytics::new(),
            strategy,
            context,
            warmup_bars: 0,
            cached_session_duration: None,
            cached_equity: None,
            last_equity_bar: 0,
            initial_capital,
            config,
        })
    }

    pub fn with_config(mut self, config: BacktestConfig) -> Self {
        self.initial_capital = config.initial_capital;
        self.position_manager = PositionManager::new(config.initial_capital);
        self.config = config;
        self
    }

    pub fn config(&self) -> &BacktestConfig {
        &self.config
    }

    pub fn context(&self) -> &StrategyContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut StrategyContext {
        &mut self.context
    }

    pub fn position_manager(&self) -> &PositionManager {
        &self.position_manager
    }

    pub fn position_manager_mut(&mut self) -> &mut PositionManager {
        &mut self.position_manager
    }

    fn build_risk_manager(strategy: &dyn Strategy) -> RiskManager {
        let mut risk_manager = RiskManager::new();

        for entry in strategy.stop_handlers() {
            risk_manager.add_stop_handler(entry.handler_name.clone(), entry.clone());
        }

        for entry in strategy.take_handlers() {
            risk_manager.add_take_handler(entry.handler_name.clone(), entry.clone());
        }

        risk_manager
    }

    fn compute_warmup_bars(&self) -> usize {
        let mut max_period: usize = 0;

        for binding in self.strategy.indicator_bindings() {
            for (_, value) in &binding.parameters {
                if let Ok(period) = value.parse::<usize>() {
                    max_period = max_period.max(period);
                }
            }
        }

        for cond in self.strategy.conditions() {
            for param in &cond.parameters {
                if let Some(val) = param.value.as_ref() {
                    if let Ok(period) = val.parse::<usize>() {
                        max_period = max_period.max(period);
                    }
                }
            }
        }

        let warmup = (max_period as f64 * 1.5).ceil() as usize;
        warmup.max(50)
    }

    pub fn run(&mut self) -> Result<BacktestReport, BacktestError> {
        if self.warmup_bars == 0 {
            self.warmup_bars = self.compute_warmup_bars();
        }

        self.position_manager.reset();
        self.risk_manager.reset();
        self.context.set_active_positions(PositionBook::default());
        self.feed_manager.reset();
        self.metrics_collector.reset();
        self.cached_equity = None;
        self.last_equity_bar = 0;

        for timeframe in self.feed_manager.frames().keys() {
            if let Ok(data) = self.context.timeframe_mut(timeframe) {
                data.set_index(0);
            } else if let Some(frame) = self.feed_manager.get_frame(timeframe) {
                let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
                self.context.insert_timeframe(timeframe.clone(), data);
            }
        }

        self.indicator_engine.populate_indicators(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;

        self.indicator_engine.populate_auxiliary_indicators(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;

        self.indicator_engine.populate_custom_data(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;

        self.condition_evaluator.populate_conditions(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;

        self.metrics_collector.push_equity_point(self.initial_capital);

        let mut processed_bars = 0usize;
        let mut needs_session_check = true;

        while self.feed_manager.step(&mut self.context) {
            processed_bars += 1;

            if processed_bars < self.warmup_bars {
                self.metrics_collector.push_equity_point(self.initial_capital);
                continue;
            }

            if needs_session_check {
                let session_state = self.session_state();
                if session_state.is_some() {
                    self.update_session_metadata(session_state);
                    needs_session_check = false;
                }
            } else if processed_bars % 10 == 0 {
                needs_session_check = true;
            }

            let has_open_positions = self.position_manager.open_position_count() > 0;
            self.metrics_collector
                .increment_bars_in_positions_if_has_positions(has_open_positions);

            if self.position_manager.open_position_count() > 0 {
                self.update_trailing_stops();
            }

            let mut decision = self
                .strategy
                .evaluate(&self.context)
                .map_err(BacktestError::Strategy)?;

            if !decision.exits.is_empty() && !decision.entries.is_empty() {
                decision.entries.clear();
            }

            let equity_changed = !decision.is_empty();
            let had_new_entries = !decision.entries.is_empty();

            if equity_changed {
                let mut report = self
                    .position_manager
                    .process_decision(&mut self.context, &decision)
                    .map_err(BacktestError::Position)?;

                for trade in &mut report.closed_trades {
                    let history = self.risk_manager.take_stop_history(&trade.position_id);
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

                self.collect_report(&report);
                self.cached_equity = None;

                if had_new_entries && self.position_manager.open_position_count() > 0 {
                    self.update_trailing_stops();
                }
            }

            if self.position_manager.open_position_count() > 0 {
                self.process_immediate_stop_checks()?;
            }

            let equity = self.calculate_equity(has_open_positions, equity_changed, processed_bars);
            self.metrics_collector.push_equity_point(equity);
        }

        self.build_report()
    }

    fn calculate_equity(
        &mut self,
        has_open_positions: bool,
        equity_changed: bool,
        processed_bars: usize,
    ) -> f64 {
        let needs_snapshot = (has_open_positions
            && (equity_changed || self.cached_equity.is_none()))
            || (has_open_positions && processed_bars - self.last_equity_bar >= 10)
            || self.cached_equity.is_none();

        if !needs_snapshot {
            return self.cached_equity.unwrap();
        }

        let total_equity = self.position_manager.portfolio_snapshot().total_equity;
        let current_equity = self.initial_capital + total_equity;

        let should_update_cache = if has_open_positions
            && (equity_changed || self.cached_equity.is_none())
        {
            self.cached_equity
                .map_or(true, |cached| (cached - current_equity).abs() > 0.01)
        } else if has_open_positions && processed_bars - self.last_equity_bar >= 10 {
            (self.cached_equity.unwrap() - current_equity).abs() > 0.01
        } else {
            true
        };

        if should_update_cache {
            self.cached_equity = Some(current_equity);
            self.last_equity_bar = processed_bars;
        }

        current_equity
    }

    fn session_state(&self) -> Option<SessionState> {
        let primary = self.feed_manager.primary_timeframe()?;
        let frame = self.feed_manager.get_frame(primary)?;
        let timeframe_data = self.context.timeframe(primary).ok()?;
        let idx = timeframe_data.index();

        if frame.len() == 0 || idx >= frame.len() {
            return None;
        }

        let duration = self.cached_session_duration?;
        let current = frame.get(idx)?;
        let mut state = SessionState::default();

        if idx == 0 {
            state.is_session_start = true;
        } else if let Some(prev) = frame.get(idx.saturating_sub(1)) {
            let delta = current.timestamp() - prev.timestamp();
            if delta > duration {
                state.is_session_start = true;
            }
        }

        if idx + 1 >= frame.len() {
            state.is_session_end = true;
        } else if let Some(next) = frame.get(idx + 1) {
            let delta = next.timestamp() - current.timestamp();
            if delta > duration {
                state.is_session_end = true;
            }
        }

        Some(state)
    }

    fn update_session_metadata(&mut self, state: Option<SessionState>) {
        if let Some(s) = state {
            if s.is_session_start {
                self.context
                    .metadata
                    .insert("session_start".to_string(), "true".to_string());
            } else {
                self.context.metadata.remove("session_start");
            }
            if s.is_session_end {
                self.context
                    .metadata
                    .insert("session_end".to_string(), "true".to_string());
            } else {
                self.context.metadata.remove("session_end");
            }
        }
    }

    fn collect_report(&mut self, report: &crate::position::ExecutionReport) {
        self.metrics_collector.record_trades(&report.closed_trades);
    }

    fn update_trailing_stops(&mut self) {
        self.risk_manager
            .update_trailing_stops(&mut self.position_manager, &self.context);
    }

    fn process_immediate_stop_checks(&mut self) -> Result<(), BacktestError> {
        let stop_decisions = self
            .risk_manager
            .check_stops(&self.position_manager, &self.context);

        if !stop_decisions.is_empty() {
            let decision = StrategyDecision {
                entries: Vec::new(),
                exits: stop_decisions,
            };

            let mut report = self
                .position_manager
                .process_decision(&mut self.context, &decision)
                .map_err(BacktestError::Position)?;

            for trade in &mut report.closed_trades {
                let history = self.risk_manager.take_stop_history(&trade.position_id);
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

            self.collect_report(&report);
            self.cached_equity = None;
        }

        Ok(())
    }

    fn build_report(&mut self) -> Result<BacktestReport, BacktestError> {
        let initial_capital = self
            .metrics_collector
            .equity_curve()
            .first()
            .copied()
            .unwrap_or(10000.0);

        let start_date = self
            .feed_manager
            .primary_timeframe()
            .and_then(|tf| self.feed_manager.get_frame(tf))
            .and_then(|frame| frame.first())
            .map(|quote| quote.timestamp());

        let end_date = self
            .feed_manager
            .primary_timeframe()
            .and_then(|tf| self.feed_manager.get_frame(tf))
            .and_then(|frame| frame.latest())
            .map(|quote| quote.timestamp());

        let total_bars = self
            .feed_manager
            .primary_timeframe()
            .and_then(|tf| self.feed_manager.get_frame(tf))
            .map(|frame| frame.len())
            .unwrap_or(0);

        let bars_in_positions = self.metrics_collector.bars_in_positions();

        Ok(self.metrics_collector.build_report(
            initial_capital,
            start_date,
            end_date,
            total_bars,
            bars_in_positions,
            None,
        ))
    }
}
