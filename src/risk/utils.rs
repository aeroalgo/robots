use std::collections::HashMap;

use crate::strategy::context::TimeframeData;
use crate::strategy::types::{PositionDirection, PriceField, StrategyParamValue};

#[derive(Debug, Clone)]
pub enum ExtractError {
    InvalidParameter(String),
}

pub fn extract_percentage(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, ExtractError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(ExtractError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

pub fn extract_number(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, ExtractError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(ExtractError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

pub fn extract_string(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
) -> Result<Option<String>, ExtractError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(s) = value.as_str() {
                    return Ok(Some(s.to_string()));
                }
                if let StrategyParamValue::Text(s) = value {
                    return Ok(Some(s.clone()));
                }
                return Err(ExtractError::InvalidParameter(format!(
                    "{} must be a string",
                    key
                )));
            }
        }
    }
    Ok(None)
}

pub fn extract_bool(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: bool,
) -> bool {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(b) = value.as_bool() {
                    return b;
                }
            }
        }
    }
    default_value
}

pub fn get_price_at_index(
    timeframe_data: &TimeframeData,
    price_field: &PriceField,
    index: usize,
    fallback: f64,
) -> f64 {
    timeframe_data
        .price_series_slice(price_field)
        .and_then(|series| series.get(index))
        .copied()
        .map(|p| p as f64)
        .unwrap_or(fallback)
}

pub fn is_stop_triggered(
    direction: &PositionDirection,
    low_price: f64,
    high_price: f64,
    stop_level: f64,
) -> bool {
    match direction {
        PositionDirection::Long => low_price <= stop_level,
        PositionDirection::Short => high_price >= stop_level,
        _ => false,
    }
}

pub fn calculate_stop_exit_price(
    direction: &PositionDirection,
    stop_level: f64,
    open_price: f64,
    fallback_price: f64,
) -> f64 {
    match direction {
        PositionDirection::Long => {
            if open_price < stop_level {
                open_price
            } else {
                stop_level
            }
        }
        PositionDirection::Short => {
            if open_price > stop_level {
                open_price
            } else {
                stop_level
            }
        }
        _ => fallback_price,
    }
}

