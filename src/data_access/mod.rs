//! Data Access Layer
//!
//! Этот модуль предоставляет унифицированный интерфейс для работы с различными источниками данных:
//! - Базы данных (ClickHouse, PostgreSQL, MongoDB)
//! - Кэш (Redis)
//! - Очереди сообщений (Kafka)
//! - Высокопроизводительная обработка данных (Arrow/Parquet)
//! - API коннекторы к биржам

pub mod api;
pub mod database;
pub mod examples;
pub mod models;
pub mod query_builder;
pub mod traits;

// Re-export основных типов для удобства использования
pub use models::*;
pub use traits::*;

// Общие типы для data_access слоя
pub type Result<T> = std::result::Result<T, DataAccessError>;

/// Основной тип ошибок для data_access слоя
#[derive(thiserror::Error, Debug)]
pub enum DataAccessError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Message queue error: {0}")]
    MessageQueue(String),

    #[error("Arrow/Parquet error: {0}")]
    Arrow(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}
