use crate::data_model::types::TimeFrame;
use crate::discovery::config::StrategyDiscoveryConfig;
use crate::discovery::strategy_converter::{StrategyConversionError, StrategyConverter};
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerInfo};
use crate::strategy::types::StrategyDefinition;

/// Основной генератор стратегий
pub struct StrategyDiscoveryEngine {
    config: StrategyDiscoveryConfig,
}

impl StrategyDiscoveryEngine {
    pub fn new(config: StrategyDiscoveryConfig) -> Self {
        Self { config }
    }
}

/// Кандидат стратегии для дальнейшей оптимизации
#[derive(Debug, Clone)]
pub struct StrategyCandidate {
    /// Базовые индикаторы (строящиеся по цене)
    pub indicators: Vec<IndicatorInfo>,
    /// Вложенные индикаторы (строящиеся по другим индикаторам)
    pub nested_indicators: Vec<NestedIndicator>,
    /// Условия входа (entry conditions)
    pub conditions: Vec<ConditionInfo>,
    /// Условия выхода (exit conditions)
    pub exit_conditions: Vec<ConditionInfo>,
    pub stop_handlers: Vec<StopHandlerInfo>,
    pub take_handlers: Vec<StopHandlerInfo>,
    pub timeframes: Vec<TimeFrame>,
    pub config: StrategyDiscoveryConfig,
}

impl StrategyCandidate {
    /// Разделяет список обработчиков на stop_handlers и take_handlers
    pub fn split_handlers(
        handlers: &[StopHandlerInfo],
    ) -> (Vec<StopHandlerInfo>, Vec<StopHandlerInfo>) {
        handlers
            .iter()
            .cloned()
            .partition(|h| h.stop_type == "stop_loss")
    }

    /// Вычисляет общее количество параметров оптимизации для этой стратегии
    pub fn total_optimization_params(&self) -> usize {
        // Параметры базовых индикаторов
        let base_indicator_params: usize = self
            .indicators
            .iter()
            .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
            .sum();

        // Параметры вложенных индикаторов
        let nested_indicator_params: usize = self
            .nested_indicators
            .iter()
            .map(|nested| {
                nested
                    .indicator
                    .parameters
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let indicator_params = base_indicator_params + nested_indicator_params;

        let entry_condition_params: usize = self
            .conditions
            .iter()
            .map(|cond| {
                cond.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let exit_condition_params: usize = self
            .exit_conditions
            .iter()
            .map(|cond| {
                cond.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let condition_params = entry_condition_params + exit_condition_params;

        // Параметры стоп-обработчиков (стоп-лосс)
        let stop_params: usize = self
            .stop_handlers
            .iter()
            .map(|stop| {
                stop.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        // Параметры тейк-обработчиков (тейк-профит)
        let take_params: usize = self
            .take_handlers
            .iter()
            .map(|take| {
                take.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        indicator_params + condition_params + stop_params + take_params
    }

    /// Проверяет, соответствует ли кандидат ограничениям конфигурации
    pub fn is_valid(&self) -> bool {
        let has_exit_conditions = !self.exit_conditions.is_empty();
        let has_stop_handlers = !self.stop_handlers.is_empty();
        let has_take_handlers = !self.take_handlers.is_empty();
        let has_any_exit = has_exit_conditions || has_stop_handlers || has_take_handlers;
        let only_take = !has_exit_conditions && !has_stop_handlers && has_take_handlers;

        self.total_optimization_params() <= self.config.max_optimization_params
            && self.timeframes.len() <= self.config.timeframe_count
            && has_any_exit
            && !only_take
    }

    /// Возвращает все индикаторы кандидата (базовые + вложенные) для удобства работы
    pub fn all_indicators(&self) -> Vec<&IndicatorInfo> {
        let mut result: Vec<&IndicatorInfo> = self.indicators.iter().collect();
        result.extend(
            self.nested_indicators
                .iter()
                .map(|nested| &nested.indicator),
        );
        result
    }

    /// Возвращает все алиасы индикаторов кандидата (базовые + вложенные)
    pub fn all_indicator_aliases(&self) -> Vec<String> {
        let mut result: Vec<String> = self
            .indicators
            .iter()
            .map(|ind| ind.alias.clone())
            .collect();
        result.extend(
            self.nested_indicators
                .iter()
                .map(|nested| nested.indicator.alias.clone()),
        );
        result
    }

    /// Получает информацию о вложенном индикаторе по его алиасу
    pub fn get_nested_indicator(&self, alias: &str) -> Option<&NestedIndicator> {
        self.nested_indicators
            .iter()
            .find(|nested| nested.indicator.alias == alias)
    }

    /// Получает алиас индикатора-источника для вложенного индикатора
    pub fn get_nested_indicator_source(&self, nested_alias: &str) -> Option<&str> {
        self.get_nested_indicator(nested_alias)
            .map(|nested| nested.input_indicator_alias.as_str())
    }

    /// Преобразует кандидата стратегии в StrategyDefinition для использования с StrategyBuilder
    pub fn to_strategy_definition(
        &self,
        base_timeframe: TimeFrame,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        StrategyConverter::candidate_to_definition(self, base_timeframe)
    }
}
