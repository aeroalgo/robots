use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Тип входных данных для индикатора
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputDataType {
    /// Простой массив значений (например, close prices)
    Simple(Vec<f32>),
    /// OHLC данные
    OHLC {
        open: Vec<f32>,
        high: Vec<f32>,
        low: Vec<f32>,
        close: Vec<f32>,
    },
    /// OHLC + Volume данные
    OHLCV {
        open: Vec<f32>,
        high: Vec<f32>,
        low: Vec<f32>,
        close: Vec<f32>,
        volume: Vec<f32>,
    },
}

/// Категория индикатора
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum IndicatorCategory {
    Trend,             // Трендовые индикаторы
    Oscillator,        // Осцилляторы
    Channel,           // Канальные индикаторы
    Volume,            // Объемные индикаторы
    SupportResistance, // Поддержка и сопротивление
    Custom,            // Пользовательские индикаторы
    Volatility,        // Волатильность
}

/// Тип индикатора по входным данным
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum IndicatorType {
    /// Простые индикаторы (работают с массивом значений)
    Simple,
    /// OHLC индикаторы (требуют OHLC данные)
    OHLC,
    /// OHLCV индикаторы (требуют OHLC + Volume данные)
    OHLCV,
    /// Универсальные индикаторы (могут работать с обоими типами)
    Universal,
}

// OutputType удален - все индикаторы возвращают Vec<f32>

/// Диапазон параметра для оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRange {
    pub start: f32,
    pub end: f32,
    pub step: f32,
    pub current: f32,
}

impl ParameterRange {
    pub fn new(start: f32, end: f32, step: f32) -> Self {
        Self {
            start,
            end,
            step,
            current: start,
        }
    }

    pub fn validate(&self) -> bool {
        self.start < self.end && self.step > 0.0
    }

    pub fn count_combinations(&self) -> usize {
        ((self.end - self.start) / self.step + 1.0) as usize
    }

    pub fn next_value(&mut self) -> Option<f32> {
        if self.current + self.step <= self.end {
            self.current += self.step;
            Some(self.current)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.current = self.start;
    }
}

/// Параметр индикатора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorParameter {
    pub name: String,
    pub value: f32,
    pub range: ParameterRange,
    pub description: String,
    pub parameter_type: ParameterType,
}

impl IndicatorParameter {
    pub fn new(
        name: &str,
        value: f32,
        range: ParameterRange,
        description: &str,
        parameter_type: ParameterType,
    ) -> Self {
        Self {
            name: name.to_string(),
            value,
            range,
            description: description.to_string(),
            parameter_type,
        }
    }

    pub fn validate(&self) -> bool {
        self.range.validate() && self.value >= self.range.start && self.value <= self.range.end
    }
}

/// Тип параметра
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Period,      // Период
    Multiplier,  // Множитель
    Threshold,   // Пороговое значение
    Coefficient, // Коэффициент
    Custom,      // Пользовательский
}

/// Набор параметров индикатора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSet {
    parameters: HashMap<String, IndicatorParameter>,
}

impl ParameterSet {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    pub fn add_parameter(&mut self, parameter: IndicatorParameter) -> Result<(), String> {
        if parameter.validate() {
            self.parameters.insert(parameter.name.clone(), parameter);
            Ok(())
        } else {
            Err(format!("Invalid parameter: {}", parameter.name))
        }
    }

    pub fn get_parameter(&self, name: &str) -> Option<&IndicatorParameter> {
        self.parameters.get(name)
    }

    pub fn get_value(&self, name: &str) -> Option<f32> {
        self.parameters.get(name).map(|p| p.value)
    }

    pub fn set_value(&mut self, name: &str, value: f32) -> Result<(), String> {
        if let Some(param) = self.parameters.get_mut(name) {
            if value >= param.range.start && value <= param.range.end {
                param.value = value;
                Ok(())
            } else {
                Err(format!(
                    "Value {} out of range for parameter {}",
                    value, name
                ))
            }
        } else {
            Err(format!("Parameter {} not found", name))
        }
    }

    pub fn get_current_values(&self) -> HashMap<String, f32> {
        self.parameters
            .iter()
            .map(|(k, v)| (k.clone(), v.value))
            .collect()
    }

    pub fn get_optimization_ranges(&self) -> HashMap<String, ParameterRange> {
        self.parameters
            .iter()
            .map(|(k, v)| (k.clone(), v.range.clone()))
            .collect()
    }

