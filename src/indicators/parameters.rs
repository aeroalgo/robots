use crate::indicators::types::{IndicatorParameter, ParameterRange, ParameterType};
use std::collections::HashMap;

/// Предустановленные диапазоны параметров для оптимизации
pub struct ParameterPresets;

impl ParameterPresets {
    /// Диапазон для периода
    pub fn period_range(start: f32, end: f32, step: f32) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для периода (стандартный)
    fn standard_period() -> ParameterRange {
        ParameterRange::new(5.0, 200.0, 1.0)
    }

    /// Диапазон для множителя
    pub fn multiplier_range(start: f32, end: f32, step: f32) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для множителя (стандартный)
    fn standard_multiplier() -> ParameterRange {
        ParameterRange::new(0.5, 5.0, 0.1)
    }

    /// Диапазон для множителя (ATR)
    fn atr_multiplier() -> ParameterRange {
        ParameterRange::new(1.0, 10.0, 0.5)
    }

    /// Диапазон для порогового значения
    pub fn threshold_range(start: f32, end: f32, step: f32) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для коэффициента
    pub fn coefficient_range(start: f32, end: f32, step: f32) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для коэффициента сглаживания
    fn smoothing_coefficient() -> ParameterRange {
        ParameterRange::new(0.1, 1.0, 0.05)
    }

    /// Централизованный метод для получения диапазона параметра
    /// Используется и для валидации, и для оптимизации
    pub fn get_range_for_parameter(
        indicator_name: &str,
        param_name: &str,
        param_type: &ParameterType,
    ) -> Option<ParameterRange> {
        match param_type {
            ParameterType::Period => Some(Self::standard_period()),
            ParameterType::Multiplier => match param_name.to_lowercase().as_str() {
                "deviation" => Some(ParameterRange::new(0.5, 4.0, 0.5)),
                "coeff_atr" | "atr_multiplier" | "atr_coefficient" => Some(Self::atr_multiplier()),
                _ => Some(Self::standard_multiplier()),
            },
            ParameterType::Threshold => Self::get_threshold_range(indicator_name, param_name),
            ParameterType::Coefficient => Some(Self::smoothing_coefficient()),
            ParameterType::Custom => match param_name.to_lowercase().as_str() {
                "period" | "length" => Some(Self::standard_period()),
                "deviation" => Some(ParameterRange::new(0.5, 4.0, 0.5)),
                "coeff_atr" | "atr_multiplier" => Some(Self::atr_multiplier()),
                _ => None,
            },
        }
    }

    /// Получить диапазон для пороговых значений осцилляторов
    fn get_threshold_range(indicator_name: &str, param_name: &str) -> Option<ParameterRange> {
        match indicator_name.to_uppercase().as_str() {
            "RSI" => Some(ParameterRange::new(20.0, 80.0, 10.0)),
            "STOCHASTIC" => Some(ParameterRange::new(10.0, 90.0, 10.0)),
            "WILLIAMSR" | "WILLIAMS_R" | "%R" => Some(ParameterRange::new(-90.0, -10.0, 10.0)),
            "CCI" => Some(ParameterRange::new(-200.0, 200.0, 40.0)),
            "MACD" => Some(ParameterRange::new(-5.0, 5.0, 1.0)),
            "MOMENTUM" => Some(ParameterRange::new(-100.0, 100.0, 20.0)),
            _ => match param_name.to_lowercase().as_str() {
                "overbought" | "upper" | "high" => Some(ParameterRange::new(60.0, 95.0, 10.0)),
                "oversold" | "lower" | "low" => Some(ParameterRange::new(5.0, 40.0, 10.0)),
                _ => Some(ParameterRange::new(0.0, 100.0, 5.0)),
            },
        }
    }
}

/// Создание параметра периода
pub fn create_period_parameter(name: &str, value: f32, description: &str) -> IndicatorParameter {
    let range = ParameterPresets::get_range_for_parameter("", name, &ParameterType::Period)
        .unwrap_or_else(|| ParameterRange::new(10.0, 200.0, 20.0));
    IndicatorParameter::new(name, value, range, description, ParameterType::Period)
}

/// Создание параметра периода с кастомным диапазоном
pub fn create_period_parameter_with_range(
    name: &str,
    value: f32,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Period)
}

/// Создание параметра множителя
pub fn create_multiplier_parameter(
    name: &str,
    value: f32,
    description: &str,
) -> IndicatorParameter {
    let range = ParameterPresets::get_range_for_parameter("", name, &ParameterType::Multiplier)
        .unwrap_or_else(|| ParameterRange::new(0.5, 5.0, 0.5));
    IndicatorParameter::new(name, value, range, description, ParameterType::Multiplier)
}

/// Создание параметра множителя с кастомным диапазоном
pub fn create_multiplier_parameter_with_range(
    name: &str,
    value: f32,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Multiplier)
}

