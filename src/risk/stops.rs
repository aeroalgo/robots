use std::collections::HashMap;

use thiserror::Error;

use crate::indicators::implementations::OptimizationRange;
use crate::position::view::ActivePosition;
use crate::strategy::context::TimeframeData;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind, StrategyParamValue};

pub struct StopEvaluationContext<'a> {
    pub position: &'a ActivePosition,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> StopEvaluationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct StopValidationContext<'a> {
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> StopValidationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct StopValidationResult {
    pub stop_level: f64,
    pub is_valid: bool,
    pub reason: Option<String>,
}

pub struct StopOutcome {
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub metadata: HashMap<String, String>,
}

pub trait StopHandler: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome>;

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        None
    }
}

pub struct StopHandlerFactory;

#[derive(Debug, Error)]
pub enum StopHandlerError {
    #[error("unknown stop handler: {0}")]
    UnknownHandler(String),
    #[error("invalid parameter {0}")]
    InvalidParameter(String),
}

pub fn get_optimization_range(handler_name: &str, param_name: &str) -> Option<OptimizationRange> {
    match handler_name.to_uppercase().as_str() {
        "STOPLOSSPCT" | "STOP_LOSS_PCT" | "STOPLOSS_PCT" => {
            match param_name.to_lowercase().as_str() {
                "percentage" | "stop_loss" | "stop" | "value" | "pct" => {
                    Some(OptimizationRange::new(2.0, 10.0, 0.5))
                }
                _ => None,
            }
        }
        "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => {
            match param_name.to_lowercase().as_str() {
                "period" => Some(OptimizationRange::new(10.0, 150.0, 10.0)),
                "coeff_atr" | "coeff" | "atr_coeff" => Some(OptimizationRange::new(2.0, 8.0, 0.2)),
                _ => None,
            }
        }
        "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
            match param_name.to_lowercase().as_str() {
                "period" => Some(OptimizationRange::new(10.0, 150.0, 10.0)),
                _ => None,
            }
        }
        "PERCENTTRAILSTOP" | "PERCENTTRAILINGSTOP" | "PERCENT_TRAIL_STOP" | "PERCENT_TRAIL" => {
            match param_name.to_lowercase().as_str() {
                "percentage" | "stop_loss" | "stop" | "value" | "pct" => {
                    Some(OptimizationRange::new(1.0, 10.0, 0.5))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

impl StopHandlerFactory {
    pub fn create(
        handler_name: &str,
        parameters: &HashMap<String, StrategyParamValue>,
    ) -> Result<Box<dyn StopHandler>, StopHandlerError> {
        match handler_name.to_ascii_uppercase().as_str() {
            "STOPLOSSPCT" | "STOP_LOSS_PCT" | "STOPLOSS_PCT" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "stop_loss", "stop", "value"],
                    0.2,
                )?;
                Ok(Box::new(StopLossPctHandler { percentage }))
            }
            "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => {
                let period = extract_number(parameters, &["period"], 14.0)?;
                let coeff_atr =
                    extract_number(parameters, &["coeff_atr", "coeff", "atr_coeff"], 5.0)?;
                Ok(Box::new(ATRTrailStopHandler { period, coeff_atr }))
            }
            "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
                let period = extract_number(parameters, &["period"], 14.0)?;
                Ok(Box::new(HILOTrailingStopHandler { period }))
            }
            "PERCENTTRAILSTOP" | "PERCENTTRAILINGSTOP" | "PERCENT_TRAIL_STOP" | "PERCENT_TRAIL" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "stop_loss", "stop", "value", "pct"],
                    1.0,
                )?;
                Ok(Box::new(PercentTrailingStopHandler { percentage }))
            }
            other => Err(StopHandlerError::UnknownHandler(other.to_string())),
        }
    }
}

