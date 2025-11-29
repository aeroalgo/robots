use crate::indicators::types::{
    DataConverter, DataValidator, IndicatorCategory, IndicatorError, IndicatorMetadata,
    IndicatorResultData, IndicatorType, InputDataType, OHLCData, ParameterSet,
};
use crate::strategy::types::{ConditionOperator, PriceField};
use std::collections::HashMap;

/// Базовый трейт для всех индикаторов
pub trait Indicator: Send + Sync {
    /// Имя индикатора
    fn name(&self) -> &str;

    /// Описание индикатора
    fn description(&self) -> &str;

    /// Категория индикатора
    fn category(&self) -> IndicatorCategory;

    /// Тип индикатора
    fn indicator_type(&self) -> IndicatorType;

    // output_type удален - все индикаторы возвращают Vec<f32>

    /// Параметры индикатора
    fn parameters(&self) -> &ParameterSet;

    /// Минимальное количество данных для расчета
    fn min_data_points(&self) -> usize;

    /// Вычислить индикатор с простыми данными
    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;

    /// Вычислить индикатор с OHLC данными
    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;

    /// Универсальный метод вычисления
    fn calculate(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        match self.indicator_type() {
            IndicatorType::Simple => self.calculate_simple(data),
            IndicatorType::OHLC => Err(IndicatorError::DataTypeMismatch {
                expected: "OHLC".to_string(),
                actual: "Simple".to_string(),
            }),
            IndicatorType::OHLCV => Err(IndicatorError::DataTypeMismatch {
                expected: "OHLCV".to_string(),
                actual: "Simple".to_string(),
            }),
            IndicatorType::Universal => self.calculate_simple(data),
        }
    }

