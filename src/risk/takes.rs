use std::collections::HashMap;

use thiserror::Error;

use crate::position::view::ActivePosition;
use crate::strategy::context::TimeframeData;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind, StrategyParamValue};

pub struct TakeEvaluationContext<'a> {
    pub position: &'a ActivePosition,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> TakeEvaluationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct TakeOutcome {
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub metadata: HashMap<String, String>,
}

pub trait TakeHandler: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &TakeEvaluationContext<'_>) -> Option<TakeOutcome>;
}

pub struct TakeHandlerFactory;

#[derive(Debug, Error)]
pub enum TakeHandlerError {
    #[error("unknown take handler: {0}")]
    UnknownHandler(String),
    #[error("invalid parameter {0}")]
    InvalidParameter(String),
}

impl TakeHandlerFactory {
    pub fn create(
        handler_name: &str,
        parameters: &HashMap<String, StrategyParamValue>,
    ) -> Result<Box<dyn TakeHandler>, TakeHandlerError> {
        match handler_name.to_ascii_uppercase().as_str() {
            "TAKEPROFITPCT" | "TAKE_PROFIT_PCT" | "TAKEPROFIT" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "take_profit", "take", "value"],
                    0.4,
                )?;
                Ok(Box::new(TakeProfitPctHandler { percentage }))
            }
            other => Err(TakeHandlerError::UnknownHandler(other.to_string())),
        }
    }
}

fn extract_percentage(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, TakeHandlerError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(TakeHandlerError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

struct TakeProfitPctHandler {
    percentage: f64,
}

impl TakeProfitPctHandler {
    fn level(&self, position: &ActivePosition) -> Option<f64> {
        let ratio = self.percentage / 100.0;
        match position.direction {
            PositionDirection::Long => Some(position.entry_price * (1.0 + ratio)),
            PositionDirection::Short => Some(position.entry_price * (1.0 - ratio)),
            _ => None,
        }
    }
}

impl TakeHandler for TakeProfitPctHandler {
    fn name(&self) -> &str {
        "TakeProfitPct"
    }

    fn evaluate(&self, ctx: &TakeEvaluationContext<'_>) -> Option<TakeOutcome> {
        let level = self.level(ctx.position)?;

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match ctx.position.direction {
            PositionDirection::Long => high_price >= level,
            PositionDirection::Short => low_price <= level,
            _ => false,
        };
        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            let triggered_price = match ctx.position.direction {
                PositionDirection::Long => high_price,
                PositionDirection::Short => low_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            return Some(TakeOutcome {
                exit_price: level,
                kind: StopSignalKind::TakeProfit,
                metadata,
            });
        }
        None
    }
}
