use crate::data_model::types::TimeFrame;
use crate::indicators::types::ParameterType;
use crate::strategy::types::ConditionOperator;
use serde::{Deserialize, Serialize};

/// Информация об индикаторе для генерации комбинаций
#[derive(Debug, Clone)]
pub struct IndicatorInfo {
    pub name: String,
    pub alias: String,
    pub parameters: Vec<IndicatorParamInfo>,
    /// Может ли индикатор строиться по другому индикатору (а не только по цене)
    pub can_use_indicator_input: bool,
    /// Тип входных данных: "price" или "indicator"
    pub input_type: String,
    /// Тип индикатора: "oscillator" (осциллятор), "trend" (трендовый), "volume" (объемный), "other" (другой)
    /// Для осцилляторов (RSI, Stochastic, %R) позволяют условия с константой (RSI > 70, RSI < 30)
    pub indicator_type: String,
}

/// Информация о параметре индикатора
#[derive(Debug, Clone)]
pub struct IndicatorParamInfo {
    pub name: String,
    pub param_type: ParameterType,
    /// Можно ли оптимизировать этот параметр
    pub optimizable: bool,
    /// Имя глобального параметра для этого параметра (если есть)
    /// Например, "period" для параметра "period" индикатора SMA
    pub global_param_name: Option<String>,
}

/// Информация об условии для генерации комбинаций
#[derive(Debug, Clone)]
pub struct ConditionInfo {
    pub id: String,
    pub name: String,
    pub operator: ConditionOperator,
    /// Тип условия:
    /// - "indicator_price" (индикатор-цена, например SMA > Close)
    /// - "indicator_indicator" (индикатор-индикатор, например SMA > EMA)
    /// - "indicator_constant" (индикатор-константа, например RSI > 70, RSI < 30)
    pub condition_type: String,
    /// Параметры оптимизации (например, процент для DualWithPercent, или значение константы для indicator_constant)
    pub optimization_params: Vec<ConditionParamInfo>,
    /// Значение константы для условий типа "indicator_constant" (например, 70 для RSI > 70)
    /// Если None, то константа берется из optimization_params
    pub constant_value: Option<f64>,
    /// Alias основного индикатора (явное поле вместо парсинга)
    pub primary_indicator_alias: String,
    /// Alias вторичного индикатора (для indicator_indicator)
    pub secondary_indicator_alias: Option<String>,
    /// Таймфрейм для primary источника (индикатор или цена)
    /// Если None, используется базовый таймфрейм стратегии
    pub primary_timeframe: Option<TimeFrame>,
    /// Таймфрейм для secondary источника (индикатор, цена или константа)
    /// Если None, используется базовый таймфрейм стратегии
    pub secondary_timeframe: Option<TimeFrame>,
    /// Поле цены для условий типа "indicator_price" (если используется цена)
    /// Если None, используется Close по умолчанию
    pub price_field: Option<String>,
}

impl crate::optimization::condition_id::ConditionInfoTrait for ConditionInfo {
    fn condition_id(&self) -> &str {
        &self.id
    }

    fn condition_type(&self) -> &str {
        &self.condition_type
    }

    fn primary_timeframe(&self) -> Option<&TimeFrame> {
        self.primary_timeframe.as_ref()
    }

    fn secondary_timeframe(&self) -> Option<&TimeFrame> {
        self.secondary_timeframe.as_ref()
    }

    fn primary_indicator_alias(&self) -> Option<String> {
        Some(self.primary_indicator_alias.clone())
    }

    fn secondary_indicator_alias(&self) -> Option<String> {
        self.secondary_indicator_alias.clone()
    }
}

/// Информация о параметре условия
#[derive(Debug, Clone)]
pub struct ConditionParamInfo {
    pub name: String,
    pub optimizable: bool,
    /// Имя глобального параметра для этого параметра (если есть)
    pub global_param_name: Option<String>,
}

/// Информация о стоп-обработчике для генерации комбинаций
#[derive(Debug, Clone)]
pub struct StopHandlerInfo {
    pub id: String,
    pub name: String,
    pub handler_name: String,
    /// Тип стопа: "stop_loss" или "take_profit"
    pub stop_type: String,
    /// Параметры оптимизации (например, percentage для StopLossPct, value для StopLossFixed)
    pub optimization_params: Vec<ConditionParamInfo>,
    /// Приоритет стопа
    pub priority: i32,
}

/// Комбинация индикаторов с информацией о вложенности
#[derive(Debug, Clone)]
pub struct IndicatorCombination {
    /// Основные индикаторы (строящиеся по цене)
    pub base_indicators: Vec<IndicatorInfo>,
    /// Индикаторы, строящиеся по другим индикаторам
    pub nested_indicators: Vec<NestedIndicator>,
}

/// Индикатор, строящийся по другому индикатору
#[derive(Debug, Clone)]
pub struct NestedIndicator {
    pub indicator: IndicatorInfo,
    /// Алиас индикатора, по которому строится этот индикатор
    pub input_indicator_alias: String,
    pub depth: usize,
}

/// Конфигурация стоп-обработчика для генерации
#[derive(Debug, Clone)]
pub struct StopHandlerConfig {
    /// Имя обработчика (например, "StopLossPct", "StopLossFixed", "TakeProfitPct")
    pub handler_name: String,
    /// Тип стопа: "stop_loss" или "take_profit"
    pub stop_type: String,
    /// Значения параметров для генерации (например, проценты [0.1, 0.2, 0.3] или фиксированные значения [10.0, 20.0, 30.0])
    pub parameter_values: Vec<f64>,
    /// Имя параметра (например, "percentage" для StopLossPct, "value" для StopLossFixed)
    pub parameter_name: String,
    /// Имя глобального параметра для оптимизации (если есть)
    pub global_param_name: Option<String>,
    /// Приоритет стопа
    pub priority: i32,
}

