use std::collections::HashMap;

use crate::position::view::ActivePosition;
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

pub fn get_bar_extremes(timeframe_data: &TimeframeData, index: usize, fallback_price: f64) -> (f64, f64) {
    let low_series = timeframe_data
        .price_series_slice(&PriceField::Low)
        .unwrap_or(&[]);

    let high_series = timeframe_data
        .price_series_slice(&PriceField::High)
        .unwrap_or(&[]);

    let current_low = low_series
        .get(index)
        .copied()
        .map(|p| p as f64)
        .unwrap_or(fallback_price);

    let current_high = high_series
        .get(index)
        .copied()
        .map(|p| p as f64)
        .unwrap_or(fallback_price);

    (current_low, current_high)
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

pub fn compute_trailing_stop(
    position: &ActivePosition,
    new_stop: f64,
    direction: &PositionDirection,
    handler_name: &str,
) -> f64 {
    let stop_key = format!("{}_current_stop", handler_name);

    if let Some(current_stop) = position
        .metadata
        .get(&stop_key)
        .and_then(|s| s.parse::<f64>().ok())
    {
        match direction {
            PositionDirection::Long => new_stop.max(current_stop),
            PositionDirection::Short => new_stop.min(current_stop),
            _ => new_stop,
        }
    } else {
        new_stop
    }
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

