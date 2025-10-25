//! Redis коннектор для кэширования и сессий (упрощенная версия)

use crate::data_access::{
    Cache, ConnectionInfo, ConnectionStatus, DataAccessError, DataSource, Result,
};
use async_trait::async_trait;
use redis::{Client, Commands, RedisResult};
use serde::{Deserialize, Serialize};

/// Redis коннектор
pub struct RedisConnector {
    client: Option<Client>,
    host: String,
    port: u16,
    password: Option<String>,
    database: Option<u8>,
}

impl RedisConnector {
    /// Создание нового Redis коннектора
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: None,
            host,
            port,
            password: None,
            database: None,
        }
    }

    /// Создание Redis коннектора с паролем
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Создание Redis коннектора с базой данных
    pub fn with_database(mut self, database: u8) -> Self {
        self.database = Some(database);
        self
    }

    /// Получение URL подключения
    fn connection_url(&self) -> String {
        let mut url = format!("redis://{}:{}", self.host, self.port);

        if let Some(password) = &self.password {
            url = format!("redis://:{}@{}:{}", password, self.host, self.port);
        }

        if let Some(db) = self.database {
            url = format!("{}/{}", url, db);
        }

        url
    }

    /// Получение соединения для операций
    fn get_connection(&self) -> Result<redis::Connection> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| DataAccessError::Connection("Not connected to Redis".to_string()))?;

        client
            .get_connection()
            .map_err(|e| DataAccessError::Connection(format!("Failed to get connection: {}", e)))
    }
}

#[async_trait]
impl DataSource for RedisConnector {
    type Error = DataAccessError;

    async fn connect(&mut self) -> Result<()> {
        let url = self.connection_url();

        self.client = Some(Client::open(url.as_str()).map_err(|e| {
            DataAccessError::Connection(format!("Failed to create Redis client: {}", e))
        })?);

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.client = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    fn connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            host: self.host.clone(),
            port: self.port,
            database: self.database.map(|db| db.to_string()),
            status: if self.is_connected() {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            },
        }
    }
}

#[async_trait]
impl Cache for RedisConnector {
    async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        let mut conn = self.get_connection()?;

        let result: RedisResult<String> = conn.get(key);

        match result {
            Ok(value) => {
                let deserialized: T = serde_json::from_str(&value).map_err(|e| {
                    DataAccessError::Serialization(format!("Failed to deserialize value: {}", e))
                })?;
                Ok(Some(deserialized))
            }
            Err(_) => {
                // Ключ не найден или другая ошибка
                Ok(None)
            }
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.get_connection()?;

        let serialized = serde_json::to_string(value).map_err(|e| {
            DataAccessError::Serialization(format!("Failed to serialize value: {}", e))
        })?;

        let result = if let Some(ttl_seconds) = ttl {
            conn.set_ex::<_, _, ()>(key, serialized, ttl_seconds)
        } else {
            conn.set::<_, _, ()>(key, serialized)
        };

        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection()?;

        conn.del::<_, ()>(key)
            .map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;

        let result: RedisResult<bool> = conn.exists(key);
        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))
    }

    async fn expire(&self, key: &str, ttl: u64) -> Result<()> {
        let mut conn = self.get_connection()?;

        conn.expire::<_, ()>(key, ttl as i64)
            .map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))?;

        Ok(())
    }

    async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection()?;

        let result: RedisResult<Vec<String>> = conn.keys(pattern);
        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))
    }
}

/// Дополнительные методы для Redis
impl RedisConnector {
    /// Получение значения как строки
    pub async fn get_string(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection()?;

        let result: RedisResult<String> = conn.get(key);

        match result {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    /// Установка строкового значения
    pub async fn set_string(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<()> {
        let mut conn = self.get_connection()?;

        let result = if let Some(ttl_seconds) = ttl {
            conn.set_ex::<_, _, ()>(key, value, ttl_seconds)
        } else {
            conn.set::<_, _, ()>(key, value)
        };

        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))?;
        Ok(())
    }

    /// Инкремент числового значения
    pub async fn increment(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection()?;

        let result: RedisResult<i64> = conn.incr(key, 1);
        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))
    }

    /// Декремент числового значения
    pub async fn decrement(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection()?;

        let result: RedisResult<i64> = conn.decr(key, 1);
        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))
    }

    /// Добавление в список
    pub async fn lpush<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.get_connection()?;

        let serialized = serde_json::to_string(value).map_err(|e| {
            DataAccessError::Serialization(format!("Failed to serialize value: {}", e))
        })?;

        conn.lpush::<_, _, ()>(key, serialized)
            .map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))?;

        Ok(())
    }

    /// Получение из списка
    pub async fn rpop<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        let mut conn = self.get_connection()?;

        let result: RedisResult<String> = conn.rpop(key, None);

        match result {
            Ok(value) => {
                let deserialized: T = serde_json::from_str(&value).map_err(|e| {
                    DataAccessError::Serialization(format!("Failed to deserialize value: {}", e))
                })?;
                Ok(Some(deserialized))
            }
            Err(_) => Ok(None),
        }
    }

    /// Получение размера списка
    pub async fn llen(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection()?;

        let result: RedisResult<i64> = conn.llen(key);
        result.map_err(|e| DataAccessError::Cache(format!("Redis error: {}", e)))
    }

    /// Получение всех элементов списка
    pub async fn lrange<T>(&self, key: &str, start: i64, stop: i64) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + Send + Sync,
    {
        let mut conn = self.get_connection()?;

        let result: RedisResult<Vec<String>> = conn.lrange(key, start as isize, stop as isize);

        match result {
            Ok(values) => {
                let mut deserialized = Vec::new();
                for value in values {
                    let item: T = serde_json::from_str(&value).map_err(|e| {
                        DataAccessError::Serialization(format!(
                            "Failed to deserialize value: {}",
                            e
                        ))
                    })?;
                    deserialized.push(item);
                }
                Ok(deserialized)
            }
            Err(e) => Err(DataAccessError::Cache(format!("Redis error: {}", e))),
        }
    }
}
