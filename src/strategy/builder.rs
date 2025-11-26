use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::condition::factory::ConditionFactory;
use crate::condition::types::{ConditionError, SignalStrength};
use crate::indicators::formula::FormulaDefinition;

use super::base::Strategy;
use super::context::StrategyContext;
use super::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionEvaluation, ConditionInputSpec,
    ConditionOperator, DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec,
    PositionDirection, PreparedCondition, PreparedStopHandler, PreparedTakeHandler, PriceField,
    RuleLogic, StopSignal, StrategyDecision, StrategyDefinition, StrategyError, StrategyId,
    StrategyMetadata, StrategyParamValue, StrategyParameterMap, StrategyRuleSpec, StrategySignal,
    StrategySignalType, StrategyUserInput, TimeframeRequirement, UserFormulaMetadata,
};
use crate::risk::stops::{StopEvaluationContext, StopHandlerError, StopHandlerFactory};
use crate::risk::takes::{TakeEvaluationContext, TakeHandlerError, TakeHandlerFactory};

#[derive(Clone)]
struct OptimizedRule {
    rule: StrategyRuleSpec,
    condition_indices: Vec<usize>,
}

#[derive(Clone)]
pub struct DynamicStrategy {
    metadata: StrategyMetadata,
    definition: StrategyDefinition,
    indicator_bindings: Vec<IndicatorBindingSpec>,
    conditions: Vec<PreparedCondition>,
    entry_rules: Vec<OptimizedRule>,
    exit_rules: Vec<OptimizedRule>,
    stop_handlers: Vec<PreparedStopHandler>,
    take_handlers: Vec<PreparedTakeHandler>,
    timeframe_requirements: Vec<TimeframeRequirement>,
    parameters: StrategyParameterMap,
    /// Служебные индикаторы для стоп-обработчиков (ATR, MINFOR, MAXFOR)
    auxiliary_specs: Vec<crate::risk::stops::AuxiliaryIndicatorSpec>,
}

impl DynamicStrategy {
    pub fn new(
        metadata: StrategyMetadata,
        definition: StrategyDefinition,
        indicator_bindings: Vec<IndicatorBindingSpec>,
        conditions: Vec<PreparedCondition>,
        entry_rules: Vec<StrategyRuleSpec>,
        exit_rules: Vec<StrategyRuleSpec>,
        stop_handlers: Vec<PreparedStopHandler>,
        take_handlers: Vec<PreparedTakeHandler>,
        timeframe_requirements: Vec<TimeframeRequirement>,
        parameters: StrategyParameterMap,
        auxiliary_specs: Vec<crate::risk::stops::AuxiliaryIndicatorSpec>,
    ) -> Self {
        let condition_lookup: HashMap<String, usize> = conditions
            .iter()
            .enumerate()
            .map(|(idx, condition)| (condition.id.clone(), idx))
            .collect();

        let optimize_rules =
            |rules: Vec<StrategyRuleSpec>, lookup: &HashMap<String, usize>| -> Vec<OptimizedRule> {
                rules
                    .into_iter()
                    .map(|rule| {
                        let condition_indices: Vec<usize> = rule
                            .conditions
                            .iter()
                            .filter_map(|id| lookup.get(id).copied())
                            .collect();
                        OptimizedRule {
                            rule,
                            condition_indices,
                        }
                    })
                    .collect()
            };

        Self {
            metadata,
            definition,
            indicator_bindings,
            conditions,
            entry_rules: optimize_rules(entry_rules, &condition_lookup),
            exit_rules: optimize_rules(exit_rules, &condition_lookup),
            stop_handlers,
            take_handlers,
            timeframe_requirements,
            parameters,
            auxiliary_specs,
        }
    }

