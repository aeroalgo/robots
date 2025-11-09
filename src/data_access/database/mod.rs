//! Database коннекторы

pub mod arrow_flight;
pub mod clickhouse;
pub mod datafusion;
pub mod mongodb;
pub mod parquet;
pub mod postgresql;
pub mod redis;

// Re-export для удобства использования
pub use arrow_flight::{
    ArrowFlightConfig, ArrowFlightConnector, ArrowFlightUtils, CandleArrowFlightConnector,
    TradeArrowFlightConnector,
};
pub use clickhouse::{
    BacktestRecord,
    ClickHouseConfig,
    ClickHouseConnector,
    ClickHouseTransaction,
    GeneticIndividual,
    Indicator,
    // Модели данных
    OhlcvData,
    OptimizationResult,
    OrderRecord,
    PortfolioSnapshot,
    Position,
    Signal,
    Strategy,
    StrategyMetric,
    SymbolInfo,
    TickData,
    TradeRecord,
    WalkForwardResult,
};
pub use datafusion::{
    AnalyticsQuery, BacktestAnalyticsConnector, CandleAnalyticsConnector, DataFusionConfig,
    DataFusionConnector, DataFusionUtils, TableStats,
};
pub use mongodb::{MongoDBConfig, MongoDBConnector, MongoDBTransaction, MongoDBUtils};
pub use parquet::{
    BacktestParquetConnector, CandleParquetConnector, ParquetCompression, ParquetConfig,
    ParquetConnector, ParquetMetadata, ParquetUtils,
};
pub use postgresql::{
    PostgreSQLConfig, PostgreSQLConnector, PostgreSQLTransaction, PostgreSQLUtils,
};
pub use redis::RedisConnector;
