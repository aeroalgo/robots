use std::collections::HashMap;
use chrono::Utc;

use crate::data_model::bar_types::BarType;
use crate::data_model::types::{Symbol, TimeFrame};
use crate::strategy::context::StrategyContext;
use crate::strategy::multitimeframe_signal_sync::{
    CombinedSignal, SignalCombiner, TimeFrameBarTypeKey,
};
use crate::strategy::types::ConditionEvaluation;

pub struct MultiTimeFrameStrategyWrapper {
    combiner: SignalCombiner,
    target_timeframe: TimeFrame,
    target_bar_type: BarType,
}

impl MultiTimeFrameStrategyWrapper {
    pub fn new(
        symbol: Symbol,
        timeframe_bar_types: Vec<(TimeFrame, BarType)>,
        target_timeframe: TimeFrame,
        target_bar_type: BarType,
    ) -> Self {
        Self {
            combiner: SignalCombiner::new(symbol, timeframe_bar_types),
            target_timeframe,
            target_bar_type,
        }
    }

    pub fn update_condition_evaluations(
        &mut self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
        condition_evaluations: &HashMap<String, ConditionEvaluation>,
    ) {
        let current_time = Utc::now();
        self.combiner
            .update_signals(timeframe, bar_type, condition_evaluations, current_time);
    }

    pub fn get_combined_signal(&self) -> Option<CombinedSignal> {
        let current_time = Utc::now();
        self.combiner
            .combine_signals(&self.target_timeframe, &self.target_bar_type, current_time)
    }

    pub fn should_open_position(&self) -> bool {
        self.get_combined_signal()
            .map(|signal| signal.should_open_position())
            .unwrap_or(false)
    }

    pub fn get_signal_strength(&self) -> f32 {
        self.get_combined_signal()
            .map(|signal| signal.get_signal_strength())
            .unwrap_or(0.0)
    }

    pub fn combiner(&self) -> &SignalCombiner {
        &self.combiner
    }

    pub fn combiner_mut(&mut self) -> &mut SignalCombiner {
        &mut self.combiner
    }
}

pub struct MultiTimeFrameExecutorExtension {
    wrappers: HashMap<TimeFrameBarTypeKey, MultiTimeFrameStrategyWrapper>,
}

impl MultiTimeFrameExecutorExtension {
    pub fn new() -> Self {
        Self {
            wrappers: HashMap::new(),
        }
    }

    pub fn register_strategy(
        &mut self,
        key: TimeFrameBarTypeKey,
        wrapper: MultiTimeFrameStrategyWrapper,
    ) {
        self.wrappers.insert(key, wrapper);
    }

    pub fn update_evaluations(
        &mut self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
        condition_evaluations: &HashMap<String, ConditionEvaluation>,
    ) {
        let key = TimeFrameBarTypeKey {
            timeframe: timeframe.clone(),
            bar_type: bar_type.clone(),
        };

        if let Some(wrapper) = self.wrappers.get_mut(&key) {
            wrapper.update_condition_evaluations(timeframe, bar_type, condition_evaluations);
        }
    }

    pub fn get_combined_signal(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> Option<CombinedSignal> {
        let key = TimeFrameBarTypeKey {
            timeframe: target_timeframe.clone(),
            bar_type: target_bar_type.clone(),
        };

        self.wrappers
            .get(&key)
            .and_then(|wrapper| wrapper.get_combined_signal())
    }

    pub fn should_open_position(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> bool {
        self.get_combined_signal(target_timeframe, target_bar_type)
            .map(|signal| signal.should_open_position())
            .unwrap_or(false)
    }

    pub fn get_signal_strength(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> f32 {
        self.get_combined_signal(target_timeframe, target_bar_type)
            .map(|signal| signal.get_signal_strength())
            .unwrap_or(0.0)
    }
}

impl Default for MultiTimeFrameExecutorExtension {
    fn default() -> Self {
        Self::new()
    }
}

pub trait MultiTimeFrameStrategyContext {
    fn update_multitimeframe_signals(
        &mut self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
        condition_evaluations: &HashMap<String, ConditionEvaluation>,
    );

    fn get_multitimeframe_signal(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> Option<CombinedSignal>;

    fn should_open_position_multitimeframe(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> bool;
}

pub struct EnhancedStrategyContext {
    base_context: StrategyContext,
    multitimeframe_extension: MultiTimeFrameExecutorExtension,
}

impl EnhancedStrategyContext {
    pub fn new(base_context: StrategyContext) -> Self {
        Self {
            base_context,
            multitimeframe_extension: MultiTimeFrameExecutorExtension::new(),
        }
    }

    pub fn with_multitimeframe_config(
        mut self,
        symbol: Symbol,
        timeframe_bar_types: Vec<(TimeFrame, BarType)>,
        target_timeframe: TimeFrame,
        target_bar_type: BarType,
    ) -> Self {
        let key = TimeFrameBarTypeKey {
            timeframe: target_timeframe.clone(),
            bar_type: target_bar_type.clone(),
        };
        let wrapper = MultiTimeFrameStrategyWrapper::new(
            symbol,
            timeframe_bar_types,
            target_timeframe,
            target_bar_type,
        );
        self.multitimeframe_extension.register_strategy(key, wrapper);
        self
    }

    pub fn base_context(&self) -> &StrategyContext {
        &self.base_context
    }

    pub fn base_context_mut(&mut self) -> &mut StrategyContext {
        &mut self.base_context
    }

    pub fn multitimeframe_extension(&self) -> &MultiTimeFrameExecutorExtension {
        &self.multitimeframe_extension
    }

    pub fn multitimeframe_extension_mut(&mut self) -> &mut MultiTimeFrameExecutorExtension {
        &mut self.multitimeframe_extension
    }
}

impl MultiTimeFrameStrategyContext for EnhancedStrategyContext {
    fn update_multitimeframe_signals(
        &mut self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
        condition_evaluations: &HashMap<String, ConditionEvaluation>,
    ) {
        self.multitimeframe_extension
            .update_evaluations(timeframe, bar_type, condition_evaluations);
    }

    fn get_multitimeframe_signal(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> Option<CombinedSignal> {
        self.multitimeframe_extension
            .get_combined_signal(target_timeframe, target_bar_type)
    }

    fn should_open_position_multitimeframe(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
    ) -> bool {
        self.multitimeframe_extension
            .should_open_position(target_timeframe, target_bar_type)
    }
}

