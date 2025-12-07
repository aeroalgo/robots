use std::collections::HashMap;

use crate::strategy::types::StrategyParamValue;

use super::errors::{StopHandlerError, TakeHandlerError};
use super::parameter_extractor::{
    extract_indicator_params_with_aliases, extract_number, extract_number_required,
    extract_percentage, extract_string_required, has_parameter, normalize_indicator_params,
    ParameterExtractionError,
};
use super::stops::{
    ATRTrailIndicatorStopHandler, ATRTrailStopHandler, HILOTrailingStopHandler,
    PercentTrailIndicatorStopHandler, PercentTrailingStopHandler, StopLossPctHandler,
};
use super::takes::TakeProfitPctHandler;
use super::traits::{StopHandler, TakeHandler};

impl From<ParameterExtractionError> for StopHandlerError {
    fn from(e: ParameterExtractionError) -> Self {
        match e {
            ParameterExtractionError::InvalidParameter(s) => StopHandlerError::InvalidParameter(s),
            ParameterExtractionError::RequiredParameterMissing(s) => {
                StopHandlerError::InvalidParameter(format!("required parameter missing: {}", s))
            }
            ParameterExtractionError::TypeMismatch(s) => {
                StopHandlerError::InvalidParameter(format!("type mismatch: {}", s))
            }
        }
    }
}

impl From<ParameterExtractionError> for TakeHandlerError {
    fn from(e: ParameterExtractionError) -> Self {
        match e {
            ParameterExtractionError::InvalidParameter(s) => TakeHandlerError::InvalidParameter(s),
            ParameterExtractionError::RequiredParameterMissing(s) => {
                TakeHandlerError::InvalidParameter(format!("required parameter missing: {}", s))
            }
            ParameterExtractionError::TypeMismatch(s) => {
                TakeHandlerError::InvalidParameter(format!("type mismatch: {}", s))
            }
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
            "ATRTRAILINDICATORSTOP" | "ATR_TRAIL_INDICATOR_STOP" | "ATR_TRAIL_IND" => {
                if !parameters.keys().any(|k| k.eq_ignore_ascii_case("period")) {
                    return Err(StopHandlerError::InvalidParameter(
                        "period parameter is required for ATRTrailIndicatorStop".to_string(),
                    ));
                }
                if !parameters.keys().any(|k| {
                    k.eq_ignore_ascii_case("coeff_atr")
                        || k.eq_ignore_ascii_case("coeff")
                        || k.eq_ignore_ascii_case("atr_coeff")
                }) {
                    return Err(StopHandlerError::InvalidParameter(
                        "coeff_atr parameter is required for ATRTrailIndicatorStop".to_string(),
                    ));
                }
                let indicator_name = extract_string_required(
                    parameters,
                    &["indicator_name", "indicator"],
                    "indicator_name",
                )?
                .to_uppercase();

                let reserved_keys = [
                    "indicator_name",
                    "indicator",
                    "period",
                    "coeff_atr",
                    "coeff",
                    "atr_coeff",
                ];

                // Извлекаем параметры индикатора с поддержкой alias (indicator_*, ind_*)
                // Функция автоматически обрабатывает префиксы и удаляет их
                let indicator_params =
                    extract_indicator_params_with_aliases(parameters, &reserved_keys);

                let indicator_params =
                    normalize_indicator_params(&indicator_name, &indicator_params);
                let period = extract_number(parameters, &["period"], 14.0)?;
                let coeff_atr =
                    extract_number(parameters, &["coeff_atr", "coeff", "atr_coeff"], 5.0)?;

                Ok(Box::new(ATRTrailIndicatorStopHandler::new(
                    period,
                    coeff_atr,
                    indicator_name,
                    indicator_params,
                )))
            }
            "PERCENTTRAILINDICATORSTOP" | "PERCENT_TRAIL_INDICATOR_STOP" | "PERCENT_TRAIL_IND" => {
                let indicator_name = extract_string_required(
                    parameters,
                    &["indicator_name", "indicator"],
                    "indicator_name",
                )?
                .to_uppercase();

                let reserved_keys = [
                    "indicator_name",
                    "indicator",
                    "percentage",
                    "stop_loss",
                    "stop",
                    "value",
                    "pct",
                ];

                // Извлекаем параметры индикатора с поддержкой alias (indicator_*, ind_*)
                // Функция автоматически обрабатывает префиксы и удаляет их
                let indicator_params =
                    extract_indicator_params_with_aliases(parameters, &reserved_keys);

                let indicator_params =
                    normalize_indicator_params(&indicator_name, &indicator_params);
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "stop_loss", "stop", "value", "pct"],
                    1.0,
                )?;

                Ok(Box::new(PercentTrailIndicatorStopHandler::new(
                    percentage,
                    indicator_name,
                    indicator_params,
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

// Функции get_stop_optimization_range и get_take_optimization_range перенесены в registry.rs
// для использования нового подхода с автоматическим сбором информации из обработчиков
