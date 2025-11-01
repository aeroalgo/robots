//! Query Builder для различных источников данных

pub mod arrow;
pub mod clickhouse;
pub mod mongodb;
pub mod postgresql;
pub mod redis;

// Re-export для удобства использования
pub use arrow::{
    Aggregation, AggregationFunction, ArrowQueryBuilder, ArrowQueryUtils,
    BacktestArrowQueryBuilder, CandleArrowQueryBuilder, FilterCondition, FilterOperator,
    FilterValue, LogicalOperator, OrderBy, SortDirection, TradeArrowQueryBuilder,
};
pub use clickhouse::{
    // Базовый билдер
    ClickHouseQueryBuilder, ClickHouseUtils,
    // Специализированные билдеры для всех таблиц
    ClickHouseBacktestQueryBuilder, ClickHouseCandleQueryBuilder, ClickHouseTradeQueryBuilder,
    SignalQueryBuilder, IndicatorQueryBuilder, StrategyQueryBuilder as ClickHouseStrategyQueryBuilder,
    StrategyMetricQueryBuilder, PositionQueryBuilder, OrderQueryBuilder,
    GeneticPopulationQueryBuilder, OptimizationResultQueryBuilder,
    PortfolioSnapshotQueryBuilder, WalkForwardQueryBuilder,
};
pub use mongodb::{
    BacktestQueryBuilder as MongoDBBacktestQueryBuilder,
    CandleQueryBuilder as MongoDBCandleQueryBuilder,
    MongoDBQuery,
    MongoDBQueryBuilder,
    MongoDBUtils,
    SortDirection as MongoDBSortDirection,
    // Новые билдеры для конфигураций и метаданных
    StrategyConfigQueryBuilder,
    StrategyQueryBuilder as MongoDBStrategyQueryBuilder,
    SystemConfigQueryBuilder,
    SystemMetadataQueryBuilder,
    TradeQueryBuilder as MongoDBTradeQueryBuilder,
    UserQueryBuilder as MongoDBUserQueryBuilder,
    UserSettingsQueryBuilder,
};
pub use postgresql::{
    BacktestQueryBuilder as PostgreSQLBacktestQueryBuilder,
    CandleQueryBuilder as PostgreSQLCandleQueryBuilder, PostgreSQLQueryBuilder, PostgreSQLUtils,
    StrategyQueryBuilder, TradeQueryBuilder as PostgreSQLTradeQueryBuilder, UserQueryBuilder,
};
pub use redis::*;