fn extract_percentage(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, StopHandlerError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(StopHandlerError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

fn extract_number(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, StopHandlerError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(StopHandlerError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

struct StopLossPctHandler {
    percentage: f64,
}

impl StopLossPctHandler {
    fn level(&self, position: &ActivePosition) -> Option<f64> {
        let ratio = self.percentage / 100.0;
        match position.direction {
            PositionDirection::Long => Some(position.entry_price * (1.0 - ratio)),
            PositionDirection::Short => Some(position.entry_price * (1.0 + ratio)),
            _ => None,
        }
    }
}

impl StopHandler for StopLossPctHandler {
    fn name(&self) -> &str {
        "StopLossPct"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let level = self.level(ctx.position)?;

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match ctx.position.direction {
            PositionDirection::Long => low_price <= level,
            PositionDirection::Short => high_price >= level,
            _ => false,
        };
        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            let triggered_price = match ctx.position.direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            return Some(StopOutcome {
                exit_price: level,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

struct ATRTrailStopHandler {
    period: f64,
    coeff_atr: f64,
}

impl ATRTrailStopHandler {
    fn get_atr_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let atr_alias = format!("ATR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&atr_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("ATR", ctx.index))
    }

    fn get_current_bar_extremes(&self, ctx: &StopEvaluationContext<'_>) -> (f64, f64) {
        let low_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .unwrap_or(&[]);

        let high_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .unwrap_or(&[]);

        let current_low = low_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let current_high = high_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        (current_low, current_high)
    }

    fn calculate_stop_level(
        &self,
        ctx: &StopEvaluationContext<'_>,
        min_price: f64,
        max_price: f64,
        atr: f32,
    ) -> f64 {
        let atr_f64 = atr as f64;
        let offset = atr_f64 * self.coeff_atr;

        match ctx.position.direction {
            PositionDirection::Long => min_price - offset,
            PositionDirection::Short => max_price + offset,
            _ => ctx.current_price,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        let stop_key = format!("{}_current_stop", self.name());

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
}

impl StopHandler for ATRTrailStopHandler {
    fn name(&self) -> &str {
        "ATRTrailStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let atr_value = self.get_atr_value(ctx)?;

        let (min_price, max_price) = self.get_current_bar_extremes(ctx);

        let new_stop = self.calculate_stop_level(ctx, min_price, max_price, atr_value);

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert("atr_value".to_string(), atr_value.to_string());
            metadata.insert("min_price".to_string(), min_price.to_string());
            metadata.insert("max_price".to_string(), max_price.to_string());
            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            metadata.insert(format!("{}_min_price", self.name()), min_price.to_string());
            metadata.insert(format!("{}_max_price", self.name()), max_price.to_string());
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

struct HILOTrailingStopHandler {
    period: f64,
}

impl HILOTrailingStopHandler {
    fn get_minfor_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let minfor_alias = format!("MINFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&minfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MINFOR", ctx.index))
    }

    fn get_maxfor_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let maxfor_alias = format!("MAXFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&maxfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MAXFOR", ctx.index))
    }

    fn get_minfor_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let minfor_alias = format!("MINFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&minfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MINFOR", ctx.index))
    }

    fn get_maxfor_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let maxfor_alias = format!("MAXFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&maxfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MAXFOR", ctx.index))
    }

    fn calculate_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        match ctx.position.direction {
            PositionDirection::Long => {
                let minfor = self.get_minfor_value(ctx)?;
                Some(minfor as f64)
            }
            PositionDirection::Short => {
                let maxfor = self.get_maxfor_value(ctx)?;
                Some(maxfor as f64)
            }
            _ => None,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        let stop_key = format!("{}_current_stop", self.name());

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
}

impl StopHandler for HILOTrailingStopHandler {
    fn name(&self) -> &str {
        "HILOTrailingStop"
    }

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        let stop_level = match ctx.direction {
            PositionDirection::Long => {
                let minfor = self.get_minfor_value_for_validation(ctx)?;
                minfor as f64
            }
            PositionDirection::Short => {
                let maxfor = self.get_maxfor_value_for_validation(ctx)?;
                maxfor as f64
            }
            _ => return None,
        };

        let is_valid = match ctx.direction {
            PositionDirection::Long => ctx.current_price > stop_level,
            PositionDirection::Short => ctx.current_price < stop_level,
            _ => false,
        };

        let reason = if !is_valid {
            match ctx.direction {
                PositionDirection::Long => Some(format!(
                    "Цена {} не выше MINFOR уровня {}",
                    ctx.current_price, stop_level
                )),
                PositionDirection::Short => Some(format!(
                    "Цена {} не ниже MAXFOR уровня {}",
                    ctx.current_price, stop_level
                )),
                _ => None,
            }
        } else {
            None
        };

        Some(StopValidationResult {
            stop_level,
            is_valid,
            reason,
        })
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let new_stop = self.calculate_stop_level(ctx)?;

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());

            if let Some(minfor) = self.get_minfor_value(ctx) {
                metadata.insert("minfor_value".to_string(), minfor.to_string());
            }
            if let Some(maxfor) = self.get_maxfor_value(ctx) {
                metadata.insert("maxfor_value".to_string(), maxfor.to_string());
            }

            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

struct PercentTrailingStopHandler {
    percentage: f64,
}

impl PercentTrailingStopHandler {
    fn get_current_bar_extremes(&self, ctx: &StopEvaluationContext<'_>) -> (f64, f64) {
        let low_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .unwrap_or(&[]);

        let high_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .unwrap_or(&[]);

        let current_low = low_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let current_high = high_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        (current_low, current_high)
    }

    fn calculate_stop_level(
        &self,
        ctx: &StopEvaluationContext<'_>,
        min_price: f64,
        max_price: f64,
    ) -> f64 {
        let ratio = self.percentage / 100.0;

        match ctx.position.direction {
            PositionDirection::Long => min_price * (1.0 - ratio),
            PositionDirection::Short => max_price * (1.0 + ratio),
            _ => ctx.current_price,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        let stop_key = format!("{}_current_stop", self.name());

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
}

impl StopHandler for PercentTrailingStopHandler {
    fn name(&self) -> &str {
        "PercentTrailingStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let (min_price, max_price) = self.get_current_bar_extremes(ctx);

        let new_stop = self.calculate_stop_level(ctx, min_price, max_price);

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert("percentage".to_string(), self.percentage.to_string());
            metadata.insert("min_price".to_string(), min_price.to_string());
            metadata.insert("max_price".to_string(), max_price.to_string());
            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            metadata.insert(format!("{}_min_price", self.name()), min_price.to_string());
            metadata.insert(format!("{}_max_price", self.name()), max_price.to_string());
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}
