use crate::indicators::types::{
    DataConverter, DataValidator, IndicatorCategory, IndicatorError, IndicatorMetadata,
    IndicatorResultData, IndicatorType, InputDataType, OHLCData, ParameterSet,
};
use async_trait::async_trait;
use std::collections::HashMap;

/// Базовый трейт для всех индикаторов
#[async_trait]
pub trait Indicator: Send + Sync {
    /// Имя индикатора
    fn name(&self) -> &str;

    /// Описание индикатора
    fn description(&self) -> &str;

    /// Категория индикатора
    fn category(&self) -> IndicatorCategory;

    /// Тип индикатора
    fn indicator_type(&self) -> IndicatorType;

    // output_type удален - все индикаторы возвращают Vec<f64>

    /// Параметры индикатора
    fn parameters(&self) -> &ParameterSet;

    /// Минимальное количество данных для расчета
    fn min_data_points(&self) -> usize;

    /// Вычислить индикатор с простыми данными
    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError>;

    /// Вычислить индикатор с OHLC данными
    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError>;

    /// Универсальный метод вычисления
    async fn calculate(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        match self.indicator_type() {
            IndicatorType::Simple => self.calculate_simple(data).await,
            IndicatorType::OHLC => Err(IndicatorError::DataTypeMismatch {
                expected: "OHLC".to_string(),
                actual: "Simple".to_string(),
            }),
            IndicatorType::OHLCV => Err(IndicatorError::DataTypeMismatch {
                expected: "OHLCV".to_string(),
                actual: "Simple".to_string(),
            }),
            IndicatorType::Universal => self.calculate_simple(data).await,
        }
    }

    /// Универсальный метод вычисления с OHLC
    async fn calculate_with_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        match self.indicator_type() {
            IndicatorType::Simple => {
                // Конвертируем OHLC в простые данные (по умолчанию используем close)
                let simple_data = data.close.clone();
                self.calculate_simple(&simple_data).await
            }
            IndicatorType::OHLC => self.calculate_ohlc(data).await,
            IndicatorType::OHLCV => {
                // Проверяем наличие volume данных
                if data.volume.is_none() {
                    return Err(IndicatorError::VolumeDataRequired);
                }
                self.calculate_ohlc(data).await
            }
            IndicatorType::Universal => self.calculate_ohlc(data).await,
        }
    }

    /// Получить метаданные
    fn metadata(&self) -> IndicatorMetadata {
        IndicatorMetadata {
            name: self.name().to_string(),
            category: self.category(),
            indicator_type: self.indicator_type(),
            input_data_type: self.get_required_input_type(),
            parameters: self.parameters().get_current_values(),
            optimization_ranges: self.parameters().get_optimization_ranges(),
            dependencies: vec![], // По умолчанию нет зависимостей
            description: self.description().to_string(),
            version: "1.0.0".to_string(),
            author: "System".to_string(),
        }
    }

    /// Получить требуемый тип входных данных
    fn get_required_input_type(&self) -> InputDataType {
        match self.indicator_type() {
            IndicatorType::Simple => InputDataType::Simple(vec![]),
            IndicatorType::OHLC => InputDataType::OHLC {
                open: vec![],
                high: vec![],
                low: vec![],
                close: vec![],
            },
            IndicatorType::OHLCV => InputDataType::OHLCV {
                open: vec![],
                high: vec![],
                low: vec![],
                close: vec![],
                volume: vec![],
            },
            IndicatorType::Universal => InputDataType::Simple(vec![]),
        }
    }

    /// Валидация параметров
    fn validate_parameters(&self) -> Result<(), IndicatorError> {
        self.parameters()
            .validate_all()
            .map_err(|e| IndicatorError::ParameterValidationError(e))
    }

    /// Валидация входных данных
    fn validate_input_data(&self, data: &[f64]) -> Result<(), IndicatorError> {
        if data.len() < self.min_data_points() {
            return Err(IndicatorError::InsufficientData {
                required: self.min_data_points(),
                actual: data.len(),
            });
        }
        Ok(())
    }

    /// Валидация OHLC данных
    fn validate_ohlc_data(&self, data: &OHLCData) -> Result<(), IndicatorError> {
        if !data.validate() {
            return Err(IndicatorError::InvalidOHLCData(
                "Invalid OHLC data structure".to_string(),
            ));
        }
        if data.len() < self.min_data_points() {
            return Err(IndicatorError::InsufficientData {
                required: self.min_data_points(),
                actual: data.len(),
            });
        }
        Ok(())
    }

    /// Получить результат с метаданными
    async fn calculate_with_metadata(
        &self,
        data: &[f64],
    ) -> Result<IndicatorResultData, IndicatorError> {
        let values = self.calculate(data).await?;
        Ok(IndicatorResultData {
            values,
            metadata: self.metadata(),
        })
    }

    /// Получить результат с метаданными для OHLC
    async fn calculate_ohlc_with_metadata(
        &self,
        data: &OHLCData,
    ) -> Result<IndicatorResultData, IndicatorError> {
        let values = self.calculate_with_ohlc(data).await?;
        Ok(IndicatorResultData {
            values,
            metadata: self.metadata(),
        })
    }

    /// Клонировать индикатор
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync>;
}

