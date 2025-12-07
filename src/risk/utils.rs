use std::collections::HashMap;

use crate::risk::context::StopValidationContext;
use crate::risk::traits::StopValidationResult;
use crate::strategy::context::TimeframeData;
use crate::strategy::types::{PositionDirection, PriceField, StrategyParamValue};

// Функции извлечения параметров перенесены в parameter_extractor.rs
// Используйте: crate::risk::extract_number, extract_string, etc.

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

pub fn validate_indicator_before_entry(
    ctx: &StopValidationContext<'_>,
    indicator_value: f64,
    indicator_description: &str,
) -> StopValidationResult {
    let stop_level = indicator_value;

    let is_valid = match ctx.direction {
        PositionDirection::Long => ctx.current_price > stop_level,
        PositionDirection::Short => ctx.current_price < stop_level,
        _ => false,
    };

    let reason = if !is_valid {
        match ctx.direction {
            PositionDirection::Long => Some(format!(
                "Цена {} не выше индикатора {} ({} = {})",
                ctx.current_price, stop_level, indicator_description, indicator_value
            )),
            PositionDirection::Short => Some(format!(
                "Цена {} не ниже индикатора {} ({} = {})",
                ctx.current_price, stop_level, indicator_description, indicator_value
            )),
            _ => None,
        }
    } else {
        None
    };

    StopValidationResult {
        stop_level,
        is_valid,
        reason,
    }
}

/// Определяет, требует ли стоп-хендлер индикатор
pub fn stop_handler_requires_indicator(handler_name: &str) -> bool {
    let name_upper = handler_name.to_ascii_uppercase();
    // Убираем индикатор из имени, если он там есть (формат "HandlerName:IndicatorName")
    let base_name = if name_upper.contains(':') {
        name_upper.split(':').next().unwrap_or(&name_upper)
    } else {
        &name_upper
    };

    matches!(
        base_name,
        "ATRTRAILINDICATORSTOP"
            | "ATR_TRAIL_INDICATOR_STOP"
            | "ATR_TRAIL_IND"
            | "PERCENTTRAILINDICATORSTOP"
            | "PERCENT_TRAIL_INDICATOR_STOP"
            | "PERCENT_TRAIL_IND"
    )
}

/// Извлекает индикатор из handler_name и нормализует имя стоп-хендлера
/// Формат handler_name: "HandlerName" или "HandlerName:IndicatorName"
/// Возвращает: (нормализованное имя, опциональное имя индикатора)
pub fn extract_indicator_from_handler_name(handler_name: &str) -> (String, Option<String>) {
    if handler_name.contains(':') {
        let parts: Vec<&str> = handler_name.split(':').collect();
        if parts.len() >= 2 {
            (
                parts[0].to_string(),
                Some(parts[1..].join(":")), // На случай, если в имени индикатора есть ':'
            )
        } else {
            (handler_name.to_string(), None)
        }
    } else {
        (handler_name.to_string(), None)
    }
}

/// Обрабатывает параметры стоп-хендлера: извлекает индикатор из handler_name
/// и добавляет его в parameters, если стоп-хендлер требует индикатор
pub fn process_stop_handler_indicator(
    handler_name: &str,
    parameters: &mut HashMap<String, StrategyParamValue>,
) -> String {
    let (normalized_name, indicator_name) = extract_indicator_from_handler_name(handler_name);

    // Если стоп-хендлер требует индикатор
    if stop_handler_requires_indicator(&normalized_name) {
        // Если indicator_name указан в handler_name (формат "HandlerName:IndicatorName"),
        // он имеет приоритет над дефолтным значением из parameters
        if let Some(indicator) = indicator_name {
            parameters.insert(
                "indicator_name".to_string(),
                StrategyParamValue::Text(indicator),
            );
        } else {
            // Если indicator_name не указан в handler_name, проверяем, есть ли он в parameters
            // (может быть добавлен из parameter_overrides)
            let has_indicator = parameters.keys().any(|k| {
                k.eq_ignore_ascii_case("indicator_name") || k.eq_ignore_ascii_case("indicator")
            });

            // Если indicator_name нет ни в handler_name, ни в parameters - это ошибка генерации кандидата
            // Стоп-хендлеры с индикаторами ДОЛЖНЫ иметь indicator_name при генерации
            if !has_indicator {
                eprintln!(
                    "ERROR: {} requires indicator_name but it's missing. This indicates a bug in candidate generation.",
                    normalized_name
                );
            }
        }
    }

    normalized_name
}
