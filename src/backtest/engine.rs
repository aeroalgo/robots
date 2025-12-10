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
    StrategyDecision, StrategyDefinition, StrategyParameterMap, StrategySignal, StrategySignalType,
};

use super::{
    constants, BacktestConfig, BacktestError, BacktestOrchestrator, ConditionEvaluator,
    EquityCalculator, FeedManager, IndicatorEngine, SessionManager, TimeFrameAggregationService,
};

pub(crate) struct BacktestBuffers {
    pub(crate) filtered_entries: Vec<StrategySignal>,
    pub(crate) stop_decision: StrategyDecision,
}

impl BacktestBuffers {
    fn new() -> Self {
        Self {
            filtered_entries: Vec::new(),
            stop_decision: StrategyDecision::empty(),
        }
    }

    fn reset(&mut self) {
        self.filtered_entries.clear();
        self.stop_decision.entries.clear();
        self.stop_decision.exits.clear();
        self.stop_decision.stop_signals.clear();
        self.stop_decision.custom.clear();
        self.stop_decision.metadata.clear();
    }
}

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
    buffers: BacktestBuffers,
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

        let (feed_manager, context) = Self::prepare_feed_and_context(strategy.as_ref(), frames)?;
        let config = BacktestConfig::default();
        let position_manager =
            Self::create_position_manager(strategy.as_ref(), &config, container.as_ref());
        let risk_manager = Self::create_risk_manager(strategy.as_ref(), container.as_ref());
        let session_duration = Self::calculate_session_duration(&feed_manager);

        Ok(Self {
            feed_manager,
            indicator_engine: IndicatorEngine::new(),
            condition_evaluator: ConditionEvaluator::new(),
            position_manager,
            risk_manager,
            metrics_collector: BacktestAnalytics::new(),
            session_manager: SessionManager::new(session_duration),
            equity_calculator: EquityCalculator::new(config.initial_capital),
            strategy,
            context,
            warmup_bars: 0,
            initial_capital: config.initial_capital,
            config,
            buffers: BacktestBuffers::new(),
        })
    }

    fn prepare_feed_and_context(
        strategy: &dyn Strategy,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<(FeedManager, StrategyContext), BacktestError> {
        let aggregation_service = TimeFrameAggregationService::new();
        let required_timeframes =
            TimeFrameAggregationService::collect_required_timeframes(strategy);

        let mut arc_frames: HashMap<TimeFrame, Arc<QuoteFrame>> =
            HashMap::with_capacity(frames.len());
        let mut base_timeframes: Vec<TimeFrame> = Vec::new();

        for (tf, frame) in frames {
            arc_frames.insert(tf.clone(), Arc::new(frame));
            base_timeframes.push(tf);
        }

        let aggregated_frames = aggregation_service.aggregate_required_timeframes(
            &arc_frames,
            &base_timeframes,
            &required_timeframes,
        )?;

        for (tf, frame) in aggregated_frames {
            arc_frames.insert(tf, Arc::new(frame));
        }

        let mut feed_manager = FeedManager::with_frames(arc_frames);
        let timeframe_order = Self::sort_timeframes(&feed_manager);

        if let Some(base_tf) = timeframe_order.first() {
            feed_manager.set_primary_timeframe(base_tf.clone());
        }

        let context = feed_manager.initialize_context_ordered(&timeframe_order);
        Ok((feed_manager, context))
    }

    fn sort_timeframes(feed_manager: &FeedManager) -> Vec<TimeFrame> {
        let mut timeframe_order: Vec<TimeFrame> = feed_manager.frames().keys().cloned().collect();
        let minutes_cache: HashMap<TimeFrame, u32> = timeframe_order
            .iter()
            .map(|tf| {
                let minutes = FeedManager::timeframe_to_minutes(tf)
                    .unwrap_or(constants::INITIAL_INDEX as u32);
                (tf.clone(), minutes)
            })
            .collect();

        timeframe_order.sort_by(|a, b| {
            let a_min = minutes_cache
                .get(a)
                .copied()
                .unwrap_or(constants::INITIAL_INDEX as u32);
            let b_min = minutes_cache
                .get(b)
                .copied()
                .unwrap_or(constants::INITIAL_INDEX as u32);
            a_min.cmp(&b_min)
        });
        timeframe_order
    }

    fn create_position_manager(
        strategy: &dyn Strategy,
        config: &BacktestConfig,
        container: Option<&Arc<ServiceContainer>>,
    ) -> PositionManager {
        if let Some(container) = container {
            container
                .resolve::<PositionManager>()
                .and_then(|pm_arc| Arc::try_unwrap(pm_arc).ok())
                .unwrap_or_else(|| {
                    PositionManager::new(strategy.id().to_string()).with_capital(
                        config.initial_capital,
                        config.use_full_capital,
                        config.reinvest_profits,
                    )
                })
        } else {
            PositionManager::new(strategy.id().to_string()).with_capital(
                config.initial_capital,
                config.use_full_capital,
                config.reinvest_profits,
            )
        }
    }

    fn create_risk_manager(
        strategy: &dyn Strategy,
        container: Option<&Arc<ServiceContainer>>,
    ) -> RiskManager {
        if let Some(container) = container {
            container
                .resolve::<RiskManager>()
                .and_then(|rm_arc| Arc::try_unwrap(rm_arc).ok())
                .unwrap_or_else(|| Self::build_risk_manager(strategy))
        } else {
            Self::build_risk_manager(strategy)
        }
    }

    fn calculate_session_duration(feed_manager: &FeedManager) -> Option<chrono::Duration> {
        feed_manager.primary_timeframe().and_then(|tf| match tf {
            TimeFrame::Minutes(m) => Some(chrono::Duration::minutes(*m as i64)),
            TimeFrame::Hours(h) => Some(chrono::Duration::hours(*h as i64)),
            TimeFrame::Days(d) => Some(chrono::Duration::days(*d as i64)),
            TimeFrame::Weeks(w) => Some(chrono::Duration::weeks(*w as i64)),
            TimeFrame::Months(m) => Some(chrono::Duration::days(
                *m as i64 * constants::DAYS_PER_MONTH as i64,
            )),
            TimeFrame::Custom(_) => None,
        })
    }

    pub fn with_config(mut self, config: BacktestConfig) -> Self {
        self.initial_capital = config.initial_capital;
        self.position_manager = PositionManager::new(self.strategy.id().to_string()).with_capital(
            config.initial_capital,
            config.use_full_capital,
            config.reinvest_profits,
        );
        self.equity_calculator = EquityCalculator::new(config.initial_capital);
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
        let primary_tf = match self.feed_manager.primary_timeframe() {
            Some(tf) => tf,
            None => return constants::MIN_WARMUP_BARS,
        };

        for binding in self.strategy.indicator_bindings() {
            let crate::strategy::types::IndicatorSourceSpec::Registry { parameters, .. } =
                &binding.source
            else {
                continue;
            };

            let period = match parameters.get("period") {
                Some(p) => p,
                None => continue,
            };

            let period_usize = period.max(1.0).round() as usize;
            let warmup_on_tf = period_usize * constants::WARMUP_PERIOD_MULTIPLIER;

            let warmup_base = Self::convert_warmup_to_base_timeframe(
                &binding.timeframe,
                primary_tf,
                warmup_on_tf,
            );
            max_warmup_bars = max_warmup_bars.max(warmup_base);
        }

        max_warmup_bars.max(constants::MIN_WARMUP_BARS)
    }

    fn convert_warmup_to_base_timeframe(
        indicator_tf: &TimeFrame,
        base_tf: &TimeFrame,
        warmup_bars: usize,
    ) -> usize {
        let indicator_minutes = FeedManager::timeframe_to_minutes(indicator_tf)
            .unwrap_or(constants::MIN_TIME_FRAME_MINUTES as u32);
        let base_minutes = FeedManager::timeframe_to_minutes(base_tf)
            .unwrap_or(constants::MIN_TIME_FRAME_MINUTES as u32);

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

    fn run_backtest(&mut self) -> Result<BacktestReport, BacktestError> {
        if self.warmup_bars == 0 {
            self.warmup_bars = self.compute_warmup_bars();
        }

        self.position_manager.reset();
        self.risk_manager.reset();
        self.context.set_active_positions(PositionBook::default());
        self.metrics_collector.reset();
        self.equity_calculator.reset();
        self.buffers.reset();

        let mut timeframe_order: Vec<TimeFrame> =
            self.feed_manager.frames().keys().cloned().collect();
        timeframe_order.sort_by(|a, b| {
            let a_min =
                FeedManager::timeframe_to_minutes(a).unwrap_or(constants::INITIAL_INDEX as u32);
            let b_min =
                FeedManager::timeframe_to_minutes(b).unwrap_or(constants::INITIAL_INDEX as u32);
            a_min.cmp(&b_min)
        });

        self.context =
            BacktestOrchestrator::initialize_context(&mut self.feed_manager, &timeframe_order);

        BacktestOrchestrator::populate_indicators_and_conditions(
            &mut self.indicator_engine,
            &self.condition_evaluator,
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;

        self.metrics_collector
            .push_equity_point(self.initial_capital);

        let mut processed_bars = 0usize;
        let mut needs_session_check = true;

        while BacktestOrchestrator::process_bar(
            &mut self.feed_manager,
            &mut self.context,
            &mut processed_bars,
            self.warmup_bars,
            self.initial_capital,
            &mut self.metrics_collector,
        ) {
            BacktestOrchestrator::check_session(
                &self.session_manager,
                &self.feed_manager,
                &mut self.context,
                processed_bars,
                &mut needs_session_check,
            );

            let has_open_positions = self.position_manager.open_position_count() > 0;
            self.metrics_collector
                .increment_bars_in_positions_if_has_positions(has_open_positions);

            if has_open_positions {
                self.update_trailing_stops();
            }

            let mut decision = self
                .strategy
                .evaluate(&self.context)
                .map_err(BacktestError::Strategy)?;

            let equity_changed = BacktestOrchestrator::process_decision(
                decision,
                &mut self.position_manager,
                &mut self.risk_manager,
                &mut self.context,
                &mut self.metrics_collector,
                &mut self.equity_calculator,
                &mut self.buffers,
            )?;

            if has_open_positions {
                BacktestOrchestrator::process_stop_checks(
                    &mut self.risk_manager,
                    &mut self.position_manager,
                    &mut self.context,
                    &mut self.metrics_collector,
                    &mut self.equity_calculator,
                    &mut self.buffers,
                )?;
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

    fn update_trailing_stops(&mut self) {
        self.risk_manager.sync_with_positions(&self.context);
        self.risk_manager.on_new_bar(&self.context);
    }

    fn build_report(&mut self) -> Result<BacktestReport, BacktestError> {
        let initial_capital = self
            .metrics_collector
            .equity_curve()
            .first()
            .copied()
            .unwrap_or(0.0);

        let primary_tf = self.feed_manager.primary_timeframe();
        let primary_frame = primary_tf.and_then(|tf| self.feed_manager.get_frame(tf));

        let start_date = primary_frame
            .and_then(|frame| frame.first())
            .map(|quote| quote.timestamp());

        let end_date = primary_frame
            .and_then(|frame| frame.latest())
            .map(|quote| quote.timestamp());

        let total_bars = primary_frame.map(|frame| frame.len()).unwrap_or(0);

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
