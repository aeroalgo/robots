use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use crate::condition::types::{
    ConditionCategory, ConditionConfig, ConditionError, ConditionResultData, SignalStrength,
    TrendDirection,
};
use crate::data_model::types::{Symbol, TimeFrame};

use super::stops::StopHandler;

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
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PriceField {
    Open,
    High,
    Low,
    Close,
    Volume,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataSeriesSource {
    Indicator { alias: String },
    Price { field: PriceField },
    Custom { key: String },
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

#[derive(Clone, Debug, PartialEq)]
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
    Indexed {
        source: DataSeriesSource,
        index_offset: usize,
    },
    Ohlc,
}

#[derive(Clone, Debug)]
pub struct ConditionBindingSpec {
    pub id: String,
    pub name: String,
    pub timeframe: TimeFrame,
    pub condition_name: String,
    pub parameters: HashMap<String, f32>,
    pub input: ConditionInputSpec,
    pub weight: f32,
    pub tags: Vec<String>,
    pub user_formula: Option<String>,
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
    pub condition_bindings: Vec<ConditionBindingSpec>,
    pub entry_rules: Vec<StrategyRuleSpec>,
    pub exit_rules: Vec<StrategyRuleSpec>,
    pub stop_handlers: Vec<StopHandlerSpec>,
    pub timeframe_requirements: Vec<TimeframeRequirement>,
    pub defaults: StrategyParameterMap,
    pub optimizer_hints: BTreeMap<String, StrategyParamValue>,
}

impl StrategyDefinition {
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
    pub trend: TrendDirection,
    pub quantity: Option<f64>,
    pub tags: Vec<String>,
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
pub struct ActivePosition {
    pub id: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub quantity: f64,
    pub opened_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_price: Option<f64>,
    pub unrealized_pnl: Option<f64>,
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
    pub trend: TrendDirection,
    pub weight: f32,
}

#[derive(Clone, Debug)]
pub struct UserIndicatorStep {
    pub alias: String,
    pub expression: String,
    pub timeframe: String,
    pub parameters: HashMap<String, StrategyParamValue>,
}

#[derive(Clone, Debug)]
pub struct UserConditionStep {
    pub id: String,
    pub expression: String,
    pub category: ConditionCategory,
    pub timeframe: String,
    pub parameters: HashMap<String, StrategyParamValue>,
}

#[derive(Clone, Debug)]
pub struct UserActionStep {
    pub rule_id: String,
    pub logic: RuleLogic,
    pub condition_ids: Vec<String>,
    pub signal: StrategySignalType,
    pub direction: PositionDirection,
    pub quantity: Option<f64>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct StrategyUserInput {
    pub name: String,
    pub description: Option<String>,
    pub indicators: Vec<UserIndicatorStep>,
    pub conditions: Vec<UserConditionStep>,
    pub actions: Vec<UserActionStep>,
    pub parameters: StrategyParameterMap,
    pub metadata: HashMap<String, String>,
}

impl From<ConditionResultData> for ConditionEvaluation {
    fn from(data: ConditionResultData) -> Self {
        let satisfied = data.signals.last().copied().unwrap_or(false);
        let strength = data
            .strengths
            .last()
            .copied()
            .unwrap_or(SignalStrength::Weak);
        let trend = data
            .directions
            .last()
            .copied()
            .unwrap_or(TrendDirection::Sideways);
        Self {
            condition_id: String::new(),
            satisfied,
            strength,
            trend,
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
