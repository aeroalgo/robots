//! Примеры использования data_access слоя

pub mod clickhouse_examples;
pub mod mongodb_examples;
pub mod postgresql_examples;
pub mod redis_example;

// Re-export для удобства использования
pub use clickhouse_examples::*;
pub use mongodb_examples::*;
pub use postgresql_examples::*;
pub use redis_example::*;
