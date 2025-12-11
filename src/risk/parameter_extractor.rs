//! Унифицированная система извлечения и нормализации параметров
//!
//! Этот модуль предоставляет централизованные функции для работы с параметрами,
//! устраняя дублирование логики из разных частей системы.

use std::collections::HashMap;

use crate::indicators::registry::IndicatorRegistry;
use crate::strategy::types::StrategyParamValue;

/// Ошибки извлечения параметров
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParameterExtractionError {
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("required parameter missing: {0}")]
    RequiredParameterMissing(String),
    
    #[error("parameter type mismatch: {0}")]
    TypeMismatch(String),
}

/// Результат извлечения параметра
pub type ParameterResult<T> = Result<T, ParameterExtractionError>;

/// Извлечение числового параметра
/// 
/// Ищет параметр по списку возможных ключей (case-insensitive).
/// Если параметр не найден, возвращает значение по умолчанию.
pub fn extract_number(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> ParameterResult<f64> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(ParameterExtractionError::InvalidParameter(
                    format!("{} must be a number", key)
                ));
            }
        }
    }
    Ok(default_value)
}

/// Извлечение обязательного числового параметра
/// 
/// Ищет параметр по списку возможных ключей (case-insensitive).
/// Если параметр не найден, возвращает ошибку.
pub fn extract_number_required(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    param_name: &str,
) -> ParameterResult<f64> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(ParameterExtractionError::TypeMismatch(
                    format!("{} must be a number", key)
                ));
            }
        }
    }
    Err(ParameterExtractionError::RequiredParameterMissing(param_name.to_string()))
}

/// Извлечение процентного параметра
/// 
/// Алиас для `extract_number` с семантикой процента.
pub fn extract_percentage(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> ParameterResult<f64> {
    extract_number(parameters, keys, default_value)
}

/// Извлечение строкового параметра
/// 
/// Ищет параметр по списку возможных ключей (case-insensitive).
/// Если параметр не найден, возвращает `None`.
pub fn extract_string(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
) -> ParameterResult<Option<String>> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(s) = value.as_str() {
                    return Ok(Some(s.to_string()));
                }
                if let StrategyParamValue::Text(s) = value {
                    return Ok(Some(s.clone()));
                }
                return Err(ParameterExtractionError::TypeMismatch(
                    format!("{} must be a string", key)
                ));
            }
        }
    }
    Ok(None)
}

/// Извлечение обязательного строкового параметра
/// 
/// Ищет параметр по списку возможных ключей (case-insensitive).
/// Если параметр не найден, возвращает ошибку.
pub fn extract_string_required(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    param_name: &str,
) -> ParameterResult<String> {
    extract_string(parameters, keys)?
        .ok_or_else(|| ParameterExtractionError::RequiredParameterMissing(param_name.to_string()))
}

/// Извлечение булевого параметра
/// 
/// Ищет параметр по списку возможных ключей (case-insensitive).
/// Если параметр не найден, возвращает значение по умолчанию.
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

/// Проверка наличия параметра
/// 
/// Проверяет, существует ли параметр с одним из указанных ключей (case-insensitive).
pub fn has_parameter(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
) -> bool {
    parameters.keys().any(|k| {
        keys.iter().any(|target| k.eq_ignore_ascii_case(target))
    })
}

/// Получение дефолтных параметров индикатора
/// 
/// Извлекает дефолтные значения параметров из реестра индикаторов.
pub fn get_default_indicator_params(indicator_name: &str) -> HashMap<String, f64> {
    let registry = IndicatorRegistry::new();
    if let Some(indicator) = registry.get_indicator(indicator_name) {
        indicator
            .parameters()
            .get_current_values()
            .into_iter()
            .map(|(k, v)| (k, v as f64))
            .collect()
    } else {
        // Fallback на стандартный период
        let mut params = HashMap::new();
        params.insert("period".to_string(), 20.0);
        params
    }
}

/// Нормализация параметров индикатора
/// 
/// Заполняет недостающие параметры дефолтными значениями из реестра индикаторов.
/// Возвращает полный набор параметров с дефолтами для отсутствующих.
pub fn normalize_indicator_params(
    indicator_name: &str,
    existing_params: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let defaults = get_default_indicator_params(indicator_name);
    let mut result = HashMap::new();

    // Сначала добавляем все дефолтные значения
    for (key, default_value) in &defaults {
        let value = existing_params.get(key).copied().unwrap_or(*default_value);
        result.insert(key.clone(), value);
    }

    // Затем добавляем любые дополнительные параметры из existing_params,
    // которых нет в дефолтах (для поддержки кастомных параметров)
    for (key, value) in existing_params {
        if !result.contains_key(key) {
            result.insert(key.clone(), *value);
        }
    }

    result
}

/// Заполнение недостающих параметров индикатора
/// 
/// Модифицирует существующий HashMap, добавляя недостающие параметры дефолтными значениями.
pub fn fill_missing_indicator_params(
    indicator_name: &str,
    existing_params: &mut HashMap<String, f64>,
) {
    let defaults = get_default_indicator_params(indicator_name);
    for (key, default_value) in defaults {
        existing_params.entry(key).or_insert(default_value);
    }
}

/// Извлечение параметров индикатора с поддержкой alias (префиксов)
/// 
/// Поддерживаемые форматы:
/// - `indicator_period` → `period` (префикс `indicator_`)
/// - `ind_coeff_atr` → `coeff_atr` (префикс `ind_`)
/// - `param1` → `param1` (без префикса, если не зарезервирован)
/// 
/// Зарезервированные ключи исключаются из результата.
pub fn extract_indicator_params_with_aliases(
    parameters: &HashMap<String, StrategyParamValue>,
    reserved_keys: &[&str],
) -> HashMap<String, f64> {
    let mut indicator_params: HashMap<String, f64> = HashMap::new();

    for (key, value) in parameters {
        let key_lower = key.to_lowercase();
        
        // Проверяем, является ли ключ зарезервированным
        if reserved_keys.iter().any(|&r| key_lower == r) {
            continue;
        }

        // Проверяем префиксы для параметров индикатора
        let param_name = if key_lower.starts_with("indicator_") {
            // Удаляем префикс "indicator_"
            Some(key_lower.strip_prefix("indicator_").unwrap().to_string())
        } else if key_lower.starts_with("ind_") {
            // Удаляем префикс "ind_"
            Some(key_lower.strip_prefix("ind_").unwrap().to_string())
        } else {
            // Параметр без префикса - это тоже параметр индикатора (если не зарезервирован)
            Some(key_lower.clone())
        };

        if let Some(param_name) = param_name {
            if let Some(num) = value.as_f64() {
                indicator_params.insert(param_name, num);
            }
        }
    }

    indicator_params
}

/// Конвертация параметров из StrategyParamValue в f64
/// 
/// Извлекает все числовые параметры из HashMap.
pub fn convert_params_to_f64(
    parameters: &HashMap<String, StrategyParamValue>,
) -> HashMap<String, f64> {
    parameters
        .iter()
        .filter_map(|(k, v)| v.as_f64().map(|n| (k.clone(), n)))
        .collect()
}

/// Конвертация параметров из f64 в StrategyParamValue
/// 
/// Преобразует HashMap<f64> в HashMap<StrategyParamValue>.
pub fn convert_params_from_f64(
    parameters: &HashMap<String, f64>,
) -> HashMap<String, StrategyParamValue> {
    parameters
        .iter()
        .map(|(k, v)| (k.clone(), StrategyParamValue::Number(*v)))
        .collect()
}