    pub fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }

    pub fn definition(&self) -> &StrategyDefinition {
        &self.definition
    }

    fn evaluate_conditions(
        &self,
        context: &StrategyContext,
    ) -> Result<Vec<Option<ConditionEvaluation>>, StrategyError> {
        let mut result = vec![None; self.conditions.len()];
        for (idx, condition) in self.conditions.iter().enumerate() {
            let condition_id = &condition.id;
            let timeframe_data = context.timeframe(&condition.timeframe)?;
            let current_index = timeframe_data.index();
            let previous_index = current_index.saturating_sub(1);

            let evaluation =
                if let Some(precomputed) = timeframe_data.condition_result_by_index(idx) {
                    let idx = self.resolve_index(previous_index, precomputed.signals.len());
                    ConditionEvaluation {
                        condition_id: condition_id.clone(),
                        satisfied: precomputed.signals.get(idx).copied().unwrap_or(false),
                        strength: precomputed
                            .strengths
                            .get(idx)
                            .copied()
                            .unwrap_or(SignalStrength::Weak),
                        weight: condition.weight(),
                    }
                } else {
                    let input = context.prepare_condition_input_with_index_offset(condition, 1)?;
                    let raw = condition.condition.check(input).map_err(|err| {
                        StrategyError::ConditionFailure {
                            condition_id: condition_id.clone(),
                            source: err,
                        }
                    })?;
                    let idx = self.resolve_index(previous_index, raw.signals.len());
                    ConditionEvaluation {
                        condition_id: condition_id.clone(),
                        satisfied: raw.signals.get(idx).copied().unwrap_or(false),
                        strength: raw
                            .strengths
                            .get(idx)
                            .copied()
                            .unwrap_or(SignalStrength::Weak),
                        weight: condition.weight(),
                    }
                };

            result[idx] = Some(evaluation);
        }
        Ok(result)
    }

    fn resolve_index(&self, requested: usize, available: usize) -> usize {
        if available == 0 {
            0
        } else if requested >= available {
            available - 1
        } else {
            requested
        }
    }

    fn evaluate_rule(
        &self,
        rule: &StrategyRuleSpec,
        condition_indices: &[usize],
        evaluations: &[Option<ConditionEvaluation>],
        context: &StrategyContext,
    ) -> Result<Option<StrategySignal>, StrategyError> {
        if condition_indices.is_empty() {
            return Err(StrategyError::DefinitionError(format!(
                "rule {} has no conditions",
                rule.id
            )));
        }
        let mut satisfied_count = 0usize;
        let mut weight_sum = 0.0f32;
        let mut weighted_score = 0.0f32;
        let mut strength_values = Vec::with_capacity(condition_indices.len());
        for &condition_idx in condition_indices {
            let evaluation = evaluations
                .get(condition_idx)
                .and_then(|opt| opt.as_ref())
                .ok_or_else(|| StrategyError::UnknownConditionReference {
                    rule_id: rule.id.clone(),
                    condition_id: String::new(),
                })?;
            strength_values.push(evaluation.strength);
            if evaluation.satisfied {
                satisfied_count += 1;
                let weight = evaluation.weight.max(0.0);
                weight_sum += weight;
                weighted_score += weight * (evaluation.strength as i32 as f32);
            }
        }
        let satisfied = match rule.logic {
            RuleLogic::All => satisfied_count == condition_indices.len(),
            RuleLogic::Any => satisfied_count > 0,
            RuleLogic::AtLeast(required) => satisfied_count >= required,
            RuleLogic::Weighted { min_total } => weighted_score >= min_total,
            RuleLogic::Expression(ref expr) => {
                return Err(StrategyError::UnsupportedRuleLogic(expr.clone()))
            }
        };
        if !satisfied {
            return Ok(None);
        }
        let average_score = if weight_sum > 0.0 {
            weighted_score / weight_sum
        } else {
            0.0
        };
        let strength = self.determine_strength(average_score, &strength_values);
        let timeframe = condition_indices
            .iter()
            .find_map(|&idx| self.conditions.get(idx).map(|cond| &cond.timeframe))
            .cloned()
            .unwrap_or_else(|| {
                self.timeframe_requirements
                    .first()
                    .map(|req| &req.timeframe)
                    .cloned()
                    .unwrap_or_else(|| crate::data_model::types::TimeFrame::Minutes(1))
            });
        let rule_id = rule.id.clone();
        let signal_type = rule.signal.clone();
        let direction = rule.direction.clone();
        let mut signal = StrategySignal {
            rule_id,
            signal_type,
            direction,
            timeframe,
            strength,
            quantity: rule.quantity,
            entry_rule_id: None,
            tags: rule.tags.clone(),
            position_group: None,
            target_entry_ids: Vec::with_capacity(rule.target_entry_ids.len()),
        };
        match signal.signal_type {
            StrategySignalType::Entry => {
                signal.position_group = Some(rule.position_group_key());
                signal.entry_rule_id = Some(rule.id.clone());
            }
            StrategySignalType::Exit => {
                signal.target_entry_ids = rule.target_entry_ids.clone();
            }
            StrategySignalType::Custom(_) => {}
        }
        if matches!(signal.signal_type, StrategySignalType::Entry)
            && self.entry_rule_already_open(context, &signal)
        {
            return Ok(None);
        }
        Ok(Some(signal))
    }

    fn determine_strength(
        &self,
        average_score: f32,
        strengths: &[SignalStrength],
    ) -> SignalStrength {
        if strengths.is_empty() {
            return SignalStrength::Weak;
        }
        if average_score >= 3.5 {
            SignalStrength::VeryStrong
        } else if average_score >= 2.5 {
            SignalStrength::Strong
        } else if average_score >= 1.5 {
            SignalStrength::Medium
        } else {
            strengths
                .iter()
                .copied()
                .max()
                .unwrap_or(SignalStrength::Weak)
        }
    }

    fn evaluate_stop_handlers(
        &self,
        context: &StrategyContext,
    ) -> Result<Vec<StopSignal>, StrategyError> {
        if context.active_positions().is_empty() {
            return Ok(Vec::new());
        }
        let positions_count = context.active_positions().len();
        let handlers_count = self.stop_handlers.len();
        let mut signals = Vec::with_capacity(positions_count * handlers_count);
        for handler in &self.stop_handlers {
            let timeframe_data = context.timeframe(&handler.timeframe)?;
            let series = timeframe_data
                .price_series_slice(&handler.price_field)
                .ok_or_else(|| StrategyError::MissingPriceSeries {
                    field: handler.price_field.clone(),
                    timeframe: handler.timeframe.clone(),
                })?;
            if series.is_empty() {
                continue;
            }
            let index = timeframe_data.index().min(series.len().saturating_sub(1));
            let current_price = series[index] as f64;
            for position in context.active_positions().values() {
                if position.timeframe != handler.timeframe {
                    continue;
                }
                if !self.stop_direction_matches(&handler.direction, &position.direction) {
                    continue;
                }
                if !handler.target_entry_ids.is_empty() {
                    let mut matches_target = false;
                    if let Some(group) = position.position_group.as_ref() {
                        if handler.target_entry_ids.iter().any(|id| id == group) {
                            matches_target = true;
                        }
                    }
                    if !matches_target {
                        if let Some(entry_id) = position.entry_rule_id.as_ref() {
                            if handler.target_entry_ids.iter().any(|id| id == entry_id) {
                                matches_target = true;
                            }
                        }
                    }
                    if !matches_target {
                        continue;
                    }
                }
                let eval_ctx = StopEvaluationContext {
                    position,
                    timeframe_data,
                    price_field: handler.price_field.clone(),
                    index,
                    current_price,
                };
                if let Some(outcome) = handler.handler.evaluate(&eval_ctx) {
                    let mut signal = StrategySignal {
                        rule_id: handler.id.clone(),
                        signal_type: StrategySignalType::Exit,
                        direction: position.direction.clone(),
                        timeframe: handler.timeframe.clone(),
                        strength: SignalStrength::VeryStrong,
                        quantity: Some(position.quantity),
                        entry_rule_id: position.entry_rule_id.clone(),
                        tags: handler.tags.clone(),
                        position_group: None,
                        target_entry_ids: Vec::with_capacity(handler.target_entry_ids.len() + 1),
                    };
                    if let Some(group) = position.position_group.as_ref() {
                        signal.target_entry_ids.push(group.clone());
                    }
                    if let Some(entry_id) = position.entry_rule_id.as_ref() {
                        if !signal.target_entry_ids.iter().any(|id| id == entry_id) {
                            signal.target_entry_ids.push(entry_id.clone());
                        }
                    }
                    if !signal.tags.iter().any(|tag| tag == "stop") {
                        signal.tags.push("stop".to_string());
                    }
                    let mut metadata = outcome.metadata;
                    metadata.insert("handler_name".to_string(), handler.name.clone());
                    signals.push(StopSignal {
                        handler_id: handler.id.clone(),
                        signal,
                        exit_price: outcome.exit_price,
                        kind: outcome.kind,
                        priority: handler.priority,
                        metadata,
                    });
                }
            }
        }
        signals.sort_by(|a, b| a.priority.cmp(&b.priority));
        Ok(signals)
    }

    fn evaluate_take_handlers(
        &self,
        context: &StrategyContext,
    ) -> Result<Vec<StopSignal>, StrategyError> {
        if context.active_positions().is_empty() {
            return Ok(Vec::new());
        }
        let positions_count = context.active_positions().len();
        let handlers_count = self.take_handlers.len();
        let mut signals = Vec::with_capacity(positions_count * handlers_count);
        for handler in &self.take_handlers {
            let timeframe_data = context.timeframe(&handler.timeframe)?;
            let series = timeframe_data
                .price_series_slice(&handler.price_field)
                .ok_or_else(|| StrategyError::MissingPriceSeries {
                    field: handler.price_field.clone(),
                    timeframe: handler.timeframe.clone(),
                })?;
            if series.is_empty() {
                continue;
            }
            let index = timeframe_data.index().min(series.len().saturating_sub(1));
            let current_price = series[index] as f64;
            for position in context.active_positions().values() {
                if position.timeframe != handler.timeframe {
                    continue;
                }
                if !self.stop_direction_matches(&handler.direction, &position.direction) {
                    continue;
                }
                if !handler.target_entry_ids.is_empty() {
                    let mut matches_target = false;
                    if let Some(group) = position.position_group.as_ref() {
                        if handler.target_entry_ids.iter().any(|id| id == group) {
                            matches_target = true;
                        }
                    }
                    if !matches_target {
                        if let Some(entry_id) = position.entry_rule_id.as_ref() {
                            if handler.target_entry_ids.iter().any(|id| id == entry_id) {
                                matches_target = true;
                            }
                        }
                    }
                    if !matches_target {
                        continue;
                    }
                }
                let eval_ctx = TakeEvaluationContext {
                    position,
                    timeframe_data,
                    price_field: handler.price_field.clone(),
                    index,
                    current_price,
                };
                if let Some(outcome) = handler.handler.evaluate(&eval_ctx) {
                    let mut signal = StrategySignal {
                        rule_id: handler.id.clone(),
                        signal_type: StrategySignalType::Exit,
                        direction: position.direction.clone(),
                        timeframe: handler.timeframe.clone(),
                        strength: SignalStrength::VeryStrong,
                        quantity: Some(position.quantity),
                        entry_rule_id: position.entry_rule_id.clone(),
                        tags: handler.tags.clone(),
                        position_group: None,
                        target_entry_ids: Vec::with_capacity(handler.target_entry_ids.len() + 1),
                    };
                    if let Some(group) = position.position_group.as_ref() {
                        signal.target_entry_ids.push(group.clone());
                    }
                    if let Some(entry_id) = position.entry_rule_id.as_ref() {
                        if !signal.target_entry_ids.iter().any(|id| id == entry_id) {
                            signal.target_entry_ids.push(entry_id.clone());
                        }
                    }
                    if !signal.tags.iter().any(|tag| tag == "take") {
                        signal.tags.push("take".to_string());
                    }
                    let mut metadata = outcome.metadata;
                    metadata.insert("handler_name".to_string(), handler.name.clone());
                    signals.push(StopSignal {
                        handler_id: handler.id.clone(),
                        signal,
                        exit_price: outcome.exit_price,
                        kind: outcome.kind,
                        priority: handler.priority,
                        metadata,
                    });
                }
            }
        }
        signals.sort_by(|a, b| a.priority.cmp(&b.priority));
        Ok(signals)
    }

    fn stop_direction_matches(
        &self,
        handler_direction: &PositionDirection,
        position_direction: &PositionDirection,
    ) -> bool {
        match handler_direction {
            PositionDirection::Both => matches!(
                position_direction,
                PositionDirection::Long | PositionDirection::Short
            ),
            PositionDirection::Long => matches!(position_direction, PositionDirection::Long),
            PositionDirection::Short => matches!(position_direction, PositionDirection::Short),
            PositionDirection::Flat => false,
        }
    }

    fn entry_rule_already_open(&self, context: &StrategyContext, signal: &StrategySignal) -> bool {
        let entry_rule_id = signal
            .entry_rule_id
            .as_deref()
            .unwrap_or_else(|| signal.rule_id.as_str());
        context.active_positions().values().any(|position| {
            position.timeframe == signal.timeframe
                && position.direction == signal.direction
                && position
                    .entry_rule_id
                    .as_deref()
                    .map(|id| id == entry_rule_id)
                    .unwrap_or(false)
        })
    }
}

