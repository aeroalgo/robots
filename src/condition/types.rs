use crate::indicators::types::OHLCData;
use thiserror::Error;

/// Ошибки системы условий
#[derive(Error, Debug)]
pub enum ConditionError {
    #[error("Недостаточно данных: требуется {required}, получено {actual}")]
    InsufficientData { required: usize, actual: usize },

    #[error("Неверный параметр: {0}")]
    InvalidParameter(String),

    #[error("Неизвестное условие: {0}")]
    UnknownCondition(String),

    #[error("Ошибка вычисления: {0}")]
    CalculationError(String),

    #[error("Несовместимые типы данных")]
    IncompatibleDataTypes,
}

/// Результат выполнения условия
pub type ConditionResult<T> = Result<T, ConditionError>;

pub use crate::indicators::base::TrendDirection;

/// Сила сигнала
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignalStrength {
    Weak = 1,
    Medium = 2,
    Strong = 3,
    VeryStrong = 4,
}

/// Тип условия
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionType {
    Comparison, // Сравнение значений (два вектора)
    Crossover,  // Пересечение линий (два вектора)
    Trend,      // Трендовые условия (один вектор)
    Percentage, // Процентные условия (два вектора)
    Momentum,   // Моментум условия (один вектор)
    Volatility, // Волатильность (один вектор)
    Custom,     // Пользовательские условия
}

/// Категория условия
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionCategory {
    Entry,        // Условия входа
    Exit,         // Условия выхода
    Filter,       // Фильтры
    Confirmation, // Подтверждения
    Divergence,   // Дивергенции
}
/// Допустимые типы входных данных условия
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConditionInput {
    Single,
    Dual,
    DualWithPercent,
    Range,
    Indexed,
    Ohlc,
}

/// Входные данные для проверки условия
#[derive(Clone, Copy, Debug)]
pub enum ConditionInputData<'a> {
    Single {
        data: &'a [f32],
    },
    Dual {
        primary: &'a [f32],
        secondary: &'a [f32],
        percent: Option<f32>,
    },
    Range {
        data: &'a [f32],
        lower: &'a [f32],
        upper: &'a [f32],
    },
    Indexed {
        data: &'a [f32],
        index: usize,
    },
    Ohlc {
        data: &'a OHLCData,
    },
}
/// Конфигурация условия
#[derive(Debug, Clone)]
pub struct ConditionConfig {
    pub name: String,
    pub description: String,
    pub condition_type: ConditionType,
    pub category: ConditionCategory,
    pub min_data_points: usize,
    pub is_reversible: bool,
    pub required_inputs: Vec<ConditionInput>,
}

/// Результат условия
#[derive(Debug, Clone)]
pub struct ConditionResultData {
    pub signals: Vec<bool>,
    pub strengths: Vec<SignalStrength>,
    pub directions: Vec<TrendDirection>,
    pub metadata: ConditionMetadata,
}

/// Метаданные условия
#[derive(Debug, Clone)]
pub struct ConditionMetadata {
    pub execution_time: std::time::Duration,
    pub data_points_processed: usize,
    pub confidence_score: f32,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl<'a> ConditionInputData<'a> {
    pub fn single(data: &'a [f32]) -> Self {
        Self::Single { data }
    }

    pub fn dual(primary: &'a [f32], secondary: &'a [f32]) -> Self {
        Self::Dual {
            primary,
            secondary,
            percent: None,
        }
    }

    pub fn dual_with_percent(primary: &'a [f32], secondary: &'a [f32], percent: f32) -> Self {
        Self::Dual {
            primary,
            secondary,
            percent: Some(percent),
        }
    }

    pub fn range(data: &'a [f32], lower: &'a [f32], upper: &'a [f32]) -> Self {
        Self::Range { data, lower, upper }
    }

    pub fn indexed(data: &'a [f32], index: usize) -> Self {
        Self::Indexed { data, index }
    }

    pub fn ohlc(data: &'a OHLCData) -> Self {
        Self::Ohlc { data }
    }

    pub fn primary_len(&self) -> usize {
        match self {
            Self::Single { data } => data.len(),
            Self::Dual { primary, .. } => primary.len(),
            Self::Range { data, .. } => data.len(),
            Self::Indexed { data, .. } => data.len(),
            Self::Ohlc { data } => data.len(),
        }
    }

    pub fn secondary_len(&self) -> Option<usize> {
        match self {
            Self::Dual { secondary, .. } => Some(secondary.len()),
            Self::Range { lower, upper, .. } => Some(std::cmp::min(lower.len(), upper.len())),
            _ => None,
        }
    }

    pub fn percent(&self) -> Option<f32> {
        match self {
            Self::Dual { percent, .. } => *percent,
            _ => None,
        }
    }
}
