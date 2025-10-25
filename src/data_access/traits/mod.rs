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

/// Трейт для работы с Arrow данными (заглушка)
#[async_trait]
pub trait ArrowDataSource {
    /// Получение данных (заглушка)
    async fn get_data(&self, query: &str) -> Result<String>;

    /// Отправка данных (заглушка)
    async fn send_data(&self, data: &str) -> Result<()>;
}

/// Трейт для работы с Parquet файлами (заглушка)
#[async_trait]
pub trait ParquetDataSource {
    /// Чтение Parquet файла (заглушка)
    async fn read_parquet(&self, path: &str) -> Result<String>;

    /// Запись в Parquet файл (заглушка)
    async fn write_parquet(&self, path: &str, data: &str) -> Result<()>;
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
pub trait ToSql {
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

impl ToSql for f64 {
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
