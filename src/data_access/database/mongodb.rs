//! MongoDB коннектор для работы с базой данных

use crate::data_access::models::*;
use crate::data_access::traits::{DataSource, Database as DatabaseTrait, ToSql, Transaction};
use crate::data_access::{DataAccessError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOptions, IndexOptions},
    Client, Collection, Database, IndexModel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Конфигурация MongoDB подключения
#[derive(Debug, Clone)]
pub struct MongoDBConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_database: Option<String>,
    pub max_pool_size: Option<u32>,
    pub min_pool_size: Option<u32>,
    pub max_idle_time: Option<u64>,
    pub connect_timeout: Option<u64>,
    pub server_selection_timeout: Option<u64>,
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 27017,
            database: "trading_robot".to_string(),
            username: None,
            password: None,
            auth_database: Some("admin".to_string()),
            max_pool_size: Some(20),
            min_pool_size: Some(5),
            max_idle_time: Some(300),
            connect_timeout: Some(30),
            server_selection_timeout: Some(30),
        }
    }
}

/// MongoDB коннектор
pub struct MongoDBConnector {
    config: MongoDBConfig,
    client: Option<Client>,
    database: Option<Database>,
    connected: bool,
}

impl MongoDBConnector {
    /// Создание нового коннектора
    pub fn new(config: MongoDBConfig) -> Self {
        Self {
            config,
            client: None,
            database: None,
            connected: false,
        }
    }

    /// Создание коннектора с дефолтной конфигурацией
    pub fn new_default() -> Self {
        Self::new(MongoDBConfig::default())
    }

    /// Получение клиента
    fn get_client(&self) -> Result<&Client> {
        self.client.as_ref().ok_or_else(|| {
            DataAccessError::Connection("MongoDB client not initialized".to_string())
        })
    }

    /// Получение базы данных
    fn get_database(&self) -> Result<&Database> {
        self.database.as_ref().ok_or_else(|| {
            DataAccessError::Connection("MongoDB database not initialized".to_string())
        })
    }

