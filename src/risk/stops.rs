use std::collections::HashMap;

use thiserror::Error;

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

pub struct StopOutcome {
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub metadata: HashMap<String, String>,
}

pub trait StopHandler: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome>;
}

pub struct StopHandlerFactory;

#[derive(Debug, Error)]
pub enum StopHandlerError {
    #[error("unknown stop handler: {0}")]
    UnknownHandler(String),
    #[error("invalid parameter {0}")]
    InvalidParameter(String),
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
            "TAKEPROFITPCT" | "TAKE_PROFIT_PCT" | "TAKEPROFIT" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "take_profit", "take", "value"],
                    0.4,
                )?;
                Ok(Box::new(TakeProfitPctHandler { percentage }))
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
        let triggered = match ctx.position.direction {
            PositionDirection::Long => ctx.current_price <= level,
            PositionDirection::Short => ctx.current_price >= level,
            _ => false,
        };
        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            return Some(StopOutcome {
                exit_price: level,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
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

impl StopHandler for TakeProfitPctHandler {
    fn name(&self) -> &str {
        "TakeProfitPct"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let level = self.level(ctx.position)?;
        let triggered = match ctx.position.direction {
            PositionDirection::Long => ctx.current_price >= level,
            PositionDirection::Short => ctx.current_price <= level,
            _ => false,
        };
        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            return Some(StopOutcome {
                exit_price: level,
                kind: StopSignalKind::TakeProfit,
                metadata,
            });
        }
        None
    }
}
