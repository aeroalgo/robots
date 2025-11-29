use chrono::Utc;
use std::collections::{BTreeMap, HashMap};

use crate::condition::ConditionParameterPresets;
use crate::data_model::types::TimeFrame;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo};
use crate::indicators::parameters::ParameterPresets;
use crate::risk::parameters::StopParameterPresets;
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, DataSeriesSource,
    IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, RuleLogic, StopHandlerSpec,
    StrategyCategory, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyParameterSpec, StrategyRuleSpec, StrategySignalType,
    TakeHandlerSpec,
};

pub struct StrategyConverter;

impl StrategyConverter {
    pub fn candidate_to_definition(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        Self::candidate_to_definition_with_params(candidate, base_timeframe, None)
    }

    pub fn candidate_to_definition_with_params(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
        param_values: Option<&StrategyParameterMap>,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        let metadata = Self::create_metadata(candidate);
        let parameters = Self::extract_parameters_with_values(candidate, param_values);
        let defaults = Self::extract_defaults_with_values(candidate, param_values);
        let base_tf = base_timeframe.clone();

        let indicator_bindings =
            Self::create_indicator_bindings_with_params(candidate, base_tf.clone(), param_values)?;
        let condition_bindings =
            Self::create_condition_bindings_with_params(candidate, base_tf.clone(), param_values)?;
        let (mut stop_handlers, mut take_handlers) =
            Self::create_stop_and_take_handlers_with_params(candidate, base_tf.clone(), param_values)?;

        let exit_condition_bindings =
            Self::create_condition_bindings_for_exit_with_params(candidate, base_tf, param_values)?;

        let entry_rules = Self::create_entry_rules(candidate, &condition_bindings)?;
        let exit_rules = Self::create_exit_rules(candidate, &exit_condition_bindings)?;

        let entry_rule_ids: Vec<String> = entry_rules.iter().map(|r| r.id.clone()).collect();
        for stop_handler in &mut stop_handlers {
            if stop_handler.target_entry_ids.is_empty() {
                stop_handler.target_entry_ids = entry_rule_ids.clone();
            }
        }
        for take_handler in &mut take_handlers {
            if take_handler.target_entry_ids.is_empty() {
                take_handler.target_entry_ids = entry_rule_ids.clone();
            }
        }

        let mut all_condition_bindings = condition_bindings;
        all_condition_bindings.extend(exit_condition_bindings);

        Ok(StrategyDefinition::new(
            metadata,
            parameters,
            indicator_bindings,
            vec![], // formulas
            all_condition_bindings,
            entry_rules,
            exit_rules,
            stop_handlers,
            take_handlers,
            defaults,
            BTreeMap::new(), // optimizer_hints
        ))
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

    fn extract_parameters_with_values(
        candidate: &StrategyCandidate,
        param_values: Option<&StrategyParameterMap>,
    ) -> Vec<StrategyParameterSpec> {
        let mut params = Vec::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&indicator.alias, &param.name);
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let range = ParameterPresets::get_range_for_parameter(
                        &indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for {}",
                            param.name, indicator.name
                        )),
                        default_value: Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            value,
                        ),
                        min: range.as_ref().map(|r| r.start as f64),
                        max: range.as_ref().map(|r| r.end as f64),
                        step: range.as_ref().map(|r| r.step as f64),
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
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let range = ParameterPresets::get_range_for_parameter(
                        &nested.indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for nested {}",
                            param.name, nested.indicator.name
                        )),
                        default_value: Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            value,
                        ),
                        min: range.as_ref().map(|r| r.start as f64),
                        max: range.as_ref().map(|r| r.end as f64),
                        step: range.as_ref().map(|r| r.step as f64),
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
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let range = ConditionParameterPresets::get_range_for_condition(&condition.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for entry condition {}",
                            param.name, condition.name
                        )),
                        default_value: StrategyParamValue::Number(value),
                        min: range.as_ref().map(|r| r.min as f64),
                        max: range.as_ref().map(|r| r.max as f64),
                        step: range.as_ref().map(|r| r.step as f64),
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
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let range = ConditionParameterPresets::get_range_for_condition(&condition.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for exit condition {}",
                            param.name, condition.name
                        )),
                        default_value: StrategyParamValue::Number(value),
                        min: range.as_ref().map(|r| r.min as f64),
                        max: range.as_ref().map(|r| r.max as f64),
                        step: range.as_ref().map(|r| r.step as f64),
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
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let range = StopParameterPresets::get_range(&stop_handler.name, &param.name);
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for stop handler {}",
                            param.name, stop_handler.name
                        )),
                        default_value: StrategyParamValue::Number(value),
                        min: range.as_ref().map(|r| r.start as f64),
                        max: range.as_ref().map(|r| r.end as f64),
                        step: range.as_ref().map(|r| r.step as f64),
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        params
    }

    fn extract_defaults_with_values(
        candidate: &StrategyCandidate,
        param_values: Option<&StrategyParameterMap>,
    ) -> StrategyParameterMap {
        let mut defaults = HashMap::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&indicator.alias, &param.name);
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    defaults.insert(
                        param_name,
                        Self::param_value_to_strategy_param_from_enum(&param.param_type, value),
                    );
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&nested.indicator.alias, &param.name);
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    defaults.insert(
                        param_name,
                        Self::param_value_to_strategy_param_from_enum(&param.param_type, value),
                    );
                }
            }
        }

        for condition in &candidate.conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&condition.id, &param.name);
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    defaults.insert(param_name, StrategyParamValue::Number(value));
                }
            }
        }

        for condition in &candidate.exit_conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&format!("exit_{}", condition.id), &param.name);
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    defaults.insert(param_name, StrategyParamValue::Number(value));
                }
            }
        }

        for stop_handler in &candidate.stop_handlers {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&stop_handler.id, &param.name);
                    let value = param_values
                        .and_then(|pv| pv.get(&param_name))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    defaults.insert(param_name, StrategyParamValue::Number(value));
                }
            }
        }

        defaults
    }

    fn create_indicator_bindings_with_params(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
        param_values: Option<&StrategyParameterMap>,
    ) -> Result<Vec<IndicatorBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();
        let mut binding_keys = std::collections::HashSet::new();

        let mut required_timeframes = std::collections::HashSet::new();
        required_timeframes.insert(base_timeframe.clone());

        for condition in candidate
            .conditions
            .iter()
            .chain(candidate.exit_conditions.iter())
        {
            let primary_tf = condition
                .primary_timeframe
                .as_ref()
                .unwrap_or(&base_timeframe);
            required_timeframes.insert(primary_tf.clone());

            let secondary_tf = condition
                .secondary_timeframe
                .as_ref()
                .unwrap_or(&base_timeframe);
            required_timeframes.insert(secondary_tf.clone());
        }

        for indicator in &candidate.indicators {
            let params =
                Self::extract_indicator_params_with_values(indicator, param_values)?;
            for timeframe in &required_timeframes {
                let key = format!("{}:{:?}", indicator.alias, timeframe);
                if !binding_keys.contains(&key) {
                    binding_keys.insert(key.clone());
                    bindings.push(IndicatorBindingSpec {
                        alias: indicator.alias.clone(),
                        timeframe: timeframe.clone(),
                        source: IndicatorSourceSpec::Registry {
                            name: indicator.name.clone(),
                            parameters: params.clone(),
                        },
                        tags: vec!["base".to_string()],
                    });
                }
            }
        }

        for nested in &candidate.nested_indicators {
            let params =
                Self::extract_indicator_params_with_values(&nested.indicator, param_values)?;
            for timeframe in &required_timeframes {
                let key = format!("{}:{:?}", nested.indicator.alias, timeframe);
                if !binding_keys.contains(&key) {
                    binding_keys.insert(key.clone());
                    bindings.push(IndicatorBindingSpec {
                        alias: nested.indicator.alias.clone(),
                        timeframe: timeframe.clone(),
                        source: IndicatorSourceSpec::Registry {
                            name: nested.indicator.name.clone(),
                            parameters: params.clone(),
                        },
                        tags: vec!["nested".to_string(), format!("depth_{}", nested.depth)],
                    });
                }
            }
        }

        Ok(bindings)
    }

    fn extract_indicator_params_with_values(
        indicator: &IndicatorInfo,
        param_values: Option<&StrategyParameterMap>,
    ) -> Result<HashMap<String, f32>, StrategyConversionError> {
        use crate::indicators::parameters::ParameterPresets;
        let mut params = HashMap::new();

        for param in &indicator.parameters {
            let param_key = format!("{}_{}", indicator.alias, param.name);
            
            let value = if let Some(values) = param_values {
                if let Some(val) = values.get(&param_key) {
                    match val {
                        StrategyParamValue::Number(n) => *n as f32,
                        StrategyParamValue::Integer(i) => *i as f32,
                        _ => {
                            let range = ParameterPresets::get_range_for_parameter(
                                &indicator.name,
                                &param.name,
                                &param.param_type,
                            )
                            .ok_or_else(|| StrategyConversionError::MissingParameterRange {
                                indicator_name: indicator.name.clone(),
                                parameter_name: param.name.clone(),
                                parameter_type: format!("{:?}", param.param_type),
                            })?;
                            range.start
                        }
                    }
                } else {
                    let range = ParameterPresets::get_range_for_parameter(
                        &indicator.name,
                        &param.name,
                        &param.param_type,
                    )
                    .ok_or_else(|| StrategyConversionError::MissingParameterRange {
                        indicator_name: indicator.name.clone(),
                        parameter_name: param.name.clone(),
                        parameter_type: format!("{:?}", param.param_type),
                    })?;
                    range.start
                }
            } else {
                let range = ParameterPresets::get_range_for_parameter(
                    &indicator.name,
                    &param.name,
                    &param.param_type,
                )
                .ok_or_else(|| StrategyConversionError::MissingParameterRange {
                    indicator_name: indicator.name.clone(),
                    parameter_name: param.name.clone(),
                    parameter_type: format!("{:?}", param.param_type),
                })?;
                range.start
            };

            params.insert(param.name.clone(), value);
        }
        Ok(params)
    }

    fn create_condition_bindings_with_params(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
        param_values: Option<&StrategyParameterMap>,
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
                    let param_key = format!("condition_{}_{}", condition.id, param.name);
                    let value = if let Some(values) = param_values {
                        if let Some(val) = values.get(&param_key) {
                            match val {
                                StrategyParamValue::Number(n) => *n as f32,
                                StrategyParamValue::Integer(i) => *i as f32,
                                _ => condition.constant_value.unwrap_or(0.0) as f32,
                            }
                        } else {
                            condition.constant_value.unwrap_or(0.0) as f32
                        }
                    } else {
                        condition.constant_value.unwrap_or(0.0) as f32
                    };
                    parameters.insert(param.name.clone(), value);
                }
            }

            let condition_timeframe = condition
                .primary_timeframe
                .clone()
                .unwrap_or_else(|| base_timeframe.clone());

            bindings.push(ConditionBindingSpec {
                id: condition.id.clone(),
                name: condition.name.clone(),
                timeframe: condition_timeframe,
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
                let price_field = if let Some(ref pf_str) = condition.price_field {
                    Self::parse_price_field_from_string(pf_str)
                        .unwrap_or_else(|| crate::strategy::types::PriceField::Close)
                } else {
                    Self::extract_price_field_from_condition_id(&condition.id)
                        .unwrap_or_else(|| crate::strategy::types::PriceField::Close)
                };

                // Используем таймфреймы из ConditionInfo, если они указаны
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(indicator_alias, tf.clone())
                } else {
                    DataSeriesSource::indicator(indicator_alias)
                };

                let secondary_source = if let Some(ref tf) = condition.secondary_timeframe {
                    DataSeriesSource::price_with_timeframe(price_field, tf.clone())
                } else {
                    DataSeriesSource::price(price_field)
                };

                // Проверяем, есть ли процент в optimization_params для создания DualWithPercent
                let percent_param = condition
                    .optimization_params
                    .iter()
                    .find(|p| p.name == "percent" || p.name == "percentage");

                if let Some(_percent_param) = percent_param {
                    // Используем constant_value из ConditionInfo для процента, если оно есть
                    // Иначе используем значение по умолчанию 1.0%
                    let percent_value = condition.constant_value.unwrap_or(1.0) as f32;
                    Ok(ConditionInputSpec::DualWithPercent {
                        primary: primary_source,
                        secondary: secondary_source,
                        percent: percent_value,
                    })
                } else {
                    Ok(ConditionInputSpec::Dual {
                        primary: primary_source,
                        secondary: secondary_source,
                    })
                }
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

                // Используем таймфреймы из ConditionInfo, если они указаны
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(aliases[0].clone(), tf.clone())
                } else {
                    DataSeriesSource::indicator(aliases[0].clone())
                };

                let secondary_source = if let Some(ref tf) = condition.secondary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(aliases[1].clone(), tf.clone())
                } else {
                    DataSeriesSource::indicator(aliases[1].clone())
                };

                // Проверяем, есть ли процент в optimization_params для создания DualWithPercent
                let percent_param = condition
                    .optimization_params
                    .iter()
                    .find(|p| p.name == "percent" || p.name == "percentage");

                if let Some(_percent_param) = percent_param {
                    // Используем constant_value из ConditionInfo для процента, если оно есть
                    // Иначе используем значение по умолчанию 1.0%
                    let percent_value = condition.constant_value.unwrap_or(1.0) as f32;
                    Ok(ConditionInputSpec::DualWithPercent {
                        primary: primary_source,
                        secondary: secondary_source,
                        percent: percent_value,
                    })
                } else {
                    Ok(ConditionInputSpec::Dual {
                        primary: primary_source,
                        secondary: secondary_source,
                    })
                }
            }
            "trend_condition" => {
                let indicator_alias =
                    Self::extract_indicator_alias_from_condition_id(&condition.id).ok_or_else(
                        || StrategyConversionError::InvalidConditionFormat {
                            condition_id: condition.id.clone(),
                            reason: "Cannot extract indicator alias".to_string(),
                        },
                    )?;

                // Используем таймфрейм из ConditionInfo для индикатора, если он указан
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(indicator_alias, tf.clone())
                } else {
                    DataSeriesSource::indicator(indicator_alias)
                };

                // Для трендовых условий используется Single input
                Ok(ConditionInputSpec::Single {
                    source: primary_source,
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

                // Используем таймфрейм из ConditionInfo для индикатора, если он указан
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(indicator_alias, tf.clone())
                } else {
                    DataSeriesSource::indicator(indicator_alias)
                };

                Ok(ConditionInputSpec::Dual {
                    primary: primary_source,
                    secondary: DataSeriesSource::custom(format!("constant_{}", constant_value)),
                })
            }
            _ => Err(StrategyConversionError::UnsupportedConditionType {
                condition_type: condition.condition_type.clone(),
            }),
        }
    }

    fn create_stop_and_take_handlers_with_params(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
        param_values: Option<&StrategyParameterMap>,
    ) -> Result<(Vec<StopHandlerSpec>, Vec<TakeHandlerSpec>), StrategyConversionError> {
        let mut stop_handlers = Vec::new();
        let mut take_handlers = Vec::new();

        for stop_handler in &candidate.stop_handlers {
            let mut parameters = StrategyParameterMap::new();
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_key = format!("stop_{}_{}", stop_handler.name, param.name);
                    let value = if let Some(values) = param_values {
                        if let Some(val) = values.get(&param_key) {
                            val.clone()
                        } else {
                            StrategyParamValue::Number(0.0)
                        }
                    } else {
                        StrategyParamValue::Number(0.0)
                    };
                    parameters.insert(param.name.clone(), value);
                }
            }

            stop_handlers.push(StopHandlerSpec {
                id: stop_handler.id.clone(),
                name: stop_handler.name.clone(),
                handler_name: stop_handler.handler_name.clone(),
                timeframe: base_timeframe.clone(),
                price_field: crate::strategy::types::PriceField::Close,
                parameters,
                direction: PositionDirection::Long,
                priority: stop_handler.priority,
                tags: vec!["stop_loss".to_string()],
                target_entry_ids: vec![],
            });
        }

        for take_handler in &candidate.take_handlers {
            let mut parameters = StrategyParameterMap::new();
            for param in &take_handler.optimization_params {
                if param.optimizable {
                    let param_key = format!("take_{}_{}", take_handler.name, param.name);
                    let value = if let Some(values) = param_values {
                        if let Some(val) = values.get(&param_key) {
                            val.clone()
                        } else {
                            StrategyParamValue::Number(0.0)
                        }
                    } else {
                        StrategyParamValue::Number(0.0)
                    };
                    parameters.insert(param.name.clone(), value);
                }
            }

            take_handlers.push(TakeHandlerSpec {
                id: take_handler.id.clone(),
                name: take_handler.name.clone(),
                handler_name: take_handler.handler_name.clone(),
                timeframe: base_timeframe.clone(),
                price_field: crate::strategy::types::PriceField::Close,
                parameters,
                direction: PositionDirection::Long,
                priority: take_handler.priority,
                tags: vec!["take_profit".to_string()],
                target_entry_ids: vec![],
            });
        }

        Ok((stop_handlers, take_handlers))
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
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["auto-generated".to_string()],
            position_group: None,
            target_entry_ids: vec![],
        }])
    }

    fn create_condition_bindings_for_exit_with_params(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
        param_values: Option<&StrategyParameterMap>,
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
                    let param_key = format!("condition_exit_{}_{}", condition.id, param.name);
                    let value = if let Some(values) = param_values {
                        if let Some(val) = values.get(&param_key) {
                            match val {
                                StrategyParamValue::Number(n) => *n as f32,
                                StrategyParamValue::Integer(i) => *i as f32,
                                _ => condition.constant_value.unwrap_or(0.0) as f32,
                            }
                        } else {
                            condition.constant_value.unwrap_or(0.0) as f32
                        }
                    } else {
                        condition.constant_value.unwrap_or(0.0) as f32
                    };
                    parameters.insert(param.name.clone(), value);
                }
            }

            let condition_timeframe = condition
                .primary_timeframe
                .clone()
                .unwrap_or_else(|| base_timeframe.clone());

            bindings.push(ConditionBindingSpec {
                id: format!("exit_{}", condition.id),
                name: format!("Exit: {}", condition.name),
                timeframe: condition_timeframe,
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
                direction: PositionDirection::Long,
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

    fn make_parameter_name(prefix: &str, param_name: &str) -> String {
        format!("{}_{}", prefix, param_name)
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
        let rest = if condition_id.starts_with("entry_") {
            condition_id.strip_prefix("entry_")?
        } else if condition_id.starts_with("exit_") {
            condition_id.strip_prefix("exit_")?
        } else if condition_id.starts_with("ind_price_") {
            condition_id.strip_prefix("ind_price_")?
        } else if condition_id.starts_with("ind_const_") {
            condition_id.strip_prefix("ind_const_")?
        } else {
            return None;
        };

        if let Some(separator_pos) = rest.find("::") {
            return Some(rest[..separator_pos].to_string());
        }
        
        if let Some(last_underscore) = rest.rfind('_') {
            return Some(rest[..last_underscore].to_string());
        }
        
        None
    }

    fn extract_indicator_aliases_from_condition_id(condition_id: &str) -> Option<Vec<String>> {
        if condition_id.starts_with("entry_") || condition_id.starts_with("exit_") {
            let rest = if condition_id.starts_with("entry_") {
                condition_id.strip_prefix("entry_")?
            } else {
                condition_id.strip_prefix("exit_")?
            };
            
            if let Some(separator_pos) = rest.find("::") {
                let alias1 = &rest[..separator_pos];
                let after_separator = &rest[separator_pos + 2..];
                if let Some(last_underscore) = after_separator.rfind('_') {
                    let alias2 = &after_separator[..last_underscore];
                    return Some(vec![alias1.to_string(), alias2.to_string()]);
                }
            }
        } else if condition_id.starts_with("ind_ind_") {
            let rest = condition_id.strip_prefix("ind_ind_")?;
            if let Some(separator_pos) = rest.find("::") {
                let alias1 = &rest[..separator_pos];
                let after_separator = &rest[separator_pos + 2..];
                if let Some(last_underscore) = after_separator.rfind('_') {
                    let alias2 = &after_separator[..last_underscore];
                    return Some(vec![alias1.to_string(), alias2.to_string()]);
                }
            }
        }
        None
    }

    fn parse_price_field_from_string(
        price_field_str: &str,
    ) -> Option<crate::strategy::types::PriceField> {
        match price_field_str {
            "Open" => Some(crate::strategy::types::PriceField::Open),
            "High" => Some(crate::strategy::types::PriceField::High),
            "Low" => Some(crate::strategy::types::PriceField::Low),
            "Close" => Some(crate::strategy::types::PriceField::Close),
            "Volume" => Some(crate::strategy::types::PriceField::Volume),
            _ => None,
        }
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
    #[error("Missing parameter range for indicator {indicator_name}, parameter {parameter_name} (type: {parameter_type})")]
    MissingParameterRange {
        indicator_name: String,
        parameter_name: String,
        parameter_type: String,
    },
}
