use crate::indicators::types::ParameterType;
use crate::strategy::types::StrategyParamValue;

pub fn param_value_to_strategy_param_from_enum(
    param_type: &ParameterType,
    default: f64,
) -> StrategyParamValue {
    match param_type {
        ParameterType::Period => StrategyParamValue::Integer(default as i64),
        ParameterType::Multiplier => StrategyParamValue::Number(default),
        ParameterType::Threshold => StrategyParamValue::Number(default),
        ParameterType::Coefficient => StrategyParamValue::Number(default),
        ParameterType::Custom => StrategyParamValue::Number(default),
    }
}

pub fn get_condition_param_range(param_name: &str) -> (f64, Option<f64>, Option<f64>, Option<f64>) {
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