impl Strategy for DynamicStrategy {
    fn id(&self) -> &StrategyId {
        &self.metadata.id
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }

    fn parameters(&self) -> &StrategyParameterMap {
        &self.parameters
    }

    fn indicator_bindings(&self) -> &[IndicatorBindingSpec] {
        &self.indicator_bindings
    }

    fn conditions(&self) -> &[PreparedCondition] {
        &self.conditions
    }

    fn entry_rules(&self) -> &[StrategyRuleSpec] {
        static EMPTY: &[StrategyRuleSpec] = &[];
        if self.entry_rules.is_empty() {
            return EMPTY;
        }
        unsafe {
            std::slice::from_raw_parts(
                self.entry_rules.as_ptr() as *const StrategyRuleSpec,
                self.entry_rules.len(),
            )
        }
    }

    fn exit_rules(&self) -> &[StrategyRuleSpec] {
        static EMPTY: &[StrategyRuleSpec] = &[];
        if self.exit_rules.is_empty() {
            return EMPTY;
        }
        unsafe {
            std::slice::from_raw_parts(
                self.exit_rules.as_ptr() as *const StrategyRuleSpec,
                self.exit_rules.len(),
            )
        }
    }

    fn timeframe_requirements(&self) -> &[TimeframeRequirement] {
        &self.timeframe_requirements
    }

