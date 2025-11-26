//! Правила построения стратегий для индикаторов
//!
//! Этот модуль централизует все правила о том, как индикаторы могут использоваться
//! в условиях стратегий: с чем могут сравниваться, какие операторы допустимы,
//! могут ли быть входом для других индикаторов и т.д.

use crate::strategy::types::ConditionOperator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Категория индикатора для правил построения
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildCategory {
    Trend,
    Oscillator,
    Channel,
    Volatility,
    Volume,
    Custom,
}

impl BuildCategory {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trend" => BuildCategory::Trend,
            "oscillator" => BuildCategory::Oscillator,
            "channel" => BuildCategory::Channel,
            "volatility" => BuildCategory::Volatility,
            "volume" => BuildCategory::Volume,
            _ => BuildCategory::Custom,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BuildCategory::Trend => "trend",
            BuildCategory::Oscillator => "oscillator",
            BuildCategory::Channel => "channel",
            BuildCategory::Volatility => "volatility",
            BuildCategory::Volume => "volume",
            BuildCategory::Custom => "custom",
        }
    }
}

/// Конфигурация для сравнения с константой
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantComparisonConfig {
    /// Разрешено ли сравнение с константой
    pub enabled: bool,
    /// Минимальное значение константы
    pub min_value: f64,
    /// Максимальное значение константы
    pub max_value: f64,
    /// Шаг изменения константы
    pub step: f64,
    /// Значения по умолчанию для overbought/oversold
    /// Например, для RSI: [30.0, 70.0]
    pub default_levels: Option<Vec<f64>>,
    /// Тип константы: "absolute" или "percentage_of_price"
    #[serde(default = "default_constant_type")]
    pub constant_type: String,
}

fn default_constant_type() -> String {
    "absolute".to_string()
}

impl Default for ConstantComparisonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_value: 0.0,
            max_value: 100.0,
            step: 1.0,
            default_levels: None,
            constant_type: "absolute".to_string(),
        }
    }
}

impl ConstantComparisonConfig {
    /// Создаёт конфиг для осциллятора (0-100)
    pub fn oscillator() -> Self {
        Self {
            enabled: true,
            min_value: 0.0,
            max_value: 100.0,
            step: 1.0,
            default_levels: None,
            constant_type: "absolute".to_string(),
        }
    }

    /// Создаёт конфиг для RSI с дефолтными уровнями
    pub fn rsi() -> Self {
        Self {
            enabled: true,
            min_value: 0.0,
            max_value: 100.0,
            step: 1.0,
            default_levels: Some(vec![30.0, 70.0]),
            constant_type: "absolute".to_string(),
        }
    }

    /// Создаёт конфиг для Stochastic с дефолтными уровнями
    pub fn stochastic() -> Self {
        Self {
            enabled: true,
            min_value: 0.0,
            max_value: 100.0,
            step: 1.0,
            default_levels: Some(vec![20.0, 80.0]),
            constant_type: "absolute".to_string(),
        }
    }

    /// Создаёт конфиг для volatility (процент от цены)
    pub fn volatility_percentage() -> Self {
        Self {
            enabled: true,
            min_value: 0.2,
            max_value: 10.0,
            step: 0.1,
            default_levels: None,
            constant_type: "percentage_of_price".to_string(),
        }
    }

    /// Возвращает значение по умолчанию для оператора GreaterThan
    pub fn default_for_greater_than(&self) -> f64 {
        if let Some(levels) = &self.default_levels {
            if levels.len() >= 2 {
                return levels[1]; // Обычно верхний уровень (70 для RSI)
            }
        }
        (self.min_value + self.max_value) / 2.0
    }

    /// Возвращает значение по умолчанию для оператора LessThan
    pub fn default_for_less_than(&self) -> f64 {
        if let Some(levels) = &self.default_levels {
            if !levels.is_empty() {
                return levels[0]; // Обычно нижний уровень (30 для RSI)
            }
        }
        (self.min_value + self.max_value) / 2.0
    }
}