/// Создание параметра порога
pub fn create_threshold_parameter(
    indicator_name: &str,
    name: &str,
    value: f32,
    description: &str,
) -> IndicatorParameter {
    let range =
        ParameterPresets::get_range_for_parameter(indicator_name, name, &ParameterType::Threshold)
            .unwrap_or_else(|| ParameterRange::new(20.0, 80.0, 5.0));
    IndicatorParameter::new(name, value, range, description, ParameterType::Threshold)
}

/// Создание параметра порога с кастомным диапазоном
pub fn create_threshold_parameter_with_range(
    name: &str,
    value: f32,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Threshold)
}

/// Создание параметра коэффициента
pub fn create_coefficient_parameter(
    name: &str,
    value: f32,
    description: &str,
) -> IndicatorParameter {
    let range = ParameterPresets::get_range_for_parameter("", name, &ParameterType::Coefficient)
        .unwrap_or_else(|| ParameterRange::new(1.0, 10.0, 0.5));
    IndicatorParameter::new(name, value, range, description, ParameterType::Coefficient)
}

/// Создание параметра коэффициента с кастомным диапазоном
pub fn create_coefficient_parameter_with_range(
    name: &str,
    value: f32,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Coefficient)
}

/// Создание параметра с кастомным типом
pub fn create_custom_parameter(
    name: &str,
    value: f32,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Custom)
}

/// Создание набора параметров для SMA
pub fn create_sma_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SMA"),
    );
    params
}

/// Создание набора параметров для EMA
pub fn create_ema_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета EMA"),
    );
    params
}

/// Создание набора параметров для RSI
pub fn create_rsi_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета RSI"),
    );
    params
}

/// Создание набора параметров для ATR
pub fn create_atr_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета ATR"),
    );
    params
}

/// Создание набора параметров для SuperTrend
pub fn create_supertrend_parameters(
    period: f32,
    coeff_atr: f32,
) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета ATR"),
    );
    params.insert(
        "coeff_atr".to_string(),
        create_multiplier_parameter_with_range(
            "coeff_atr",
            coeff_atr,
            ParameterPresets::get_range_for_parameter(
                "SuperTrend",
                "coeff_atr",
                &ParameterType::Multiplier,
            )
            .unwrap_or_else(|| ParameterRange::new(1.0, 10.0, 0.1)),
            "Коэффициент ATR для SuperTrend",
        ),
    );
    params
}

/// Создание набора параметров для MACD
pub fn create_macd_parameters(
    fast_period: f32,
    slow_period: f32,
    signal_period: f32,
) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "fast_period".to_string(),
        create_period_parameter_with_range(
            "fast_period",
            fast_period,
            ParameterRange::new(2.0, 50.0, 1.0),
            "Быстрый период для MACD",
        ),
    );
    params.insert(
        "slow_period".to_string(),
        create_period_parameter_with_range(
            "slow_period",
            slow_period,
            ParameterRange::new(20.0, 200.0, 5.0),
            "Медленный период для MACD",
        ),
    );
    params.insert(
        "signal_period".to_string(),
        create_period_parameter_with_range(
            "signal_period",
            signal_period,
            ParameterRange::new(2.0, 50.0, 1.0),
            "Период сигнальной линии MACD",
        ),
    );
    params
}

/// Создание набора параметров для Bollinger Bands
pub fn create_bollinger_parameters(
    period: f32,
    std_dev: f32,
) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SMA"),
    );
    params.insert(
        "std_dev".to_string(),
        create_multiplier_parameter_with_range(
            "std_dev",
            std_dev,
            ParameterRange::new(1.0, 3.0, 0.1),
            "Стандартное отклонение для полос",
        ),
    );
    params
}

/// Создание набора параметров для Stochastic
pub fn create_stochastic_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета Stochastic"),
    );
    params
}

/// Создание набора параметров для WMA
pub fn create_wma_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета WMA"),
    );
    params
}

/// Создание набора параметров для AMA
pub fn create_ama_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета AMA"),
    );
    params
}

/// Создание набора параметров для ZLEMA
pub fn create_zlema_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета ZLEMA"),
    );
    params
}

/// Создание набора параметров для GEOMEAN
pub fn create_geomean_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета GEOMEAN"),
    );
    params
}

/// Создание набора параметров для AMMA
pub fn create_amma_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета AMMA"),
    );
    params
}

/// Создание набора параметров для SQWMA
pub fn create_sqwma_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SQWMA"),
    );
    params
}

/// Создание набора параметров для SINEWMA
pub fn create_sinewma_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SINEWMA"),
    );
    params
}

/// Создание набора параметров для TPBF
pub fn create_tpbf_parameters(period: f32) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета TPBF"),
    );
    params
}
