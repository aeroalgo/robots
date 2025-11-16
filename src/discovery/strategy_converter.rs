use chrono::Utc;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::data_model::types::TimeFrame;
use crate::discovery::config::StrategyDiscoveryConfig;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerInfo};
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperandSpec,
    ConditionOperator, DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec,
    PositionDirection, RuleLogic, StopHandlerSpec, StrategyCategory, StrategyDefinition,
    StrategyMetadata, StrategyParamValue, StrategyParameterMap, StrategyParameterSpec,
    StrategyRuleSpec, StrategySignalType, TimeframeRequirement,
};

pub struct StrategyConverter;

impl StrategyConverter {
    pub fn candidate_to_definition(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        let metadata = Self::create_metadata(candidate);
        let parameters = Self::extract_parameters(candidate);
        let defaults = Self::extract_defaults(candidate);

        let indicator_bindings =
            Self::create_indicator_bindings(candidate, base_timeframe.clone())?;
        let condition_bindings =
            Self::create_condition_bindings(candidate, base_timeframe.clone())?;
        let stop_handlers = Self::create_stop_handlers(candidate, base_timeframe.clone())?;

        let exit_condition_bindings =
            Self::create_condition_bindings_for_exit(candidate, base_timeframe.clone())?;

        let entry_rules = Self::create_entry_rules(candidate, &condition_bindings)?;
        let exit_rules = Self::create_exit_rules(candidate, &exit_condition_bindings)?;

        let timeframe_requirements = Self::create_timeframe_requirements(candidate);

        Ok(StrategyDefinition {
            metadata,
            parameters,
            indicator_bindings,
            formulas: vec![],
            condition_bindings,
            entry_rules,
            exit_rules,
            stop_handlers,
            timeframe_requirements,
            defaults,
            optimizer_hints: BTreeMap::new(),
        })
    }