/// Конфигурация для сравнения с ценой
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceComparisonConfig {
    /// Разрешено ли сравнение с ценой
    pub enabled: bool,
    /// Допустимые поля цены (Close, High, Low, Open)
    pub allowed_price_fields: Vec<String>,
    /// Поле цены по умолчанию
    #[serde(default = "default_price_field")]
    pub default_price_field: String,
}

fn default_price_field() -> String {
    "Close".to_string()
}

impl Default for PriceComparisonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_price_fields: vec![
                "Close".to_string(),
                "High".to_string(),
                "Low".to_string(),
                "Open".to_string(),
            ],
            default_price_field: "Close".to_string(),
        }
    }
}

impl PriceComparisonConfig {
    /// Сравнение запрещено
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            allowed_price_fields: vec![],
            default_price_field: "Close".to_string(),
        }
    }

    /// Только Close
    pub fn close_only() -> Self {
        Self {
            enabled: true,
            allowed_price_fields: vec!["Close".to_string()],
            default_price_field: "Close".to_string(),
        }
    }

    /// Close, High, Low
    pub fn standard() -> Self {
        Self {
            enabled: true,
            allowed_price_fields: vec!["Close".to_string(), "High".to_string(), "Low".to_string()],
            default_price_field: "Close".to_string(),
        }
    }
}

/// Конфигурация для сравнения с другими индикаторами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorComparisonConfig {
    /// Разрешено ли сравнение с другими индикаторами
    pub enabled: bool,
    /// Категории индикаторов, с которыми можно сравнивать
    pub allowed_categories: Vec<BuildCategory>,
    /// Конкретные индикаторы, с которыми можно сравнивать (имена)
    /// Если пусто - используются allowed_categories
    pub allowed_indicators: Vec<String>,
    /// Категории индикаторов, с которыми НЕЛЬЗЯ сравнивать
    pub denied_categories: Vec<BuildCategory>,
    /// Конкретные индикаторы, с которыми НЕЛЬЗЯ сравнивать
    pub denied_indicators: Vec<String>,
    /// Можно ли сравнивать с самим собой (разные экземпляры)
    #[serde(default)]
    pub can_compare_with_self: bool,
}

impl Default for IndicatorComparisonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_categories: vec![
                BuildCategory::Trend,
                BuildCategory::Channel,
                BuildCategory::Oscillator,
            ],
            allowed_indicators: vec![],
            denied_categories: vec![],
            denied_indicators: vec![],
            can_compare_with_self: true,
        }
    }
}

impl IndicatorComparisonConfig {
    /// Сравнение запрещено
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            allowed_categories: vec![],
            allowed_indicators: vec![],
            denied_categories: vec![],
            denied_indicators: vec![],
            can_compare_with_self: false,
        }
    }

    /// Только trend и channel
    pub fn trend_and_channel() -> Self {
        Self {
            enabled: true,
            allowed_categories: vec![BuildCategory::Trend, BuildCategory::Channel],
            allowed_indicators: vec![],
            denied_categories: vec![BuildCategory::Oscillator],
            denied_indicators: vec![],
            can_compare_with_self: true,
        }
    }

    /// Только с индикаторами, построенными по осцилляторам
    /// (специальное правило, обрабатывается в runtime)
    pub fn only_nested_on_oscillator() -> Self {
        Self {
            enabled: true,
            allowed_categories: vec![], // Пустые категории = специальная логика
            allowed_indicators: vec![],
            denied_categories: vec![BuildCategory::Oscillator], // Нельзя с чистыми осцилляторами
            denied_indicators: vec![],
            can_compare_with_self: false,
        }
    }
}

/// Конфигурация для использования как входа других индикаторов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedInputConfig {
    /// Может ли быть входом для других индикаторов
    pub can_be_input: bool,
    /// Какие категории индикаторов могут его использовать как вход
    pub can_be_input_for: Vec<BuildCategory>,
    /// Может ли принимать другие индикаторы как вход
    pub can_accept_input: bool,
    /// Какие категории индикаторов может принимать как вход
    pub accepts_from_categories: Vec<BuildCategory>,
}

