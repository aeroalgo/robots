pub mod collector;
pub mod condition;
pub mod config;
pub mod engine;
pub mod indicator;
pub mod stop_handler;
pub mod strategy_converter;
pub mod timeframe;
pub mod types;

// Реэкспорт для удобства использования
pub use collector::IndicatorInfoCollector;
pub use condition::ConditionCombinationGenerator;
pub use config::StrategyDiscoveryConfig;
pub use engine::{StrategyCandidate, StrategyDiscoveryEngine};
pub use indicator::IndicatorCombinationGenerator;
pub use stop_handler::StopHandlerCombinationGenerator;
pub use strategy_converter::{StrategyConversionError, StrategyConverter};
pub use timeframe::TimeFrameGenerator;
pub use types::{
    ConditionInfo, ConditionParamInfo, IndicatorCombination, IndicatorInfo, IndicatorParamInfo,
    NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::types::TimeFrame;

    #[test]
    fn test_timeframe_generator() {
        let base = TimeFrame::Minutes(60);
        let combinations = TimeFrameGenerator::generate_combinations(base, 3, 1440);

        assert!(!combinations.is_empty());
        // Должны быть комбинации из 3 таймфреймов: 60, 120, 180
    }
}
