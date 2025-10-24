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

/// Направление тренда
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Rising,
    Falling,
    Sideways,
}

/// Сила сигнала
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
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

/// Конфигурация условия
#[derive(Debug, Clone)]
pub struct ConditionConfig {
    pub name: String,
    pub description: String,
    pub condition_type: ConditionType,
    pub category: ConditionCategory,
    pub min_data_points: usize,
    pub is_reversible: bool,
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
