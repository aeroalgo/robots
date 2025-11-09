use crate::condition::types::{
    ConditionConfig, ConditionError, ConditionResult, ConditionResultData, SignalStrength,
    TrendDirection,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Базовый трейт для всех условий
#[async_trait]
pub trait Condition: Send + Sync {
    /// Имя условия
    fn name(&self) -> &str;

    /// Описание условия
    fn description(&self) -> &str;

    /// Конфигурация условия
    fn config(&self) -> &ConditionConfig;

    /// Минимальное количество точек данных
    fn min_data_points(&self) -> usize;

    /// Проверить условие на простых данных
    async fn check_simple(&self, data: &[f64]) -> ConditionResult<ConditionResultData>;

    /// Проверить условие на OHLC данных
    async fn check_ohlc(
        &self,
        data: &crate::indicators::types::OHLCData,
    ) -> ConditionResult<ConditionResultData>;

    /// Проверить условие на двух наборах данных (например, цена и индикатор)
    async fn check_dual(
        &self,
        data1: &[f64],
        data2: &[f64],
    ) -> ConditionResult<ConditionResultData>;

    /// Проверить условие на одном индексе
    async fn check_single(&self, index: usize, data: &[f64]) -> ConditionResult<bool>;

    /// Валидация входных данных
    fn validate_input_data(&self, data: &[f64]) -> Result<(), ConditionError>;

    /// Клонирование условия
    fn clone_box(&self) -> Box<dyn Condition + Send + Sync>;
}

/// Трейт для условий сравнения (два вектора)
#[async_trait]
pub trait ComparisonCondition: Condition {
    /// Проверить, что первый вектор выше второго
    async fn above(&self, data1: &[f64], data2: &[f64]) -> ConditionResult<ConditionResultData>;

    /// Проверить, что первый вектор ниже второго
    async fn below(&self, data1: &[f64], data2: &[f64]) -> ConditionResult<ConditionResultData>;

    /// Проверить, что вектора равны (с допуском)
    async fn equals(
        &self,
        data1: &[f64],
        data2: &[f64],
        tolerance: f64,
    ) -> ConditionResult<ConditionResultData>;

    /// Проверить, что значения в диапазоне между двумя векторами
    async fn in_range(
        &self,
        data: &[f64],
        min_data: &[f64],
        max_data: &[f64],
    ) -> ConditionResult<ConditionResultData>;
}

/// Трейт для процентных условий (два вектора)
#[async_trait]
pub trait PercentageCondition: Condition {
    /// Проверить, что первый вектор выше второго на процент
    async fn greater_percent(
        &self,
        data1: &[f64],
        data2: &[f64],
        percent: f64,
    ) -> ConditionResult<ConditionResultData>;

    /// Проверить, что первый вектор ниже второго на процент
    async fn lower_percent(
        &self,
        data1: &[f64],
        data2: &[f64],
        percent: f64,
    ) -> ConditionResult<ConditionResultData>;
}

/// Трейт для условий пересечения
#[async_trait]
pub trait CrossoverCondition: Condition {
    /// Проверить пересечение выше
    async fn crosses_above(&self, data1: &[f64], data2: &[f64], index: usize) -> bool;

    /// Проверить пересечение ниже
    async fn crosses_below(&self, data1: &[f64], data2: &[f64], index: usize) -> bool;

    /// Проверить пересечение с порогом выше
    async fn crosses_above_threshold(&self, data: &[f64], threshold: f64, index: usize) -> bool;

    /// Проверить пересечение с порогом ниже
    async fn crosses_below_threshold(&self, data: &[f64], threshold: f64, index: usize) -> bool;
}

/// Трейт для трендовых условий (один вектор)
#[async_trait]
pub trait TrendCondition: Condition {
    /// Определить направление тренда
    async fn get_trend_direction(
        &self,
        data: &[f64],
        period: usize,
    ) -> ConditionResult<TrendDirection>;

    /// Проверить растущий тренд
    async fn is_rising(&self, data: &[f64], period: usize) -> bool;

    /// Проверить падающий тренд
    async fn is_falling(&self, data: &[f64], period: usize) -> bool;

    /// Проверить боковой тренд
    async fn is_sideways(&self, data: &[f64], period: usize) -> bool;

    /// Получить силу тренда
    async fn get_trend_strength(
        &self,
        data: &[f64],
        period: usize,
    ) -> ConditionResult<SignalStrength>;

    /// Проверить растущие бары
    async fn check_rising_bars(
        &self,
        data: &[f64],
        window_size: usize,
    ) -> ConditionResult<ConditionResultData>;

    /// Проверить падающие бары
    async fn check_falling_bars(
        &self,
        data: &[f64],
        window_size: usize,
    ) -> ConditionResult<ConditionResultData>;

    /// Проверить разворот от падения к росту
    async fn check_falling_to_rising(&self, data: &[f64]) -> ConditionResult<ConditionResultData>;

    /// Проверить разворот от роста к падению
    async fn check_rising_to_falling(&self, data: &[f64]) -> ConditionResult<ConditionResultData>;
}

/// Трейт для моментум условий
#[async_trait]
pub trait MomentumCondition: Condition {
    /// Проверить ускорение
    async fn is_accelerating(&self, data: &[f64], period: usize) -> bool;

    /// Проверить замедление
    async fn is_decelerating(&self, data: &[f64], period: usize) -> bool;

    /// Проверить разворот
    async fn is_reversing(&self, data: &[f64], period: usize) -> bool;

    /// Получить моментум
    async fn get_momentum(&self, data: &[f64], period: usize) -> ConditionResult<f64>;
}

/// Трейт для волатильности
#[async_trait]
pub trait VolatilityCondition: Condition {
    /// Проверить высокую волатильность
    async fn is_high_volatility(&self, data: &[f64], period: usize, threshold: f64) -> bool;

    /// Проверить низкую волатильность
    async fn is_low_volatility(&self, data: &[f64], period: usize, threshold: f64) -> bool;

    /// Получить уровень волатильности
    async fn get_volatility_level(&self, data: &[f64], period: usize) -> ConditionResult<f64>;

    /// Проверить расширение волатильности
    async fn is_expanding(&self, data: &[f64], period: usize) -> bool;

    /// Проверить сжатие волатильности
    async fn is_contracting(&self, data: &[f64], period: usize) -> bool;
}

/// Трейт для пользовательских условий
#[async_trait]
pub trait CustomCondition: Condition {
    /// Выполнить пользовательскую логику
    async fn execute_custom(
        &self,
        data: &[f64],
        context: &HashMap<String, f64>,
    ) -> ConditionResult<ConditionResultData>;

    /// Валидация пользовательских параметров
    fn validate_custom_parameters(
        &self,
        params: &HashMap<String, f64>,
    ) -> Result<(), ConditionError>;
}

/// Трейт для оптимизации параметров условий
pub trait ParameterOptimizer {
    /// Получить диапазоны параметров для оптимизации
    fn get_parameter_ranges(&self) -> HashMap<String, (f64, f64, f64)>;

    /// Оптимизировать параметры на исторических данных
    async fn optimize_parameters(
        &self,
        data: &[f64],
        target_metric: &str,
    ) -> ConditionResult<HashMap<String, f64>>;

    /// Валидация оптимизированных параметров
    fn validate_optimized_parameters(
        &self,
        params: &HashMap<String, f64>,
    ) -> Result<(), ConditionError>;
}

/// Трейт для клонирования условий
pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn Condition + Send + Sync>;
}

impl<T> CloneBox for T
where
    T: Condition + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(self.clone())
    }
}
