//! Базовые трейты для унификации работы с источниками данных

use crate::data_access::models::*;
use crate::data_access::{DataAccessError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Базовый трейт для всех источников данных
#[async_trait]
pub trait DataSource {
    /// Тип ошибки для конкретного источника данных
    type Error: Into<DataAccessError>;

    /// Подключение к источнику данных
    async fn connect(&mut self) -> Result<()>;

    /// Отключение от источника данных
    async fn disconnect(&mut self) -> Result<()>;

    /// Проверка состояния подключения
    fn is_connected(&self) -> bool;

    /// Получение информации о подключении
    fn connection_info(&self) -> ConnectionInfo;
}

/// Информация о подключении
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
    pub status: ConnectionStatus,
}

/// Статус подключения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

/// Трейт для кэширования данных
#[async_trait]
pub trait Cache {
    /// Получение значения по ключу
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;

    /// Установка значения с опциональным TTL
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Удаление значения по ключу
    async fn delete(&self, key: &str) -> Result<()>;

    /// Проверка существования ключа
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Установка TTL для существующего ключа
    async fn expire(&self, key: &str, ttl: u64) -> Result<()>;

    /// Получение всех ключей по паттерну
    async fn keys(&self, pattern: &str) -> Result<Vec<String>>;
}

/// Трейт для работы с базами данных
#[async_trait]
pub trait Database {
    /// Выполнение запроса с возвратом результатов
    async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;

    /// Выполнение запроса без возврата результатов
    async fn execute(&self, query: &str) -> Result<u64>;

    /// Выполнение запроса с параметрами
    async fn query_with_params<T>(&self, query: &str, params: &[&dyn ToSql]) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;

    /// Выполнение запроса с параметрами без возврата результатов
    async fn execute_with_params(&self, query: &str, params: &[&dyn ToSql]) -> Result<u64>;

    /// Начало транзакции
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction + Send + Sync>>;

    /// Проверка подключения
    async fn ping(&self) -> Result<()>;
}

/// Трейт для работы с транзакциями (упрощенный)
#[async_trait]
pub trait Transaction {
    /// Выполнение запроса в транзакции без возврата результатов
    async fn execute(&self, query: &str) -> Result<u64>;

    /// Подтверждение транзакции
    async fn commit(self: Box<Self>) -> Result<()>;

    /// Откат транзакции
    async fn rollback(self: Box<Self>) -> Result<()>;
}

/// Трейт для работы с очередями сообщений
#[async_trait]
pub trait MessageQueue {
    /// Отправка сообщения в очередь
    async fn send<T>(&self, topic: &str, message: &T) -> Result<()>
    where
        T: Serialize + Send + Sync;

    /// Получение сообщений из очереди
    async fn receive<T>(&self, topic: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync;

    /// Подписка на топик
    async fn subscribe<T>(&self, topic: &str) -> Result<Box<dyn MessageStream<T>>>;

    /// Создание топика
    async fn create_topic(&self, topic: &str) -> Result<()>;

    /// Удаление топика
    async fn delete_topic(&self, topic: &str) -> Result<()>;
}

/// Трейт для работы с потоком сообщений
#[async_trait]
pub trait MessageStream<T> {
    /// Получение следующего сообщения
    async fn next(&mut self) -> Result<Option<T>>;

    /// Отписка от потока
    async fn unsubscribe(self: Box<Self>) -> Result<()>;
}

/// Трейт для работы с Arrow данными
#[async_trait]
pub trait ArrowDataSource {
    /// Получение данных в виде RecordBatch
    async fn get_data(&self, query: &str) -> Result<Vec<arrow::record_batch::RecordBatch>>;

    /// Отправка данных в виде RecordBatch
    async fn send_data(&self, batches: Vec<arrow::record_batch::RecordBatch>) -> Result<()>;

    /// Получение схемы данных
    async fn get_schema(&self, query: &str) -> Result<String>;

    /// Выполнение действия
    async fn do_action(&self, action: arrow_flight::Action) -> Result<Vec<u8>>;
}

/// Трейт для работы с Parquet файлами
#[async_trait]
pub trait ParquetDataSource {
    /// Чтение Parquet файла
    async fn read_parquet(&self, path: &str) -> Result<Vec<arrow::record_batch::RecordBatch>>;

    /// Запись в Parquet файл
    async fn write_parquet(
        &self,
        path: &str,
        batches: Vec<arrow::record_batch::RecordBatch>,
    ) -> Result<()>;

    /// Получение схемы Parquet файла
    async fn get_schema(&self, path: &str) -> Result<String>;

    /// Получение метаданных файла
    async fn get_metadata(&self, path: &str) -> Result<ParquetMetadata>;

    /// Список файлов в директории
    async fn list_files(&self, directory: &str) -> Result<Vec<String>>;

    /// Удаление файла
    async fn delete_file(&self, path: &str) -> Result<()>;
}

/// Метаданные Parquet файла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetMetadata {
    pub file_path: String,
    pub file_size: u64,
    pub num_rows: usize,
    pub num_columns: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Трейт для работы с DataFusion
#[async_trait]
pub trait DataFusionDataSource {
    /// Выполнение SQL запроса
    async fn execute_sql(&self, sql: &str) -> Result<Vec<arrow::record_batch::RecordBatch>>;

    /// Регистрация таблицы из Parquet файла
    async fn register_parquet_table(&mut self, table_name: &str, file_path: &str) -> Result<()>;

    /// Регистрация таблицы из Arrow данных
    async fn register_arrow_table(
        &mut self,
        table_name: &str,
        batches: Vec<arrow::record_batch::RecordBatch>,
    ) -> Result<()>;

    /// Получение схемы таблицы
    async fn get_table_schema(&self, table_name: &str) -> Result<arrow::datatypes::Schema>;

    /// Создание DataFrame из SQL запроса
    async fn create_dataframe(&self, sql: &str) -> Result<datafusion::dataframe::DataFrame>;

    /// Выполнение аналитического запроса
    async fn execute_analytics_query(
        &self,
        query: &AnalyticsQuery,
    ) -> Result<Vec<arrow::record_batch::RecordBatch>>;

    /// Получение статистики по таблице
    async fn get_table_stats(&self, table_name: &str) -> Result<TableStats>;
}

/// Аналитический запрос
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub table_name: String,
    pub aggregations: Option<Vec<String>>,
    pub filters: Option<String>,
    pub group_by: Option<String>,
    pub order_by: Option<String>,
    pub limit: Option<usize>,
}

/// Статистика таблицы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub estimated_size_bytes: u64,
}

/// Трейт для работы с API бирж
#[async_trait]
pub trait ExchangeApi {
    /// Получение текущих цен
    async fn get_ticker(&self, symbol: &str) -> Result<Ticker>;

    /// Получение исторических данных
    async fn get_historical_data(
        &self,
        symbol: &str,
        interval: &str,
        limit: u32,
    ) -> Result<Vec<Candle>>;

    /// Получение баланса аккаунта
    async fn get_balance(&self) -> Result<Vec<Balance>>;

    /// Размещение ордера
    async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse>;

    /// Отмена ордера
    async fn cancel_order(&self, order_id: &str) -> Result<()>;

    /// Получение открытых ордеров
    async fn get_open_orders(&self, symbol: Option<&str>) -> Result<Vec<Order>>;
}

/// Вспомогательный трейт для параметров SQL запросов
pub trait ToSql: Sync {
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for i64 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for f32 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

impl ToSql for String {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace("'", "''"))
    }
}

impl ToSql for &str {
    fn to_sql(&self) -> String {
        format!("'{}'", self.replace("'", "''"))
    }
}
