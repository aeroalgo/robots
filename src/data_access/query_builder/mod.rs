//! Query Builder для различных источников данных

pub mod clickhouse;
pub mod mongodb;
pub mod postgresql;
pub mod redis;

// Re-export для удобства использования
pub use clickhouse::{
    BacktestQueryBuilder as ClickHouseBacktestQueryBuilder,
    CandleQueryBuilder as ClickHouseCandleQueryBuilder, ClickHouseQueryBuilder, ClickHouseUtils,
    TradeQueryBuilder as ClickHouseTradeQueryBuilder,
};
pub use mongodb::{
    BacktestQueryBuilder as MongoDBBacktestQueryBuilder,
    CandleQueryBuilder as MongoDBCandleQueryBuilder,
    MongoDBQuery,
    MongoDBQueryBuilder,
    MongoDBUtils,
    SortDirection,
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