impl Default for NestedInputConfig {
    fn default() -> Self {
        Self {
            can_be_input: true,
            can_be_input_for: vec![BuildCategory::Trend, BuildCategory::Oscillator],
            can_accept_input: false,
            accepts_from_categories: vec![],
        }
    }
}

impl NestedInputConfig {
    /// Может быть входом для трендовых индикаторов (как осциллятор)
    pub fn oscillator() -> Self {
        Self {
            can_be_input: true,
            can_be_input_for: vec![BuildCategory::Trend],
            can_accept_input: false,
            accepts_from_categories: vec![],
        }
    }

    /// Трендовый индикатор - может и быть входом, и принимать вход
    pub fn trend() -> Self {
        Self {
            can_be_input: true,
            can_be_input_for: vec![BuildCategory::Trend, BuildCategory::Oscillator],
            can_accept_input: true,
            accepts_from_categories: vec![BuildCategory::Trend, BuildCategory::Oscillator],
        }
    }

    /// Не участвует в nested
    pub fn disabled() -> Self {
        Self {
            can_be_input: false,
            can_be_input_for: vec![],
            can_accept_input: false,
            accepts_from_categories: vec![],
        }
    }
}

/// Правила построения для индикатора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorBuildRules {
    /// Категория индикатора
    pub category: BuildCategory,

    /// Сравнение с ценой
    pub price_comparison: PriceComparisonConfig,

    /// Сравнение с константой
    pub constant_comparison: ConstantComparisonConfig,

    /// Сравнение с другими индикаторами
    pub indicator_comparison: IndicatorComparisonConfig,

    /// Допустимые операторы для условий
    pub allowed_operators: Vec<ConditionOperator>,

    /// Конфигурация вложенных индикаторов
    pub nested_config: NestedInputConfig,

    /// Может ли использоваться в первой фазе построения
    #[serde(default = "default_true")]
    pub allowed_in_phase_1: bool,

    /// Можно ли использовать с процентом от значения
    /// (например, SMA > Close * 1.02%)
    #[serde(default)]
    pub supports_percentage_condition: bool,

    /// Может ли создавать трендовые условия (RisingTrend/FallingTrend)
    #[serde(default = "default_true")]
    pub supports_trend_condition: bool,

    /// Приоритет при выборе (больше = чаще выбирается)
    #[serde(default = "default_priority")]
    pub selection_priority: f64,
}

fn default_true() -> bool {
    true
}

fn default_priority() -> f64 {
    1.0
}

impl Default for IndicatorBuildRules {
    fn default() -> Self {
        Self {
            category: BuildCategory::Trend,
            price_comparison: PriceComparisonConfig::standard(),
            constant_comparison: ConstantComparisonConfig::default(),
            indicator_comparison: IndicatorComparisonConfig::trend_and_channel(),
            allowed_operators: vec![
                ConditionOperator::GreaterThan,
                ConditionOperator::LessThan,
                ConditionOperator::CrossesAbove,
                ConditionOperator::CrossesBelow,
            ],
            nested_config: NestedInputConfig::trend(),
            allowed_in_phase_1: true,
            supports_percentage_condition: true,
            supports_trend_condition: true,
            selection_priority: 1.0,
        }
    }
}

impl IndicatorBuildRules {
    /// Правила для осциллятора
    pub fn oscillator() -> Self {
        Self {
            category: BuildCategory::Oscillator,
            price_comparison: PriceComparisonConfig::disabled(),
            constant_comparison: ConstantComparisonConfig::oscillator(),
            indicator_comparison: IndicatorComparisonConfig::only_nested_on_oscillator(),
            allowed_operators: vec![
                ConditionOperator::GreaterThan,
                ConditionOperator::LessThan,
                ConditionOperator::CrossesAbove,
                ConditionOperator::CrossesBelow,
            ],
            nested_config: NestedInputConfig::oscillator(),
            allowed_in_phase_1: true,
            supports_percentage_condition: false,
            supports_trend_condition: true,
            selection_priority: 0.5,
        }
    }

