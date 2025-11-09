use crate::indicators::types::{IndicatorParameter, ParameterRange, ParameterType};
use std::collections::HashMap;

/// Предустановленные диапазоны параметров для оптимизации
pub struct ParameterPresets;

impl ParameterPresets {
    /// Диапазон для периода
    pub fn period_range(start: f64, end: f64, step: f64) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для периода (стандартный)
    pub fn standard_period() -> ParameterRange {
        ParameterRange::new(5.0, 100.0, 1.0)
    }

    /// Диапазон для периода (короткий)
    pub fn short_period() -> ParameterRange {
        ParameterRange::new(2.0, 50.0, 1.0)
    }

    /// Диапазон для периода (длинный)
    pub fn long_period() -> ParameterRange {
        ParameterRange::new(20.0, 200.0, 5.0)
    }

    /// Диапазон для множителя
    pub fn multiplier_range(start: f64, end: f64, step: f64) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для множителя (стандартный)
    pub fn standard_multiplier() -> ParameterRange {
        ParameterRange::new(1.0, 5.0, 0.1)
    }

    /// Диапазон для множителя (ATR)
    pub fn atr_multiplier() -> ParameterRange {
        ParameterRange::new(1.5, 4.0, 0.1)
    }

    /// Диапазон для порогового значения
    pub fn threshold_range(start: f64, end: f64, step: f64) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для RSI порогов
    pub fn rsi_thresholds() -> ParameterRange {
        ParameterRange::new(20.0, 80.0, 5.0)
    }

    /// Диапазон для коэффициента
    pub fn coefficient_range(start: f64, end: f64, step: f64) -> ParameterRange {
        ParameterRange::new(start, end, step)
    }

    /// Диапазон для коэффициента сглаживания
    pub fn smoothing_coefficient() -> ParameterRange {
        ParameterRange::new(0.1, 0.9, 0.05)
    }
}

/// Создание параметра периода
pub fn create_period_parameter(name: &str, value: f64, description: &str) -> IndicatorParameter {
    IndicatorParameter::new(
        name,
        value,
        ParameterPresets::standard_period(),
        description,
        ParameterType::Period,
    )
}

/// Создание параметра периода с кастомным диапазоном
pub fn create_period_parameter_with_range(
    name: &str,
    value: f64,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Period)
}

/// Создание параметра множителя
pub fn create_multiplier_parameter(
    name: &str,
    value: f64,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(
        name,
        value,
        ParameterPresets::standard_multiplier(),
        description,
        ParameterType::Multiplier,
    )
}

/// Создание параметра множителя с кастомным диапазоном
pub fn create_multiplier_parameter_with_range(
    name: &str,
    value: f64,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Multiplier)
}

/// Создание параметра порога
pub fn create_threshold_parameter(name: &str, value: f64, description: &str) -> IndicatorParameter {
    IndicatorParameter::new(
        name,
        value,
        ParameterPresets::rsi_thresholds(),
        description,
        ParameterType::Threshold,
    )
}

/// Создание параметра порога с кастомным диапазоном
pub fn create_threshold_parameter_with_range(
    name: &str,
    value: f64,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Threshold)
}

/// Создание параметра коэффициента
pub fn create_coefficient_parameter(
    name: &str,
    value: f64,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(
        name,
        value,
        ParameterPresets::smoothing_coefficient(),
        description,
        ParameterType::Coefficient,
    )
}

/// Создание параметра коэффициента с кастомным диапазоном
pub fn create_coefficient_parameter_with_range(
    name: &str,
    value: f64,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Coefficient)
}

/// Создание параметра с кастомным типом
pub fn create_custom_parameter(
    name: &str,
    value: f64,
    range: ParameterRange,
    description: &str,
) -> IndicatorParameter {
    IndicatorParameter::new(name, value, range, description, ParameterType::Custom)
}

/// Создание набора параметров для SMA
pub fn create_sma_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SMA"),
    );
    params
}

/// Создание набора параметров для EMA
pub fn create_ema_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета EMA"),
    );
    params
}

/// Создание набора параметров для RSI
pub fn create_rsi_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета RSI"),
    );
    params
}

/// Создание набора параметров для ATR
pub fn create_atr_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета ATR"),
    );
    params
}

/// Создание набора параметров для SuperTrend
pub fn create_supertrend_parameters(
    period: f64,
    coeff_atr: f64,
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
            ParameterPresets::atr_multiplier(),
            "Коэффициент ATR для SuperTrend",
        ),
    );
    params
}

/// Создание набора параметров для MACD
pub fn create_macd_parameters(
    fast_period: f64,
    slow_period: f64,
    signal_period: f64,
) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "fast_period".to_string(),
        create_period_parameter_with_range(
            "fast_period",
            fast_period,
            ParameterPresets::short_period(),
            "Быстрый период для MACD",
        ),
    );
    params.insert(
        "slow_period".to_string(),
        create_period_parameter_with_range(
            "slow_period",
            slow_period,
            ParameterPresets::long_period(),
            "Медленный период для MACD",
        ),
    );
    params.insert(
        "signal_period".to_string(),
        create_period_parameter_with_range(
            "signal_period",
            signal_period,
            ParameterPresets::short_period(),
            "Период сигнальной линии MACD",
        ),
    );
    params
}

/// Создание набора параметров для Bollinger Bands
pub fn create_bollinger_parameters(
    period: f64,
    std_dev: f64,
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
pub fn create_stochastic_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета Stochastic"),
    );
    params
}

/// Создание набора параметров для WMA
pub fn create_wma_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета WMA"),
    );
    params
}

/// Создание набора параметров для AMA
pub fn create_ama_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета AMA"),
    );
    params
}

/// Создание набора параметров для ZLEMA
pub fn create_zlema_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета ZLEMA"),
    );
    params
}

/// Создание набора параметров для GEOMEAN
pub fn create_geomean_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета GEOMEAN"),
    );
    params
}

/// Создание набора параметров для AMMA
pub fn create_amma_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета AMMA"),
    );
    params
}

/// Создание набора параметров для SQWMA
pub fn create_sqwma_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SQWMA"),
    );
    params
}

/// Создание набора параметров для SINEWMA
pub fn create_sinewma_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета SINEWMA"),
    );
    params
}

/// Создание набора параметров для TPBF
pub fn create_tpbf_parameters(period: f64) -> HashMap<String, IndicatorParameter> {
    let mut params = HashMap::new();
    params.insert(
        "period".to_string(),
        create_period_parameter("period", period, "Период для расчета TPBF"),
    );
    params
}