    fn evaluate(&self, context: &StrategyContext) -> Result<StrategyDecision, StrategyError> {
        let evaluations = self.evaluate_conditions(context)?;
        let has_active_positions = !context.active_positions().is_empty();
        let (stop_signals, take_signals) = if has_active_positions {
            (
                self.evaluate_stop_handlers(context)?,
                self.evaluate_take_handlers(context)?,
            )
        } else {
            (Vec::new(), Vec::new())
        };
        let mut decision = StrategyDecision::empty();
        let mut metadata_key_buf = String::with_capacity(32);
        for stop in &stop_signals {
            decision.exits.push(stop.signal.clone());
            metadata_key_buf.clear();
            metadata_key_buf.push_str("stop.");
            metadata_key_buf.push_str(&stop.handler_id);
            metadata_key_buf.push_str(".exit_price");
            decision
                .metadata
                .insert(metadata_key_buf.clone(), stop.exit_price.to_string());
        }
        for take in &take_signals {
            decision.exits.push(take.signal.clone());
            metadata_key_buf.clear();
            metadata_key_buf.push_str("take.");
            metadata_key_buf.push_str(&take.handler_id);
            metadata_key_buf.push_str(".exit_price");
            decision
                .metadata
                .insert(metadata_key_buf.clone(), take.exit_price.to_string());
        }
        let mut all_stop_signals = stop_signals;
        all_stop_signals.extend(take_signals);
        all_stop_signals.sort_by(|a, b| a.priority.cmp(&b.priority));

        let mut has_exit_rule_signals = false;

        if has_active_positions {
            for optimized_rule in &self.exit_rules {
                if let Some(signal) = self.evaluate_rule(
                    &optimized_rule.rule,
                    &optimized_rule.condition_indices,
                    &evaluations,
                    context,
                )? {
                    match signal.signal_type {
                        StrategySignalType::Entry => decision.entries.push(signal),
                        StrategySignalType::Exit => {
                            decision.exits.push(signal);
                            has_exit_rule_signals = true;
                        }
                        StrategySignalType::Custom(_) => decision.custom.push(signal),
                    }
                }
            }
        }

        if !has_exit_rule_signals {
            for optimized_rule in &self.entry_rules {
                if let Some(signal) = self.evaluate_rule(
                    &optimized_rule.rule,
                    &optimized_rule.condition_indices,
                    &evaluations,
                    context,
                )? {
                    match signal.signal_type {
                        StrategySignalType::Entry => decision.entries.push(signal),
                        StrategySignalType::Exit => decision.exits.push(signal),
                        StrategySignalType::Custom(_) => decision.custom.push(signal),
                    }
                }
            }
        }

        decision.stop_signals = all_stop_signals;
        Ok(decision)
    }

