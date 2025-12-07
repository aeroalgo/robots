use std::collections::HashMap;

use crate::indicators::types::OHLCData;
use crate::strategy::types::StrategyParamValue;

use super::errors::StopHandlerError;

/// Универсальная функция для получения спецификаций вспомогательных индикаторов
///
/// Использует `StopHandlerFactory` для создания временного обработчика и получения
/// спецификаций через метод `required_auxiliary_indicators()`. Это позволяет
/// автоматически работать с любыми новыми обработчиками без необходимости
/// добавления специального кода.
pub fn get_auxiliary_specs_from_handler_spec_universal(
    handler_name: &str,
    parameters: &HashMap<String, StrategyParamValue>,
) -> Result<Vec<AuxiliaryIndicatorSpec>, StopHandlerError> {
    use super::factory::StopHandlerFactory;

    // Создаем временный обработчик для получения спецификаций
    // Это универсальный подход - работает для всех обработчиков автоматически
    let handler = StopHandlerFactory::create(handler_name, parameters)?;

    // Получаем спецификации через метод обработчика
    Ok(handler.required_auxiliary_indicators())
}

#[derive(Clone, Debug)]
pub struct AuxiliaryIndicatorSpec {
    pub indicator_name: String,
    pub parameters: HashMap<String, f64>,
    pub alias: String,
}

impl AuxiliaryIndicatorSpec {
    pub fn atr(period: u32) -> Self {
        Self {
            indicator_name: "ATR".to_string(),
            parameters: [("period".to_string(), period as f64)]
                .into_iter()
                .collect(),
            alias: format!("aux_ATR_{}", period),
        }
    }

    pub fn minfor(period: u32) -> Self {
        Self {
            indicator_name: "MINFOR".to_string(),
            parameters: [("period".to_string(), period as f64)]
                .into_iter()
                .collect(),
            alias: format!("aux_MINFOR_{}", period),
        }
    }

    pub fn maxfor(period: u32) -> Self {
        Self {
            indicator_name: "MAXFOR".to_string(),
            parameters: [("period".to_string(), period as f64)]
                .into_iter()
                .collect(),
            alias: format!("aux_MAXFOR_{}", period),
        }
    }
}

// Функции get_default_indicator_params, normalize_indicator_params, fill_missing_indicator_params
// перенесены в parameter_extractor.rs для устранения дублирования
// Используйте: crate::risk::get_default_indicator_params, etc.

pub fn collect_required_auxiliary_indicators(
    handlers: &[Box<dyn super::traits::StopHandler>],
) -> Vec<AuxiliaryIndicatorSpec> {
    let mut specs = Vec::new();
    let mut seen_aliases = std::collections::HashSet::new();

    for handler in handlers {
        for spec in handler.required_auxiliary_indicators() {
            if !seen_aliases.contains(&spec.alias) {
                seen_aliases.insert(spec.alias.clone());
                specs.push(spec);
            }
        }
    }

    specs
}

pub fn compute_auxiliary_indicators(
    specs: &[AuxiliaryIndicatorSpec],
    ohlc: &OHLCData,
) -> Result<HashMap<String, Vec<f32>>, StopHandlerError> {
    use crate::indicators::registry::IndicatorFactory;

    let mut results = HashMap::new();

    for spec in specs {
        let parameters: HashMap<String, f32> = spec
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), *v as f32))
            .collect();

        let indicator = IndicatorFactory::create_indicator(&spec.indicator_name, parameters)
            .map_err(|e| {
                StopHandlerError::InvalidParameter(format!(
                    "Failed to create auxiliary indicator {}: {:?}",
                    spec.indicator_name, e
                ))
            })?;

        let values = indicator.calculate_ohlc(ohlc).map_err(|e| {
            StopHandlerError::InvalidParameter(format!(
                "Failed to compute auxiliary indicator {}: {:?}",
                spec.indicator_name, e
            ))
        })?;

        results.insert(spec.alias.clone(), values);
    }

    Ok(results)
}

// Функция extract_indicator_params_with_aliases перенесена в parameter_extractor.rs
// Используйте: crate::risk::extract_indicator_params_with_aliases

// Устаревшие функции удалены - теперь используется универсальный подход через
// get_auxiliary_specs_from_handler_spec_universal(), который автоматически работает
// с любыми обработчиками через StopHandlerFactory и метод required_auxiliary_indicators()

/// Получает спецификации вспомогательных индикаторов из параметров обработчика
///
/// Универсальная функция, которая автоматически работает с любыми обработчиками
/// через создание временного экземпляра и вызов `required_auxiliary_indicators()`.
///
/// Для новых обработчиков достаточно реализовать метод `required_auxiliary_indicators()`
/// в trait `StopHandler` - никакого дополнительного кода не требуется.
pub fn get_auxiliary_specs_from_handler_spec(
    handler_name: &str,
    parameters: &HashMap<String, StrategyParamValue>,
) -> Result<Vec<AuxiliaryIndicatorSpec>, StopHandlerError> {
    get_auxiliary_specs_from_handler_spec_universal(handler_name, parameters)
}

pub fn collect_auxiliary_specs_from_stop_handlers(
    stop_handlers: &[crate::strategy::types::StopHandlerSpec],
) -> Result<Vec<AuxiliaryIndicatorSpec>, StopHandlerError> {
    let mut specs = Vec::new();
    let mut seen_aliases = std::collections::HashSet::new();

    for handler in stop_handlers {
        for spec in
            get_auxiliary_specs_from_handler_spec(&handler.handler_name, &handler.parameters)?
        {
            if !seen_aliases.contains(&spec.alias) {
                seen_aliases.insert(spec.alias.clone());
                specs.push(spec);
            }
        }
    }

    Ok(specs)
}
