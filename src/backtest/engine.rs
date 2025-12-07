use std::collections::HashMap;
use std::sync::Arc;

use crate::candles::aggregator::TimeFrameAggregator;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::di::ServiceContainer;
use crate::metrics::{BacktestAnalytics, BacktestReport};
use crate::position::{PositionBook, PositionManager};
use crate::risk::RiskManager;
use crate::strategy::base::Strategy;
use crate::strategy::builder::StrategyBuilder;
use crate::strategy::context::{StrategyContext, TimeframeData};
use crate::strategy::types::{
    StrategyDecision, StrategyDefinition, StrategyParameterMap, StrategySignalType,
};

use super::{
    constants, BacktestConfig, BacktestError, ConditionEvaluator, EquityCalculator, FeedManager,
    IndicatorEngine, SessionManager,
};

pub struct BacktestEngine {
    feed_manager: FeedManager,
    indicator_engine: IndicatorEngine,
    condition_evaluator: ConditionEvaluator,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    metrics_collector: BacktestAnalytics,
    session_manager: SessionManager,
    equity_calculator: EquityCalculator,
    strategy: Box<dyn Strategy>,
    context: StrategyContext,
    warmup_bars: usize,
    initial_capital: f64,
    config: BacktestConfig,
}

impl BacktestEngine {
    pub fn new(
        strategy: Box<dyn Strategy>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, BacktestError> {
        Self::new_with_provider(strategy, frames, None)
    }

    pub fn new_with_provider(
        strategy: Box<dyn Strategy>,
        frames: HashMap<TimeFrame, QuoteFrame>,
        container: Option<Arc<ServiceContainer>>,
    ) -> Result<Self, BacktestError> {
        if frames.is_empty() {
            return Err(BacktestError::Feed(
                "frames collection is empty".to_string(),
            ));
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
        let mut base_timeframes: Vec<TimeFrame> = Vec::new();

        for (tf, frame) in frames {
            let tf_clone = tf.clone();
            arc_frames.insert(tf_clone.clone(), Arc::new(frame));
            base_timeframes.push(tf_clone);
        }

        let mut aggregated_frames = HashMap::new();
        for base_tf in &base_timeframes {
            if let Some(base_frame) = arc_frames.get(base_tf) {
                let all_required: Vec<TimeFrame> = required_timeframes
                    .iter()
                    .filter(|tf| {
                        FeedManager::is_higher_timeframe(tf, base_tf)
                            && FeedManager::is_multiple_of(base_tf, tf)
                    })
                    .cloned()
                    .collect();

                for target_tf in all_required {
                    if arc_frames.contains_key(&target_tf) {
                        continue;
                    }
                    if aggregated_frames.contains_key(&target_tf) {
                        continue;
                    }

                    match TimeFrameAggregator::aggregate(base_frame.as_ref(), target_tf.clone()) {
                        Ok(aggregated) => {
                            aggregated_frames.insert(target_tf.clone(), aggregated.frame);
                        }
                        Err(e) => {
                            return Err(BacktestError::Feed(format!(
                                "Failed to aggregate timeframe {:?} from {:?}: {}",
                                target_tf, base_tf, e
                            )));
                        }
                    }
                }

                let base_minutes = FeedManager::timeframe_to_minutes(base_tf);
                if let Some(base_mins) = base_minutes {
                    let multipliers = match base_mins {
                        5 => vec![2, 3, 4],
                        15 => vec![2, 4],
                        30 => vec![2, 4],
                        60 => vec![2, 3, 4, 6, 8, 12],
                        120 => vec![2, 3],
                        180 => vec![2],
                        240 => vec![],
                        _ => vec![],
                    };

                    for mult in multipliers {
                        if let Some(target_tf) =
                            FeedManager::create_derived_timeframe(base_tf, mult)
                        {
                            if arc_frames.contains_key(&target_tf) {
                                continue;
                            }
                            if aggregated_frames.contains_key(&target_tf) {
                                continue;
                            }
                            if !required_timeframes.contains(&target_tf) {
                                continue;
                            }

                            match TimeFrameAggregator::aggregate(
                                base_frame.as_ref(),
                                target_tf.clone(),
                            ) {
                                Ok(aggregated) => {
                                    aggregated_frames.insert(target_tf.clone(), aggregated.frame);
                                }
                                Err(e) => {
                                    return Err(BacktestError::Feed(format!(
                                        "Failed to aggregate timeframe {:?} from {:?}: {}",
                                        target_tf, base_tf, e
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        for (tf, frame) in aggregated_frames {
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

        let position_manager = if let Some(container) = &container {
            container
                .resolve::<PositionManager>()
                .and_then(|pm_arc| Arc::try_unwrap(pm_arc).ok())
                .unwrap_or_else(|| {
                    PositionManager::new(strategy.id().to_string()).with_capital(
                        initial_capital,
                        config.use_full_capital,
                        config.reinvest_profits,
                    )
                })
        } else {
            PositionManager::new(strategy.id().to_string()).with_capital(
                initial_capital,
                config.use_full_capital,
                config.reinvest_profits,
            )
        };

        let risk_manager = if let Some(container) = &container {
            container
                .resolve::<RiskManager>()
                .and_then(|rm_arc| Arc::try_unwrap(rm_arc).ok())
                .unwrap_or_else(|| Self::build_risk_manager(strategy.as_ref()))
        } else {
            Self::build_risk_manager(strategy.as_ref())
        };

        let cached_session_duration = feed_manager.primary_timeframe().and_then(|tf| match tf {
            TimeFrame::Minutes(m) => Some(chrono::Duration::minutes(*m as i64)),
            TimeFrame::Hours(h) => Some(chrono::Duration::hours(*h as i64)),
            TimeFrame::Days(d) => Some(chrono::Duration::days(*d as i64)),
            TimeFrame::Weeks(w) => Some(chrono::Duration::weeks(*w as i64)),
            TimeFrame::Months(m) => Some(chrono::Duration::days(*m as i64 * 30)),
            TimeFrame::Custom(_) => None,
        });

        Ok(Self {
            feed_manager,
            indicator_engine: IndicatorEngine::new(),
            condition_evaluator: ConditionEvaluator::new(),
            position_manager,
            risk_manager,
            metrics_collector: BacktestAnalytics::new(),
            session_manager: SessionManager::new(cached_session_duration),
            equity_calculator: EquityCalculator::new(initial_capital),
            strategy,
            context,
            warmup_bars: 0,
            initial_capital,
            config,
        })
    }

    pub fn with_config(mut self, config: BacktestConfig) -> Self {
        self.initial_capital = config.initial_capital;
        self.position_manager = PositionManager::new(self.strategy.id().to_string()).with_capital(
            config.initial_capital,
            config.use_full_capital,
            config.reinvest_profits,
        );
        self.equity_calculator
            .set_initial_capital(config.initial_capital);
        self.config = config;
        self
    }

    pub fn config(&self) -> &BacktestConfig {
        &self.config
    }

    pub fn from_definition(
        definition: StrategyDefinition,
        parameter_overrides: Option<StrategyParameterMap>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, BacktestError> {
        let mut builder = StrategyBuilder::new(definition);
        if let Some(overrides) = parameter_overrides {
            builder = builder.with_parameters(overrides);
        }
        let strategy = builder.build().map_err(|e| BacktestError::Strategy(e))?;

        let mut engine = Self::new(Box::new(strategy), frames)?;
        engine.warmup_bars = engine.compute_warmup_bars();
        Ok(engine)
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

        use crate::risk::factory::StopHandlerFactory;
        use crate::risk::StopHandlerEntry;
        use std::sync::Arc;

        for spec in strategy.stop_handler_specs() {
            if let Ok(handler) = StopHandlerFactory::create(&spec.handler_name, &spec.parameters) {
                let entry = StopHandlerEntry {
                    handler: Arc::from(handler),
                    timeframe: spec.timeframe.clone(),
                    price_field: spec.price_field.clone(),
                    direction: spec.direction.clone(),
                    priority: spec.priority,
                };
                risk_manager.add_handler(entry);
            }
        }

        risk_manager
    }

    fn compute_warmup_bars(&self) -> usize {
        let mut max_warmup_bars = 0usize;

        for binding in self.strategy.indicator_bindings() {
            if let crate::strategy::types::IndicatorSourceSpec::Registry { parameters, .. } =
                &binding.source
            {
                if let Some(period) = parameters.get("period") {
                    let period_usize = period.max(1.0).round() as usize;
                    let warmup_on_tf = period_usize * 2;

                    if let Some(primary_tf) = self.feed_manager.primary_timeframe() {
                        let warmup_base = Self::convert_warmup_to_base_timeframe(
                            &binding.timeframe,
                            primary_tf,
                            warmup_on_tf,
                        );
                        max_warmup_bars = max_warmup_bars.max(warmup_base);
                    }
                }
            }
        }

        max_warmup_bars.max(constants::MIN_WARMUP_BARS)
    }

    fn convert_warmup_to_base_timeframe(
        indicator_tf: &TimeFrame,
        base_tf: &TimeFrame,
        warmup_bars: usize,
    ) -> usize {
        let indicator_minutes = FeedManager::timeframe_to_minutes(indicator_tf).unwrap_or(1);
        let base_minutes = FeedManager::timeframe_to_minutes(base_tf).unwrap_or(1);

        if indicator_minutes >= base_minutes {
            let ratio = indicator_minutes / base_minutes;
            warmup_bars * ratio as usize
        } else {
            let ratio = base_minutes / indicator_minutes;
            (warmup_bars + ratio as usize - 1) / ratio as usize
        }
    }

    pub fn run(&mut self) -> Result<BacktestReport, BacktestError> {
        self.run_backtest()
    }

    pub fn run_backtest(&mut self) -> Result<BacktestReport, BacktestError> {
        if self.warmup_bars == 0 {
            self.warmup_bars = self.compute_warmup_bars();
        }

        self.position_manager.reset();
        self.risk_manager.reset();
        self.context.set_active_positions(PositionBook::default());
        self.feed_manager.reset();
        self.metrics_collector.reset();
        self.equity_calculator.reset();

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

        self.metrics_collector
            .push_equity_point(self.initial_capital);

        let mut processed_bars = 0usize;
        let mut needs_session_check = true;

        while self.feed_manager.step(&mut self.context) {
            processed_bars += 1;

            if processed_bars < self.warmup_bars {
                self.metrics_collector
                    .push_equity_point(self.initial_capital);
                continue;
            }

            if needs_session_check {
                if let Some(primary_tf) = self.feed_manager.primary_timeframe() {
                    if let Some(frame) = self.feed_manager.get_frame(primary_tf) {
                        let session_state =
                            self.session_manager
                                .session_state(primary_tf, frame, &self.context);
                        if session_state.is_some() {
                            self.session_manager
                                .update_metadata(&mut self.context, session_state);
                            needs_session_check = false;
                        }
                    }
                }
            } else if processed_bars % constants::SESSION_CHECK_INTERVAL == 0 {
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
                let mut validated_decision = decision;

                if !validated_decision.entries.is_empty() {
                    let mut filtered_entries = Vec::new();
                    for entry in &validated_decision.entries {
                        let entry_price = self
                            .context
                            .timeframe(&entry.timeframe)
                            .ok()
                            .and_then(|tf| {
                                tf.price_series_slice(&crate::strategy::types::PriceField::Close)
                                    .and_then(|series| series.get(tf.index()).copied())
                                    .map(|p| p as f64)
                            })
                            .unwrap_or(0.0);

                        if entry_price > 0.0 {
                            if let Some(_reason) = self.risk_manager.validate_before_entry(
                                &self.context,
                                &entry.direction,
                                entry_price,
                                &entry.timeframe,
                                crate::strategy::types::PriceField::Close,
                            ) {
                                continue;
                            }
                        }
                        filtered_entries.push(entry.clone());
                    }
                    validated_decision.entries = filtered_entries;
                }

                let mut report = self
                    .position_manager
                    .process_decision(&mut self.context, &validated_decision)
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
                self.equity_calculator.reset();

                if had_new_entries && self.position_manager.open_position_count() > 0 {
                    self.update_trailing_stops();
                }
            }

            if self.position_manager.open_position_count() > 0 {
                self.process_immediate_stop_checks()?;
            }

            let equity = self.equity_calculator.calculate(
                &self.position_manager,
                has_open_positions,
                equity_changed,
                processed_bars,
            );
            self.metrics_collector.push_equity_point(equity);
        }

        self.build_report()
    }

    fn collect_report(&mut self, report: &crate::position::ExecutionReport) {
        self.metrics_collector.absorb_execution_report(report);
    }

    fn update_trailing_stops(&mut self) {
        self.risk_manager.sync_with_positions(&self.context);
        self.risk_manager.on_new_bar(&self.context);
    }

    fn process_immediate_stop_checks(&mut self) -> Result<(), BacktestError> {
        loop {
            let stop_signals = self.risk_manager.check_stops(&self.context);
            if stop_signals.is_empty() {
                break;
            }

            let mut decision = StrategyDecision::empty();
            decision.stop_signals = stop_signals;

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
            self.equity_calculator.reset();
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