/// Трейт для трендовых индикаторов
#[async_trait]
pub trait TrendIndicator: Indicator {
    /// Получить направление тренда
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError>;
}

/// Трейт для осцилляторов
#[async_trait]
pub trait OscillatorIndicator: Indicator {
    /// Получить зоны перекупленности/перепроданности
    async fn get_overbought_oversold_zones(
        &self,
        data: &[f64],
    ) -> Result<OverboughtOversoldZones, IndicatorError>;
}

/// Трейт для индикаторов волатильности
#[async_trait]
pub trait VolatilityIndicator: Indicator {
    /// Получить уровень волатильности
    async fn get_volatility_level(&self, data: &[f64]) -> Result<f64, IndicatorError>;
}

/// Трейт для простых индикаторов
#[async_trait]
pub trait SimpleIndicator: Indicator {
    /// Вычислить индикатор (алиас для calculate_simple)
    async fn compute(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        self.calculate_simple(data).await
    }
}

/// Трейт для OHLC индикаторов
#[async_trait]
pub trait OHLCIndicator: Indicator {
    /// Вычислить индикатор (алиас для calculate_ohlc)
    async fn compute(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        self.calculate_ohlc(data).await
    }
}

/// Направление тренда
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Up,       // Восходящий тренд
    Down,     // Нисходящий тренд
    Sideways, // Боковой тренд
    Unknown,  // Неизвестно
}

/// Зоны перекупленности/перепроданности
#[derive(Debug, Clone)]
pub struct OverboughtOversoldZones {
    pub overbought_threshold: f64,
    pub oversold_threshold: f64,
    pub current_value: f64,
    pub is_overbought: bool,
    pub is_oversold: bool,
}

impl OverboughtOversoldZones {
    pub fn new(overbought_threshold: f64, oversold_threshold: f64, current_value: f64) -> Self {
        Self {
            overbought_threshold,
            oversold_threshold,
            current_value,
            is_overbought: current_value > overbought_threshold,
            is_oversold: current_value < oversold_threshold,
        }
    }
}

/// Трейт для оптимизации параметров
pub trait ParameterOptimizer {
    /// Получить все возможные комбинации параметров
    fn get_parameter_combinations(&self) -> Vec<HashMap<String, f64>>;

    /// Оптимизировать параметры на основе данных
    async fn optimize_parameters(
        &self,
        data: &[f64],
        target_metric: &str,
    ) -> Result<HashMap<String, f64>, IndicatorError>;

    /// Получить количество комбинаций параметров
    fn get_total_combinations(&self) -> usize;
}

/// Реализация оптимизатора параметров для базового индикатора
impl<T: Indicator> ParameterOptimizer for T {
    fn get_parameter_combinations(&self) -> Vec<HashMap<String, f64>> {
        let mut combinations = Vec::new();
        let ranges = self.parameters().get_optimization_ranges();

        // Простая реализация: генерируем все комбинации
        let mut current_values: HashMap<String, f64> =
            ranges.iter().map(|(k, v)| (k.clone(), v.start)).collect();

        loop {
            combinations.push(current_values.clone());

            // Генерируем следующую комбинацию
            let mut has_next = false;
            for (name, range) in &ranges {
                if let Some(current) = current_values.get_mut(name) {
                    if *current + range.step <= range.end {
                        *current += range.step;
                        has_next = true;
                        break;
                    } else {
                        *current = range.start;
                    }
                }
            }

            if !has_next {
                break;
            }
        }

        combinations
    }

    fn get_total_combinations(&self) -> usize {
        let ranges = self.parameters().get_optimization_ranges();
        ranges
            .values()
            .map(|range| range.count_combinations())
            .product()
    }

    async fn optimize_parameters(
        &self,
        _data: &[f64],
        _target_metric: &str,
    ) -> Result<HashMap<String, f64>, IndicatorError> {
        // Базовая реализация: возвращаем текущие параметры
        // В реальной реализации здесь должна быть логика оптимизации
        Ok(self.parameters().get_current_values())
    }
}
