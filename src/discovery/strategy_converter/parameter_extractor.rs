use std::collections::HashMap;

use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerInfo};
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::StrategyParameterSpec;

pub struct ParameterExtractor;

impl ParameterExtractor {
    pub fn extract_all(candidate: &StrategyCandidate) -> Vec<StrategyParameterSpec> {
        let mut params = Vec::new();

        params.extend(Self::extract_indicator_parameters(
            &candidate.indicators,
            false,
        ));
        params.extend(Self::extract_nested_indicator_parameters(
            &candidate.nested_indicators,
        ));
        params.extend(Self::extract_condition_parameters(
            &candidate.conditions,
            "entry",
        ));
        params.extend(Self::extract_condition_parameters(
            &candidate.exit_conditions,
            "exit",
        ));
        params.extend(Self::extract_stop_handler_parameters(
            &candidate.stop_handlers,
        ));
        params.extend(Self::extract_take_handler_parameters(
            &candidate.take_handlers,
        ));

        params
    }

    fn extract_indicator_parameters(
        indicators: &[IndicatorInfo],
        is_nested: bool,
    ) -> Vec<StrategyParameterSpec> {
        use crate::indicators::parameters::ParameterPresets;

        let mut params = Vec::new();

        for indicator in indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name =
                        ConditionId::indicator_parameter_name(&indicator.alias, &param.name);
                    let range = ParameterPresets::get_range_for_parameter(
                        &indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    let (default_val, min_val, max_val, step_val) = if let Some(r) = range {
                        let default = ((r.start + r.end) / 2.0) as f64;
                        (
                            default,
                            Some(r.start as f64),
                            Some(r.end as f64),
                            Some(r.step as f64),
                        )
                    } else {
                        (50.0, Some(10.0), Some(200.0), Some(10.0))
                    };

                    let description_prefix = if is_nested { "nested" } else { "" };

                    params.push(StrategyParameterSpec::new_numeric(
                        param_name,
                        Some(format!(
                            "{} parameter for {} {}",
                            param.name, description_prefix, indicator.name
                        )),
                        Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            default_val,
                        ),
                        min_val,
                        max_val,
                        step_val,
                        param.optimizable,
                        param.mutatable,
                    ));
                }
            }
        }

        params
    }

    fn extract_nested_indicator_parameters(
        nested_indicators: &[NestedIndicator],
    ) -> Vec<StrategyParameterSpec> {
        let indicators: Vec<IndicatorInfo> = nested_indicators
            .iter()
            .map(|nested| nested.indicator.clone())
            .collect();
        Self::extract_indicator_parameters(&indicators, true)
    }

    fn extract_condition_parameters(
        conditions: &[ConditionInfo],
        prefix: &str,
    ) -> Vec<StrategyParameterSpec> {
        let mut params = Vec::new();

        for condition in conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name = if prefix == "exit" {
                        ConditionId::parameter_name(&format!("exit_{}", condition.id), &param.name)
                    } else {
                        ConditionId::parameter_name(&condition.id, &param.name)
                    };

                    let (default_val, min_val, max_val, step_val) =
                        Self::get_condition_param_range(&param.name);

                    params.push(StrategyParameterSpec::new_numeric(
                        param_name,
                        Some(format!(
                            "{} parameter for {} condition {}",
                            param.name, prefix, condition.name
                        )),
                        crate::strategy::types::StrategyParamValue::Number(default_val),
                        min_val,
                        max_val,
                        step_val,
                        param.optimizable,
                        param.mutatable,
                    ));
                }
            }
        }

        params
    }

    fn get_condition_param_range(param_name: &str) -> (f64, Option<f64>, Option<f64>, Option<f64>) {
        if param_name == "period" {
            let range = crate::condition::parameters::ConditionParameterPresets::trend_period();
            (
                ((range.min + range.max) / 2.0) as f64,
                Some(range.min as f64),
                Some(range.max as f64),
                Some(range.step as f64),
            )
        } else if param_name == "percentage" || param_name == "percent" {
            let range = crate::condition::parameters::ConditionParameterPresets::percentage();
            (
                ((range.min + range.max) / 2.0) as f64,
                Some(range.min as f64),
                Some(range.max as f64),
                Some(range.step as f64),
            )
        } else {
            (1.0, None, None, None)
        }
    }

    fn extract_stop_handler_parameters(
        stop_handlers: &[StopHandlerInfo],
    ) -> Vec<StrategyParameterSpec> {
        use crate::risk::utils::stop_handler_requires_indicator;

        let mut params = Vec::new();

        for stop_handler in stop_handlers {
            let requires_indicator = stop_handler_requires_indicator(&stop_handler.handler_name);

            if requires_indicator {
                let category =
                    Self::determine_indicator_category_for_stop(&stop_handler.handler_name);
                let compatible_indicators = Self::get_compatible_indicators(&category);

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

                params.push(StrategyParameterSpec::new_indicator_name(
                    indicator_name_key.clone(),
                    Some(format!(
                        "indicator_name parameter for stop handler {}",
                        stop_handler.name
                    )),
                    crate::strategy::types::StrategyParamValue::Text(default_indicator),
                    category,
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
                    indicator_name_key,
                    Some(10.0),
                    Some(200.0),
                    Some(10.0),
                    true,
                    true,
                ));
            }

            let temp_params = Self::get_default_stop_params(&stop_handler.handler_name);
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
                        params.extend(Self::extract_handler_params_from_info(
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

    fn determine_indicator_category_for_stop(handler_name: &str) -> String {
        let name_upper = handler_name.to_uppercase();
        if name_upper.contains("TRAIL") || name_upper.contains("ATR") {
            "trend".to_string()
        } else {
            "trend".to_string()
        }
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
        indicators
            .iter()
            .map(|ind| ind.name().to_string())
            .collect()
    }

    fn extract_take_handler_parameters(
        take_handlers: &[StopHandlerInfo],
    ) -> Vec<StrategyParameterSpec> {
        let mut params = Vec::new();

        for take_handler in take_handlers {
            let temp_params = Self::get_default_take_params(&take_handler.handler_name);
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
                        params.extend(Self::extract_handler_params_from_info(
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

    fn param_value_to_strategy_param_from_enum(
        param_type: &crate::indicators::types::ParameterType,
        default: f64,
    ) -> crate::strategy::types::StrategyParamValue {
        match param_type {
            crate::indicators::types::ParameterType::Period => {
                crate::strategy::types::StrategyParamValue::Integer(default as i64)
            }
            crate::indicators::types::ParameterType::Multiplier => {
                crate::strategy::types::StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Threshold => {
                crate::strategy::types::StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Coefficient => {
                crate::strategy::types::StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Custom => {
                crate::strategy::types::StrategyParamValue::Number(default)
            }
        }
    }

    pub fn get_default_stop_params(
        handler_name: &str,
    ) -> HashMap<String, crate::strategy::types::StrategyParamValue> {
        use crate::risk::factory::StopHandlerFactory;
        use crate::strategy::types::StrategyParamValue;

        let params = StopHandlerFactory::get_default_parameters(handler_name);

        if let Ok(temp_handler) = StopHandlerFactory::create(handler_name, &params) {
            let handler_params = temp_handler.parameters();
            let mut result = HashMap::new();
            for (param_name, param_value) in handler_params.get_current_values() {
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
            let mut result = HashMap::new();
            for (param_name, param_value) in handler_params.get_current_values() {
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
}