    pub fn validate_all(&self) -> Result<(), String> {
        for (name, param) in &self.parameters {
            if !param.validate() {
                return Err(format!("Invalid parameter: {}", name));
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }
}

/// Метаданные индикатора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorMetadata {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    pub input_data_type: InputDataType,
    pub parameters: HashMap<String, f32>,
    pub optimization_ranges: HashMap<String, ParameterRange>,
    pub dependencies: Vec<String>, // Имена зависимых индикаторов
    pub description: String,
    pub version: String,
    pub author: String,
}

/// OHLC структура данных
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCData {
    pub open: Vec<f32>,
    pub high: Vec<f32>,
    pub low: Vec<f32>,
    pub close: Vec<f32>,
    pub volume: Option<Vec<f32>>,
    pub timestamp: Option<Vec<i64>>,
}

impl OHLCData {
    pub fn new(open: Vec<f32>, high: Vec<f32>, low: Vec<f32>, close: Vec<f32>) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume: None,
            timestamp: None,
        }
    }

    pub fn with_volume(mut self, volume: Vec<f32>) -> Self {
        self.volume = Some(volume);
        self
    }

    pub fn with_timestamp(mut self, timestamp: Vec<i64>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn len(&self) -> usize {
        self.close.len()
    }

    pub fn is_empty(&self) -> bool {
        self.close.is_empty()
    }

    pub fn validate(&self) -> bool {
        self.open.len() == self.high.len()
            && self.high.len() == self.low.len()
            && self.low.len() == self.close.len()
            && self
                .volume
                .as_ref()
                .map_or(true, |v| v.len() == self.close.len())
            && self
                .timestamp
                .as_ref()
                .map_or(true, |t| t.len() == self.close.len())
    }

    pub fn get_median_price(&self) -> Vec<f32> {
        self.high
            .iter()
            .zip(self.low.iter())
            .map(|(h, l)| (h + l) / 2.0)
            .collect()
    }

    pub fn get_typical_price(&self) -> Vec<f32> {
        self.high
            .iter()
            .zip(self.low.iter())
            .zip(self.close.iter())
            .map(|((h, l), c)| (h + l + c) / 3.0)
            .collect()
    }

    pub fn get_weighted_close(&self) -> Vec<f32> {
        self.high
            .iter()
            .zip(self.low.iter())
            .zip(self.close.iter())
            .map(|((h, l), c)| (h + l + c + c) / 4.0)
            .collect()
    }
}

/// Ошибки индикаторов
#[derive(Debug, thiserror::Error)]
pub enum IndicatorError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Insufficient data: required {required}, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Dependency error: {0}")]
    DependencyError(String),

    #[error("Parameter validation failed: {0}")]
    ParameterValidationError(String),

    #[error("Data type mismatch: expected {expected}, got {actual}")]
    DataTypeMismatch { expected: String, actual: String },

    #[error("OHLC data required but not provided")]
    OHLCDataRequired,

    #[error("Volume data required but not provided")]
    VolumeDataRequired,

    #[error("Invalid OHLC data: {0}")]
    InvalidOHLCData(String),

    #[error("Optimization error: {0}")]
    OptimizationError(String),

    #[error("Formula error: {0}")]
    FormulaError(String),
}

/// Результат операции с индикатором
pub type IndicatorResult<T> = Result<T, IndicatorError>;

/// Трейт для валидации входных данных
pub trait DataValidator {
    fn validate_simple_data(&self, data: &[f32]) -> Result<(), IndicatorError>;
    fn validate_ohlc_data(&self, data: &OHLCData) -> Result<(), IndicatorError>;
    fn get_required_data_type(&self) -> InputDataType;
}

/// Трейт для конвертации данных
pub trait DataConverter {
    fn convert_to_simple(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
    fn convert_to_ohlc(&self, data: &[f32]) -> Result<OHLCData, IndicatorError>;
}

/// Результат вычисления индикатора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResultData {
    pub values: Vec<f32>,
    pub metadata: IndicatorMetadata,
}

/// Идентификатор индикатора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorId {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    pub parameters: HashMap<String, f32>,
}

impl IndicatorId {
    pub fn new(name: &str, category: IndicatorCategory, indicator_type: IndicatorType) -> Self {
        Self {
            name: name.to_string(),
            category,
            indicator_type,
            parameters: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, key: &str, value: f32) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }
}