    /// Универсальный метод вычисления с OHLC
    fn calculate_with_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        match self.indicator_type() {
            IndicatorType::Simple => {
                // Конвертируем OHLC в простые данные (по умолчанию используем close)
                let simple_data = data.close.clone();
                self.calculate_simple(&simple_data)
            }
            IndicatorType::OHLC => self.calculate_ohlc(data),
            IndicatorType::OHLCV => {
                // Проверяем наличие volume данных
                if data.volume.is_none() {
                    return Err(IndicatorError::VolumeDataRequired);
                }
                self.calculate_ohlc(data)
            }
            IndicatorType::Universal => self.calculate_ohlc(data),
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

    /// Правила построения стратегий для этого индикатора
    /// По умолчанию возвращает правила на основе категории
    fn build_rules(&self) -> IndicatorBuildRules {
        match self.category() {
            IndicatorCategory::Oscillator => IndicatorBuildRules::OSCILLATOR,
            IndicatorCategory::Trend => IndicatorBuildRules::TREND,
            IndicatorCategory::Channel => IndicatorBuildRules::CHANNEL,
            IndicatorCategory::Volatility => IndicatorBuildRules::VOLATILITY,
            IndicatorCategory::Volume => IndicatorBuildRules::VOLUME,
            _ => IndicatorBuildRules::default(),
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
    fn validate_input_data(&self, data: &[f32]) -> Result<(), IndicatorError> {
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
    fn calculate_with_metadata(&self, data: &[f32]) -> Result<IndicatorResultData, IndicatorError> {
        let values = self.calculate(data)?;
        Ok(IndicatorResultData {
            values,
            metadata: self.metadata(),
        })
    }

    /// Получить результат с метаданными для OHLC
    fn calculate_ohlc_with_metadata(
        &self,
        data: &OHLCData,
    ) -> Result<IndicatorResultData, IndicatorError> {
        let values = self.calculate_with_ohlc(data)?;
        Ok(IndicatorResultData {
            values,
            metadata: self.metadata(),
        })
    }

    /// Клонировать индикатор
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync>;
}

/// Трейт для трендовых индикаторов
pub trait TrendIndicator: Indicator {
    /// Получить направление тренда
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError>;
}

/// Трейт для осцилляторов
pub trait OscillatorIndicator: Indicator {
    /// Получить зоны перекупленности/перепроданности
    fn get_overbought_oversold_zones(
        &self,
        data: &[f32],
    ) -> Result<OverboughtOversoldZones, IndicatorError>;
}

/// Трейт для индикаторов волатильности
pub trait VolatilityIndicator: Indicator {
    /// Получить уровень волатильности
    fn get_volatility_level(&self, data: &[f32]) -> Result<f32, IndicatorError>;
}

/// Трейт для простых индикаторов
pub trait SimpleIndicator: Indicator {
    /// Вычислить индикатор (алиас для calculate_simple)
    fn compute(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(data)
    }
}

/// Трейт для OHLC индикаторов
pub trait OHLCIndicator: Indicator {
    /// Вычислить индикатор (алиас для calculate_ohlc)
    fn compute(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_ohlc(data)
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
    pub overbought_threshold: f32,
    pub oversold_threshold: f32,
    pub current_value: f32,
    pub is_overbought: bool,
    pub is_oversold: bool,
}

impl OverboughtOversoldZones {
    pub fn new(overbought_threshold: f32, oversold_threshold: f32, current_value: f32) -> Self {
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
    fn get_parameter_combinations(&self) -> Vec<HashMap<String, f32>>;

    /// Оптимизировать параметры на основе данных
    fn optimize_parameters(
        &self,
        data: &[f32],
        target_metric: &str,
    ) -> Result<HashMap<String, f32>, IndicatorError>;

    /// Получить количество комбинаций параметров
    fn get_total_combinations(&self) -> usize;
}

/// Реализация оптимизатора параметров для базового индикатора
impl<T: Indicator> ParameterOptimizer for T {
    fn get_parameter_combinations(&self) -> Vec<HashMap<String, f32>> {
        let mut combinations = Vec::new();
        let ranges = self.parameters().get_optimization_ranges();

        // Простая реализация: генерируем все комбинации
        let mut current_values: HashMap<String, f32> =
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

    fn optimize_parameters(
        &self,
        _data: &[f32],
        _target_metric: &str,
    ) -> Result<HashMap<String, f32>, IndicatorError> {
        // Базовая реализация: возвращаем текущие параметры
        // В реальной реализации здесь должна быть логика оптимизации
        Ok(self.parameters().get_current_values())
    }
}

// ============================================================================
// ПРАВИЛА ПОСТРОЕНИЯ СТРАТЕГИЙ
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThresholdType {
    None,
    Absolute,
    PercentOfPrice {
        base_price_fields: &'static [PriceField],
    },
}

impl ThresholdType {
    pub const fn percent_of_close() -> Self {
        Self::PercentOfPrice {
            base_price_fields: &[PriceField::Close],
        }
    }

    pub const fn percent_of(fields: &'static [PriceField]) -> Self {
        Self::PercentOfPrice {
            base_price_fields: fields,
        }
    }

    pub fn get_base_price_fields(&self) -> &[PriceField] {
        match self {
            Self::PercentOfPrice { base_price_fields } => base_price_fields,
            _ => &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PriceCompareConfig {
    pub enabled: bool,
    pub price_fields: &'static [PriceField],
}

impl PriceCompareConfig {
    pub const DISABLED: Self = Self {
        enabled: false,
        price_fields: &[],
    };

    pub const STANDARD: Self = Self {
        enabled: true,
        price_fields: &[PriceField::Close, PriceField::High, PriceField::Low],
    };

    pub const CLOSE_ONLY: Self = Self {
        enabled: true,
        price_fields: &[PriceField::Close],
    };
}

/// Конфигурация сравнения с другими индикаторами
#[derive(Debug, Clone)]
pub struct IndicatorCompareConfig {
    /// Можно ли сравнивать с другими индикаторами
    pub enabled: bool,
    /// Допустимые категории для сравнения (если пусто - игнорируется)
    pub allowed_categories: &'static [IndicatorCategory],
    /// Запрещённые категории
    pub denied_categories: &'static [IndicatorCategory],
    /// Конкретные индикаторы, с которыми можно сравнивать (по имени)
    /// Если не пусто - используется вместо allowed_categories
    pub specific_indicators: &'static [&'static str],
    /// Поддерживает ли процентные условия при сравнении с индикаторами
    /// Например: SMA > EMA * 1.02%
    pub supports_percent: bool,
}

impl IndicatorCompareConfig {
    pub const DISABLED: Self = Self {
        enabled: false,
        allowed_categories: &[],
        denied_categories: &[],
        specific_indicators: &[],
        supports_percent: false,
    };

    pub const TREND_AND_CHANNEL: Self = Self {
        enabled: true,
        allowed_categories: &[IndicatorCategory::Trend, IndicatorCategory::Channel],
        denied_categories: &[IndicatorCategory::Oscillator],
        specific_indicators: &[],
        supports_percent: true,
    };

    /// Создаёт конфиг для сравнения только с конкретными индикаторами
    pub const fn only_with(indicators: &'static [&'static str]) -> Self {
        Self {
            enabled: true,
            allowed_categories: &[],
            denied_categories: &[],
            specific_indicators: indicators,
            supports_percent: true,
        }
    }

    /// Создаёт конфиг для сравнения с конкретными индикаторами без процентных условий
    pub const fn only_with_no_percent(indicators: &'static [&'static str]) -> Self {
        Self {
            enabled: true,
            allowed_categories: &[],
            denied_categories: &[],
            specific_indicators: indicators,
            supports_percent: false,
        }
    }
}

/// Конфигурация вложенности индикаторов
#[derive(Debug, Clone)]
pub struct NestingConfig {
    /// Может ли этот индикатор быть входом для других индикаторов
    pub can_be_input: bool,
    /// Категории индикаторов, которые могут использовать этот как вход
    pub input_for_categories: &'static [IndicatorCategory],
    /// Конкретные индикаторы, которые могут использовать этот как вход (по имени)
    /// Если не пусто - используется вместо input_for_categories
    pub input_for_indicators: &'static [&'static str],
    /// Может ли принимать другие индикаторы как вход
    pub accepts_input: bool,
    /// От каких категорий может принимать вход
    pub accepts_from_categories: &'static [IndicatorCategory],
    /// Конкретные индикаторы, от которых может принимать вход (по имени)
    /// Если не пусто - используется вместо accepts_from_categories
    pub accepts_from_indicators: &'static [&'static str],
}

impl NestingConfig {
    /// Осциллятор: может быть входом для трендовых, не принимает вход
    pub const OSCILLATOR: Self = Self {
        can_be_input: true,
        input_for_categories: &[IndicatorCategory::Trend],
        input_for_indicators: &[],
        accepts_input: false,
        accepts_from_categories: &[],
        accepts_from_indicators: &[],
    };

    /// Трендовый: может быть входом и принимать вход
    pub const TREND: Self = Self {
        can_be_input: true,
        input_for_categories: &[IndicatorCategory::Trend, IndicatorCategory::Oscillator],
        input_for_indicators: &[],
        accepts_input: true,
        accepts_from_categories: &[IndicatorCategory::Trend, IndicatorCategory::Oscillator],
        accepts_from_indicators: &[],
    };

    /// Отключено: не участвует в вложенности
    pub const DISABLED: Self = Self {
        can_be_input: false,
        input_for_categories: &[],
        input_for_indicators: &[],
        accepts_input: false,
        accepts_from_categories: &[],
        accepts_from_indicators: &[],
    };

    /// Volatility: может быть входом для трендовых, не принимает вход
    pub const VOLATILITY: Self = Self {
        can_be_input: true,
        input_for_categories: &[IndicatorCategory::Trend],
        input_for_indicators: &[],
        accepts_input: false,
        accepts_from_categories: &[],
        accepts_from_indicators: &[],
    };

    /// Проверяет, может ли этот индикатор быть входом для указанного индикатора
    pub fn can_be_input_for(&self, indicator_name: &str, category: IndicatorCategory) -> bool {
        if !self.can_be_input {
            return false;
        }
        // Если указаны конкретные индикаторы - проверяем по имени
        if !self.input_for_indicators.is_empty() {
            return self.input_for_indicators.contains(&indicator_name);
        }
        // Иначе проверяем по категории
        self.input_for_categories.contains(&category)
    }

    /// Проверяет, может ли этот индикатор принять указанный индикатор как вход
    pub fn can_accept_from(&self, indicator_name: &str, category: IndicatorCategory) -> bool {
        if !self.accepts_input {
            return false;
        }
        // Если указаны конкретные индикаторы - проверяем по имени
        if !self.accepts_from_indicators.is_empty() {
            return self.accepts_from_indicators.contains(&indicator_name);
        }
        // Иначе проверяем по категории
        self.accepts_from_categories.contains(&category)
    }
}

#[derive(Debug, Clone)]
pub struct IndicatorBuildRules {
    pub allowed_conditions: &'static [ConditionOperator],
    pub price_compare: PriceCompareConfig,
    pub threshold_type: ThresholdType,
    pub indicator_compare: IndicatorCompareConfig,
    pub nesting: NestingConfig,
    pub phase_1_allowed: bool,
    pub supports_percent_condition: bool,
    pub can_compare_with_input_source: bool,
    pub can_compare_with_nested_result: bool,
    pub nested_compare_conditions: &'static [ConditionOperator],
}

impl Default for IndicatorBuildRules {
    fn default() -> Self {
        Self {
            allowed_conditions: &[
                ConditionOperator::Above,
                ConditionOperator::Below,
                ConditionOperator::CrossesAbove,
                ConditionOperator::CrossesBelow,
            ],
            price_compare: PriceCompareConfig::STANDARD,
            threshold_type: ThresholdType::None,
            indicator_compare: IndicatorCompareConfig::TREND_AND_CHANNEL,
            nesting: NestingConfig::TREND,
            phase_1_allowed: true,
            supports_percent_condition: true,
            can_compare_with_input_source: true,
            can_compare_with_nested_result: true,
            nested_compare_conditions: &[],
        }
    }
}

impl IndicatorBuildRules {
    pub const OSCILLATOR: Self = Self {
        allowed_conditions: &[
            ConditionOperator::Above,
            ConditionOperator::Below,
            ConditionOperator::CrossesAbove,
            ConditionOperator::CrossesBelow,
            ConditionOperator::RisingTrend,
            ConditionOperator::FallingTrend,
        ],
        price_compare: PriceCompareConfig::DISABLED,
        threshold_type: ThresholdType::Absolute,
        indicator_compare: IndicatorCompareConfig::DISABLED,
        nesting: NestingConfig::OSCILLATOR,
        phase_1_allowed: true,
        supports_percent_condition: false,
        can_compare_with_input_source: false,
        can_compare_with_nested_result: true,
        nested_compare_conditions: &[],
    };

    pub const TREND: Self = Self {
        allowed_conditions: &[
            ConditionOperator::Above,
            ConditionOperator::Below,
            ConditionOperator::CrossesAbove,
            ConditionOperator::CrossesBelow,
            ConditionOperator::RisingTrend,
            ConditionOperator::FallingTrend,
            ConditionOperator::GreaterPercent,
            ConditionOperator::LowerPercent,
        ],
        price_compare: PriceCompareConfig::STANDARD,
        threshold_type: ThresholdType::None,
        indicator_compare: IndicatorCompareConfig::TREND_AND_CHANNEL,
        nesting: NestingConfig::TREND,
        phase_1_allowed: true,
        supports_percent_condition: true,
        can_compare_with_input_source: true,
        can_compare_with_nested_result: true,
        nested_compare_conditions: &[],
    };

    pub const CHANNEL: Self = Self {
        allowed_conditions: &[
            ConditionOperator::Above,
            ConditionOperator::Below,
            ConditionOperator::CrossesAbove,
            ConditionOperator::CrossesBelow,
            ConditionOperator::GreaterPercent,
            ConditionOperator::LowerPercent,
        ],
        price_compare: PriceCompareConfig::STANDARD,
        threshold_type: ThresholdType::None,
        indicator_compare: IndicatorCompareConfig::TREND_AND_CHANNEL,
        nesting: NestingConfig::DISABLED,
        phase_1_allowed: true,
        supports_percent_condition: true,
        can_compare_with_input_source: false,
        can_compare_with_nested_result: false,
        nested_compare_conditions: &[],
    };

    pub const VOLATILITY: Self = Self {
        allowed_conditions: &[ConditionOperator::Above, ConditionOperator::Below],
        price_compare: PriceCompareConfig::DISABLED,
        threshold_type: ThresholdType::PercentOfPrice {
            base_price_fields: &[PriceField::Close],
        },
        indicator_compare: IndicatorCompareConfig::DISABLED,
        nesting: NestingConfig::VOLATILITY,
        phase_1_allowed: false,
        supports_percent_condition: false,
        can_compare_with_input_source: false,
        can_compare_with_nested_result: true,
        nested_compare_conditions: &[
            ConditionOperator::Above,
            ConditionOperator::Below,
            ConditionOperator::CrossesAbove,
            ConditionOperator::CrossesBelow,
            ConditionOperator::GreaterPercent,
            ConditionOperator::LowerPercent,
        ],
    };

    pub const VOLUME: Self = Self {
        allowed_conditions: &[ConditionOperator::Above, ConditionOperator::Below],
        price_compare: PriceCompareConfig::DISABLED,
        threshold_type: ThresholdType::None,
        indicator_compare: IndicatorCompareConfig::DISABLED,
        nesting: NestingConfig::DISABLED,
        phase_1_allowed: false,
        supports_percent_condition: false,
        can_compare_with_input_source: false,
        can_compare_with_nested_result: false,
        nested_compare_conditions: &[],
    };

    pub fn is_condition_allowed(&self, condition: &ConditionOperator) -> bool {
        self.allowed_conditions.contains(condition)
    }

    pub fn is_nested_condition_allowed(&self, condition: &ConditionOperator) -> bool {
        if self.nested_compare_conditions.is_empty() {
            self.allowed_conditions.contains(condition)
        } else {
            self.nested_compare_conditions.contains(condition)
        }
    }

    pub fn can_compare_with_price(&self, price_field: &PriceField) -> bool {
        self.price_compare.enabled && self.price_compare.price_fields.contains(price_field)
    }

    /// Проверяет, можно ли сравнивать с порогом
    pub fn can_compare_with_threshold(&self) -> bool {
        !matches!(self.threshold_type, ThresholdType::None)
    }

    /// Проверяет, является ли порог процентом от цены
    pub fn is_percent_of_price_threshold(&self) -> bool {
        matches!(self.threshold_type, ThresholdType::PercentOfPrice { .. })
    }

    pub fn get_percent_base_price_fields(&self) -> &[PriceField] {
        self.threshold_type.get_base_price_fields()
    }

    /// Проверяет, можно ли сравнивать с индикатором указанной категории
    pub fn can_compare_with_category(&self, category: IndicatorCategory) -> bool {
        if !self.indicator_compare.enabled {
            return false;
        }
        if self.indicator_compare.denied_categories.contains(&category) {
            return false;
        }
        if self.indicator_compare.allowed_categories.is_empty() {
            return true; // Если allowed пусто, разрешено всё кроме denied
        }
        self.indicator_compare
            .allowed_categories
            .contains(&category)
    }

    /// Проверяет, может ли быть входом для индикатора указанной категории
    pub fn can_be_input_for(&self, category: IndicatorCategory) -> bool {
        self.nesting.can_be_input && self.nesting.input_for_categories.contains(&category)
    }

    /// Проверяет, может ли принимать вход от индикатора указанной категории
    pub fn can_accept_input_from(&self, category: IndicatorCategory) -> bool {
        self.nesting.accepts_input && self.nesting.accepts_from_categories.contains(&category)
    }
}