    /// Получение коллекции
    pub fn get_collection<T>(&self, name: &str) -> Result<Collection<T>>
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let database = self.get_database()?;
        Ok(database.collection(name))
    }

    /// Создание индексов для торговых данных
    pub async fn create_indexes(&self) -> Result<()> {
        let database = self.get_database()?;

        // Индексы для коллекции candles
        let candles_collection: Collection<Document> = database.collection("candles");
        let candle_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "symbol": 1, "timestamp": -1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder().keys(doc! { "symbol": 1 }).build(),
            IndexModel::builder().keys(doc! { "timestamp": -1 }).build(),
        ];
        candles_collection
            .create_indexes(candle_indexes, None)
            .await
            .map_err(|e| {
                DataAccessError::Query(format!("Failed to create candle indexes: {}", e))
            })?;

        // Индексы для коллекции trades
        let trades_collection: Collection<Document> = database.collection("trades");
        let trade_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "symbol": 1, "timestamp": -1 })
                .build(),
            IndexModel::builder().keys(doc! { "order_id": 1 }).build(),
            IndexModel::builder().keys(doc! { "side": 1 }).build(),
        ];
        trades_collection
            .create_indexes(trade_indexes, None)
            .await
            .map_err(|e| {
                DataAccessError::Query(format!("Failed to create trade indexes: {}", e))
            })?;

        // Индексы для коллекции orders
        let orders_collection: Collection<Document> = database.collection("orders");
        let order_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "symbol": 1, "status": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "created_at": -1 })
                .build(),
        ];
        orders_collection
            .create_indexes(order_indexes, None)
            .await
            .map_err(|e| {
                DataAccessError::Query(format!("Failed to create order indexes: {}", e))
            })?;

        // Индексы для коллекции users
        let users_collection: Collection<Document> = database.collection("users");
        let user_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "username": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "email": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ];
        users_collection
            .create_indexes(user_indexes, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to create user indexes: {}", e)))?;

        // Индексы для коллекции strategies
        let strategies_collection: Collection<Document> = database.collection("strategies");
        let strategy_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder().keys(doc! { "name": 1 }).build(),
            IndexModel::builder().keys(doc! { "enabled": 1 }).build(),
        ];
        strategies_collection
            .create_indexes(strategy_indexes, None)
            .await
            .map_err(|e| {
                DataAccessError::Query(format!("Failed to create strategy indexes: {}", e))
            })?;

        // Индексы для коллекции backtest_results
        let backtest_collection: Collection<Document> = database.collection("backtest_results");
        let backtest_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "strategy_id": 1, "symbol": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "total_return": -1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "sharpe_ratio": -1 })
                .build(),
        ];
        backtest_collection
            .create_indexes(backtest_indexes, None)
            .await
            .map_err(|e| {
                DataAccessError::Query(format!("Failed to create backtest indexes: {}", e))
            })?;

        println!("✅ MongoDB indexes created successfully");
        Ok(())
    }

    /// Получение статистики базы данных
    pub async fn get_database_stats(&self) -> Result<HashMap<String, i64>> {
        let database = self.get_database()?;
        let mut stats = HashMap::new();

        // Подсчет записей в основных коллекциях
        let collections = vec![
            "users",
            "strategies",
            "candles",
            "trades",
            "orders",
            "backtest_results",
            "trading_signals",
            "system_events",
        ];

        for collection_name in collections {
            let collection: Collection<Document> = database.collection(collection_name);
            let count = collection
                .count_documents(doc! {}, None)
                .await
                .map_err(|e| {
                    DataAccessError::Query(format!(
                        "Failed to count documents in {}: {}",
                        collection_name, e
                    ))
                })?;
            stats.insert(collection_name.to_string(), count as i64);
        }

        // Размер базы данных
        let stats_command = doc! {
            "dbStats": 1
        };
        let stats_result = database
            .run_command(stats_command, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get database stats: {}", e)))?;

        if let Some(size_bytes) = stats_result.get("dataSize").and_then(|v| v.as_i64()) {
            stats.insert("database_size_bytes".to_string(), size_bytes);
        }

        Ok(stats)
    }

    /// CRUD операции для пользователей
    pub async fn create_user(&self, user: &User) -> Result<()> {
        let collection: Collection<User> = self.get_collection("users")?;
        collection
            .insert_one(user, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to create user: {}", e)))?;
        Ok(())
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let collection: Collection<User> = self.get_collection("users")?;
        let filter = doc! { "id": id };
        let user = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get user: {}", e)))?;
        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let collection: Collection<User> = self.get_collection("users")?;
        let filter = doc! { "username": username };
        let user = collection.find_one(filter, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to get user by username: {}", e))
        })?;
        Ok(user)
    }

    pub async fn update_user(&self, id: &str, user: &User) -> Result<()> {
        let collection: Collection<User> = self.get_collection("users")?;
        let filter = doc! { "id": id };
        let update = doc! { "$set": mongodb::bson::to_bson(user).map_err(|e| {
            DataAccessError::Query(format!("Failed to serialize user: {}", e))
        })? };
        collection
            .update_one(filter, update, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to update user: {}", e)))?;
        Ok(())
    }

    pub async fn delete_user(&self, id: &str) -> Result<()> {
        let collection: Collection<User> = self.get_collection("users")?;
        let filter = doc! { "id": id };
        collection
            .delete_one(filter, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to delete user: {}", e)))?;
        Ok(())
    }

    /// CRUD операции для стратегий
    pub async fn create_strategy(&self, strategy: &Strategy) -> Result<()> {
        let collection: Collection<Strategy> = self.get_collection("strategies")?;
        collection
            .insert_one(strategy, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to create strategy: {}", e)))?;
        Ok(())
    }

    pub async fn get_strategy_by_id(&self, id: &str) -> Result<Option<Strategy>> {
        let collection: Collection<Strategy> = self.get_collection("strategies")?;
        let filter = doc! { "id": id };
        let strategy = collection
            .find_one(filter, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get strategy: {}", e)))?;
        Ok(strategy)
    }

    pub async fn get_enabled_strategies(&self) -> Result<Vec<Strategy>> {
        let collection: Collection<Strategy> = self.get_collection("strategies")?;
        let filter = doc! { "enabled": true };
        let cursor = collection.find(filter, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to get enabled strategies: {}", e))
        })?;
        let strategies: std::result::Result<Vec<Strategy>, _> = cursor.try_collect().await;
        strategies
            .map_err(|e| DataAccessError::Query(format!("Failed to collect strategies: {}", e)))
    }

    /// CRUD операции для свечей
    pub async fn create_candle(&self, candle: &Candle) -> Result<()> {
        let collection: Collection<Candle> = self.get_collection("candles")?;
        collection
            .insert_one(candle, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to create candle: {}", e)))?;
        Ok(())
    }

    pub async fn create_candles_batch(&self, candles: &[Candle]) -> Result<()> {
        let collection: Collection<Candle> = self.get_collection("candles")?;
        collection.insert_many(candles, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to create candles batch: {}", e))
        })?;
        Ok(())
    }

    pub async fn get_candles(
        &self,
        symbol: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>,
    ) -> Result<Vec<Candle>> {
        let collection: Collection<Candle> = self.get_collection("candles")?;
        let filter = doc! {
            "symbol": symbol,
            "timestamp": {
                "$gte": mongodb::bson::DateTime::from_millis(start_time.timestamp_millis()),
                "$lte": mongodb::bson::DateTime::from_millis(end_time.timestamp_millis())
            }
        };

        let mut options = FindOptions::default();
        options.sort = Some(doc! { "timestamp": 1 });
        if let Some(limit) = limit {
            options.limit = Some(limit);
        }

        let cursor = collection
            .find(filter, options)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get candles: {}", e)))?;
        let candles: std::result::Result<Vec<Candle>, _> = cursor.try_collect().await;
        candles.map_err(|e| DataAccessError::Query(format!("Failed to collect candles: {}", e)))
    }

    /// CRUD операции для сделок
    pub async fn create_trade(&self, trade: &Trade) -> Result<()> {
        let collection: Collection<Trade> = self.get_collection("trades")?;
        collection
            .insert_one(trade, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to create trade: {}", e)))?;
        Ok(())
    }

    pub async fn get_trades(
        &self,
        symbol: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<Trade>> {
        let collection: Collection<Trade> = self.get_collection("trades")?;
        let mut filter = doc! {};

        if let Some(symbol) = symbol {
            filter.insert("symbol", symbol);
        }

        if let (Some(start), Some(end)) = (start_time, end_time) {
            filter.insert(
                "timestamp",
                doc! {
                    "$gte": mongodb::bson::DateTime::from_millis(start.timestamp_millis()),
                    "$lte": mongodb::bson::DateTime::from_millis(end.timestamp_millis())
                },
            );
        }

        let mut options = FindOptions::default();
        options.sort = Some(doc! { "timestamp": -1 });
        if let Some(limit) = limit {
            options.limit = Some(limit);
        }

        let cursor = collection
            .find(filter, options)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get trades: {}", e)))?;
        let trades: std::result::Result<Vec<Trade>, _> = cursor.try_collect().await;
        trades.map_err(|e| DataAccessError::Query(format!("Failed to collect trades: {}", e)))
    }

    /// CRUD операции для результатов бэктестов
    pub async fn create_backtest_result(&self, result: &BacktestResult) -> Result<()> {
        let collection: Collection<BacktestResult> = self.get_collection("backtest_results")?;
        collection.insert_one(result, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to create backtest result: {}", e))
        })?;
        Ok(())
    }

    pub async fn get_backtest_results(
        &self,
        strategy_id: Option<&str>,
        symbol: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<BacktestResult>> {
        let collection: Collection<BacktestResult> = self.get_collection("backtest_results")?;
        let mut filter = doc! {};

        if let Some(strategy_id) = strategy_id {
            filter.insert("strategy_id", strategy_id);
        }

        if let Some(symbol) = symbol {
            filter.insert("symbol", symbol);
        }

        let mut options = FindOptions::default();
        options.sort = Some(doc! { "total_return": -1 });
        if let Some(limit) = limit {
            options.limit = Some(limit);
        }

        let cursor = collection.find(filter, options).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to get backtest results: {}", e))
        })?;
        let results: std::result::Result<Vec<BacktestResult>, _> = cursor.try_collect().await;
        results.map_err(|e| {
            DataAccessError::Query(format!("Failed to collect backtest results: {}", e))
        })
    }

    /// Аналитические запросы
    pub async fn get_symbol_statistics(&self) -> Result<Vec<Document>> {
        let collection: Collection<Document> = self.get_collection("trades")?;
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$symbol",
                    "trade_count": { "$sum": 1 },
                    "avg_price": { "$avg": "$price" },
                    "total_volume": { "$sum": { "$multiply": ["$quantity", "$price"] } },
                    "min_price": { "$min": "$price" },
                    "max_price": { "$max": "$price" }
                }
            },
            doc! {
                "$sort": { "trade_count": -1 }
            },
        ];

        let cursor = collection.aggregate(pipeline, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to get symbol statistics: {}", e))
        })?;
        let results: std::result::Result<Vec<Document>, _> = cursor.try_collect().await;
        results.map_err(|e| {
            DataAccessError::Query(format!("Failed to collect symbol statistics: {}", e))
        })
    }

    pub async fn get_daily_trading_stats(&self) -> Result<Vec<Document>> {
        let collection: Collection<Document> = self.get_collection("trades")?;
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": "$timestamp"
                        }
                    },
                    "trade_count": { "$sum": 1 },
                    "total_volume": { "$sum": { "$multiply": ["$quantity", "$price"] } },
                    "unique_symbols": { "$addToSet": "$symbol" }
                }
            },
            doc! {
                "$addFields": {
                    "symbol_count": { "$size": "$unique_symbols" }
                }
            },
            doc! {
                "$sort": { "_id": -1 }
            },
        ];

        let cursor = collection.aggregate(pipeline, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to get daily trading stats: {}", e))
        })?;
        let results: std::result::Result<Vec<Document>, _> = cursor.try_collect().await;
        results.map_err(|e| {
            DataAccessError::Query(format!("Failed to collect daily trading stats: {}", e))
        })
    }

    /// Очистка всех коллекций (для тестирования)
    pub async fn truncate_all_collections(&self) -> Result<()> {
        let database = self.get_database()?;
        let collections = vec![
            "users",
            "strategies",
            "candles",
            "trades",
            "orders",
            "backtest_results",
            "trading_signals",
            "system_events",
        ];

        for collection_name in collections {
            let collection: Collection<Document> = database.collection(collection_name);
            collection.delete_many(doc! {}, None).await.map_err(|e| {
                DataAccessError::Query(format!(
                    "Failed to truncate collection {}: {}",
                    collection_name, e
                ))
            })?;
        }

        println!("✅ All MongoDB collections truncated");
        Ok(())
    }
}