    /// Правила для RSI
    pub fn rsi() -> Self {
        let mut rules = Self::oscillator();
        rules.constant_comparison = ConstantComparisonConfig::rsi();
        rules
    }

    /// Правила для Stochastic
    pub fn stochastic() -> Self {
        let mut rules = Self::oscillator();
        rules.constant_comparison = ConstantComparisonConfig::stochastic();
        rules
    }

    /// Правила для трендового индикатора
    pub fn trend() -> Self {
        Self {
            category: BuildCategory::Trend,
            price_comparison: PriceComparisonConfig::standard(),
            constant_comparison: ConstantComparisonConfig::default(), // disabled
            indicator_comparison: IndicatorComparisonConfig::trend_and_channel(),
            allowed_operators: vec![
                ConditionOperator::GreaterThan,
                ConditionOperator::LessThan,
                ConditionOperator::CrossesAbove,
                ConditionOperator::CrossesBelow,
            ],
            nested_config: NestedInputConfig::trend(),
            allowed_in_phase_1: true,
            supports_percentage_condition: true,
            supports_trend_condition: true,
            selection_priority: 0.6,
        }
    }

    /// Правила для канального индикатора
    pub fn channel() -> Self {
        Self {
            category: BuildCategory::Channel,
            price_comparison: PriceComparisonConfig::standard(),
            constant_comparison: ConstantComparisonConfig::default(), // disabled
            indicator_comparison: IndicatorComparisonConfig::trend_and_channel(),
            allowed_operators: vec![
                ConditionOperator::GreaterThan,
                ConditionOperator::LessThan,
                ConditionOperator::CrossesAbove,
                ConditionOperator::CrossesBelow,
            ],
            nested_config: NestedInputConfig::disabled(), // Канальные обычно не вкладываются
            allowed_in_phase_1: true,
            supports_percentage_condition: true,
            supports_trend_condition: false, // Каналы обычно не используют trend conditions
            selection_priority: 0.5,
        }
    }

    /// Правила для volatility индикатора (ATR, WATR и т.д.)
    pub fn volatility() -> Self {
        Self {
            category: BuildCategory::Volatility,
            price_comparison: PriceComparisonConfig::close_only(),
            constant_comparison: ConstantComparisonConfig::volatility_percentage(),
            indicator_comparison: IndicatorComparisonConfig::disabled(),
            allowed_operators: vec![ConditionOperator::GreaterThan, ConditionOperator::LessThan],
            nested_config: NestedInputConfig::disabled(),
            allowed_in_phase_1: false, // Volatility не в первой фазе
            supports_percentage_condition: false, // Сам работает через percentage
            supports_trend_condition: false,
            selection_priority: 0.4,
        }
    }

    /// Правила для volume индикатора
    pub fn volume() -> Self {
        Self {
            category: BuildCategory::Volume,
            price_comparison: PriceComparisonConfig::disabled(),
            constant_comparison: ConstantComparisonConfig::default(),
            indicator_comparison: IndicatorComparisonConfig::disabled(),
            allowed_operators: vec![ConditionOperator::GreaterThan, ConditionOperator::LessThan],
            nested_config: NestedInputConfig::disabled(),
            allowed_in_phase_1: false, // Volume не в первой фазе
            supports_percentage_condition: false,
            supports_trend_condition: false,
            selection_priority: 0.3,
        }
    }
}

/// Реестр правил построения для индикаторов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRulesRegistry {
    /// Правила по умолчанию для каждой категории
    pub category_defaults: HashMap<BuildCategory, IndicatorBuildRules>,
    /// Переопределения для конкретных индикаторов (по имени)
    pub indicator_overrides: HashMap<String, IndicatorBuildRules>,
    /// Индикаторы, полностью исключённые из построения
    pub excluded_indicators: HashSet<String>,
    /// Глобальные настройки
    pub global_settings: GlobalBuildSettings,
}