    fn evaluate_stop_signals(
        &self,
        context: &StrategyContext,
    ) -> Result<Vec<StopSignal>, StrategyError> {
        if context.active_positions().is_empty() {
            return Ok(Vec::new());
        }
        let stop_signals = self.evaluate_stop_handlers(context)?;
        let take_signals = self.evaluate_take_handlers(context)?;
        let mut all_stop_signals = stop_signals;
        all_stop_signals.extend(take_signals);
        all_stop_signals.sort_by(|a, b| a.priority.cmp(&b.priority));
        Ok(all_stop_signals)
    }

    fn clone_box(&self) -> Box<dyn Strategy> {
        Box::new(self.clone())
    }

    fn auxiliary_indicator_specs(&self) -> &[crate::risk::stops::AuxiliaryIndicatorSpec] {
        &self.auxiliary_specs
    }
}

pub struct StrategyBuilder {
    definition: StrategyDefinition,
    parameter_overrides: StrategyParameterMap,
}

impl StrategyBuilder {
    pub fn new(definition: StrategyDefinition) -> Self {
        Self {
            definition,
            parameter_overrides: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, name: impl Into<String>, value: StrategyParamValue) -> Self {
        self.parameter_overrides.insert(name.into(), value);
        self
    }

    pub fn with_parameters(mut self, parameters: StrategyParameterMap) -> Self {
        for (key, value) in parameters {
            self.parameter_overrides.insert(key, value);
        }
        self
    }

    pub fn build(self) -> Result<DynamicStrategy, StrategyError> {
        use crate::indicators::parameters::ParameterPresets;
        use crate::indicators::types::ParameterType;
        let mut indicator_bindings = self.definition.indicator_bindings.clone();
        for binding in &mut indicator_bindings {
            if let IndicatorSourceSpec::Registry { name, parameters } = &mut binding.source {
                let alias = &binding.alias;
                let prefix = format!("{}_", alias);
                for (key, value) in &self.parameter_overrides {
                    if let Some(param_name) = key.strip_prefix(&prefix) {
                        let param_value = if let StrategyParamValue::Number(num_value) = value {
                            *num_value as f32
                        } else if let StrategyParamValue::Integer(int_value) = value {
                            *int_value as f32
                        } else {
                            continue;
                        };

                        let param_name_lower = param_name.to_lowercase();
                        let param_type = if param_name_lower.contains("period")
                            || param_name_lower.contains("length")
                        {
                            ParameterType::Period
                        } else if param_name_lower == "deviation" {
                            ParameterType::Multiplier
                        } else if param_name_lower.contains("multiplier")
                            || param_name_lower.contains("coeff")
                        {
                            ParameterType::Multiplier
                        } else if param_name_lower.contains("threshold")
                            || param_name_lower.contains("level")
                        {
                            ParameterType::Threshold
                        } else {
                            ParameterType::Custom
                        };

                        if let Some(range) =
                            ParameterPresets::get_range_for_parameter(name, param_name, &param_type)
                        {
                            let clamped_value = param_value.max(range.start).min(range.end);
                            parameters.insert(param_name.to_string(), clamped_value);
                        } else {
                            parameters.insert(param_name.to_string(), param_value);
                        }
                    }
                }
            }
        }
        let mut prepared_conditions = Vec::with_capacity(self.definition.condition_bindings.len());
        for binding in &self.definition.condition_bindings {
            let factory_name = binding.factory_name();
            let condition =
                ConditionFactory::create_condition(factory_name, binding.parameters.clone())
                    .map_err(|err| map_condition_error(factory_name, err))?;
            let metadata = ConditionFactory::get_condition_info(factory_name);
            prepared_conditions.push(PreparedCondition {
                id: binding.id.clone(),
                condition: Arc::from(condition),
                input: binding.input.clone(),
                timeframe: binding.timeframe.clone(),
                weight: binding.weight,
                metadata,
                tags: binding.tags.clone(),
            });
        }
        let condition_ids: HashSet<String> = prepared_conditions
            .iter()
            .map(|condition| condition.id.clone())
            .collect();
        for rule in self
            .definition
            .entry_rules
            .iter()
            .chain(self.definition.exit_rules.iter())
        {
            for condition_id in &rule.conditions {
                if !condition_ids.contains(condition_id) {
                    return Err(StrategyError::UnknownConditionReference {
                        rule_id: rule.id.clone(),
                        condition_id: condition_id.clone(),
                    });
                }
            }
        }
        let mut prepared_stop_handlers = Vec::with_capacity(self.definition.stop_handlers.len());
        let mut auxiliary_specs_collector = Vec::new();
        let mut seen_auxiliary_aliases = std::collections::HashSet::new();

        for handler in &self.definition.stop_handlers {
            // Применяем параметры из parameter_overrides к stop handlers
            // Формат: "stop_{handler.name}_{param.name}"
            let mut handler_params = handler.parameters.clone();
            let handler_prefix = format!("stop_{}_", handler.name);
            for (key, value) in &self.parameter_overrides {
                if let Some(param_name) = key.strip_prefix(&handler_prefix) {
                    handler_params.insert(param_name.to_string(), value.clone());
                }
            }

            // Нормализуем ключи параметров (в нижний регистр) для StopHandlerFactory
            let mut normalized_params = HashMap::with_capacity(handler_params.len());
            for (key, value) in &handler_params {
                normalized_params.insert(key.to_ascii_lowercase(), value.clone());
            }

            // Собираем auxiliary specs с учетом примененных параметров
            for spec in crate::risk::stops::get_auxiliary_specs_from_handler_spec(
                &handler.handler_name,
                &handler_params,
            ) {
                if !seen_auxiliary_aliases.contains(&spec.alias) {
                    seen_auxiliary_aliases.insert(spec.alias.clone());
                    auxiliary_specs_collector.push(spec);
                }
            }

            let instance = StopHandlerFactory::create(&handler.handler_name, &normalized_params)
                .map_err(|err| map_stop_error(&handler.handler_name, err))?;
            prepared_stop_handlers.push(PreparedStopHandler {
                id: handler.id.clone(),
                name: handler.name.clone(),
                handler: Arc::from(instance),
                timeframe: handler.timeframe.clone(),
                price_field: handler.price_field.clone(),
                direction: handler.direction.clone(),
                priority: handler.priority,
                tags: handler.tags.clone(),
                target_entry_ids: handler.target_entry_ids.clone(),
            });
        }

        let mut prepared_take_handlers = Vec::with_capacity(self.definition.take_handlers.len());
        for handler in &self.definition.take_handlers {
            // Применяем параметры из parameter_overrides к take handlers
            // Формат: "take_{handler.name}_{param.name}"
            let mut handler_params = handler.parameters.clone();
            let handler_prefix = format!("take_{}_", handler.name);
            for (key, value) in &self.parameter_overrides {
                if let Some(param_name) = key.strip_prefix(&handler_prefix) {
                    handler_params.insert(param_name.to_string(), value.clone());
                }
            }

            // Нормализуем ключи параметров (в нижний регистр) для TakeHandlerFactory
            let mut normalized_params = HashMap::with_capacity(handler_params.len());
            for (key, value) in &handler_params {
                normalized_params.insert(key.to_ascii_lowercase(), value.clone());
            }
            let instance = TakeHandlerFactory::create(&handler.handler_name, &normalized_params)
                .map_err(|err| map_take_error(&handler.handler_name, err))?;
            prepared_take_handlers.push(PreparedTakeHandler {
                id: handler.id.clone(),
                name: handler.name.clone(),
                handler: Arc::from(instance),
                timeframe: handler.timeframe.clone(),
                price_field: handler.price_field.clone(),
                direction: handler.direction.clone(),
                priority: handler.priority,
                tags: handler.tags.clone(),
                target_entry_ids: handler.target_entry_ids.clone(),
            });
        }

        let mut parameters = self.definition.defaults.clone();
        for (key, value) in self.parameter_overrides {
            parameters.insert(key, value);
        }

        // auxiliary_specs уже собраны при обработке stop_handlers с учетом parameter_overrides
        let auxiliary_specs = auxiliary_specs_collector;

        let strategy = DynamicStrategy::new(
            self.definition.metadata.clone(),
            self.definition.clone(),
            indicator_bindings,
            prepared_conditions,
            self.definition.entry_rules.clone(),
            self.definition.exit_rules.clone(),
            prepared_stop_handlers,
            prepared_take_handlers,
            self.definition.timeframe_requirements.clone(),
            parameters,
            auxiliary_specs,
        );
        Ok(strategy)
    }

    // =========================================================================
    // User Input API - для динамического создания стратегий
    // НЕ УДАЛЯТЬ: Этот функционал используется для создания кастомных стратегий,
    // формул (например, разница скользящих средних), парсинга условий и т.д.
    // Позволяет создавать стратегии без написания кода через API/UI.
    // =========================================================================

    /// Создаёт StrategyBuilder из пользовательского ввода.
    /// Позволяет создавать стратегии динамически без написания Rust кода.
    pub fn from_user_input(input: StrategyUserInput) -> Result<Self, StrategyError> {
        let metadata = StrategyMetadata::with_id(input.name.clone(), input.name.clone());
        let mut indicator_bindings = Vec::with_capacity(input.indicators.len());
        let mut formula_metadata = Vec::with_capacity(input.indicators.len());
        for indicator in &input.indicators {
            let timeframe =
                crate::data_model::types::TimeFrame::from_identifier(&indicator.timeframe);
            let mut numeric_params = HashMap::with_capacity(indicator.parameters.len());
            let mut string_params = HashMap::with_capacity(indicator.parameters.len());
            for (key, value) in &indicator.parameters {
                if let Some(number) = value.as_f64() {
                    numeric_params.insert(key.clone(), number as f32);
                } else if let Some(text) = value.as_str() {
                    string_params.insert(key.clone(), text.to_string());
                }
            }
            let source = if let Some(name) = string_params.get("name") {
                IndicatorSourceSpec::Registry {
                    name: name.clone(),
                    parameters: numeric_params.clone(),
                }
            } else {
                let expression = indicator.expression.trim();
                if expression.is_empty() {
                    return Err(StrategyError::DefinitionError(format!(
                        "indicator {} requires expression",
                        indicator.alias
                    )));
                }
                let definition = FormulaDefinition::parse(expression).map_err(|err| {
                    StrategyError::DefinitionError(format!(
                        "formula {} parse error: {}",
                        indicator.alias, err
                    ))
                })?;
                let inputs = definition.data_dependencies().cloned().collect::<Vec<_>>();
                formula_metadata.push(UserFormulaMetadata {
                    id: indicator.alias.clone(),
                    name: indicator.alias.clone(),
                    expression: expression.to_string(),
                    description: None,
                    tags: Vec::new(),
                    inputs,
                });
                IndicatorSourceSpec::Formula {
                    expression: expression.to_string(),
                }
            };
            indicator_bindings.push(IndicatorBindingSpec {
                alias: indicator.alias.clone(),
                timeframe,
                source,
                tags: Vec::new(),
            });
        }
        let mut condition_bindings = Vec::with_capacity(input.conditions.len());
        for condition in &input.conditions {
            let timeframe =
                crate::data_model::types::TimeFrame::from_identifier(&condition.timeframe);
            let operator = extract_condition_operator(condition)?;
            let parameters = extract_numeric_parameters(&condition.parameters);
            let input_spec = build_condition_input_spec(&operator, &condition.parameters)?;
            let declarative = ConditionDeclarativeSpec::from_input(operator, &input_spec);
            condition_bindings.push(ConditionBindingSpec {
                id: condition.id.clone(),
                name: condition.id.clone(),
                timeframe,
                declarative,
                parameters,
                input: input_spec,
                weight: 1.0,
                tags: Vec::new(),
                user_formula: Some(condition.expression.clone()),
            });
        }
        let mut entry_rules = Vec::new();
        let mut exit_rules = Vec::new();
        entry_rules.reserve(input.actions.len());
        exit_rules.reserve(input.actions.len());
        for action in &input.actions {
            let rule = StrategyRuleSpec {
                id: action.rule_id.clone(),
                name: action.rule_id.clone(),
                logic: action.logic.clone(),
                conditions: action.condition_ids.clone(),
                signal: action.signal.clone(),
                direction: action.direction.clone(),
                quantity: action.quantity,
                tags: action.tags.clone(),
                position_group: None,
                target_entry_ids: Vec::new(),
            };
            match action.signal {
                StrategySignalType::Entry => entry_rules.push(rule),
                StrategySignalType::Exit => exit_rules.push(rule),
                StrategySignalType::Custom(_) => entry_rules.push(rule),
            }
        }
        let timeframe_requirements = indicator_bindings
            .iter()
            .map(|binding| TimeframeRequirement {
                alias: binding.alias.clone(),
                timeframe: binding.timeframe.clone(),
            })
            .collect();
        let defaults = input.parameters.clone();
        let definition = StrategyDefinition {
            metadata,
            parameters: Vec::new(),
            indicator_bindings,
            formulas: formula_metadata,
            condition_bindings,
            entry_rules,
            exit_rules,
            stop_handlers: Vec::new(),
            take_handlers: vec![],
            timeframe_requirements,
            defaults,
            optimizer_hints: BTreeMap::new(),
        };
        Ok(Self::new(definition))
    }
}

// =============================================================================
// Helper functions for User Input parsing
// НЕ УДАЛЯТЬ: Используются функцией from_user_input для парсинга пользовательского ввода
// =============================================================================

fn map_condition_error(name: &str, error: ConditionError) -> StrategyError {
    StrategyError::DefinitionError(format!("condition {} creation failed: {}", name, error))
}

fn map_stop_error(name: &str, error: StopHandlerError) -> StrategyError {
    StrategyError::DefinitionError(format!("stop handler {} creation failed: {}", name, error))
}

fn map_take_error(name: &str, error: TakeHandlerError) -> StrategyError {
    StrategyError::DefinitionError(format!("take handler {} creation failed: {}", name, error))
}

/// Извлекает оператор условия из пользовательского шага условия
fn extract_condition_operator(
    step: &super::types::UserConditionStep,
) -> Result<ConditionOperator, StrategyError> {
    if let Some(operator) = step
        .parameters
        .get("operator")
        .and_then(|value| value.as_str())
        .and_then(parse_condition_operator)
    {
        return Ok(operator);
    }
    if let Some(operator) = step
        .parameters
        .get("condition_name")
        .and_then(|value| value.as_str())
        .and_then(parse_condition_operator)
    {
        return Ok(operator);
    }
    if let Some(operator) = step
        .parameters
        .get("name")
        .and_then(|value| value.as_str())
        .and_then(parse_condition_operator)
    {
        return Ok(operator);
    }
    if !step.expression.is_empty() {
        if let Some(operator) = parse_condition_operator(&step.expression) {
            return Ok(operator);
        }
    }
    Err(StrategyError::DefinitionError(format!(
        "condition {} has no identifiable operator",
        step.id
    )))
}

/// Парсит строковое представление оператора условия
fn parse_condition_operator(value: &str) -> Option<ConditionOperator> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let upper = trimmed.to_ascii_uppercase();
    match upper.as_str() {
        ">" | "GT" | "GREATER" | "GREATERTHAN" | "ABOVE" => {
            return Some(ConditionOperator::GreaterThan)
        }
        "<" | "LT" | "LESS" | "LESSTHAN" | "BELOW" => return Some(ConditionOperator::LessThan),
        "CROSSESABOVE" | "CROSSABOVE" | "CROSSUP" | "CROSS_UP" => {
            return Some(ConditionOperator::CrossesAbove)
        }
        "CROSSESBELOW" | "CROSSBELOW" | "CROSSDOWN" | "CROSS_DOWN" => {
            return Some(ConditionOperator::CrossesBelow)
        }
        "BETWEEN" => return Some(ConditionOperator::Between),
        _ => {}
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("cross") && lower.contains("above") {
        return Some(ConditionOperator::CrossesAbove);
    }
    if lower.contains("cross") && lower.contains("below") {
        return Some(ConditionOperator::CrossesBelow);
    }
    if lower.contains("between") {
        return Some(ConditionOperator::Between);
    }
    if lower.contains('>') {
        return Some(ConditionOperator::GreaterThan);
    }
    if lower.contains('<') {
        return Some(ConditionOperator::LessThan);
    }
    None
}

/// Извлекает числовые параметры из StrategyParameterMap
fn extract_numeric_parameters(parameters: &StrategyParameterMap) -> HashMap<String, f32> {
    let mut result = HashMap::with_capacity(parameters.len());
    for (key, value) in parameters {
        if let Some(number) = value.as_f64() {
            result.insert(key.clone(), number as f32);
        }
    }
    result
}

/// Строит спецификацию входных данных условия из параметров пользователя
fn build_condition_input_spec(
    operator: &ConditionOperator,
    parameters: &StrategyParameterMap,
) -> Result<ConditionInputSpec, StrategyError> {
    let primary = parameters
        .get("primary")
        .and_then(parse_series_source)
        .ok_or_else(|| {
            StrategyError::DefinitionError("condition primary source not specified".to_string())
        })?;

    if matches!(operator, ConditionOperator::Between) {
        let lower = parameters
            .get("lower")
            .and_then(parse_series_source)
            .ok_or_else(|| {
                StrategyError::DefinitionError(
                    "between condition requires lower bound series".to_string(),
                )
            })?;
        let upper = parameters
            .get("upper")
            .and_then(parse_series_source)
            .ok_or_else(|| {
                StrategyError::DefinitionError(
                    "between condition requires upper bound series".to_string(),
                )
            })?;
        return Ok(ConditionInputSpec::Range {
            source: primary,
            lower,
            upper,
        });
    }

    if let Some(secondary_value) = parameters.get("secondary").and_then(parse_series_source) {
        if let Some(percent_value) = parameters.get("percent").and_then(|value| value.as_f64()) {
            return Ok(ConditionInputSpec::DualWithPercent {
                primary,
                secondary: secondary_value,
                percent: percent_value as f32,
            });
        }
        return Ok(ConditionInputSpec::Dual {
            primary,
            secondary: secondary_value,
        });
    }

    if let Some(index_offset) = parameters
        .get("index_offset")
        .and_then(|value| value.as_f64())
        .map(|value| value.max(0.0) as usize)
    {
        return Ok(ConditionInputSpec::Indexed {
            source: primary,
            index_offset,
        });
    }

    match operator {
        ConditionOperator::GreaterThan
        | ConditionOperator::LessThan
        | ConditionOperator::CrossesAbove
        | ConditionOperator::CrossesBelow => Err(StrategyError::DefinitionError(
            "condition requires secondary source".to_string(),
        )),
        ConditionOperator::Between => unreachable!("handled above"),
    }
}

/// Парсит источник данных серии из параметра стратегии
fn parse_series_source(value: &StrategyParamValue) -> Option<DataSeriesSource> {
    if let Some(text) = value.as_str() {
        parse_series_source_text(text)
    } else {
        None
    }
}

/// Парсит текстовое представление источника данных
/// Поддерживает форматы: "indicator:alias", "price:close", "custom:key"
fn parse_series_source_text(value: &str) -> Option<DataSeriesSource> {
    let lower = value.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("indicator:") {
        return Some(DataSeriesSource::indicator(rest));
    }
    if let Some(rest) = lower.strip_prefix("price:") {
        let field = match rest {
            "open" => PriceField::Open,
            "high" => PriceField::High,
            "low" => PriceField::Low,
            "close" => PriceField::Close,
            "volume" => PriceField::Volume,
            _ => PriceField::Close,
        };
        return Some(DataSeriesSource::price(field));
    }
    if let Some(rest) = lower.strip_prefix("custom:") {
        return Some(DataSeriesSource::custom(rest));
    }
    Some(DataSeriesSource::indicator(value))
}