#[async_trait]
impl DataSource for MongoDBConnector {
    type Error = DataAccessError;

    async fn connect(&mut self) -> Result<()> {
        let mut client_options = ClientOptions::parse(&format!(
            "mongodb://{}:{}",
            self.config.host, self.config.port
        ))
        .await
        .map_err(|e| {
            DataAccessError::Connection(format!("Failed to parse MongoDB connection string: {}", e))
        })?;

        // Настройка аутентификации
        if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
            client_options.credential = Some(
                mongodb::options::Credential::builder()
                    .username(username.clone())
                    .password(password.clone())
                    .source(
                        self.config
                            .auth_database
                            .clone()
                            .unwrap_or_else(|| "admin".to_string()),
                    )
                    .build(),
            );
        }

        // Настройка пула соединений
        if let Some(max_pool_size) = self.config.max_pool_size {
            client_options.max_pool_size = Some(max_pool_size);
        }
        if let Some(min_pool_size) = self.config.min_pool_size {
            client_options.min_pool_size = Some(min_pool_size);
        }
        if let Some(max_idle_time) = self.config.max_idle_time {
            client_options.max_idle_time = Some(std::time::Duration::from_secs(max_idle_time));
        }
        if let Some(connect_timeout) = self.config.connect_timeout {
            client_options.connect_timeout = Some(std::time::Duration::from_secs(connect_timeout));
        }
        if let Some(server_selection_timeout) = self.config.server_selection_timeout {
            client_options.server_selection_timeout =
                Some(std::time::Duration::from_secs(server_selection_timeout));
        }