/// Глобальные настройки построения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalBuildSettings {
    /// Допустимые поля цены по умолчанию
    pub default_price_fields: Vec<String>,
    /// Максимальная глубина вложенности индикаторов
    pub max_nesting_depth: usize,
    /// Разрешить сравнение осциллятор-осциллятор
    #[serde(default)]
    pub allow_oscillator_oscillator_comparison: bool,
}

impl Default for GlobalBuildSettings {
    fn default() -> Self {
        Self {
            default_price_fields: vec!["Close".to_string()],
            max_nesting_depth: 2,
            allow_oscillator_oscillator_comparison: false,
        }
    }
}

impl Default for BuildRulesRegistry {
    fn default() -> Self {
        let mut category_defaults = HashMap::new();

        // Трендовые индикаторы
        category_defaults.insert(BuildCategory::Trend, IndicatorBuildRules::trend());

        // Осцилляторы
        category_defaults.insert(BuildCategory::Oscillator, IndicatorBuildRules::oscillator());

        // Канальные
        category_defaults.insert(BuildCategory::Channel, IndicatorBuildRules::channel());

        // Volatility
        category_defaults.insert(BuildCategory::Volatility, IndicatorBuildRules::volatility());

        // Volume
        category_defaults.insert(BuildCategory::Volume, IndicatorBuildRules::volume());

        // Custom (по умолчанию как trend)
        category_defaults.insert(BuildCategory::Custom, IndicatorBuildRules::trend());

        // Переопределения для конкретных индикаторов
        let mut indicator_overrides = HashMap::new();

        // RSI - особые уровни
        indicator_overrides.insert("RSI".to_string(), IndicatorBuildRules::rsi());

        // Stochastic - особые уровни
        indicator_overrides.insert("Stochastic".to_string(), IndicatorBuildRules::stochastic());

        // WATR, ATR, VTRAND - volatility с процентом
        indicator_overrides.insert("WATR".to_string(), IndicatorBuildRules::volatility());
        indicator_overrides.insert("ATR".to_string(), IndicatorBuildRules::volatility());
        indicator_overrides.insert("VTRAND".to_string(), IndicatorBuildRules::volatility());
        indicator_overrides.insert("TrueRange".to_string(), IndicatorBuildRules::volatility());

        // SuperTrend - специальный трендовый
        let mut supertrend = IndicatorBuildRules::trend();
        supertrend.nested_config = NestedInputConfig::disabled(); // SuperTrend не вкладывается
        indicator_overrides.insert("SuperTrend".to_string(), supertrend);

        // Канальные индикаторы
        indicator_overrides.insert("BBUpper".to_string(), IndicatorBuildRules::channel());
        indicator_overrides.insert("BBLower".to_string(), IndicatorBuildRules::channel());
        indicator_overrides.insert("BBMiddle".to_string(), IndicatorBuildRules::channel());
        indicator_overrides.insert("KCUpper".to_string(), IndicatorBuildRules::channel());
        indicator_overrides.insert("KCLower".to_string(), IndicatorBuildRules::channel());
        indicator_overrides.insert("KCMiddle".to_string(), IndicatorBuildRules::channel());

        // Исключённые индикаторы (вспомогательные)
        let mut excluded = HashSet::new();
        excluded.insert("MAXFOR".to_string());
        excluded.insert("MINFOR".to_string());

        Self {
            category_defaults,
            indicator_overrides,
            excluded_indicators: excluded,
            global_settings: GlobalBuildSettings::default(),
        }
    }
}

impl BuildRulesRegistry {
    /// Создаёт новый реестр с настройками по умолчанию
    pub fn new() -> Self {
        Self::default()
    }

    /// Получает правила для индикатора
    pub fn get_rules(&self, indicator_name: &str, category: &str) -> &IndicatorBuildRules {
        // Сначала проверяем переопределения по имени
        if let Some(rules) = self.indicator_overrides.get(indicator_name) {
            return rules;
        }

        // Затем по категории
        let build_category = BuildCategory::from_str(category);
        self.category_defaults
            .get(&build_category)
            .unwrap_or_else(|| {
                self.category_defaults
                    .get(&BuildCategory::Custom)
                    .expect("Custom category should always exist")
            })
    }

