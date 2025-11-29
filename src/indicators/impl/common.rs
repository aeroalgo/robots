use crate::indicators::base::TrendDirection;
use crate::indicators::types::IndicatorError;

pub fn adjust_period(period: usize, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(period.max(1).min(len))
    }
}

pub fn default_trend_direction(values: Vec<f32>) -> Result<TrendDirection, IndicatorError> {
    if let Some(last_value) = values.last() {
        if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
            if last_value > prev_value {
                Ok(TrendDirection::Rising)
            } else if last_value < prev_value {
                Ok(TrendDirection::Falling)
            } else {
                Ok(TrendDirection::Sideways)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    } else {
        Ok(TrendDirection::Unknown)
    }
}
