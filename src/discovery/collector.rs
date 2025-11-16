use crate::discovery::types::{IndicatorInfo, IndicatorParamInfo};
use crate::indicators::base::Indicator;
use crate::indicators::registry::IndicatorRegistry;
use crate::indicators::types::{IndicatorCategory, ParameterType};

/// Собирает информацию об индикаторах из IndicatorRegistry и преобразует в IndicatorInfo
pub struct IndicatorInfoCollector;

impl IndicatorInfoCollector {
    /// Собирает информацию обо всех индикаторах из реестра
    pub fn collect_from_registry(registry: &IndicatorRegistry) -> Vec<IndicatorInfo> {
        let mut indicators = Vec::new();

        // Получаем все категории индикаторов
        for category in [
            IndicatorCategory::Trend,
            IndicatorCategory::Oscillator,
            IndicatorCategory::Channel,
            IndicatorCategory::Volume,
            IndicatorCategory::SupportResistance,
            IndicatorCategory::Custom,
            IndicatorCategory::Volatility,
        ] {
            let category_indicators = registry.get_indicators_by_category(&category);
            for indicator in category_indicators {
                if let Some(indicator_info) = Self::convert_indicator_to_info(indicator) {
                    indicators.push(indicator_info);
                }
            }
        }

        indicators
    }

    /// Преобразует индикатор из реестра в IndicatorInfo
    fn convert_indicator_to_info(indicator: &Box<dyn Indicator + Send + Sync>) -> Option<IndicatorInfo> {
        let category = indicator.category();
        let parameters = indicator.parameters();

        // Преобразуем категорию в строку для indicator_type
        let indicator_type_str = Self::category_to_string(&category);

        // Собираем информацию о параметрах
        // Получаем все имена параметров через get_current_values
        let param_names: Vec<String> = parameters.get_current_values().keys().cloned().collect();
        let param_infos: Vec<IndicatorParamInfo> = param_names
            .iter()
            .filter_map(|name| {
                parameters.get_parameter(name).map(|param| {
                    // Определяем, можно ли оптимизировать параметр
                    let optimizable = param.range.start != param.range.end;

                    // Определяем тип параметра (упрощенно, можно улучшить)
                    let param_type = Self::infer_parameter_type(name, param.value);

                    IndicatorParamInfo {
                        name: name.clone(),
                        param_type,
                        optimizable,
                        global_param_name: Self::infer_global_param_name(name),
                    }
                })
            })
            .collect();

        // Определяем, может ли индикатор строиться по другому индикатору
        // Это зависит от типа индикатора - простые индикаторы могут строиться по другим
        let can_use_indicator_input = matches!(
            indicator.indicator_type(),
            crate::indicators::types::IndicatorType::Simple
        );

        Some(IndicatorInfo {
            name: indicator.name().to_string(),
            alias: Self::generate_alias(indicator.name()),
            parameters: param_infos,
            can_use_indicator_input,
            input_type: "price".to_string(), // По умолчанию, можно улучшить
            indicator_type: indicator_type_str,
        })
    }

    /// Преобразует IndicatorCategory в строку для indicator_type
    fn category_to_string(category: &IndicatorCategory) -> String {
        match category {
            IndicatorCategory::Oscillator => "oscillator".to_string(),
            IndicatorCategory::Trend => "trend".to_string(),
            IndicatorCategory::Volume => "volume".to_string(),
            IndicatorCategory::Channel => "channel".to_string(),
            IndicatorCategory::SupportResistance => "support_resistance".to_string(),
            IndicatorCategory::Volatility => "volatility".to_string(),
            IndicatorCategory::Custom => "other".to_string(),
        }
    }

    /// Генерирует алиас из имени индикатора (например, "RSI" -> "rsi")
    fn generate_alias(name: &str) -> String {
        name.to_lowercase().replace(' ', "_")
    }

    /// Определяет тип параметра на основе имени и значения
    fn infer_parameter_type(name: &str, _value: f32) -> ParameterType {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("period") || name_lower.contains("length") {
            ParameterType::Period
        } else if name_lower.contains("multiplier") || name_lower.contains("coeff") {
            ParameterType::Multiplier
        } else if name_lower.contains("threshold") || name_lower.contains("level") {
            ParameterType::Threshold
        } else if name_lower.contains("coefficient") || name_lower.contains("coeff") {
            ParameterType::Coefficient
        } else {
            ParameterType::Custom
        }
    }

    /// Определяет имя глобального параметра на основе имени параметра
    fn infer_global_param_name(param_name: &str) -> Option<String> {
        let name_lower = param_name.to_lowercase();
        
        if name_lower.contains("period") || name_lower.contains("length") {
            Some("period".to_string())
        } else if name_lower.contains("coeff_atr") || name_lower.contains("atr_coeff") {
            Some("coeff_atr".to_string())
        } else if name_lower.contains("pct") || name_lower.contains("percentage") {
            Some("pct".to_string())
        } else {
            None
        }
    }
}

