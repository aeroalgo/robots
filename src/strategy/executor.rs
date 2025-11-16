use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;

use crate::candles::aggregator::TimeFrameAggregator;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::{Symbol, TimeFrame};
use crate::indicators::formula::{FormulaDefinition, FormulaEvaluationContext};
use crate::indicators::runtime::IndicatorRuntimeEngine;

use super::base::Strategy;
use super::builder::StrategyBuilder;
use super::context::{StrategyContext, TimeframeData};
use super::types::{
    IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, StrategyDecision,
    StrategyDefinition, StrategyError, StrategyParameterMap,
};
use crate::metrics::{BacktestAnalytics, BacktestReport};
use crate::position::{ExecutionReport, PositionBook, PositionError, PositionManager};

#[derive(Debug, Error)]
pub enum StrategyExecutionError {
    #[error("strategy evaluation error: {0}")]
    Strategy(#[from] StrategyError),
    #[error("position manager error: {0}")]
    Position(#[from] PositionError),
    #[error("feed error: {0}")]
    Feed(String),
}

pub struct BacktestExecutor {
    strategy: Box<dyn Strategy>,
    position_manager: PositionManager,
    feed: HistoricalFeed,
    context: StrategyContext,
    analytics: BacktestAnalytics,
    warmup_bars: usize,
    deferred_decision: Option<StrategyDecision>,
}

#[derive(Clone, Copy, Debug, Default)]
struct SessionState {
    is_session_start: bool,
    is_session_end: bool,
}

impl BacktestExecutor {
    pub fn new(
        strategy: Box<dyn Strategy>,
        mut frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, StrategyExecutionError> {
        if frames.is_empty() {
            return Err(StrategyExecutionError::Feed(
                "frames collection is empty".to_string(),
            ));
        }

        let mut required_timeframes: std::collections::HashSet<TimeFrame> = std::collections::HashSet::new();
        
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

        frames = Self::generate_missing_timeframes(frames, &required_timeframes)?;

        let feed = HistoricalFeed::new(frames);
        let context = feed.initialize_context();
        let position_manager = PositionManager::new(strategy.id().clone());
        Ok(Self {
            strategy,
            position_manager,
            feed,
            context,
            analytics: BacktestAnalytics::new(),
            warmup_bars: 0,
            deferred_decision: None,
        })
    }

    fn compute_warmup_bars(&self) -> usize {
        let max_period = self
            .strategy
            .indicator_bindings()
            .iter()
            .filter_map(|binding| match &binding.source {
                IndicatorSourceSpec::Registry { parameters, .. } => parameters
                    .get("period")
                    .map(|value| value.max(1.0).round() as usize),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        max_period.saturating_mul(2)
    }

    pub fn from_definition(
        definition: StrategyDefinition,
        parameter_overrides: Option<StrategyParameterMap>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, StrategyExecutionError> {
        let mut builder = StrategyBuilder::new(definition);
        if let Some(overrides) = parameter_overrides {
            builder = builder.with_parameters(overrides);
        }
        let strategy = builder.build().map_err(StrategyExecutionError::Strategy)?;
        let mut executor = Self::new(Box::new(strategy), frames)?;
        executor.warmup_bars = executor.compute_warmup_bars();
        Ok(executor)
    }

    fn generate_missing_timeframes(
        mut frames: HashMap<TimeFrame, QuoteFrame>,
        required: &[TimeFrame],
    ) -> Result<HashMap<TimeFrame, QuoteFrame>, StrategyExecutionError> {
        let mut generated = HashMap::new();

        for required_tf in required {
            if frames.contains_key(required_tf) {
                continue;
            }

            let source_frame = Self::find_best_source_frame(&frames, required_tf)
                .ok_or_else(|| {
                    StrategyExecutionError::Feed(format!(
                        "Cannot generate timeframe {:?}: no suitable source timeframe found",
                        required_tf
                    ))
                })?;

            let aggregated = TimeFrameAggregator::aggregate(&source_frame, required_tf.clone())
                .map_err(|e| {
                    StrategyExecutionError::Feed(format!(
                        "Failed to aggregate timeframe {:?}: {}",
                        required_tf, e
                    ))
                })?;

            generated.insert(required_tf.clone(), aggregated.frame);
        }

        frames.extend(generated);
        Ok(frames)
    }

    fn find_best_source_frame<'a>(
        frames: &'a HashMap<TimeFrame, QuoteFrame>,
        target: &TimeFrame,
    ) -> Option<&'a QuoteFrame> {
        let target_minutes = Self::timeframe_to_minutes(target)?;

        frames
            .iter()
            .filter_map(|(tf, frame)| {
                let source_minutes = Self::timeframe_to_minutes(tf)?;
                if source_minutes < target_minutes && target_minutes % source_minutes == 0 {
                    Some((source_minutes, frame))
                } else {
                    None
                }
            })
            .max_by_key(|(minutes, _)| *minutes)
            .map(|(_, frame)| frame)
    }

    fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        match tf {
            TimeFrame::Minutes(m) => Some(*m),
            TimeFrame::Hours(h) => Some(h * 60),
            TimeFrame::Days(d) => Some(d * 24 * 60),
            TimeFrame::Weeks(w) => Some(w * 7 * 24 * 60),
            TimeFrame::Months(m) => Some(m * 30 * 24 * 60),
            TimeFrame::Custom(_) => None,
        }
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

    pub async fn run_backtest(&mut self) -> Result<BacktestReport, StrategyExecutionError> {
        if self.warmup_bars == 0 {
            self.warmup_bars = self.compute_warmup_bars();
        }
        self.position_manager.reset();
        self.context.set_active_positions(PositionBook::default());
        self.feed.reset();
        self.analytics.reset();
        self.deferred_decision = None;
        for timeframe in self.feed.frames.keys() {
            if let Ok(data) = self.context.timeframe_mut(timeframe) {
                data.set_index(0);
            } else if let Some(frame) = self.feed.frames.get(timeframe) {
                let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
                self.context.insert_timeframe(timeframe.clone(), data);
            }
        }
        self.populate_indicators().await?;
        self.analytics
            .push_equity_point(self.position_manager.portfolio_snapshot().total_equity);
        let mut processed_bars = 0usize;
        while self.feed.step(&mut self.context) {
            processed_bars += 1;
            let session_state = self.session_state();
            self.update_session_metadata(session_state);
            if processed_bars < self.warmup_bars {
                self.analytics
                    .push_equity_point(self.position_manager.portfolio_snapshot().total_equity);
                continue;
            }
            if let Some(pending) = self.deferred_decision.take() {
                if !pending.is_empty() {
                    self.context
                        .metadata
                        .insert("deferred_entries".to_string(), "true".to_string());
                    let result = self
                        .position_manager
                        .process_decision(&mut self.context, &pending)
                        .await;
                    self.context.metadata.remove("deferred_entries");
                    let report = result.map_err(StrategyExecutionError::Position)?;
                    self.collect_report(&report);
                    self.process_immediate_stop_checks().await?;
                }
            }
            let decision = self
                .strategy
                .evaluate(&self.context)
                .await
                .map_err(StrategyExecutionError::Strategy)?;
            if session_state
                .map(|state| state.is_session_end)
                .unwrap_or(false)
            {
                if !decision.is_empty() {
                    self.deferred_decision = Some(decision);
                }
                self.analytics
                    .push_equity_point(self.position_manager.portfolio_snapshot().total_equity);
                continue;
            }
            if !decision.is_empty() {
                let report = self
                    .position_manager
                    .process_decision(&mut self.context, &decision)
                    .await
                    .map_err(StrategyExecutionError::Position)?;
                self.collect_report(&report);
                self.process_immediate_stop_checks().await?;
            }
            self.analytics
                .push_equity_point(self.position_manager.portfolio_snapshot().total_equity);
        }
        if let Some(pending) = self.deferred_decision.take() {
            if !pending.is_empty() {
                self.context
                    .metadata
                    .insert("deferred_entries".to_string(), "true".to_string());
                let result = self
                    .position_manager
                    .process_decision(&mut self.context, &pending)
                    .await;
                self.context.metadata.remove("deferred_entries");
                let report = result.map_err(StrategyExecutionError::Position)?;
                self.collect_report(&report);
                self.process_immediate_stop_checks().await?;
            }
        }
        Ok(self
            .analytics
            .build_report(self.position_manager.portfolio_snapshot().realized_pnl))
    }

    async fn populate_indicators(&mut self) -> Result<(), StrategyExecutionError> {
        let mut grouped: HashMap<TimeFrame, Vec<IndicatorBindingSpec>> = HashMap::new();
        for binding in self.strategy.indicator_bindings() {
            grouped
                .entry(binding.timeframe.clone())
                .or_default()
                .push(binding.clone());
        }
        let mut engine = IndicatorRuntimeEngine::new();
        for (timeframe, bindings) in grouped {
            let frame = {
                let frame_ref = self.feed.frames.get(&timeframe).ok_or_else(|| {
                    StrategyExecutionError::Feed(format!(
                        "timeframe {:?} not available in feed",
                        timeframe
                    ))
                })?;
                Arc::clone(frame_ref)
            };
            self.ensure_timeframe_data(&timeframe, &frame);
            let ohlc = frame.to_indicator_ohlc();
            let plan = IndicatorComputationPlan::build(&bindings)?;
            let mut computed: HashMap<String, Arc<Vec<f32>>> = HashMap::new();
            for binding in plan.ordered() {
                match &binding.source {
                    IndicatorSourceSpec::Registry { name, parameters } => {
                        let values = engine
                            .compute_registry(&timeframe, name, parameters, &ohlc)
                            .await
                            .map_err(|err| {
                                StrategyExecutionError::Feed(format!(
                                    "indicator {} calculation failed: {}",
                                    name, err
                                ))
                            })?;
                        self.store_indicator_series(&timeframe, &binding.alias, values.clone())?;
                        computed.insert(binding.alias.clone(), values);
                    }
                    IndicatorSourceSpec::Formula { .. } => {
                        let definition = plan.formula(&binding.alias).ok_or_else(|| {
                            StrategyExecutionError::Feed(format!(
                                "missing formula definition for alias {}",
                                binding.alias
                            ))
                        })?;
                        let context = FormulaEvaluationContext::new(&ohlc, &computed);
                        let values = engine
                            .compute_formula(&timeframe, definition, &context)
                            .map_err(|err| StrategyExecutionError::Feed(err.to_string()))?;
                        self.store_indicator_series(&timeframe, &binding.alias, values.clone())?;
                        computed.insert(binding.alias.clone(), values);
                    }
                }
            }
        }
        Ok(())
    }

    fn ensure_timeframe_data(&mut self, timeframe: &TimeFrame, frame: &Arc<QuoteFrame>) {
        if self.context.timeframe(timeframe).is_err() {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            self.context.insert_timeframe(timeframe.clone(), data);
        }
    }

    fn store_indicator_series(
        &mut self,
        timeframe: &TimeFrame,
        alias: &str,
        values: Arc<Vec<f32>>,
    ) -> Result<(), StrategyExecutionError> {
        let data = self
            .context
            .timeframe_mut(timeframe)
            .map_err(StrategyExecutionError::Strategy)?;
        data.insert_indicator_arc(alias.to_string(), values);
        Ok(())
    }

    fn session_state(&self) -> Option<SessionState> {
        let primary = &self.feed.primary_timeframe;
        let frame = self.feed.frames.get(primary)?;
        let timeframe_data = self.context.timeframe(primary).ok()?;
        let idx = timeframe_data.index();
        if frame.len() == 0 || idx >= frame.len() {
            return None;
        }
        let duration = primary.duration()?;
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
        match state {
            Some(state) => {
                self.context.metadata.insert(
                    "session_start".to_string(),
                    state.is_session_start.to_string(),
                );
                self.context
                    .metadata
                    .insert("session_end".to_string(), state.is_session_end.to_string());
            }
            None => {
                self.context.metadata.remove("session_start");
                self.context.metadata.remove("session_end");
            }
        }
    }

    fn collect_report(&mut self, report: &ExecutionReport) {
        self.analytics.absorb_execution_report(report);
    }

    async fn process_immediate_stop_checks(&mut self) -> Result<(), StrategyExecutionError> {
        loop {
            let stop_signals = self
                .strategy
                .evaluate_stop_signals(&self.context)
                .map_err(StrategyExecutionError::Strategy)?;
            if stop_signals.is_empty() {
                break;
            }
            let mut decision = StrategyDecision::empty();
            decision.stop_signals = stop_signals;
            let report = self
                .position_manager
                .process_decision(&mut self.context, &decision)
                .await
                .map_err(StrategyExecutionError::Position)?;
            self.collect_report(&report);
        }
        Ok(())
    }
}

struct HistoricalFeed {
    frames: HashMap<TimeFrame, Arc<QuoteFrame>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: TimeFrame,
}

impl HistoricalFeed {
    fn new(frames: HashMap<TimeFrame, QuoteFrame>) -> Self {
        let mut arc_frames = HashMap::new();
        let mut primary_timeframe = None;
        let mut max_len = 0usize;
        for (timeframe, frame) in frames {
            let len = frame.len();
            if len > max_len {
                max_len = len;
                primary_timeframe = Some(timeframe.clone());
            }
            arc_frames.insert(timeframe, Arc::new(frame));
        }
        let primary_timeframe =
            primary_timeframe.expect("HistoricalFeed requires at least one timeframe");
        Self {
            frames: arc_frames,
            indices: HashMap::new(),
            primary_timeframe,
        }
    }

    fn initialize_context(&self) -> StrategyContext {
        let mut map = HashMap::new();
        for (timeframe, frame) in &self.frames {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            map.insert(timeframe.clone(), data);
        }
        StrategyContext::with_timeframes(map)
    }

    fn reset(&mut self) {
        self.indices.clear();
    }

    fn step(&mut self, context: &mut StrategyContext) -> bool {
        let primary_frame = match self.frames.get(&self.primary_timeframe) {
            Some(frame) => frame,
            None => return false,
        };
        let primary_len = primary_frame.len();
        if primary_len == 0 {
            return false;
        }
        let current_primary_index = {
            let entry = self
                .indices
                .entry(self.primary_timeframe.clone())
                .or_insert(0);
            if *entry >= primary_len {
                return false;
            }
            *entry
        };
        for (timeframe, frame) in &self.frames {
            let len = frame.len();
            if len == 0 {
                continue;
            }
            let idx = if timeframe == &self.primary_timeframe {
                current_primary_index.min(len - 1)
            } else {
                let entry = self.indices.entry(timeframe.clone()).or_insert(0);
                *entry = (*entry).min(len - 1);
                *entry
            };
            match context.timeframe_mut(timeframe) {
                Ok(data) => {
                    data.set_index(idx);
                    if data.symbol().is_none() {
                        data.set_symbol(frame.symbol().clone());
                    }
                }
                Err(_) => {
                    let mut data = TimeframeData::with_quote_frame(frame.as_ref(), idx);
                    data.set_index(idx);
                    context.insert_timeframe(timeframe.clone(), data);
                }
            }
        }
        {
            let entry = self
                .indices
                .entry(self.primary_timeframe.clone())
                .or_insert(0);
            *entry = current_primary_index.saturating_add(1);
        }
        for (timeframe, frame) in &self.frames {
            if timeframe == &self.primary_timeframe {
                continue;
            }
            let len = frame.len();
            let entry = self.indices.entry(timeframe.clone()).or_insert(0);
            if *entry + 1 < len {
                *entry += 1;
            }
        }
        true
    }
}

struct IndicatorComputationPlan<'a> {
    ordered: Vec<&'a IndicatorBindingSpec>,
    formulas: HashMap<String, FormulaDefinition>,
}

impl<'a> IndicatorComputationPlan<'a> {
    fn build(bindings: &'a [IndicatorBindingSpec]) -> Result<Self, StrategyExecutionError> {
        let mut binding_map: HashMap<String, &'a IndicatorBindingSpec> = HashMap::new();
        for binding in bindings {
            if binding_map.insert(binding.alias.clone(), binding).is_some() {
                return Err(StrategyExecutionError::Feed(format!(
                    "duplicate indicator alias {}",
                    binding.alias
                )));
            }
        }
        let mut formulas = HashMap::new();
        for binding in bindings {
            if let IndicatorSourceSpec::Formula { expression } = &binding.source {
                let definition = FormulaDefinition::parse(expression)
                    .map_err(|err| StrategyExecutionError::Feed(err.to_string()))?;
                formulas.insert(binding.alias.clone(), definition);
            }
        }
        let mut indegree: HashMap<String, usize> =
            binding_map.keys().map(|alias| (alias.clone(), 0)).collect();
        let mut edges: HashMap<String, Vec<String>> = HashMap::new();
        for binding in bindings {
            if let Some(definition) = formulas.get(&binding.alias) {
                for dependency in definition.data_dependencies() {
                    if !binding_map.contains_key(dependency) {
                        return Err(StrategyExecutionError::Feed(format!(
                            "missing dependency {} for alias {}",
                            dependency, binding.alias
                        )));
                    }
                    edges
                        .entry(dependency.clone())
                        .or_default()
                        .push(binding.alias.clone());
                    if let Some(value) = indegree.get_mut(&binding.alias) {
                        *value += 1;
                    }
                }
            }
        }
        let mut queue = VecDeque::new();
        for (alias, degree) in indegree.iter() {
            if *degree == 0 {
                queue.push_back(alias.clone());
            }
        }
        let mut ordered = Vec::new();
        while let Some(alias) = queue.pop_front() {
            let binding = *binding_map
                .get(&alias)
                .expect("binding must exist for alias");
            ordered.push(binding);
            if let Some(children) = edges.get(&alias) {
                for child in children {
                    if let Some(entry) = indegree.get_mut(child) {
                        *entry -= 1;
                        if *entry == 0 {
                            queue.push_back(child.clone());
                        }
                    }
                }
            }
        }
        if ordered.len() != bindings.len() {
            return Err(StrategyExecutionError::Feed(
                "circular indicator dependency".to_string(),
            ));
        }
        Ok(Self { ordered, formulas })
    }

    fn ordered(&self) -> &[&'a IndicatorBindingSpec] {
        &self.ordered
    }

    fn formula(&self, alias: &str) -> Option<&FormulaDefinition> {
        self.formulas.get(alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::types::{SignalStrength, TrendDirection};
    use crate::data_model::quote::Quote;
    use crate::data_model::quote_frame::QuoteFrame;
    use crate::strategy::context::StrategyContext;
    use crate::strategy::types::{
        IndicatorBindingSpec, PreparedCondition, PriceField, StrategyDecision, StrategyId,
        StrategyMetadata, StrategyParameterMap, StrategyRuleSpec, StrategySignal,
        StrategySignalType, TimeframeRequirement,
    };
    use async_trait::async_trait;

    #[derive(Clone)]
    struct SimpleStrategy {
        id: StrategyId,
        metadata: StrategyMetadata,
        timeframe: TimeFrame,
        requirements: Vec<TimeframeRequirement>,
        parameters: StrategyParameterMap,
    }

    impl SimpleStrategy {
        fn new(timeframe: TimeFrame) -> Self {
            let id = "SIMPLE_ENTRY_EXIT".to_string();
            let metadata = StrategyMetadata::with_id(&id, "Simple Entry Exit");
            let requirements = vec![TimeframeRequirement {
                alias: "primary".to_string(),
                timeframe: timeframe.clone(),
            }];
            Self {
                id,
                metadata,
                timeframe,
                requirements,
                parameters: StrategyParameterMap::new(),
            }
        }
    }

    #[async_trait]
    impl Strategy for SimpleStrategy {
        fn id(&self) -> &StrategyId {
            &self.id
        }

        fn metadata(&self) -> &StrategyMetadata {
            &self.metadata
        }

        fn parameters(&self) -> &StrategyParameterMap {
            &self.parameters
        }

        fn indicator_bindings(&self) -> &[IndicatorBindingSpec] {
            &[]
        }

        fn conditions(&self) -> &[PreparedCondition] {
            &[]
        }

        fn entry_rules(&self) -> &[StrategyRuleSpec] {
            &[]
        }

        fn exit_rules(&self) -> &[StrategyRuleSpec] {
            &[]
        }

        fn timeframe_requirements(&self) -> &[TimeframeRequirement] {
            &self.requirements
        }

        async fn evaluate(
            &self,
            context: &StrategyContext,
        ) -> Result<StrategyDecision, StrategyError> {
            let data = context.timeframe(&self.timeframe)?;
            let idx = data.index();
            let series_len = data
                .price_series_slice(&PriceField::Close)
                .map(|slice| slice.len())
                .unwrap_or(0);
            dbg!(("evaluate", idx, series_len));
            let mut decision = StrategyDecision::empty();
            if idx == 0 {
                let signal = StrategySignal {
                    rule_id: "enter".to_string(),
                    signal_type: StrategySignalType::Entry,
                    direction: PositionDirection::Long,
                    timeframe: self.timeframe.clone(),
                    strength: SignalStrength::Strong,
                    trend: TrendDirection::Rising,
                    quantity: Some(1.0),
                    entry_rule_id: Some("enter".to_string()),
                    tags: Vec::new(),
                    position_group: Some("enter".to_string()),
                    target_entry_ids: Vec::new(),
                };
                decision.entries.push(signal);
            } else if series_len > 0 && idx + 1 == series_len {
                let signal = StrategySignal {
                    rule_id: "exit".to_string(),
                    signal_type: StrategySignalType::Exit,
                    direction: PositionDirection::Long,
                    timeframe: self.timeframe.clone(),
                    strength: SignalStrength::Strong,
                    trend: TrendDirection::Falling,
                    quantity: Some(1.0),
                    entry_rule_id: None,
                    tags: Vec::new(),
                    position_group: None,
                    target_entry_ids: vec!["enter".to_string()],
                };
                decision.exits.push(signal);
            }
            Ok(decision)
        }

        fn clone_box(&self) -> Box<dyn Strategy> {
            Box::new(self.clone())
        }
    }

    fn build_frame(prices: &[f32], timeframe: &TimeFrame) -> QuoteFrame {
        let symbol = Symbol::from_descriptor("TEST.TEST");
        let mut frame = QuoteFrame::new(symbol.clone(), timeframe.clone());
        for (idx, price) in prices.iter().enumerate() {
            let quote = Quote::from_parts(
                symbol.clone(),
                timeframe.clone(),
                Utc::now() + chrono::Duration::minutes(idx as i64),
                *price,
                *price,
                *price,
                *price,
                1.0,
            );
            frame.push(quote).unwrap();
        }
        frame
    }

    #[tokio::test]
    async fn backtest_executor_produces_trade_and_metrics() {
        let timeframe = TimeFrame::minutes(1);
        let frame = build_frame(&[100.0, 102.0, 105.0], &timeframe);
        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), frame);
        let strategy: Box<dyn Strategy> = Box::new(SimpleStrategy::new(timeframe.clone()));
        let mut executor = BacktestExecutor::new(strategy, frames).expect("executor creation");
        let report = executor.run_backtest().await.expect("backtest run");
        dbg!(&report.trades);
        assert_eq!(report.trades.len(), 1);
        let trade = &report.trades[0];
        assert!((trade.pnl - 5.0).abs() < 1e-6);
        assert_eq!(report.metrics.total_trades, 1);
        assert!((report.metrics.total_pnl - 5.0).abs() < 1e-6);
        assert!((report.metrics.win_rate - 1.0).abs() < 1e-6);
        assert_eq!(report.equity_curve.last().copied().unwrap_or_default(), 5.0);
    }
}
