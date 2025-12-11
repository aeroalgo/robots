use std::collections::HashMap;

use crate::discovery::types::StopHandlerInfo;
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::StrategyParameterSpec;

use super::helpers::get_condition_param_range;

pub fn extract_stop_handler_parameters(
    stop_handlers: &[StopHandlerInfo],
) -> Vec<StrategyParameterSpec> {
    use crate::risk::utils::stop_handler_requires_indicator;

    let mut params = Vec::with_capacity(stop_handlers.len() * 3);

    for stop_handler in stop_handlers {
        let requires_indicator = stop_handler_requires_indicator(&stop_handler.handler_name);

        if requires_indicator {
            let category = determine_indicator_category_for_stop(&stop_handler.handler_name);
            let compatible_indicators = get_compatible_indicators(category);

            let indicator_name_key =
                ConditionId::stop_handler_parameter_name(&stop_handler.id, "indicator_name");

            let default_indicator = compatible_indicators
                .first()
                .cloned()
                .unwrap_or_else(|| "SMA".to_string());

            let discrete_values: Vec<crate::strategy::types::StrategyParamValue> =
                compatible_indicators
                    .iter()
                    .map(|ind| crate::strategy::types::StrategyParamValue::Text(ind.clone()))
                    .collect();

            let indicator_name_key_clone = indicator_name_key.clone();
            params.push(StrategyParameterSpec::new_indicator_name(
                indicator_name_key_clone.clone(),
                Some(format!(
                    "indicator_name parameter for stop handler {}",
                    stop_handler.name
                )),
                crate::strategy::types::StrategyParamValue::Text(default_indicator),
                category.to_string(),
                discrete_values,
                true,
                true,
            ));

            let indicator_period_key =
                ConditionId::stop_handler_parameter_name(&stop_handler.id, "indicator_period");

            params.push(StrategyParameterSpec::new_indicator_parameter(
                indicator_period_key,
                Some(format!(
                    "indicator_period parameter for stop handler {}",
                    stop_handler.name
                )),
                crate::strategy::types::StrategyParamValue::Number(20.0),
                indicator_name_key_clone,
                Some(10.0),
                Some(200.0),
                Some(10.0),
                true,
                true,
            ));
        }

        let temp_params = get_default_stop_params(&stop_handler.handler_name);
        if let Ok(temp_handler) = crate::risk::factory::StopHandlerFactory::create(
            &stop_handler.handler_name,
            &temp_params,
        ) {
            let handler_params = temp_handler.parameters();
            for (param_name, param_value) in handler_params.get_current_values() {
                if param_name == "indicator_name" || param_name == "indicator_period" {
                    continue;
                }

                if let Some(param_info) = handler_params.get_parameter(&param_name) {
                    let param_key =
                        ConditionId::stop_handler_parameter_name(&stop_handler.id, &param_name);
                    params.push(StrategyParameterSpec::new_numeric(
                        param_key,
                        Some(format!(
                            "{} parameter for stop handler {}",
                            param_name, stop_handler.name
                        )),
                        crate::strategy::types::StrategyParamValue::Number(param_value as f64),
                        Some(param_info.range.start as f64),
                        Some(param_info.range.end as f64),
                        Some(param_info.range.step as f64),
                        true,
                        true,
                    ));
                }
            }
        } else {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    params.extend(extract_handler_params_from_info(
                        stop_handler,
                        &param.name,
                        "stop",
                    ));
                }
            }
        }
    }

    params
}

pub fn extract_take_handler_parameters(
    take_handlers: &[StopHandlerInfo],
) -> Vec<StrategyParameterSpec> {
    let mut params = Vec::with_capacity(take_handlers.len() * 3);

    for take_handler in take_handlers {
        let temp_params = get_default_take_params(&take_handler.handler_name);
        if let Ok(temp_handler) = crate::risk::factory::TakeHandlerFactory::create(
            &take_handler.handler_name,
            &temp_params,
        ) {
            let handler_params = temp_handler.parameters();
            for (param_name, param_value) in handler_params.get_current_values() {
                if let Some(param_info) = handler_params.get_parameter(&param_name) {
                    let param_key =
                        ConditionId::take_handler_parameter_name(&take_handler.id, &param_name);
                    params.push(StrategyParameterSpec::new_numeric(
                        param_key,
                        Some(format!(
                            "{} parameter for take handler {}",
                            param_name, take_handler.name
                        )),
                        crate::strategy::types::StrategyParamValue::Number(param_value as f64),
                        Some(param_info.range.start as f64),
                        Some(param_info.range.end as f64),
                        Some(param_info.range.step as f64),
                        true,
                        true,
                    ));
                }
            }
        } else {
            for param in &take_handler.optimization_params {
                if param.optimizable {
                    params.extend(extract_handler_params_from_info(
                        take_handler,
                        &param.name,
                        "take",
                    ));
                }
            }
        }
    }

    params
}

