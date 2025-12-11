use crate::discovery::types::{IndicatorInfo, NestedIndicator};
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::StrategyParameterSpec;

use super::helpers::param_value_to_strategy_param_from_enum;

pub fn extract_indicator_parameters(
    indicators: &[IndicatorInfo],
    is_nested: bool,
) -> Vec<StrategyParameterSpec> {
    use crate::indicators::parameters::ParameterPresets;

    let estimated_capacity = indicators.len() * 2;
    let mut params = Vec::with_capacity(estimated_capacity);

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
                    param_value_to_strategy_param_from_enum(&param.param_type, default_val),
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

pub fn extract_nested_indicator_parameters(
    nested_indicators: &[NestedIndicator],
) -> Vec<StrategyParameterSpec> {
    let indicators: Vec<IndicatorInfo> = nested_indicators
        .iter()
        .map(|nested| nested.indicator.clone())
        .collect();
    extract_indicator_parameters(&indicators, true)
}