    fn create_metadata(candidate: &StrategyCandidate) -> StrategyMetadata {
        let indicator_names: Vec<String> = candidate
            .indicators
            .iter()
            .map(|ind| ind.name.clone())
            .collect();
        let nested_names: Vec<String> = candidate
            .nested_indicators
            .iter()
            .map(|nested| nested.indicator.name.clone())
            .collect();
        let all_names = [indicator_names, nested_names].concat();
        let name = format!("Auto Strategy: {}", all_names.join(" + "));

        let condition_names: Vec<String> = candidate
            .conditions
            .iter()
            .map(|cond| cond.name.clone())
            .collect();
        let description = Some(format!(
            "Автоматически сгенерированная стратегия. Индикаторы: {}. Условия: {}.",
            all_names.join(", "),
            condition_names.join(", ")
        ));

        StrategyMetadata {
            id: format!("auto_strategy_{}", Utc::now().timestamp()),
            name,
            description,
            version: Some("1.0.0".to_string()),
            author: Some("Strategy Discovery Engine".to_string()),
            categories: vec![StrategyCategory::Custom("Auto Generated".to_string())],
            tags: vec!["auto-generated".to_string(), "discovery".to_string()],
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    fn extract_parameters(candidate: &StrategyCandidate) -> Vec<StrategyParameterSpec> {
        let mut params = Vec::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&indicator.alias, &param.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for {}",
                            param.name, indicator.name
                        )),
                        default_value: Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            0.0,
                        ),
                        min: None,
                        max: None,
                        step: None,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&nested.indicator.alias, &param.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for nested {}",
                            param.name, nested.indicator.name
                        )),
                        default_value: Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            0.0,
                        ),
                        min: None,
                        max: None,
                        step: None,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for condition in &candidate.conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&condition.id, &param.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for entry condition {}",
                            param.name, condition.name
                        )),
                        default_value: StrategyParamValue::Number(0.0),
                        min: None,
                        max: None,
                        step: None,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for condition in &candidate.exit_conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&format!("exit_{}", condition.id), &param.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for exit condition {}",
                            param.name, condition.name
                        )),
                        default_value: StrategyParamValue::Number(0.0),
                        min: None,
                        max: None,
                        step: None,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for stop_handler in &candidate.stop_handlers {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&stop_handler.id, &param.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for stop handler {}",
                            param.name, stop_handler.name
                        )),
                        default_value: StrategyParamValue::Number(0.0),
                        min: None,
                        max: None,
                        step: None,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        params
    }

    fn extract_defaults(candidate: &StrategyCandidate) -> StrategyParameterMap {
        let mut defaults = HashMap::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&indicator.alias, &param.name);
                    defaults.insert(
                        param_name,
                        Self::param_value_to_strategy_param_from_enum(&param.param_type, 0.0),
                    );
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&nested.indicator.alias, &param.name);
                    defaults.insert(
                        param_name,
                        Self::param_value_to_strategy_param_from_enum(&param.param_type, 0.0),
                    );
                }
            }
        }

        for condition in &candidate.conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&condition.id, &param.name);
                    defaults.insert(param_name, StrategyParamValue::Number(0.0));
                }
            }
        }

        for condition in &candidate.exit_conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&format!("exit_{}", condition.id), &param.name);
                    defaults.insert(param_name, StrategyParamValue::Number(0.0));
                }
            }
        }

        for stop_handler in &candidate.stop_handlers {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&stop_handler.id, &param.name);
                    defaults.insert(param_name, StrategyParamValue::Number(0.0));
                }
            }
        }

        defaults
    }

    fn create_indicator_bindings(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<IndicatorBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();

        for indicator in &candidate.indicators {
            let params = Self::extract_indicator_params(indicator);
            bindings.push(IndicatorBindingSpec {
                alias: indicator.alias.clone(),
                timeframe: base_timeframe.clone(),
                source: IndicatorSourceSpec::Registry {
                    name: indicator.name.clone(),
                    parameters: params,
                },
                tags: vec!["base".to_string()],
            });
        }

        for nested in &candidate.nested_indicators {
            let params = Self::extract_indicator_params(&nested.indicator);
            bindings.push(IndicatorBindingSpec {
                alias: nested.indicator.alias.clone(),
                timeframe: base_timeframe.clone(),
                source: IndicatorSourceSpec::Registry {
                    name: nested.indicator.name.clone(),
                    parameters: params,
                },
                tags: vec!["nested".to_string(), format!("depth_{}", nested.depth)],
            });
        }

        Ok(bindings)
    }

    fn extract_indicator_params(indicator: &IndicatorInfo) -> HashMap<String, f32> {
        let mut params = HashMap::new();
        for param in &indicator.parameters {
            let default_value = match param.param_type {
                crate::indicators::types::ParameterType::Period => 20.0,
                crate::indicators::types::ParameterType::Multiplier => 2.0,
                crate::indicators::types::ParameterType::Threshold => 0.5,
                crate::indicators::types::ParameterType::Coefficient => 1.0,
                crate::indicators::types::ParameterType::Custom => 0.0,
            };
            params.insert(param.name.clone(), default_value);
        }
        params
    }

    fn create_condition_bindings(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<ConditionBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();

        for condition in &candidate.conditions {
            let input = Self::create_condition_input(condition, candidate)?;
            let declarative = ConditionDeclarativeSpec {
                operator: condition.operator.clone(),
                operands: vec![],
                description: Some(condition.name.clone()),
            };

            let mut parameters = HashMap::new();
            for param in &condition.optimization_params {
                if param.optimizable {
                    let value = condition.constant_value.unwrap_or(0.0) as f32;
                    parameters.insert(param.name.clone(), value);
                }
            }

            bindings.push(ConditionBindingSpec {
                id: condition.id.clone(),
                name: condition.name.clone(),
                timeframe: base_timeframe.clone(),
                declarative,
                parameters,
                input,
                weight: 1.0,
                tags: vec![condition.condition_type.clone()],
                user_formula: None,
            });
        }

        Ok(bindings)
    }

    fn create_condition_input(
        condition: &ConditionInfo,
        candidate: &StrategyCandidate,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        match condition.condition_type.as_str() {
            "indicator_price" => {
                let indicator_alias =
                    Self::extract_indicator_alias_from_condition_id(&condition.id).ok_or_else(
                        || StrategyConversionError::InvalidConditionFormat {
                            condition_id: condition.id.clone(),
                            reason: "Cannot extract indicator alias".to_string(),
                        },
                    )?;
                let price_field = Self::extract_price_field_from_condition_id(&condition.id)
                    .unwrap_or_else(|| crate::strategy::types::PriceField::Close);

                Ok(ConditionInputSpec::Dual {
                    primary: DataSeriesSource::indicator(indicator_alias),
                    secondary: DataSeriesSource::price(price_field),
                })
            }
            "indicator_indicator" => {
                let aliases = Self::extract_indicator_aliases_from_condition_id(&condition.id)
                    .ok_or_else(|| StrategyConversionError::InvalidConditionFormat {
                        condition_id: condition.id.clone(),
                        reason: "Cannot extract indicator aliases".to_string(),
                    })?;
                if aliases.len() < 2 {
                    return Err(StrategyConversionError::InvalidConditionFormat {
                        condition_id: condition.id.clone(),
                        reason: "Need at least 2 indicators for indicator_indicator condition"
                            .to_string(),
                    });
                }

                Ok(ConditionInputSpec::Dual {
                    primary: DataSeriesSource::indicator(aliases[0].clone()),
                    secondary: DataSeriesSource::indicator(aliases[1].clone()),
                })
            }
            "indicator_constant" => {
                let indicator_alias =
                    Self::extract_indicator_alias_from_condition_id(&condition.id).ok_or_else(
                        || StrategyConversionError::InvalidConditionFormat {
                            condition_id: condition.id.clone(),
                            reason: "Cannot extract indicator alias".to_string(),
                        },
                    )?;
                let constant_value = condition.constant_value.unwrap_or(0.0) as f32;

                Ok(ConditionInputSpec::Dual {
                    primary: DataSeriesSource::indicator(indicator_alias),
                    secondary: DataSeriesSource::custom(format!("constant_{}", constant_value)),
                })
            }
            _ => Err(StrategyConversionError::UnsupportedConditionType {
                condition_type: condition.condition_type.clone(),
            }),
        }
    }

    fn create_stop_handlers(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<StopHandlerSpec>, StrategyConversionError> {
        let mut handlers = Vec::new();

        for stop_handler in &candidate.stop_handlers {
            let mut parameters = StrategyParameterMap::new();
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    parameters.insert(param.name.clone(), StrategyParamValue::Number(0.0));
                }
            }

            handlers.push(StopHandlerSpec {
                id: stop_handler.id.clone(),
                name: stop_handler.name.clone(),
                handler_name: stop_handler.handler_name.clone(),
                timeframe: base_timeframe.clone(),
                price_field: crate::strategy::types::PriceField::Close,
                parameters,
                direction: PositionDirection::Both,
                priority: stop_handler.priority,
                tags: vec![stop_handler.stop_type.clone()],
                target_entry_ids: vec![],
            });
        }

        Ok(handlers)
    }

    fn create_entry_rules(
        candidate: &StrategyCandidate,
        condition_bindings: &[ConditionBindingSpec],
    ) -> Result<Vec<StrategyRuleSpec>, StrategyConversionError> {
        if condition_bindings.is_empty() {
            return Ok(vec![]);
        }

        let condition_ids: Vec<String> = condition_bindings.iter().map(|c| c.id.clone()).collect();

        Ok(vec![StrategyRuleSpec {
            id: "entry_rule_1".to_string(),
            name: "Entry Rule".to_string(),
            logic: RuleLogic::All,
            conditions: condition_ids,
            signal: StrategySignalType::Entry,
            direction: PositionDirection::Both,
            quantity: None,
            tags: vec!["auto-generated".to_string()],
            position_group: None,
            target_entry_ids: vec![],
        }])
    }

    fn create_condition_bindings_for_exit(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<ConditionBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();

        for condition in &candidate.exit_conditions {
            let input = Self::create_condition_input(condition, candidate)?;
            let declarative = ConditionDeclarativeSpec {
                operator: condition.operator.clone(),
                operands: vec![],
                description: Some(condition.name.clone()),
            };

            let mut parameters = HashMap::new();
            for param in &condition.optimization_params {
                if param.optimizable {
                    let value = condition.constant_value.unwrap_or(0.0) as f32;
                    parameters.insert(param.name.clone(), value);
                }
            }

            bindings.push(ConditionBindingSpec {
                id: format!("exit_{}", condition.id),
                name: format!("Exit: {}", condition.name),
                timeframe: base_timeframe.clone(),
                declarative,
                parameters,
                input,
                weight: 1.0,
                tags: vec![condition.condition_type.clone(), "exit".to_string()],
                user_formula: None,
            });
        }

        Ok(bindings)
    }

    fn create_exit_rules(
        candidate: &StrategyCandidate,
        exit_condition_bindings: &[ConditionBindingSpec],
    ) -> Result<Vec<StrategyRuleSpec>, StrategyConversionError> {
        let mut exit_rules = Vec::new();

        // Создаем exit rule из exit conditions, если они есть
        if !exit_condition_bindings.is_empty() {
            let condition_ids: Vec<String> = exit_condition_bindings
                .iter()
                .map(|c| c.id.clone())
                .collect();
            exit_rules.push(StrategyRuleSpec {
                id: "exit_rule_1".to_string(),
                name: "Exit Rule".to_string(),
                logic: RuleLogic::All,
                conditions: condition_ids,
                signal: StrategySignalType::Exit,
                direction: PositionDirection::Both,
                quantity: None,
                tags: vec!["auto-generated".to_string(), "exit-conditions".to_string()],
                position_group: None,
                target_entry_ids: vec![],
            });
        }

        // Exit rules также могут быть созданы из stop handlers
        // Но stop handlers обрабатываются отдельно через StopHandlerSpec
        // Здесь мы создаем exit rules только из условий

        Ok(exit_rules)
    }

    fn create_timeframe_requirements(candidate: &StrategyCandidate) -> Vec<TimeframeRequirement> {
        let mut timeframes = HashSet::new();
        for tf in &candidate.timeframes {
            timeframes.insert(tf.clone());
        }

        timeframes
            .into_iter()
            .enumerate()
            .map(|(idx, tf)| TimeframeRequirement {
                alias: format!("tf_{}", idx),
                timeframe: tf,
            })
            .collect()
    }

    fn make_parameter_name(prefix: &str, param_name: &str) -> String {
        format!("{}__{}", prefix, param_name)
    }

    fn param_value_to_strategy_param_from_enum(
        param_type: &crate::indicators::types::ParameterType,
        default: f64,
    ) -> StrategyParamValue {
        match param_type {
            crate::indicators::types::ParameterType::Period => {
                StrategyParamValue::Integer(default as i64)
            }
            crate::indicators::types::ParameterType::Multiplier => {
                StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Threshold => {
                StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Coefficient => {
                StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Custom => StrategyParamValue::Number(default),
        }
    }

    fn extract_indicator_alias_from_condition_id(condition_id: &str) -> Option<String> {
        if condition_id.starts_with("ind_price_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 3 {
                return Some(parts[2].to_string());
            }
        } else if condition_id.starts_with("ind_const_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 3 {
                return Some(parts[2].to_string());
            }
        }
        None
    }

    fn extract_indicator_aliases_from_condition_id(condition_id: &str) -> Option<Vec<String>> {
        if condition_id.starts_with("ind_ind_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 4 {
                return Some(vec![parts[2].to_string(), parts[3].to_string()]);
            }
        }
        None
    }

    fn extract_price_field_from_condition_id(
        condition_id: &str,
    ) -> Option<crate::strategy::types::PriceField> {
        if condition_id.starts_with("ind_price_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 4 {
                match parts[3] {
                    "Open" => Some(crate::strategy::types::PriceField::Open),
                    "High" => Some(crate::strategy::types::PriceField::High),
                    "Low" => Some(crate::strategy::types::PriceField::Low),
                    "Close" => Some(crate::strategy::types::PriceField::Close),
                    "Volume" => Some(crate::strategy::types::PriceField::Volume),
                    _ => Some(crate::strategy::types::PriceField::Close),
                }
            } else {
                Some(crate::strategy::types::PriceField::Close)
            }
        } else {
            Some(crate::strategy::types::PriceField::Close)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StrategyConversionError {
    #[error("Invalid condition format: {condition_id} - {reason}")]
    InvalidConditionFormat {
        condition_id: String,
        reason: String,
    },
    #[error("Unsupported condition type: {condition_type}")]
    UnsupportedConditionType { condition_type: String },
}
