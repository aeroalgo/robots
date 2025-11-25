use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use crate::condition::types::{
    ConditionConfig, ConditionError, ConditionResultData, SignalStrength,
};
use crate::data_model::types::TimeFrame;
use crate::risk::stops::StopHandler;
use crate::risk::takes::TakeHandler;
use serde::{Deserialize, Serialize};

pub type StrategyId = String;
pub type StrategyParameterMap = HashMap<String, StrategyParamValue>;
pub type StrategyUserSettings = HashMap<String, StrategyParamValue>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StrategyCategory {
    TrendFollowing,
    MeanReversion,
    Volatility,
    Arbitrage,
    MarketMaking,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum StrategyParamValue {
    Number(f64),
    Integer(i64),
    Text(String),
    Flag(bool),
    List(Vec<StrategyParamValue>),
}

impl StrategyParamValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(v) => Some(*v),
            Self::Integer(v) => Some(*v as f64),
            Self::Text(_) => None,
            Self::Flag(v) => Some(if *v { 1.0 } else { 0.0 }),
            Self::List(_) => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Flag(v) => Some(*v),
            Self::Number(v) => Some(*v != 0.0),
            Self::Integer(v) => Some(*v != 0),
            Self::Text(_) => None,
            Self::List(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(v) => Some(v.as_str()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrategyParameterSpec {
    pub name: String,
    pub description: Option<String>,
    pub default_value: StrategyParamValue,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub discrete_values: Option<Vec<StrategyParamValue>>,
    pub optimize: bool,
}

#[derive(Clone, Debug)]
pub struct StopHandlerSpec {
    pub id: String,
    pub name: String,
    pub handler_name: String,
    pub timeframe: TimeFrame,
    pub price_field: PriceField,
    pub parameters: StrategyParameterMap,
    pub direction: PositionDirection,
    pub priority: i32,
    pub tags: Vec<String>,
    pub target_entry_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TakeHandlerSpec {
    pub id: String,
    pub name: String,
    pub handler_name: String,
    pub timeframe: TimeFrame,
    pub price_field: PriceField,
    pub parameters: StrategyParameterMap,
    pub direction: PositionDirection,
    pub priority: i32,
    pub tags: Vec<String>,
    pub target_entry_ids: Vec<String>,
}

#[derive(Clone)]
pub struct PreparedStopHandler {
    pub id: String,
    pub name: String,
    pub handler: Arc<dyn StopHandler>,
    pub timeframe: TimeFrame,
    pub price_field: PriceField,
    pub direction: PositionDirection,
    pub priority: i32,
    pub tags: Vec<String>,
    pub target_entry_ids: Vec<String>,
}

#[derive(Clone)]
pub struct PreparedTakeHandler {
    pub id: String,
    pub name: String,
    pub handler: Arc<dyn TakeHandler>,
    pub timeframe: TimeFrame,
    pub price_field: PriceField,
    pub direction: PositionDirection,
    pub priority: i32,
    pub tags: Vec<String>,
    pub target_entry_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PriceField {
    Open,
    High,
    Low,
    Close,
    Volume,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSeriesSource {
    Indicator {
        alias: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeframe: Option<TimeFrame>,
    },
    Price {
        field: PriceField,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeframe: Option<TimeFrame>,
    },
    Custom {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeframe: Option<TimeFrame>,
    },
}

impl DataSeriesSource {
    pub fn indicator(alias: impl Into<String>) -> Self {
        Self::Indicator {
            alias: alias.into(),
            timeframe: None,
        }
    }

    pub fn indicator_with_timeframe(alias: impl Into<String>, timeframe: TimeFrame) -> Self {
        Self::Indicator {
            alias: alias.into(),
            timeframe: Some(timeframe),
        }
    }

    pub fn price(field: PriceField) -> Self {
        Self::Price {
            field,
            timeframe: None,
        }
    }

    pub fn price_with_timeframe(field: PriceField, timeframe: TimeFrame) -> Self {
        Self::Price {
            field,
            timeframe: Some(timeframe),
        }
    }

    pub fn custom(key: impl Into<String>) -> Self {
        Self::Custom {
            key: key.into(),
            timeframe: None,
        }
    }

    pub fn custom_with_timeframe(key: impl Into<String>, timeframe: TimeFrame) -> Self {
        Self::Custom {
            key: key.into(),
            timeframe: Some(timeframe),
        }
    }

    pub fn timeframe(&self) -> Option<&TimeFrame> {
        match self {
            Self::Indicator { timeframe, .. } => timeframe.as_ref(),
            Self::Price { timeframe, .. } => timeframe.as_ref(),
            Self::Custom { timeframe, .. } => timeframe.as_ref(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IndicatorSourceSpec {
    Registry {
        name: String,
        parameters: HashMap<String, f32>,
    },
    Formula {
        expression: String,
    },
}

#[derive(Clone, Debug)]
pub struct IndicatorBindingSpec {
    pub alias: String,
    pub timeframe: TimeFrame,
    pub source: IndicatorSourceSpec,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserFormulaMetadata {
    pub id: String,
    pub name: String,
    pub expression: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub inputs: Vec<String>,
}

impl UserFormulaMetadata {
    pub fn as_indicator_binding(
        &self,
        alias: impl Into<String>,
        timeframe: TimeFrame,
    ) -> IndicatorBindingSpec {
        IndicatorBindingSpec {
            alias: alias.into(),
            timeframe,
            source: IndicatorSourceSpec::Formula {
                expression: self.expression.clone(),
            },
            tags: self.tags.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConditionInputSpec {
    Single {
        source: DataSeriesSource,
    },
    Dual {
        primary: DataSeriesSource,
        secondary: DataSeriesSource,
    },
    DualWithPercent {
        primary: DataSeriesSource,
        secondary: DataSeriesSource,
        percent: f32,
    },
    Range {
        source: DataSeriesSource,
        lower: DataSeriesSource,
        upper: DataSeriesSource,
    },
    Indexed {
        source: DataSeriesSource,
        index_offset: usize,
    },
    Ohlc,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConditionOperator {
    GreaterThan,
    LessThan,
    CrossesAbove,
    CrossesBelow,
    Between,
}

impl ConditionOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GreaterThan => "greater_than",
            Self::LessThan => "less_than",
            Self::CrossesAbove => "crosses_above",
            Self::CrossesBelow => "crosses_below",
            Self::Between => "between",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConditionOperandSpec {
    Series(DataSeriesSource),
    Scalar(f32),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConditionDeclarativeSpec {
    pub operator: ConditionOperator,
    pub operands: Vec<ConditionOperandSpec>,
    pub description: Option<String>,
}

impl ConditionDeclarativeSpec {
    pub fn new(operator: ConditionOperator, operands: Vec<ConditionOperandSpec>) -> Self {
        Self {
            operator,
            operands,
            description: None,
        }
    }

    pub fn from_input(operator: ConditionOperator, input: &ConditionInputSpec) -> Self {
        let mut operands = Vec::new();
        match input {
            ConditionInputSpec::Single { source } => {
                operands.push(ConditionOperandSpec::Series(source.clone()));
            }
            ConditionInputSpec::Dual { primary, secondary } => {
                operands.push(ConditionOperandSpec::Series(primary.clone()));
                operands.push(ConditionOperandSpec::Series(secondary.clone()));
            }
            ConditionInputSpec::DualWithPercent {
                primary,
                secondary,
                percent,
            } => {
                operands.push(ConditionOperandSpec::Series(primary.clone()));
                operands.push(ConditionOperandSpec::Series(secondary.clone()));
                operands.push(ConditionOperandSpec::Scalar(*percent));
            }
            ConditionInputSpec::Range {
                source,
                lower,
                upper,
            } => {
                operands.push(ConditionOperandSpec::Series(source.clone()));
                operands.push(ConditionOperandSpec::Series(lower.clone()));
                operands.push(ConditionOperandSpec::Series(upper.clone()));
            }
            ConditionInputSpec::Indexed { source, .. } => {
                operands.push(ConditionOperandSpec::Series(source.clone()));
            }
            ConditionInputSpec::Ohlc => {}
        }
        Self::new(operator, operands)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConditionBindingSpec {
    pub id: String,
    pub name: String,
    pub timeframe: TimeFrame,
    pub declarative: ConditionDeclarativeSpec,
    pub parameters: HashMap<String, f32>,
    pub input: ConditionInputSpec,
    pub weight: f32,
    pub tags: Vec<String>,
    pub user_formula: Option<String>,
}

impl ConditionBindingSpec {
    pub fn factory_name(&self) -> &'static str {
        // Проверяем, является ли это трендовым условием (есть параметр "period" и Single input)
        let is_trend_condition = self.parameters.contains_key("period")
            && matches!(self.input, ConditionInputSpec::Single { .. });

        match self.declarative.operator {
            ConditionOperator::GreaterThan => {
                if is_trend_condition {
                    "RISINGTREND"
                } else {
                    match self.input {
                        ConditionInputSpec::DualWithPercent { .. } => "GREATERPERCENT",
                        _ => "ABOVE",
                    }
                }
            }
            ConditionOperator::LessThan => {
                if is_trend_condition {
                    "FALLINGTREND"
                } else {
                    match self.input {
                        ConditionInputSpec::DualWithPercent { .. } => "LOWERPERCENT",
                        _ => "BELOW",
                    }
                }
            }
            ConditionOperator::CrossesAbove => {
                // Для трендовых условий CrossesAbove не поддерживается, преобразуем в RisingTrend
                if is_trend_condition {
                    "RISINGTREND"
                } else {
                    "CROSSESABOVE"
                }
            }
            ConditionOperator::CrossesBelow => {
                // Для трендовых условий CrossesBelow не поддерживается, преобразуем в FallingTrend
                if is_trend_condition {
                    "FALLINGTREND"
                } else {
                    "CROSSESBELOW"
                }
            }
            ConditionOperator::Between => "BETWEEN",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuleLogic {
    All,
    Any,
    AtLeast(usize),
    Weighted { min_total: f32 },
    Expression(String),
}

#[derive(Clone, Debug)]
pub struct StrategyRuleSpec {
    pub id: String,
    pub name: String,
    pub logic: RuleLogic,
    pub conditions: Vec<String>,
    pub signal: StrategySignalType,
    pub direction: PositionDirection,
    pub quantity: Option<f64>,
    pub tags: Vec<String>,
    pub position_group: Option<String>,
    pub target_entry_ids: Vec<String>,
}

impl StrategyRuleSpec {
    pub fn position_group_key(&self) -> String {
        self.position_group
            .clone()
            .unwrap_or_else(|| self.id.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TimeframeRequirement {
    pub alias: String,
    pub timeframe: TimeFrame,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StrategyMetadata {
    pub id: StrategyId,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub categories: Vec<StrategyCategory>,
    pub tags: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl StrategyMetadata {
    pub fn with_id(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: None,
            author: None,
            categories: Vec::new(),
            tags: Vec::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StrategyDefinition {
    pub metadata: StrategyMetadata,
    pub parameters: Vec<StrategyParameterSpec>,
    pub indicator_bindings: Vec<IndicatorBindingSpec>,
    pub formulas: Vec<UserFormulaMetadata>,
    pub condition_bindings: Vec<ConditionBindingSpec>,
    pub entry_rules: Vec<StrategyRuleSpec>,
    pub exit_rules: Vec<StrategyRuleSpec>,
    pub stop_handlers: Vec<StopHandlerSpec>,
    pub take_handlers: Vec<TakeHandlerSpec>,
    pub timeframe_requirements: Vec<TimeframeRequirement>,
    pub defaults: StrategyParameterMap,
    pub optimizer_hints: BTreeMap<String, StrategyParamValue>,
}

impl StrategyDefinition {
    pub fn new(
        metadata: StrategyMetadata,
        parameters: Vec<StrategyParameterSpec>,
        indicator_bindings: Vec<IndicatorBindingSpec>,
        formulas: Vec<UserFormulaMetadata>,
        condition_bindings: Vec<ConditionBindingSpec>,
        entry_rules: Vec<StrategyRuleSpec>,
        exit_rules: Vec<StrategyRuleSpec>,
        stop_handlers: Vec<StopHandlerSpec>,
        take_handlers: Vec<TakeHandlerSpec>,
        defaults: StrategyParameterMap,
        optimizer_hints: BTreeMap<String, StrategyParamValue>,
    ) -> Self {
        let timeframe_requirements =
            Self::timeframe_requirements_from_indicators(&indicator_bindings);
        Self {
            metadata,
            parameters,
            indicator_bindings,
            formulas,
            condition_bindings,
            entry_rules,
            exit_rules,
            stop_handlers,
            take_handlers,
            timeframe_requirements,
            defaults,
            optimizer_hints,
        }
    }

    pub fn all_timeframes(&self) -> HashSet<TimeFrame> {
        let mut set = HashSet::new();
        for req in &self.timeframe_requirements {
            set.insert(req.timeframe.clone());
        }
        for binding in &self.indicator_bindings {
            set.insert(binding.timeframe.clone());
        }
        for binding in &self.condition_bindings {
            set.insert(binding.timeframe.clone());
        }
        for handler in &self.stop_handlers {
            set.insert(handler.timeframe.clone());
        }
        set
    }

    pub fn formula_by_id(&self, id: &str) -> Option<&UserFormulaMetadata> {
        self.formulas.iter().find(|formula| formula.id == id)
    }

    fn timeframe_requirements_from_indicators(
        indicators: &[IndicatorBindingSpec],
    ) -> Vec<TimeframeRequirement> {
        indicators
            .iter()
            .map(|binding| TimeframeRequirement {
                alias: binding.alias.clone(),
                timeframe: binding.timeframe.clone(),
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StrategySignalType {
    Entry,
    Exit,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StopSignalKind {
    StopLoss,
    TakeProfit,
    Trailing,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PositionDirection {
    Long,
    Short,
    Flat,
    Both,
}

#[derive(Clone, Debug)]
pub struct StrategySignal {
    pub rule_id: String,
    pub signal_type: StrategySignalType,
    pub direction: PositionDirection,
    pub timeframe: TimeFrame,
    pub strength: SignalStrength,
    pub quantity: Option<f64>,
    pub entry_rule_id: Option<String>,
    pub tags: Vec<String>,
    pub position_group: Option<String>,
    pub target_entry_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct StopSignal {
    pub handler_id: String,
    pub signal: StrategySignal,
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub priority: i32,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct StrategyDecision {
    pub entries: Vec<StrategySignal>,
    pub exits: Vec<StrategySignal>,
    pub stop_signals: Vec<StopSignal>,
    pub custom: Vec<StrategySignal>,
    pub metadata: HashMap<String, String>,
}

impl StrategyDecision {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
            exits: Vec::new(),
            stop_signals: Vec::new(),
            custom: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
            && self.exits.is_empty()
            && self.stop_signals.is_empty()
            && self.custom.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StrategyError {
    #[error("timeframe data not found: {0:?}")]
    MissingTimeframe(TimeFrame),
    #[error("indicator data not found: alias={alias}, timeframe={timeframe:?}")]
    MissingIndicator { alias: String, timeframe: TimeFrame },
    #[error("custom data not found: key={key}, timeframe={timeframe:?}")]
    MissingCustomData { key: String, timeframe: TimeFrame },
    #[error("price series not available: {field:?} {timeframe:?}")]
    MissingPriceSeries {
        field: PriceField,
        timeframe: TimeFrame,
    },
    #[error("condition evaluation failed: {condition_id}")]
    ConditionFailure {
        condition_id: String,
        source: ConditionError,
    },
    #[error("rule references unknown condition: {rule_id} -> {condition_id}")]
    UnknownConditionReference {
        rule_id: String,
        condition_id: String,
    },
    #[error("unsupported rule logic: {0}")]
    UnsupportedRuleLogic(String),
    #[error("strategy definition error: {0}")]
    DefinitionError(String),
}

#[derive(Clone, Debug)]
pub struct ConditionEvaluation {
    pub condition_id: String,
    pub satisfied: bool,
    pub strength: SignalStrength,
    pub weight: f32,
}

impl From<ConditionResultData> for ConditionEvaluation {
    fn from(data: ConditionResultData) -> Self {
        let satisfied = data.signals.last().copied().unwrap_or(false);
        let strength = data
            .strengths
            .last()
            .copied()
            .unwrap_or(SignalStrength::Weak);
        Self {
            condition_id: String::new(),
            satisfied,
            strength,
            weight: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct PreparedCondition {
    pub id: String,
    pub condition: Arc<dyn crate::condition::Condition + Send + Sync>,
    pub input: ConditionInputSpec,
    pub timeframe: TimeFrame,
    pub weight: f32,
    pub metadata: Option<ConditionConfig>,
    pub tags: Vec<String>,
}

impl fmt::Debug for PreparedCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedCondition")
            .field("id", &self.id)
            .field("input", &self.input)
            .field("timeframe", &self.timeframe)
            .field("weight", &self.weight)
            .field("metadata", &self.metadata)
            .field("tags", &self.tags)
            .finish()
    }
}

impl PreparedCondition {
    pub fn weight(&self) -> f32 {
        if self.weight <= 0.0 {
            1.0
        } else {
            self.weight
        }
    }
}
