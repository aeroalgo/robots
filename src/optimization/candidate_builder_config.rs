use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBuilderConfig {
    pub probabilities: ElementProbabilities,
    pub constraints: ElementConstraints,
    pub condition_config: ConditionConfig,
    pub rules: BuildRules,
}

impl Default for CandidateBuilderConfig {
    fn default() -> Self {
        Self {
            probabilities: ElementProbabilities::default(),
            constraints: ElementConstraints::default(),
            condition_config: ConditionConfig::default(),
            rules: BuildRules::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementProbabilities {
    pub indicators: IndicatorProbabilities,
    pub conditions: ConditionProbabilities,
    pub stop_handlers: StopHandlerProbabilities,
    pub take_handlers: TakeHandlerProbabilities,
    pub timeframes: TimeframeProbabilities,
    pub nested_indicators: NestedIndicatorProbabilities,
    pub phases: PhaseProbabilities,
}

impl Default for ElementProbabilities {
    fn default() -> Self {
        Self {
            indicators: IndicatorProbabilities::default(),
            conditions: ConditionProbabilities::default(),
            stop_handlers: StopHandlerProbabilities::default(),
            take_handlers: TakeHandlerProbabilities::default(),
            timeframes: TimeframeProbabilities::default(),
            nested_indicators: NestedIndicatorProbabilities::default(),
            phases: PhaseProbabilities::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorProbabilities {
    pub add_base_indicator: f64,
    pub add_trend_indicator: f64,
    pub add_oscillator_indicator: f64,
    pub add_volume_indicator: f64,
    pub add_volatility_indicator: f64,
    pub add_channel_indicator: f64,
}

impl Default for IndicatorProbabilities {
    fn default() -> Self {
        Self {
            add_base_indicator: 0.8,
            add_trend_indicator: 0.6,
            add_oscillator_indicator: 0.5,
            add_volume_indicator: 0.3,
            add_volatility_indicator: 0.4,
            add_channel_indicator: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionProbabilities {
    pub add_entry_condition: f64,
    pub use_indicator_price_condition: f64,
    pub use_indicator_indicator_condition: f64,
    pub use_crosses_operator: f64,
    pub use_trend_condition: f64,
    pub use_percent_condition: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProbabilities {
    /// Вероятность продолжить сборку после первой фазы (фазы 2, 3, 4...)
    pub continue_building: f64,
    /// Вероятность добавить exit rules в фазе
    pub add_exit_rules: f64,
}

impl Default for PhaseProbabilities {
    fn default() -> Self {
        Self {
            continue_building: 0.6,
            add_exit_rules: 0.4,
        }
    }
}

impl Default for ConditionProbabilities {
    fn default() -> Self {
        Self {
            add_entry_condition: 0.7,
            use_indicator_price_condition: 0.7,
            use_indicator_indicator_condition: 0.5,
            use_crosses_operator: 0.4,
            use_trend_condition: 0.2,
            use_percent_condition: 0.15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopHandlerProbabilities {
    pub add_stop_loss: f64,
}

impl Default for StopHandlerProbabilities {
    fn default() -> Self {
        Self { add_stop_loss: 0.9 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeHandlerProbabilities {
    pub add_take_profit: f64,
}

impl Default for TakeHandlerProbabilities {
    fn default() -> Self {
        Self {
            add_take_profit: 0.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeProbabilities {
    pub use_base_timeframe: f64,
    pub use_higher_timeframe: f64,
    pub use_multiple_timeframes: f64,
}

impl Default for TimeframeProbabilities {
    fn default() -> Self {
        Self {
            use_base_timeframe: 1.0,
            use_higher_timeframe: 0.5,
            use_multiple_timeframes: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedIndicatorProbabilities {
    pub add_nested_indicator: f64,
    pub max_nesting_depth: usize,
}

impl Default for NestedIndicatorProbabilities {
    fn default() -> Self {
        Self {
            add_nested_indicator: 0.3,
            max_nesting_depth: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConstraints {
    pub min_indicators: usize,
    pub max_indicators: usize,
    pub min_entry_conditions: usize,
    pub max_entry_conditions: usize,
    pub min_exit_conditions: usize,
    pub max_exit_conditions: usize,
    pub min_stop_handlers: usize,
    pub max_stop_handlers: usize,
    pub min_take_handlers: usize,
    pub max_take_handlers: usize,
    pub min_timeframes: usize,
    pub max_timeframes: usize,
}

impl Default for ElementConstraints {
    fn default() -> Self {
        Self {
            min_indicators: 1,
            max_indicators: 4,
            min_entry_conditions: 1,
            max_entry_conditions: 4,
            min_exit_conditions: 0,
            max_exit_conditions: 2,
            min_stop_handlers: 1,
            max_stop_handlers: 1,
            min_take_handlers: 0,
            max_take_handlers: 1,
            min_timeframes: 1,
            max_timeframes: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionConfig {
    /// Поля цены, из которых случайно выбирается для условий типа indicator_price (по умолчанию только Close)
    #[serde(default = "default_price_fields")]
    pub price_fields: Vec<String>,
}

fn default_price_fields() -> Vec<String> {
    vec!["Close".to_string()]
}

impl Default for ConditionConfig {
    fn default() -> Self {
        Self {
            price_fields: vec!["Close".to_string()],
        }
    }
}

impl CandidateBuilderConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: CandidateBuilderConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRules {
    pub dependencies: Vec<DependencyRule>,
    pub exclusions: Vec<ExclusionRule>,
    pub conditions: Vec<ConditionalRule>,
    pub indicator_parameter_rules: Vec<IndicatorParameterRule>,
    /// Список индикаторов, которые исключены из выборки для оптимизации
    pub excluded_indicators: Vec<String>,
}

impl Default for BuildRules {
    fn default() -> Self {
        Self {
            dependencies: vec![DependencyRule {
                trigger: ElementSelector::stop_handler("StopLossPct"),
                required: ElementSelector::take_handler("TakeProfitPct"),
                strict: true,
            }],
            exclusions: vec![ExclusionRule {
                element: ElementSelector::stop_handler("ATRTrailStop"),
                excluded: ElementSelector::take_handler("TakeProfitPct"),
            }],
            conditions: Vec::new(),
            indicator_parameter_rules: vec![
                IndicatorParameterRule {
                    indicator_type: "volatility".to_string(),
                    indicator_names: Vec::new(),
                    price_field_constraint: Some(PriceFieldConstraint {
                        required_price_field: "Close".to_string(),
                        parameter_constraint: ParameterConstraint::PercentageFromPrice {
                            min_percent: 0.2,
                            max_percent: 10.0,
                            step: 0.1,
                        },
                    }),
                },
                IndicatorParameterRule {
                    indicator_type: "oscillator".to_string(),
                    indicator_names: Vec::new(),
                    price_field_constraint: None,
                },
            ],
            excluded_indicators: vec!["MAXFOR".to_string(), "MINFOR".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRule {
    pub trigger: ElementSelector,
    pub required: ElementSelector,
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionRule {
    pub element: ElementSelector,
    pub excluded: ElementSelector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRule {
    pub condition: RuleCondition,
    pub action: RuleAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuleCondition {
    And { conditions: Vec<ElementSelector> },
    Or { conditions: Vec<ElementSelector> },
    Not { condition: Box<ElementSelector> },
    Element { element: ElementSelector },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RuleAction {
    Require {
        element: ElementSelector,
        strict: bool,
    },
    Exclude {
        element: ElementSelector,
    },
    SetProbability {
        element: ElementSelector,
        probability: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum ElementSelector {
    Indicator { name: String },
    StopHandler { name: String },
    TakeHandler { name: String },
    Condition { condition_type: String },
    Timeframe { timeframe: String },
    AnyIndicator,
    AnyStopHandler,
    AnyTakeHandler,
    AnyCondition,
}

impl ElementSelector {
    pub fn stop_handler(name: impl Into<String>) -> Self {
        Self::StopHandler { name: name.into() }
    }

    pub fn take_handler(name: impl Into<String>) -> Self {
        Self::TakeHandler { name: name.into() }
    }

    pub fn indicator(name: impl Into<String>) -> Self {
        Self::Indicator { name: name.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorParameterRule {
    pub indicator_type: String,
    pub indicator_names: Vec<String>,
    pub price_field_constraint: Option<PriceFieldConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFieldConstraint {
    pub required_price_field: String,
    pub parameter_constraint: ParameterConstraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParameterConstraint {
    PercentageFromPrice {
        min_percent: f64,
        max_percent: f64,
        step: f64,
    },
    FixedRange {
        min_value: f64,
        max_value: f64,
        step: f64,
    },
}