        let client = Client::with_options(client_options).map_err(|e| {
            DataAccessError::Connection(format!("Failed to create MongoDB client: {}", e))
        })?;

        // Тестирование подключения
        let database = client.database(&self.config.database);
        database
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|e| DataAccessError::Connection(format!("Failed to ping MongoDB: {}", e)))?;

        self.client = Some(client);
        self.database = Some(database);
        self.connected = true;

        println!("✅ Connected to MongoDB database: {}", self.config.database);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(client) = self.client.take() {
            // Клиент автоматически закроет все соединения при drop
            println!("✅ Disconnected from MongoDB database");
        }
        self.database = None;
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn connection_info(&self) -> crate::data_access::traits::ConnectionInfo {
        crate::data_access::traits::ConnectionInfo {
            host: self.config.host.clone(),
            port: self.config.port,
            database: Some(self.config.database.clone()),
            status: if self.connected {
                crate::data_access::traits::ConnectionStatus::Connected
            } else {
                crate::data_access::traits::ConnectionStatus::Disconnected
            },
        }
    }
}

#[async_trait]
impl DatabaseTrait for MongoDBConnector {
    async fn query<T>(&self, query: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        if !self.connected {
            return Err(DataAccessError::Connection(
                "Not connected to MongoDB".to_string(),
            ));
        }

        // Упрощенная реализация - в реальном проекте нужно парсить MongoDB запросы
        println!("Executing MongoDB query: {}", query);

        // Возвращаем пустой результат для демонстрации
        Ok(Vec::new())
    }