    /// Проверяет, исключён ли индикатор
    pub fn is_excluded(&self, indicator_name: &str) -> bool {
        self.excluded_indicators.contains(indicator_name)
    }

    /// Проверяет, можно ли использовать индикатор в фазе 1
    pub fn can_use_in_phase_1(&self, indicator_name: &str, category: &str) -> bool {
        if self.is_excluded(indicator_name) {
            return false;
        }
        self.get_rules(indicator_name, category).allowed_in_phase_1
    }

    /// Проверяет, может ли индикатор сравниваться с ценой
    pub fn can_compare_with_price(&self, indicator_name: &str, category: &str) -> bool {
        self.get_rules(indicator_name, category)
            .price_comparison
            .enabled
    }

    /// Возвращает допустимые поля цены для индикатора
    pub fn allowed_price_fields(&self, indicator_name: &str, category: &str) -> &[String] {
        &self
            .get_rules(indicator_name, category)
            .price_comparison
            .allowed_price_fields
    }

    /// Проверяет, может ли индикатор сравниваться с константой
    pub fn can_compare_with_constant(&self, indicator_name: &str, category: &str) -> bool {
        self.get_rules(indicator_name, category)
            .constant_comparison
            .enabled
    }

    /// Возвращает конфигурацию константы для индикатора
    pub fn get_constant_config(
        &self,
        indicator_name: &str,
        category: &str,
    ) -> &ConstantComparisonConfig {
        &self.get_rules(indicator_name, category).constant_comparison
    }

    /// Проверяет, может ли индикатор сравниваться с другим индикатором
    pub fn can_compare_with_indicator(
        &self,
        indicator_name: &str,
        indicator_category: &str,
        other_name: &str,
        other_category: &str,
    ) -> bool {
        let rules = self.get_rules(indicator_name, indicator_category);

        if !rules.indicator_comparison.enabled {
            return false;
        }

        let other_build_category = BuildCategory::from_str(other_category);

        // Проверяем denied
        if rules
            .indicator_comparison
            .denied_categories
            .contains(&other_build_category)
        {
            return false;
        }
        if rules
            .indicator_comparison
            .denied_indicators
            .contains(&other_name.to_string())
        {
            return false;
        }

        // Проверяем allowed
        if !rules.indicator_comparison.allowed_indicators.is_empty() {
            return rules
                .indicator_comparison
                .allowed_indicators
                .contains(&other_name.to_string());
        }

        if !rules.indicator_comparison.allowed_categories.is_empty() {
            return rules
                .indicator_comparison
                .allowed_categories
                .contains(&other_build_category);
        }

        // Если allowed пустой, но enabled = true, значит специальная логика
        // (например, only_nested_on_oscillator)
        true
    }

    /// Возвращает допустимые операторы для индикатора
    pub fn allowed_operators(&self, indicator_name: &str, category: &str) -> &[ConditionOperator] {
        &self.get_rules(indicator_name, category).allowed_operators
    }

    /// Проверяет, может ли индикатор быть входом для других
    pub fn can_be_nested_input(&self, indicator_name: &str, category: &str) -> bool {
        self.get_rules(indicator_name, category)
            .nested_config
            .can_be_input
    }

    /// Проверяет, может ли индикатор принимать другие индикаторы как вход
    pub fn can_accept_nested_input(&self, indicator_name: &str, category: &str) -> bool {
        self.get_rules(indicator_name, category)
            .nested_config
            .can_accept_input
    }

