use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;

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

        let mut feed = HistoricalFeed::new_empty();

        for (tf, frame) in frames {
            feed.frames.insert(tf.clone(), Arc::new(frame));
            if feed.primary_timeframe.is_none() {
                feed.primary_timeframe = Some(tf);
            } else {
                let current_len = feed
                    .frames
                    .get(&feed.primary_timeframe.clone().unwrap())
                    .map(|f| f.len())
                    .unwrap_or(0);
                let new_len = feed.frames.get(&tf).map(|f| f.len()).unwrap_or(0);
                if new_len > current_len {
                    feed.primary_timeframe = Some(tf);
                }
            }
        }

        if feed.primary_timeframe.is_none() {
            return Err(StrategyExecutionError::Feed(
                "No timeframes available after generation".to_string(),
            ));
        }

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
        // Находим самый длинный период и его таймфрейм
        let mut max_warmup_bars = 0usize;

        for binding in self.strategy.indicator_bindings() {
            if let IndicatorSourceSpec::Registry { parameters, .. } = &binding.source {
                if let Some(period) = parameters.get("period") {
                    let period_usize = period.max(1.0).round() as usize;
                    // Warmup = период * 2 на данном таймфрейме
                    let warmup_on_tf = period_usize * 2;

                    // Пересчитываем warmup в бары базового таймфрейма
                    if let Some(primary_tf) = self.feed.primary_timeframe.as_ref() {
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

        max_warmup_bars
    }

    fn convert_warmup_to_base_timeframe(
        indicator_tf: &TimeFrame,
        base_tf: &TimeFrame,
        warmup_bars: usize,
    ) -> usize {
        let indicator_minutes = Self::timeframe_to_minutes(indicator_tf).unwrap_or(1);
        let base_minutes = Self::timeframe_to_minutes(base_tf).unwrap_or(1);

        if indicator_minutes >= base_minutes {
            // Старший таймфрейм: умножаем на соотношение
            let ratio = indicator_minutes / base_minutes;
            warmup_bars * ratio as usize
        } else {
            // Младший таймфрейм: делим на соотношение
            let ratio = base_minutes / indicator_minutes;
            (warmup_bars + ratio as usize - 1) / ratio as usize // Округление вверх
        }
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

    pub fn run_backtest(&mut self) -> Result<BacktestReport, StrategyExecutionError> {
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
        self.populate_indicators()?;
        self.populate_conditions()?;
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

            // Отслеживаем бары в позициях (только после warmup)
            let has_open_positions = self.position_manager.open_position_count() > 0;
            self.analytics
                .increment_bars_in_positions_if_has_positions(has_open_positions);

            if let Some(pending) = self.deferred_decision.take() {
                if !pending.is_empty() {
                    self.context
                        .metadata
                        .insert("deferred_entries".to_string(), "true".to_string());
                    let result = self
                        .position_manager
                        .process_decision(&mut self.context, &pending);
                    self.context.metadata.remove("deferred_entries");
                    let report = result.map_err(StrategyExecutionError::Position)?;
                    self.collect_report(&report);
                    self.process_immediate_stop_checks()?;
                }
            }
            let decision = self
                .strategy
                .evaluate(&self.context)
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
                    .map_err(StrategyExecutionError::Position)?;
                self.collect_report(&report);
                self.process_immediate_stop_checks()?;
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
                    .process_decision(&mut self.context, &pending);
                self.context.metadata.remove("deferred_entries");
                let report = result.map_err(StrategyExecutionError::Position)?;
                self.collect_report(&report);
                self.process_immediate_stop_checks()?;
            }
        }
        let initial_capital = self
            .analytics
            .equity_curve()
            .first()
            .copied()
            .unwrap_or(10000.0);

        let start_date = self
            .feed
            .primary_timeframe
            .as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .and_then(|frame| frame.first())
            .map(|quote| quote.timestamp());

        let end_date = self
            .feed
            .primary_timeframe
            .as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .and_then(|frame| frame.latest())
            .map(|quote| quote.timestamp());

        let total_bars = self
            .feed
            .primary_timeframe
            .as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .map(|frame| frame.len())
            .unwrap_or(0);

        let bars_in_positions = self.analytics.bars_in_positions();

        Ok(self.analytics.build_report(
            initial_capital,
            start_date,
            end_date,
            total_bars,
            bars_in_positions,
            None,
        ))
    }

    fn populate_indicators(&mut self) -> Result<(), StrategyExecutionError> {
        let bindings_count = self.strategy.indicator_bindings().len();
        let mut grouped: HashMap<TimeFrame, Vec<IndicatorBindingSpec>> = HashMap::with_capacity(bindings_count / 2 + 1);
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
            let mut computed: HashMap<String, Arc<Vec<f32>>> = HashMap::with_capacity(bindings.len());

            for binding in plan.ordered() {
                match &binding.source {
                    IndicatorSourceSpec::Registry { name, parameters } => {
                        let values = engine
                            .compute_registry(&timeframe, name, parameters, &ohlc)
                            .map_err(|err| {
                                StrategyExecutionError::Feed(format!(
                                    "indicator {} calculation failed: {}",
                                    name, err
                                ))
                            })?;
                        self.store_indicator_series(&timeframe, &binding.alias, values.clone())?;
                        computed.insert(binding.alias.clone(), values.clone());
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
                        computed.insert(binding.alias.clone(), values.clone());
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

    fn populate_conditions(&mut self) -> Result<(), StrategyExecutionError> {
        let conditions_count = self.strategy.conditions().len();
        let mut grouped: HashMap<TimeFrame, Vec<String>> = HashMap::with_capacity(conditions_count / 2 + 1);
        for condition in self.strategy.conditions() {
            grouped
                .entry(condition.timeframe.clone())
                .or_default()
                .push(condition.id.clone());
        }

        for (timeframe, condition_ids) in grouped {
            let frame = {
                let frame_ref = self.feed.frames.get(&timeframe).ok_or_else(|| {
                    StrategyExecutionError::Feed(format!(
                        "timeframe {:?} not available in feed for conditions",
                        timeframe
                    ))
                })?;
                Arc::clone(frame_ref)
            };

            self.ensure_timeframe_data(&timeframe, &frame);

            for condition_id in condition_ids {
                let condition = self
                    .strategy
                    .conditions()
                    .iter()
                    .find(|c| c.id == condition_id)
                    .ok_or_else(|| {
                        StrategyExecutionError::Feed(format!(
                            "condition {} not found",
                            condition_id
                        ))
                    })?;

                let input = self
                    .context
                    .prepare_condition_input(condition)
                    .map_err(|err| StrategyExecutionError::Strategy(err))?;

                let result = condition.condition.check(input).map_err(|err| {
                    StrategyExecutionError::Strategy(StrategyError::ConditionFailure {
                        condition_id: condition.id.clone(),
                        source: err,
                    })
                })?;

                let data = self
                    .context
                    .timeframe_mut(&timeframe)
                    .map_err(StrategyExecutionError::Strategy)?;
                data.insert_condition_result(condition.id.clone(), result);
            }
        }
        Ok(())
    }

    fn session_state(&self) -> Option<SessionState> {
        let primary = self.feed.primary_timeframe.as_ref()?;
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

    fn process_immediate_stop_checks(&mut self) -> Result<(), StrategyExecutionError> {
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
                .map_err(StrategyExecutionError::Position)?;
            self.collect_report(&report);
        }
        Ok(())
    }
}

struct HistoricalFeed {
    frames: HashMap<TimeFrame, Arc<QuoteFrame>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: Option<TimeFrame>,
}

impl HistoricalFeed {
    fn new(frames: HashMap<TimeFrame, QuoteFrame>) -> Self {
        let frames_len = frames.len();
        let mut arc_frames = HashMap::with_capacity(frames_len);
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
        Self {
            frames: arc_frames,
            indices: HashMap::with_capacity(frames_len),
            primary_timeframe,
        }
    }

    fn new_empty() -> Self {
        Self {
            frames: HashMap::new(),
            indices: HashMap::new(),
            primary_timeframe: None,
        }
    }

    fn initialize_context(&self) -> StrategyContext {
        let mut map = HashMap::with_capacity(self.frames.len());
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
        let primary_timeframe = match &self.primary_timeframe {
            Some(tf) => tf,
            None => return false,
        };
        let primary_frame = match self.frames.get(primary_timeframe) {
            Some(frame) => frame,
            None => return false,
        };
        let primary_len = primary_frame.len();
        if primary_len == 0 {
            return false;
        }
        let current_primary_index = {
            let entry = self.indices.entry(primary_timeframe.clone()).or_insert(0);
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
            let idx = if timeframe == primary_timeframe {
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
            let entry = self.indices.entry(primary_timeframe.clone()).or_insert(0);
            *entry = current_primary_index.saturating_add(1);
        }
        for (timeframe, frame) in &self.frames {
            if timeframe == primary_timeframe {
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
        let mut binding_map: HashMap<String, &'a IndicatorBindingSpec> = HashMap::with_capacity(bindings.len());
        for binding in bindings {
            if binding_map.insert(binding.alias.clone(), binding).is_some() {
                return Err(StrategyExecutionError::Feed(format!(
                    "duplicate indicator alias {}",
                    binding.alias
                )));
            }
        }
        let formula_count = bindings.iter().filter(|b| matches!(b.source, IndicatorSourceSpec::Formula { .. })).count();
        let mut formulas = HashMap::with_capacity(formula_count);
        for binding in bindings {
            if let IndicatorSourceSpec::Formula { expression } = &binding.source {
                let definition = FormulaDefinition::parse(expression)
                    .map_err(|err| StrategyExecutionError::Feed(err.to_string()))?;
                formulas.insert(binding.alias.clone(), definition);
            }
        }
        let mut indegree: HashMap<String, usize> =
            binding_map.keys().map(|alias| (alias.clone(), 0)).collect();
        let mut edges: HashMap<String, Vec<String>> = HashMap::with_capacity(bindings.len());
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
        let mut queue = VecDeque::with_capacity(bindings.len());
        for (alias, degree) in indegree.iter() {
            if *degree == 0 {
                queue.push_back(alias.clone());
            }
        }
        let mut ordered = Vec::with_capacity(bindings.len());
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

        fn evaluate(&self, context: &StrategyContext) -> Result<StrategyDecision, StrategyError> {
            let data = context.timeframe(&self.timeframe)?;
            let idx = data.index();
            let series_len = data
                .price_series_slice(&PriceField::Close)
                .map(|slice| slice.len())
                .unwrap_or(0);
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
        let report = executor.run_backtest().expect("backtest run");
        assert_eq!(report.trades.len(), 1);
        let trade = &report.trades[0];
        assert!((trade.pnl - 5.0).abs() < 1e-6);
        assert_eq!(report.metrics.total_trades, 1);
        assert!((report.metrics.total_profit - 5.0).abs() < 1e-6);
        assert!((report.metrics.winning_percentage - 1.0).abs() < 1e-6);
        assert_eq!(report.equity_curve.last().copied().unwrap_or_default(), 5.0);
    }
}
