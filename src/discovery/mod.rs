pub mod collector;
pub mod condition;
pub mod config;
pub mod engine;
pub mod strategy_converter;
pub mod types;

// Реэкспорт для удобства использования
pub use collector::IndicatorInfoCollector;
pub use condition::ConditionCombinationGenerator;
pub use config::StrategyDiscoveryConfig;
pub use engine::{StrategyCandidate, StrategyDiscoveryEngine};
pub use strategy_converter::{StrategyConversionError, StrategyConverter};
pub use types::{
    ConditionInfo, ConditionParamInfo, IndicatorCombination, IndicatorInfo, IndicatorParamInfo,
    NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};