fn extract_handler_params_from_info(
    handler: &StopHandlerInfo,
    param_name: &str,
    handler_type: &str,
) -> Vec<StrategyParameterSpec> {
    let range_opt = crate::risk::get_stop_optimization_range(&handler.handler_name, param_name);
    let (default_val, min_val, max_val, step_val) = if let Some(range) = range_opt {
        (
            ((range.start + range.end) / 2.0) as f64,
            Some(range.start as f64),
            Some(range.end as f64),
            Some(range.step as f64),
        )
    } else {
        if handler_type == "stop" {
            (50.0, Some(10.0), Some(150.0), Some(10.0))
        } else {
            (10.0, Some(5.0), Some(30.0), Some(1.0))
        }
    };

    let param_id = if handler_type == "stop" {
        ConditionId::stop_handler_parameter_name(&handler.id, param_name)
    } else {
        ConditionId::take_handler_parameter_name(&handler.id, param_name)
    };

    vec![StrategyParameterSpec::new_numeric(
        param_id,
        Some(format!(
            "{} parameter for {} handler {}",
            param_name, handler_type, handler.name
        )),
        crate::strategy::types::StrategyParamValue::Number(default_val),
        min_val,
        max_val,
        step_val,
        true,
        true,
    )]
}

fn determine_indicator_category_for_stop(_handler_name: &str) -> &'static str {
    "trend"
}

fn get_compatible_indicators(category: &str) -> Vec<String> {
    use crate::indicators::registry::IndicatorRegistry;
    use crate::indicators::types::IndicatorCategory;

    let registry = IndicatorRegistry::new();

    let category_enum = match category {
        "trend" => IndicatorCategory::Trend,
        "oscillator" => IndicatorCategory::Oscillator,
        "volatility" => IndicatorCategory::Volatility,
        "volume" => IndicatorCategory::Volume,
        _ => IndicatorCategory::Trend,
    };

    let indicators = registry.get_indicators_by_category(&category_enum);
    let capacity = indicators.len();
    let mut result = Vec::with_capacity(capacity);
    for ind in indicators {
        result.push(ind.name().to_string());
    }
    result
}

pub fn get_default_stop_params(
    handler_name: &str,
) -> HashMap<String, crate::strategy::types::StrategyParamValue> {
    use crate::risk::factory::StopHandlerFactory;
    use crate::strategy::types::StrategyParamValue;

    let params = StopHandlerFactory::get_default_parameters(handler_name);

    if let Ok(temp_handler) = StopHandlerFactory::create(handler_name, &params) {
        let handler_params = temp_handler.parameters();
        let values = handler_params.get_current_values();
        let mut result = HashMap::with_capacity(values.len());
        for (param_name, param_value) in values {
            result.insert(
                param_name.clone(),
                StrategyParamValue::Number(param_value as f64),
            );
        }
        result
    } else {
        params
    }
}

pub fn get_default_take_params(
    handler_name: &str,
) -> HashMap<String, crate::strategy::types::StrategyParamValue> {
    use crate::risk::factory::TakeHandlerFactory;
    use crate::strategy::types::StrategyParamValue;

    let params = TakeHandlerFactory::get_default_parameters(handler_name);

    if let Ok(temp_handler) = TakeHandlerFactory::create(handler_name, &params) {
        let handler_params = temp_handler.parameters();
        let values = handler_params.get_current_values();
        let mut result = HashMap::with_capacity(values.len());
        for (param_name, param_value) in values {
            result.insert(
                param_name.clone(),
                StrategyParamValue::Number(param_value as f64),
            );
        }
        result
    } else {
        params
    }
}