    /// Проверяет совместимость двух индикаторов для сравнения
    /// Учитывает правила обоих индикаторов
    pub fn are_comparable(
        &self,
        ind1_name: &str,
        ind1_category: &str,
        ind2_name: &str,
        ind2_category: &str,
    ) -> bool {
        // Глобальное правило: осцилляторы не сравниваются друг с другом
        if !self.global_settings.allow_oscillator_oscillator_comparison
            && ind1_category == "oscillator"
            && ind2_category == "oscillator"
        {
            return false;
        }

        // Проверяем правила первого индикатора
        if !self.can_compare_with_indicator(ind1_name, ind1_category, ind2_name, ind2_category) {
            return false;
        }

        // Проверяем правила второго индикатора
        if !self.can_compare_with_indicator(ind2_name, ind2_category, ind1_name, ind1_category) {
            return false;
        }

        true
    }

    /// Добавляет переопределение для индикатора
    pub fn add_override(&mut self, indicator_name: &str, rules: IndicatorBuildRules) {
        self.indicator_overrides
            .insert(indicator_name.to_string(), rules);
    }

    /// Добавляет индикатор в список исключённых
    pub fn exclude_indicator(&mut self, indicator_name: &str) {
        self.excluded_indicators.insert(indicator_name.to_string());
    }

    /// Удаляет индикатор из списка исключённых
    pub fn include_indicator(&mut self, indicator_name: &str) {
        self.excluded_indicators.remove(indicator_name);
    }

    /// Обновляет правила категории
    pub fn update_category_rules(&mut self, category: BuildCategory, rules: IndicatorBuildRules) {
        self.category_defaults.insert(category, rules);
    }

    /// Возвращает приоритет выбора индикатора
    pub fn selection_priority(&self, indicator_name: &str, category: &str) -> f64 {
        self.get_rules(indicator_name, category).selection_priority
    }

    /// Проверяет, поддерживает ли индикатор процентные условия
    pub fn supports_percentage_condition(&self, indicator_name: &str, category: &str) -> bool {
        self.get_rules(indicator_name, category)
            .supports_percentage_condition
    }

    /// Проверяет, поддерживает ли индикатор трендовые условия
    pub fn supports_trend_condition(&self, indicator_name: &str, category: &str) -> bool {
        self.get_rules(indicator_name, category)
            .supports_trend_condition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let registry = BuildRulesRegistry::new();

        // RSI - осциллятор
        assert!(!registry.can_compare_with_price("RSI", "oscillator"));
        assert!(registry.can_compare_with_constant("RSI", "oscillator"));

        // SMA - трендовый
        assert!(registry.can_compare_with_price("SMA", "trend"));
        assert!(!registry.can_compare_with_constant("SMA", "trend"));

        // ATR - volatility
        assert!(registry.can_compare_with_constant("ATR", "volatility"));
        assert!(!registry.can_use_in_phase_1("ATR", "volatility"));

        // MAXFOR - исключён
        assert!(registry.is_excluded("MAXFOR"));
    }

    #[test]
    fn test_oscillator_comparison_rules() {
        let registry = BuildRulesRegistry::new();

        // Осцилляторы не сравниваются друг с другом
        assert!(!registry.are_comparable("RSI", "oscillator", "Stochastic", "oscillator"));

        // Трендовые могут сравниваться друг с другом
        assert!(registry.are_comparable("SMA", "trend", "EMA", "trend"));

        // Трендовые могут сравниваться с канальными
        assert!(registry.are_comparable("SMA", "trend", "BBUpper", "channel"));
    }

    #[test]
    fn test_constant_config() {
        let registry = BuildRulesRegistry::new();

        let rsi_config = registry.get_constant_config("RSI", "oscillator");
        assert!(rsi_config.enabled);
        assert_eq!(rsi_config.default_levels, Some(vec![30.0, 70.0]));

        let atr_config = registry.get_constant_config("ATR", "volatility");
        assert!(atr_config.enabled);
        assert_eq!(atr_config.constant_type, "percentage_of_price");
    }

    #[test]
    fn test_serialization() {
        let registry = BuildRulesRegistry::new();
        let json = serde_json::to_string_pretty(&registry).unwrap();
        let restored: BuildRulesRegistry = serde_json::from_str(&json).unwrap();

        assert_eq!(
            registry.excluded_indicators.len(),
            restored.excluded_indicators.len()
        );
    }
}
