//! Data Access Layer
//!
//! Этот модуль предоставляет унифицированный интерфейс для работы с различными источниками данных:
//! - Базы данных (ClickHouse, PostgreSQL, MongoDB)
//! - Кэш (Redis)
//! - Очереди сообщений (Kafka)
//! - Высокопроизводительная обработка данных (Arrow Flight, Parquet, DataFusion)
//! - API коннекторы к биржам
//!
//! ## Arrow/Parquet инфраструктура
//!
//! ### Arrow Flight Server (Port 8815)
//! - Высокоскоростная передача данных между компонентами
//! - Нулевое копирование данных, векторизованные операции
//! - Поддержка сжатия и оптимизации
//!
//! ### Parquet Files (Port 8816)
//! - Колоночное хранение исторических данных
//! - Высокое сжатие, быстрые аналитические запросы
//! - Поддержка различных типов сжатия (Snappy, Gzip, LZ4, Zstd)
//!
//! ### DataFusion (Port 8817)
//! - SQL-запросы к Arrow/Parquet данным
//! - Быстрые агрегации и фильтрация данных
//! - Поддержка аналитических запросов
//!
//! ## Архитектура хранения данных
//!
//! ```
//! Redis (L1 Cache) -> Arrow Flight (L2 Transfer) -> Parquet (L3 Storage) -> ClickHouse (L4 Analytics)
//! ```
//!
//! - **Redis**: Горячие данные, кэш, сессии
//! - **Arrow Flight**: Высокоскоростная передача данных
//! - **Parquet**: Архивное хранение, промежуточные вычисления
//! - **ClickHouse**: Основное хранилище для аналитики
//! - **DataFusion**: SQL-запросы к Arrow/Parquet данным

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

impl From<arrow::error::ArrowError> for DataAccessError {
    fn from(value: arrow::error::ArrowError) -> Self {
        DataAccessError::Arrow(value.to_string())
    }
}