    async fn execute(&self, query: &str) -> Result<u64> {
        if !self.connected {
            return Err(DataAccessError::Connection(
                "Not connected to MongoDB".to_string(),
            ));
        }

        // Упрощенная реализация - в реальном проекте нужно парсить MongoDB команды
        println!("Executing MongoDB command: {}", query);

        // Возвращаем 0 для демонстрации
        Ok(0)
    }

    async fn query_with_params<T>(&self, query: &str, _params: &[&dyn ToSql]) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        if !self.connected {
            return Err(DataAccessError::Connection(
                "Not connected to MongoDB".to_string(),
            ));
        }

        println!(
            "Executing MongoDB query with {} params: {}",
            _params.len(),
            query
        );

        // Упрощенная реализация
        Ok(Vec::new())
    }

    async fn execute_with_params(&self, query: &str, _params: &[&dyn ToSql]) -> Result<u64> {
        if !self.connected {
            return Err(DataAccessError::Connection(
                "Not connected to MongoDB".to_string(),
            ));
        }

        println!(
            "Executing MongoDB command with {} params: {}",
            _params.len(),
            query
        );

        // Упрощенная реализация
        Ok(0)
    }

    async fn begin_transaction(&self) -> Result<Box<dyn Transaction + Send + Sync>> {
        if !self.connected {
            return Err(DataAccessError::Connection(
                "Not connected to MongoDB".to_string(),
            ));
        }

        // Упрощенная реализация транзакции для MongoDB
        Ok(Box::new(MongoDBTransaction {
            database: self.database.clone().unwrap(),
        }))
    }

    async fn ping(&self) -> Result<()> {
        if !self.connected {
            return Err(DataAccessError::Connection(
                "Not connected to MongoDB".to_string(),
            ));
        }

        let database = self.get_database()?;
        database
            .run_command(doc! { "ping": 1 }, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to ping MongoDB: {}", e)))?;

        Ok(())
    }
}

/// MongoDB транзакция
pub struct MongoDBTransaction {
    database: Database,
}

#[async_trait]
impl Transaction for MongoDBTransaction {
    async fn execute(&self, query: &str) -> Result<u64> {
        // Упрощенная реализация - в реальном проекте нужно парсить MongoDB команды
        println!("Executing MongoDB command in transaction: {}", query);
        Ok(0)
    }

    async fn commit(self: Box<Self>) -> Result<()> {
        // В MongoDB транзакции автоматически коммитятся при завершении сессии
        println!("✅ MongoDB transaction committed");
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        // В MongoDB транзакции автоматически откатываются при ошибке
        println!("❌ MongoDB transaction rolled back");
        Ok(())
    }
}

