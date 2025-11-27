use crate::indicators::types::{IndicatorParameter, ParameterRange, ParameterType};
use std::collections::HashMap;

pub struct ParameterPresets;

impl ParameterPresets {
    fn standard_period() -> ParameterRange {
        ParameterRange::new(5.0, 200.0, 1.0)
    }

    fn standard_multiplier() -> ParameterRange {
        ParameterRange::new(0.5, 5.0, 0.1)
    }

    fn atr_multiplier() -> ParameterRange {
        ParameterRange::new(1.0, 10.0, 0.5)
    }

    fn deviation() -> ParameterRange {
        ParameterRange::new(0.5, 4.0, 0.5)
    }

    fn smoothing_coefficient() -> ParameterRange {
        ParameterRange::new(0.1, 1.0, 0.05)
    }

    fn default_for_type(param_type: &ParameterType) -> ParameterRange {
        match param_type {
            ParameterType::Period => Self::standard_period(),
            ParameterType::Multiplier => Self::standard_multiplier(),
            ParameterType::Threshold => ParameterRange::new(20.0, 80.0, 5.0),
            ParameterType::Coefficient => Self::smoothing_coefficient(),
            ParameterType::Custom => ParameterRange::new(0.0, 100.0, 1.0),
        }
    }

    pub fn get_range_for_parameter(
        indicator_name: &str,
        param_name: &str,
        param_type: &ParameterType,
    ) -> Option<ParameterRange> {
        match param_type {
            ParameterType::Period => Some(Self::standard_period()),
            ParameterType::Multiplier => Self::get_multiplier_range(param_name),
            ParameterType::Threshold => Self::get_threshold_range(indicator_name, param_name),
            ParameterType::Coefficient => Some(Self::smoothing_coefficient()),
            ParameterType::Custom => Self::get_custom_range(param_name),
        }
    }

    fn get_multiplier_range(param_name: &str) -> Option<ParameterRange> {
        match param_name.to_lowercase().as_str() {
            "deviation" => Some(Self::deviation()),
            "coeff_atr" | "atr_multiplier" | "atr_coefficient" => Some(Self::atr_multiplier()),
            _ => Some(Self::standard_multiplier()),
        }
    }

    fn get_custom_range(param_name: &str) -> Option<ParameterRange> {
        match param_name.to_lowercase().as_str() {
            "period" | "length" => Some(Self::standard_period()),
            "deviation" => Some(Self::deviation()),
            "coeff_atr" | "atr_multiplier" => Some(Self::atr_multiplier()),
            _ => None,
        }
    }

    fn get_threshold_range(indicator_name: &str, param_name: &str) -> Option<ParameterRange> {
        match indicator_name.to_uppercase().as_str() {
            "RSI" => Some(ParameterRange::new(20.0, 80.0, 10.0)),
            "STOCHASTIC" => Some(ParameterRange::new(10.0, 90.0, 10.0)),
            "WILLIAMSR" | "WILLIAMS_R" | "%R" => Some(ParameterRange::new(-90.0, -10.0, 10.0)),
            "CCI" => Some(ParameterRange::new(-200.0, 200.0, 40.0)),
            "MACD" => Some(ParameterRange::new(-5.0, 5.0, 1.0)),
            "MOMENTUM" => Some(ParameterRange::new(-100.0, 100.0, 20.0)),
            _ => Self::get_threshold_range_by_param_name(param_name),
        }
    }

    fn get_threshold_range_by_param_name(param_name: &str) -> Option<ParameterRange> {
        match param_name.to_lowercase().as_str() {
            "overbought" | "upper" | "high" => Some(ParameterRange::new(60.0, 95.0, 10.0)),
            "oversold" | "lower" | "low" => Some(ParameterRange::new(5.0, 40.0, 10.0)),
            _ => Some(ParameterRange::new(0.0, 100.0, 5.0)),
        }
    }

    pub fn get_optimization_range(
        indicator_name: &str,
        param_name: &str,
        param_type: &ParameterType,
    ) -> Option<ParameterRange> {
        Self::get_range_for_parameter(indicator_name, param_name, param_type).map(|range| {
            let step = match param_type {
                ParameterType::Period => 10.0,
                ParameterType::Multiplier => {
                    if matches!(
                        param_name.to_lowercase().as_str(),
                        "coeff_atr" | "atr_multiplier" | "atr_coefficient"
                    ) {
                        0.2
                    } else {
                        0.2
                    }
                }
                _ => range.step,
            };
            ParameterRange::new(range.start, range.end, step)
        })
    }

    pub fn get_oscillator_threshold_range(
        indicator_name: &str,
        param_name: &str,
    ) -> Option<ParameterRange> {
        Self::get_range_for_parameter(indicator_name, param_name, &ParameterType::Threshold)
    }
}

pub fn create_period_parameter(name: &str, value: f32, description: &str) -> IndicatorParameter {
    let range = ParameterPresets::get_range_for_parameter("", name, &ParameterType::Period)
        .unwrap_or_else(|| ParameterPresets::default_for_type(&ParameterType::Period));
    IndicatorParameter::new(name, value, range, description, ParameterType::Period)
}

pub fn create_multiplier_parameter(
    name: &str,
    value: f32,
    description: &str,
) -> IndicatorParameter {
    let range = ParameterPresets::get_range_for_parameter("", name, &ParameterType::Multiplier)
        .unwrap_or_else(|| ParameterPresets::default_for_type(&ParameterType::Multiplier));
    IndicatorParameter::new(name, value, range, description, ParameterType::Multiplier)
}
