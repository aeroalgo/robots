use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;

use crate::candles::aggregator::{AggregatedQuoteFrame, TimeFrameAggregator};
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

        let mut feed = HistoricalFeed::new_empty();
        frames = Self::generate_missing_timeframes(frames, &required_timeframes, &mut feed)?;
        
        for (tf, frame) in frames {
            feed.frames.insert(tf.clone(), Arc::new(frame));
            if feed.primary_timeframe.is_none() {
                feed.primary_timeframe = Some(tf);
            } else {
                let current_len = feed.frames.get(&feed.primary_timeframe.clone().unwrap())
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

    fn generate_missing_timeframes(
        mut frames: HashMap<TimeFrame, QuoteFrame>,
        required: &[TimeFrame],
        feed: &mut HistoricalFeed,
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

            let source_tf = source_frame.timeframe().clone();
            let aggregated = TimeFrameAggregator::aggregate(&source_frame, required_tf.clone())
                .map_err(|e| {
                    StrategyExecutionError::Feed(format!(
                        "Failed to aggregate timeframe {:?}: {}",
                        required_tf, e
                    ))
                })?;

            let aggregated_frame = aggregated.frame;
            let aggregated_metadata = aggregated.metadata;
            let aggregated_source_indices = aggregated.source_indices;
            
            let mut frame_copy = QuoteFrame::new(
                aggregated_frame.symbol().clone(),
                aggregated_frame.timeframe().clone(),
            );
            for quote in aggregated_frame.iter() {
                frame_copy.push(quote.clone()).map_err(|e| {
                    StrategyExecutionError::Feed(format!("Failed to copy quote: {}", e))
                })?;
            }
            
            let aggregated_arc = Arc::new(AggregatedQuoteFrame {
                frame: frame_copy,
                metadata: aggregated_metadata.clone(),
                source_indices: aggregated_source_indices.clone(),
            });
            
            feed.aggregated_frames.insert(
                required_tf.clone(),
                (aggregated_arc, source_tf),
            );

            generated.insert(required_tf.clone(), aggregated_frame);
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
        self.populate_conditions().await?;
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
            self.analytics.increment_bars_in_positions_if_has_positions(has_open_positions);
            
            self.feed.expand_aggregated_timeframes(&mut self.context, self.strategy.indicator_bindings())?;
            
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
        let initial_capital = self.analytics.equity_curve().first()
            .copied()
            .unwrap_or(10000.0);
        
        let start_date = self.feed.primary_timeframe.as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .and_then(|frame| frame.first())
            .map(|quote| quote.timestamp());
        
        let end_date = self.feed.primary_timeframe.as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .and_then(|frame| frame.latest())
            .map(|quote| quote.timestamp());
        
        let total_bars = self.feed.primary_timeframe.as_ref()
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
            let mut aggregated_indicators = HashMap::new();
            
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
                        computed.insert(binding.alias.clone(), values.clone());
                        
                        if self.feed.aggregated_frames.contains_key(&timeframe) {
                            aggregated_indicators.insert(binding.alias.clone(), values);
                        }
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
                        
                        if self.feed.aggregated_frames.contains_key(&timeframe) {
                            aggregated_indicators.insert(binding.alias.clone(), values);
                        }
                    }
                }
            }
            
            if !aggregated_indicators.is_empty() {
                self.feed.save_aggregated_indicators(&timeframe, aggregated_indicators);
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

    async fn populate_conditions(&mut self) -> Result<(), StrategyExecutionError> {
        let mut grouped: HashMap<TimeFrame, Vec<String>> = HashMap::new();
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

                let result = condition.condition.check(input).await.map_err(|err| {
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
    aggregated_frames: HashMap<TimeFrame, (Arc<AggregatedQuoteFrame>, TimeFrame)>,
    aggregated_indicators: HashMap<TimeFrame, HashMap<String, Arc<Vec<f32>>>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: Option<TimeFrame>,
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
        Self {
            frames: arc_frames,
            aggregated_frames: HashMap::new(),
            aggregated_indicators: HashMap::new(),
            indices: HashMap::new(),
            primary_timeframe,
        }
    }

    fn new_empty() -> Self {
        Self {
            frames: HashMap::new(),
            aggregated_frames: HashMap::new(),
            aggregated_indicators: HashMap::new(),
            indices: HashMap::new(),
            primary_timeframe: None,
        }
    }
    
    fn save_aggregated_indicators(&mut self, timeframe: &TimeFrame, indicators: HashMap<String, Arc<Vec<f32>>>) {
        self.aggregated_indicators.insert(timeframe.clone(), indicators);
    }
    
    fn get_aggregated_indicators(&self, timeframe: &TimeFrame) -> Option<&HashMap<String, Arc<Vec<f32>>>> {
        self.aggregated_indicators.get(timeframe)
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

    fn is_aggregated_bar_closed(
        aggregated: &AggregatedQuoteFrame,
        aggregated_bar_index: usize,
        current_primary_index: usize,
    ) -> bool {
        if let Some(source_indices) = aggregated.source_indices.get(&aggregated_bar_index) {
            if source_indices.is_empty() {
                return false;
            }
            if let Some(&last_source_index) = source_indices.last() {
                return last_source_index < current_primary_index;
            }
        }
        false
    }

    fn expand_aggregated_timeframes(
        &self,
        context: &mut StrategyContext,
        indicator_bindings: &[IndicatorBindingSpec],
    ) -> Result<(), StrategyExecutionError> {
        for (aggregated_tf, (aggregated, source_tf)) in &self.aggregated_frames {
            let source_frame = self.frames.get(source_tf)
                .ok_or_else(|| {
                    StrategyExecutionError::Feed(format!(
                        "Source timeframe {:?} not found in frames",
                        source_tf
                    ))
                })?;
            
            let expanded = aggregated
                .expand(source_frame.as_ref())
                .map_err(|e| {
                    StrategyExecutionError::Feed(format!(
                        "Failed to expand timeframe {:?}: {}",
                        aggregated_tf, e
                    ))
                })?;

            let primary_timeframe = self.primary_timeframe.as_ref().unwrap_or(source_tf);
            let primary_index = context.timeframe(primary_timeframe)
                .map(|d| d.index())
                .unwrap_or(0);
            
            let ratio = aggregated.metadata.aggregation_ratio as usize;
            
            let aggregated_bar_index = primary_index / ratio;
            let max_aggregated_index = aggregated.frame.len().saturating_sub(1);
            let safe_aggregated_index = aggregated_bar_index.min(max_aggregated_index);
            
            let aggregated_bar_is_closed = Self::is_aggregated_bar_closed(
                aggregated,
                safe_aggregated_index,
                primary_index,
            );
            
            let closed_aggregated_index = if aggregated_bar_is_closed && safe_aggregated_index > 0 {
                safe_aggregated_index
            } else if safe_aggregated_index > 0 {
                safe_aggregated_index - 1
            } else {
                0
            };
            
            let expanded_index = (closed_aggregated_index * ratio).min(expanded.len().saturating_sub(1));
            
            let mut data = TimeframeData::with_quote_frame(&expanded, expanded_index);
            
            if let Some(saved_indicators) = self.get_aggregated_indicators(aggregated_tf) {
                for binding in indicator_bindings {
                    if binding.timeframe == *aggregated_tf {
                        if let Some(original_values) = saved_indicators.get(&binding.alias) {
                            if !original_values.is_empty() {
                                let expanded_values = Self::expand_indicator_values_by_source_indices(
                                    original_values.as_ref(),
                                    aggregated,
                                    &expanded,
                                );
                                if expanded_values.len() == expanded.len() {
                                    data.insert_indicator_arc(binding.alias.clone(), Arc::new(expanded_values));
                                }
                            }
                        }
                    }
                }
            } else if let Some(existing_data) = context.timeframe(aggregated_tf).ok() {
                for binding in indicator_bindings {
                    if binding.timeframe == *aggregated_tf {
                        if let Some(original_values) = existing_data.indicator_series_slice(&binding.alias) {
                            if !original_values.is_empty() {
                                let expanded_values = Self::expand_indicator_values_by_source_indices(
                                    original_values,
                                    aggregated,
                                    &expanded,
                                );
                                if expanded_values.len() == expanded.len() {
                                    data.insert_indicator_arc(binding.alias.clone(), Arc::new(expanded_values));
                                }
                            }
                        }
                    }
                }
            }
            
            if let Ok(existing_data) = context.timeframe_mut(aggregated_tf) {
                *existing_data = data;
                existing_data.set_index(expanded_index);
            } else {
                context.insert_timeframe(aggregated_tf.clone(), data);
            }
        }
        Ok(())
    }
    
    fn expand_indicator_values(
        original_values: &[f32],
        ratio: usize,
        target_len: usize,
    ) -> Vec<f32> {
        if original_values.is_empty() {
            return vec![0.0; target_len];
        }
        
        let mut expanded = Vec::with_capacity(target_len);
        
        for i in 0..target_len {
            let original_idx = i / ratio;
            if original_idx < original_values.len() {
                expanded.push(original_values[original_idx]);
            } else if !original_values.is_empty() {
                expanded.push(original_values[original_values.len() - 1]);
            } else {
                expanded.push(0.0);
            }
        }
        
        expanded
    }
    
    fn expand_indicator_values_by_source_indices(
        original_values: &[f32],
        aggregated_frame: &AggregatedQuoteFrame,
        expanded_frame: &QuoteFrame,
    ) -> Vec<f32> {
        if original_values.is_empty() {
            return vec![];
        }
        
        let expanded_len = expanded_frame.len();
        let mut expanded = vec![0.0; expanded_len];
        
        // Создаем маппинг: временная метка агрегированного бара -> индекс в развернутом фрейме
        let mut timestamp_to_expanded_idx: HashMap<i64, Vec<usize>> = HashMap::new();
        for (idx, quote) in expanded_frame.iter().enumerate() {
            let ts = quote.timestamp_millis();
            timestamp_to_expanded_idx.entry(ts).or_insert_with(Vec::new).push(idx);
        }
        
        for agg_idx in 0..aggregated_frame.frame.len() {
            if let Some(source_indices) = aggregated_frame.source_indices.get(&agg_idx) {
                if source_indices.is_empty() {
                    continue;
                }
                
                let indicator_value = if agg_idx < original_values.len() {
                    original_values[agg_idx]
                } else if !original_values.is_empty() {
                    original_values[original_values.len() - 1]
                } else {
                    0.0
                };
                
                // Получаем временную метку агрегированного бара (выровненную до границы таймфрейма)
                let aggregated_quote = aggregated_frame.frame.get(agg_idx);
                if let Some(agg_quote) = aggregated_quote {
                    let agg_timestamp = agg_quote.timestamp_millis();
                    
                    // Находим все развернутые бары, которые начинаются с этой временной метки
                    // и последующие бары в количестве source_indices.len()
                    if let Some(expanded_indices) = timestamp_to_expanded_idx.get(&agg_timestamp) {
                        // Берем первый индекс с совпадающей временной меткой
                        if let Some(&first_expanded_idx) = expanded_indices.first() {
                            // Ставим значение индикатора на все развернутые бары,
                            // начиная с первого, который имеет совпадающую временную метку
                            let count = source_indices.len();
                            for i in 0..count {
                                let idx = first_expanded_idx + i;
                                if idx < expanded_len {
                                    expanded[idx] = indicator_value;
                                }
                            }
                        }
                    } else {
                        // Если не нашли точного совпадения, используем source_indices
                        // для определения позиций (fallback)
                        for &source_idx in source_indices {
                            if source_idx < expanded_len {
                                expanded[source_idx] = indicator_value;
                            }
                        }
                    }
                }
            }
        }
        
        // Заполняем пропуски предыдущим значением (forward fill)
        for i in 1..expanded.len() {
            if expanded[i] == 0.0 && expanded[i - 1] != 0.0 {
                expanded[i] = expanded[i - 1];
            }
        }
        
        expanded
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
            let entry = self
                .indices
                .entry(primary_timeframe.clone())
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
            let entry = self
                .indices
                .entry(primary_timeframe.clone())
                .or_insert(0);
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
        assert_eq!(report.trades.len(), 1);
        let trade = &report.trades[0];
        assert!((trade.pnl - 5.0).abs() < 1e-6);
        assert_eq!(report.metrics.total_trades, 1);
        assert!((report.metrics.total_pnl - 5.0).abs() < 1e-6);
        assert!((report.metrics.win_rate - 1.0).abs() < 1e-6);
        assert_eq!(report.equity_curve.last().copied().unwrap_or_default(), 5.0);
    }
}
