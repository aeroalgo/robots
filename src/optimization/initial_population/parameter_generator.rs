use crate::condition::parameters::ConditionParameterPresets;
use crate::discovery::StrategyCandidate;
use crate::indicators::parameters::ParameterPresets;
use crate::indicators::registry::IndicatorRegistry;
use crate::indicators::types::{IndicatorCategory, ParameterType};
use crate::optimization::condition_id::ConditionId;
use crate::optimization::candidate_builder_config::CandidateBuilderConfig;
use crate::risk::get_stop_optimization_range;
use crate::risk::utils::stop_handler_requires_indicator;
use crate::strategy::types::StrategyParameterMap;
use crate::strategy::types::StrategyParamValue;
use rand::Rng;
use std::collections::HashMap;

pub fn generate_random_parameters(
    candidate: &StrategyCandidate,
    candidate_builder_config: &CandidateBuilderConfig,
) -> StrategyParameterMap {
    let mut rng = rand::thread_rng();
    let total_params: usize = candidate
        .indicators
        .iter()
        .map(|i| i.parameters.len())
        .sum();
    let mut params = HashMap::with_capacity(total_params);

    for indicator in &candidate.indicators {
        for param in &indicator.parameters {
            if param.optimizable {
                let param_type_str = format!("{:?}", param.param_type);
                let param_value = if param_type_str.contains("Boolean") {
                    StrategyParamValue::Flag(rng.gen())
                } else {
                    let range = if should_apply_volatility_constraint(
                        indicator,
                        &candidate.conditions,
                        &candidate.exit_conditions,
                        candidate_builder_config,
                    ) {
                        get_volatility_percentage_range(candidate_builder_config, indicator)
                    } else {
                        ParameterPresets::get_optimization_range(
                            &indicator.name,
                            &param.name,
                            &param.param_type,
                        )
                    };

                    if let Some(range) = range {
                        let steps = ((range.end - range.start) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.start + (step_index as f32 * range.step);
                        if param_type_str.contains("Integer") {
                            StrategyParamValue::Integer(value as i64)
                        } else {
                            StrategyParamValue::Number(value as f64)
                        }
                    } else {
                        continue;
                    }
                };
                let param_key = format!("{}_{}", indicator.alias, param.name);
                params.insert(param_key, param_value);
            }
        }
    }

    for nested in &candidate.nested_indicators {
        for param in &nested.indicator.parameters {
            if param.optimizable {
                let param_type_str = format!("{:?}", param.param_type);
                let param_value = if param_type_str.contains("Boolean") {
                    StrategyParamValue::Flag(rng.gen())
                } else {
                    if let Some(range) = ParameterPresets::get_optimization_range(
                        &nested.indicator.name,
                        &param.name,
                        &param.param_type,
                    ) {
                        let steps = ((range.end - range.start) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.start + (step_index as f32 * range.step);
                        if param_type_str.contains("Integer") {
                            StrategyParamValue::Integer(value as i64)
                        } else {
                            StrategyParamValue::Number(value as f64)
                        }
                    } else {
                        continue;
                    }
                };
                let param_key = format!("{}_{}", nested.indicator.alias, param.name);
                params.insert(param_key, param_value);
            }
        }
    }

    for stop_handler in &candidate.stop_handlers {
        let requires_indicator = stop_handler_requires_indicator(&stop_handler.handler_name);

        if requires_indicator {
            let category = if stop_handler.handler_name.to_uppercase().contains("TRAIL")
                || stop_handler.handler_name.to_uppercase().contains("ATR")
            {
                "trend"
            } else {
                "trend"
            };

            let registry = IndicatorRegistry::new();
            let category_enum = match category {
                "trend" => IndicatorCategory::Trend,
                "oscillator" => IndicatorCategory::Oscillator,
                "volatility" => IndicatorCategory::Volatility,
                "volume" => IndicatorCategory::Volume,
                _ => IndicatorCategory::Trend,
            };

            let indicators = registry.get_indicators_by_category(&category_enum);
            let indicator_names: Vec<String> = indicators
                .iter()
                .map(|ind| ind.name().to_string())
                .collect();

            if !indicator_names.is_empty() {
                let selected_indicator =
                    indicator_names[rng.gen_range(0..indicator_names.len())].clone();

                let indicator_name_key =
                    ConditionId::stop_handler_parameter_name(&stop_handler.id, "indicator_name");
                params.insert(
                    indicator_name_key,
                    StrategyParamValue::Text(selected_indicator.clone()),
                );

                let indicator_period_key =
                    ConditionId::stop_handler_parameter_name(&stop_handler.id, "indicator_period");

                if let Some(range) =
                    ParameterPresets::get_optimization_range(&selected_indicator, "period", &ParameterType::Period)
                {
                    let steps = ((range.end - range.start) / range.step) as usize;
                    let step_index = rng.gen_range(0..=steps);
                    let value = range.start + (step_index as f32 * range.step);
                    params.insert(
                        indicator_period_key,
                        StrategyParamValue::Number(value as f64),
                    );
                } else {
                    params.insert(indicator_period_key, StrategyParamValue::Number(20.0));
                }
            }
        }

        for param in &stop_handler.optimization_params {
            if param.optimizable
                && param.name != "indicator_name"
                && param.name != "indicator_period"
            {
                if let Some(range) =
                    get_stop_optimization_range(&stop_handler.handler_name, &param.name)
                {
                    let steps = ((range.end - range.start) / range.step) as usize;
                    let step_index = rng.gen_range(0..=steps);
                    let value = range.start + (step_index as f32 * range.step);
                    let param_key =
                        ConditionId::stop_handler_parameter_name(&stop_handler.id, &param.name);
                    params.insert(param_key, StrategyParamValue::Number(value as f64));
                }
            }
        }
    }

    for take_handler in &candidate.take_handlers {
        for param in &take_handler.optimization_params {
            if param.optimizable {
                if let Some(range) =
                    get_stop_optimization_range(&take_handler.handler_name, &param.name)
                {
                    let steps = ((range.end - range.start) / range.step) as usize;
                    let step_index = rng.gen_range(0..=steps);
                    let value = range.start + (step_index as f32 * range.step);
                    params.insert(
                        ConditionId::take_handler_parameter_name(&take_handler.id, &param.name),
                        StrategyParamValue::Number(value as f64),
                    );
                }
            }
        }
    }

    for condition in candidate
        .conditions
        .iter()
        .chain(candidate.exit_conditions.iter())
    {
        let indicator_name = extract_indicator_name_from_condition(candidate, condition);

        for param in &condition.optimization_params {
            if param.optimizable {
                let param_value = if param.name.to_lowercase() == "threshold"
                    && condition.constant_value.is_some()
                {
                    StrategyParamValue::Number(condition.constant_value.unwrap())
                } else if param.name.to_lowercase() == "percentage"
                    || param.name.to_lowercase() == "percent"
                {
                    let condition_name = condition.operator.factory_name();
                    if let Some(range) =
                        ConditionParameterPresets::get_range_for_condition(condition_name)
                    {
                        let steps = ((range.max - range.min) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.min + (step_index as f32 * range.step);
                        StrategyParamValue::Number(value as f64)
                    } else if condition.condition_type == "indicator_constant"
                        && indicator_name.is_some()
                    {
                        if let Some(ind_name) = &indicator_name {
                            if let Some(indicator) = candidate.indicators.iter().find(|i| {
                                i.name == *ind_name && i.indicator_type == "volatility"
                            }) {
                                if let Some(range) =
                                    get_volatility_percentage_range(candidate_builder_config, indicator)
                                {
                                    let steps = ((range.end - range.start) / range.step) as usize;
                                    let step_index = rng.gen_range(0..=steps);
                                    let value = range.start + (step_index as f32 * range.step);
                                    StrategyParamValue::Number(value as f64)
                                } else {
                                    StrategyParamValue::Number(
                                        condition.constant_value.unwrap_or(2.0),
                                    )
                                }
                            } else {
                                if let Some(range) = ParameterPresets::get_optimization_range(
                                    ind_name,
                                    &param.name,
                                    &ParameterType::Multiplier,
                                ) {
                                    let steps = ((range.end - range.start) / range.step) as usize;
                                    let step_index = rng.gen_range(0..=steps);
                                    let value = range.start + (step_index as f32 * range.step);
                                    StrategyParamValue::Number(value as f64)
                                } else {
                                    continue;
                                }
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else if param.name.to_lowercase() == "percentage"
                    && condition.condition_type == "indicator_constant"
                    && indicator_name.is_some()
                {
                    if let Some(ind_name) = &indicator_name {
                        if let Some(indicator) = candidate
                            .indicators
                            .iter()
                            .find(|i| i.name == *ind_name && i.indicator_type == "volatility")
                        {
                            if let Some(range) =
                                get_volatility_percentage_range(candidate_builder_config, indicator)
                            {
                                let steps = ((range.end - range.start) / range.step) as usize;
                                let step_index = rng.gen_range(0..=steps);
                                let value = range.start + (step_index as f32 * range.step);
                                StrategyParamValue::Number(value as f64)
                            } else {
                                StrategyParamValue::Number(condition.constant_value.unwrap_or(2.0))
                            }
                        } else {
                            if let Some(range) = ParameterPresets::get_optimization_range(
                                ind_name,
                                &param.name,
                                &ParameterType::Multiplier,
                            ) {
                                let steps = ((range.end - range.start) / range.step) as usize;
                                let step_index = rng.gen_range(0..=steps);
                                let value = range.start + (step_index as f32 * range.step);
                                StrategyParamValue::Number(value as f64)
                            } else {
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                } else if param.name.to_lowercase() == "period" {
                    let condition_name = condition.operator.factory_name();
                    if let Some(range) =
                        ConditionParameterPresets::get_range_for_condition(condition_name)
                    {
                        let steps = ((range.max - range.min) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.min + (step_index as f32 * range.step);
                        StrategyParamValue::Number(value as f64)
                    } else {
                        continue;
                    }
                } else if let Some(ind_name) = &indicator_name {
                    let param_type = match param.name.to_lowercase().as_str() {
                        "threshold" => ParameterType::Threshold,
                        "percentage" | "percent" => ParameterType::Multiplier,
                        _ => ParameterType::Threshold,
                    };

                    if let Some(range) =
                        ParameterPresets::get_optimization_range(ind_name, &param.name, &param_type)
                    {
                        let steps = ((range.end - range.start) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.start + (step_index as f32 * range.step);
                        StrategyParamValue::Number(value as f64)
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };
                params.insert(
                    ConditionId::parameter_name(&condition.id, &param.name),
                    param_value,
                );
            }
        }
    }

    params
}

pub fn extract_indicator_name_from_condition(
    candidate: &StrategyCandidate,
    condition: &crate::discovery::ConditionInfo,
) -> Option<String> {
    let alias = &condition.primary_indicator_alias;

    if let Some(ind) = candidate.indicators.iter().find(|i| i.alias == *alias) {
        return Some(ind.name.clone());
    }

    if let Some(nested) = candidate
        .nested_indicators
        .iter()
        .find(|n| n.indicator.alias == *alias)
    {
        return Some(nested.indicator.name.clone());
    }

    None
}

pub fn should_apply_volatility_constraint(
    indicator: &crate::discovery::IndicatorInfo,
    conditions: &[crate::discovery::ConditionInfo],
    exit_conditions: &[crate::discovery::ConditionInfo],
    config: &CandidateBuilderConfig,
) -> bool {
    if indicator.indicator_type != "volatility" {
        return false;
    }

    let rules = &config.rules.indicator_parameter_rules;
    for rule in rules {
        if rule.indicator_type == "volatility" {
            if !rule.indicator_names.is_empty() {
                if !rule.indicator_names.contains(&indicator.name) {
                    continue;
                }
            }

            if let Some(constraint) = &rule.price_field_constraint {
                if constraint.required_price_field == "Close" {
                    for condition in conditions.iter().chain(exit_conditions.iter()) {
                        if let Some(price_field) = &condition.price_field {
                            if price_field == "Close" {
                                if condition.primary_indicator_alias == indicator.alias {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn get_volatility_percentage_range(
    config: &CandidateBuilderConfig,
    indicator: &crate::discovery::IndicatorInfo,
) -> Option<crate::indicators::types::ParameterRange> {
    let rules = &config.rules.indicator_parameter_rules;
    for rule in rules {
        if rule.indicator_type == "volatility" {
            if !rule.indicator_names.is_empty() {
                if !rule.indicator_names.contains(&indicator.name) {
                    continue;
                }
            }

            if let Some(constraint) = &rule.price_field_constraint {
                if let super::super::candidate_builder_config::ParameterConstraint::PercentageFromPrice {
                    min_percent,
                    max_percent,
                    step,
                } = &constraint.parameter_constraint
                {
                    return Some(crate::indicators::types::ParameterRange::new(
                        *min_percent as f32,
                        *max_percent as f32,
                        *step as f32,
                    ));
                }
            }
        }
    }
    None
}
