//! Query Builder для различных источников данных

pub mod arrow;
pub mod clickhouse;
pub mod common;
pub mod mongodb;
pub mod postgresql;
pub mod redis;

// Re-export общих типов
pub use common::{LogicalOperator, SortDirection};

// Re-export для удобства использования
pub use arrow::{
    Aggregation, AggregationFunction, ArrowQueryBuilder, ArrowQueryUtils,
    BacktestArrowQueryBuilder, CandleArrowQueryBuilder, FilterCondition, FilterOperator,
    FilterValue, OrderBy, TradeArrowQueryBuilder,
};
pub use clickhouse::{
    // Специализированные билдеры для всех таблиц
    ClickHouseBacktestQueryBuilder,
    ClickHouseCandleQueryBuilder,
    // Базовый билдер
    ClickHouseQueryBuilder,
    ClickHouseTradeQueryBuilder,
    ClickHouseUtils,
    GeneticPopulationQueryBuilder,
    IndicatorQueryBuilder,
    OptimizationResultQueryBuilder,
    OrderQueryBuilder,
    PortfolioSnapshotQueryBuilder,
    PositionQueryBuilder,
    SignalQueryBuilder,
    StrategyMetricQueryBuilder,
    StrategyQueryBuilder as ClickHouseStrategyQueryBuilder,
    WalkForwardQueryBuilder,
};
pub use mongodb::{
    BacktestQueryBuilder as MongoDBBacktestQueryBuilder,
    CandleQueryBuilder as MongoDBCandleQueryBuilder,
    MongoDBQuery,
    MongoDBQueryBuilder,
    MongoDBUtils,
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