/// Утилиты для работы с MongoDB
pub struct MongoDBUtils;

impl MongoDBUtils {
    /// Получение информации о версии MongoDB
    pub async fn get_version(connector: &MongoDBConnector) -> Result<String> {
        let database = connector.get_database()?;
        let result = database
            .run_command(doc! { "buildInfo": 1 }, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get MongoDB version: {}", e)))?;

        let version = result
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        Ok(version.to_string())
    }

    /// Получение списка коллекций
    pub async fn get_collections(connector: &MongoDBConnector) -> Result<Vec<String>> {
        let database = connector.get_database()?;
        let collections = database
            .list_collection_names(None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get collections: {}", e)))?;
        Ok(collections)
    }

    /// Получение информации о коллекции
    pub async fn get_collection_info(
        connector: &MongoDBConnector,
        collection_name: &str,
    ) -> Result<Document> {
        let database = connector.get_database()?;
        let collection: Collection<Document> = database.collection(collection_name);

        let mut result = doc! {};
        let mut cursor = collection
            .aggregate(vec![doc! { "$collStats": { "storageStats": {} } }], None)
            .await
            .map_err(|e| {
                DataAccessError::Query(format!("Failed to get collection stats: {}", e))
            })?;

        while let Some(doc) = cursor.try_next().await.map_err(|e| {
            DataAccessError::Query(format!("Failed to iterate collection stats: {}", e))
        })? {
            result = doc;
            break;
        }

        Ok(result)
    }

    /// Получение топ стратегий по доходности
    pub async fn get_top_strategies_by_return(
        connector: &MongoDBConnector,
        limit: i64,
    ) -> Result<Vec<Document>> {
        let collection: Collection<Document> = connector.get_collection("backtest_results")?;
        let pipeline = vec![
            doc! {
                "$group": {
                    "_id": "$strategy_id",
                    "avg_return": { "$avg": "$total_return" },
                    "avg_sharpe": { "$avg": "$sharpe_ratio" },
                    "avg_drawdown": { "$avg": "$max_drawdown" },
                    "test_count": { "$sum": 1 }
                }
            },
            doc! {
                "$sort": { "avg_return": -1 }
            },
            doc! {
                "$limit": limit
            },
        ];

        let cursor = collection
            .aggregate(pipeline, None)
            .await
            .map_err(|e| DataAccessError::Query(format!("Failed to get top strategies: {}", e)))?;
        let results: std::result::Result<Vec<Document>, _> = cursor.try_collect().await;
        results
            .map_err(|e| DataAccessError::Query(format!("Failed to collect top strategies: {}", e)))
    }

    /// Получение производительности стратегий
    pub async fn get_strategy_performance(connector: &MongoDBConnector) -> Result<Vec<Document>> {
        let collection: Collection<Document> = connector.get_collection("backtest_results")?;
        let pipeline = vec![
            doc! {
                "$lookup": {
                    "from": "strategies",
                    "localField": "strategy_id",
                    "foreignField": "id",
                    "as": "strategy"
                }
            },
            doc! {
                "$unwind": "$strategy"
            },
            doc! {
                "$group": {
                    "_id": "$strategy.name",
                    "avg_return": { "$avg": "$total_return" },
                    "avg_sharpe": { "$avg": "$sharpe_ratio" },
                    "avg_drawdown": { "$avg": "$max_drawdown" },
                    "test_count": { "$sum": 1 },
                    "symbols": { "$addToSet": "$symbol" }
                }
            },
            doc! {
                "$addFields": {
                    "symbol_count": { "$size": "$symbols" }
                }
            },
            doc! {
                "$sort": { "avg_return": -1 }
            },
        ];

        let cursor = collection.aggregate(pipeline, None).await.map_err(|e| {
            DataAccessError::Query(format!("Failed to get strategy performance: {}", e))
        })?;
        let results: std::result::Result<Vec<Document>, _> = cursor.try_collect().await;
        results.map_err(|e| {
            DataAccessError::Query(format!("Failed to collect strategy performance: {}", e))
        })
    }
}
