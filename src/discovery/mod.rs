pub mod collector;
pub mod condition;
pub mod config;
pub mod engine;
pub mod indicator;
pub mod strategy_converter;
pub mod timeframe;
pub mod types;

// Реэкспорт для удобства использования
pub use collector::IndicatorInfoCollector;
pub use condition::ConditionCombinationGenerator;
pub use config::StrategyDiscoveryConfig;
pub use engine::{StrategyCandidate, StrategyDiscoveryEngine};
pub use indicator::IndicatorCombinationGenerator;
pub use strategy_converter::{StrategyConversionError, StrategyConverter};
pub use timeframe::TimeFrameGenerator;
pub use types::{
    ConditionInfo, ConditionParamInfo, IndicatorCombination, IndicatorInfo, IndicatorParamInfo,
    NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};
