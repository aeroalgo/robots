use std::collections::HashMap;

use crate::indicators::types::ParameterRange;
use crate::strategy::types::StrategyParamValue;

use super::auxiliary::normalize_indicator_params;
use super::errors::{StopHandlerError, TakeHandlerError};
use super::parameters::StopParameterPresets;
use super::stops::{
    ATRTrailStopHandler, HILOTrailingStopHandler, IndicatorStopHandler, PercentTrailingStopHandler,
    StopLossPctHandler,
};
use super::takes::TakeProfitPctHandler;
use super::traits::{StopHandler, TakeHandler};
use super::utils::{
    extract_bool, extract_number, extract_percentage, extract_string, ExtractError,
};

impl From<ExtractError> for StopHandlerError {
    fn from(e: ExtractError) -> Self {
        match e {
            ExtractError::InvalidParameter(s) => StopHandlerError::InvalidParameter(s),
        }
    }
}

impl From<ExtractError> for TakeHandlerError {
    fn from(e: ExtractError) -> Self {
        match e {
            ExtractError::InvalidParameter(s) => TakeHandlerError::InvalidParameter(s),
        }
    }
}

pub struct StopHandlerFactory;

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
                Ok(Box::new(StopLossPctHandler::new(percentage)))
            }
            "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => {
                if !parameters.keys().any(|k| k.eq_ignore_ascii_case("period")) {
                    return Err(StopHandlerError::InvalidParameter(
                        "period parameter is required for ATRTrailStop".to_string(),
                    ));
                }
                if !parameters.keys().any(|k| {
                    k.eq_ignore_ascii_case("coeff_atr")
                        || k.eq_ignore_ascii_case("coeff")
                        || k.eq_ignore_ascii_case("atr_coeff")
                }) {
                    return Err(StopHandlerError::InvalidParameter(
                        "coeff_atr parameter is required for ATRTrailStop".to_string(),
                    ));
                }
                let period = extract_number(parameters, &["period"], 14.0)?;
                let coeff_atr =
                    extract_number(parameters, &["coeff_atr", "coeff", "atr_coeff"], 5.0)?;
                Ok(Box::new(ATRTrailStopHandler::new(period, coeff_atr)))
            }
            "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
                if !parameters.keys().any(|k| k.eq_ignore_ascii_case("period")) {
                    return Err(StopHandlerError::InvalidParameter(
                        "period parameter is required for HILOTrailingStop".to_string(),
                    ));
                }
                let period = extract_number(parameters, &["period"], 14.0)?;
                Ok(Box::new(HILOTrailingStopHandler::new(period)))
            }
            "PERCENTTRAILSTOP" | "PERCENTTRAILINGSTOP" | "PERCENT_TRAIL_STOP" | "PERCENT_TRAIL" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "stop_loss", "stop", "value", "pct"],
                    1.0,
                )?;
                Ok(Box::new(PercentTrailingStopHandler::new(percentage)))
            }
            "INDICATORSTOP" | "INDICATOR_STOP" | "IND_STOP" => {
                let indicator_name = extract_string(parameters, &["indicator_name", "indicator"])?
                    .unwrap_or_else(|| "SMA".to_string())
                    .to_uppercase();

                let reserved_keys = [
                    "indicator_name",
                    "indicator",
                    "offset_percent",
                    "offset",
                    "offset_pct",
                    "trailing",
                    "trail",
                ];
                let mut indicator_params: HashMap<String, f64> = HashMap::new();

                for (key, value) in parameters {
                    let key_lower = key.to_lowercase();
                    if !reserved_keys.iter().any(|&r| key_lower == r) {
                        if let Some(num) = value.as_f64() {
                            indicator_params.insert(key_lower, num);
                        }
                    }
                }

                let indicator_params =
                    normalize_indicator_params(&indicator_name, &indicator_params);

                let offset_percent =
                    extract_number(parameters, &["offset_percent", "offset", "offset_pct"], 0.0)?;
                let trailing = extract_bool(parameters, &["trailing", "trail"], true);

                Ok(Box::new(IndicatorStopHandler::new(
                    indicator_name,
                    indicator_params,
                    offset_percent,
                    trailing,
                )))
            }
            other => Err(StopHandlerError::UnknownHandler(other.to_string())),
        }
    }
}

pub struct TakeHandlerFactory;

impl TakeHandlerFactory {
    pub fn create(
        handler_name: &str,
        parameters: &HashMap<String, StrategyParamValue>,
    ) -> Result<Box<dyn TakeHandler>, TakeHandlerError> {
        match handler_name.to_ascii_uppercase().as_str() {
            "TAKEPROFITPCT" | "TAKE_PROFIT_PCT" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "take_profit", "take", "value"],
                    0.4,
                )?;
                Ok(Box::new(TakeProfitPctHandler::new(percentage)))
            }
            other => Err(TakeHandlerError::UnknownHandler(other.to_string())),
        }
    }
}

pub fn get_stop_optimization_range(handler_name: &str, param_name: &str) -> Option<ParameterRange> {
    StopParameterPresets::get_range(handler_name, param_name)
}

pub fn get_take_optimization_range(handler_name: &str, param_name: &str) -> Option<ParameterRange> {
    StopParameterPresets::get_range(handler_name, param_name)
}
