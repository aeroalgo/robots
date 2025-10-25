//! Query Builder для Redis

use crate::data_access::{Result, DataAccessError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Базовый Query Builder для Redis
pub struct RedisQueryBuilder {
    operations: Vec<RedisOperation>,
}

/// Операции Redis
#[derive(Debug, Clone)]
pub enum RedisOperation {
    Get(String),
    Set(String, String, Option<u64>),
    Delete(String),
    Exists(String),
    Expire(String, u64),
    Keys(String),
    Increment(String),
    Decrement(String),
    LPush(String, String),
    RPop(String),
    LLen(String),
    LRange(String, i64, i64),
}

impl RedisQueryBuilder {
    /// Создание нового Query Builder
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Получение значения по ключу
    pub fn get(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::Get(key.to_string()));
        self
    }

    /// Установка значения
    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.operations.push(RedisOperation::Set(key.to_string(), value.to_string(), None));
        self
    }

    /// Установка значения с TTL
    pub fn set_with_ttl(mut self, key: &str, value: &str, ttl: u64) -> Self {
        self.operations.push(RedisOperation::Set(key.to_string(), value.to_string(), Some(ttl)));
        self
    }

    /// Удаление ключа
    pub fn delete(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::Delete(key.to_string()));
        self
    }

    /// Проверка существования ключа
    pub fn exists(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::Exists(key.to_string()));
        self
    }

    /// Установка TTL для ключа
    pub fn expire(mut self, key: &str, ttl: u64) -> Self {
        self.operations.push(RedisOperation::Expire(key.to_string(), ttl));
        self
    }

    /// Поиск ключей по паттерну
    pub fn keys(mut self, pattern: &str) -> Self {
        self.operations.push(RedisOperation::Keys(pattern.to_string()));
        self
    }

    /// Инкремент значения
    pub fn increment(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::Increment(key.to_string()));
        self
    }

    /// Декремент значения
    pub fn decrement(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::Decrement(key.to_string()));
        self
    }

    /// Добавление в начало списка
    pub fn lpush(mut self, key: &str, value: &str) -> Self {
        self.operations.push(RedisOperation::LPush(key.to_string(), value.to_string()));
        self
    }

    /// Получение из конца списка
    pub fn rpop(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::RPop(key.to_string()));
        self
    }

    /// Получение размера списка
    pub fn llen(mut self, key: &str) -> Self {
        self.operations.push(RedisOperation::LLen(key.to_string()));
        self
    }

    /// Получение диапазона списка
    pub fn lrange(mut self, key: &str, start: i64, stop: i64) -> Self {
        self.operations.push(RedisOperation::LRange(key.to_string(), start, stop));
        self
    }

    /// Получение операций
    pub fn operations(&self) -> &[RedisOperation] {
        &self.operations
    }

    /// Очистка операций
    pub fn clear(mut self) -> Self {
        self.operations.clear();
        self
    }
}

impl Default for RedisQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для кэширования
pub struct CacheQueryBuilder {
    builder: RedisQueryBuilder,
}

impl CacheQueryBuilder {
    /// Создание нового Cache Query Builder
    pub fn new() -> Self {
        Self {
            builder: RedisQueryBuilder::new(),
        }
    }

    /// Кэширование значения
    pub fn cache<T>(mut self, key: &str, value: &T, ttl: Option<u64>) -> Self
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(value)
            .unwrap_or_else(|_| "{}".to_string());
        
        self.builder = if let Some(ttl_seconds) = ttl {
            self.builder.set_with_ttl(key, &serialized, ttl_seconds)
        } else {
            self.builder.set(key, &serialized)
        };
        
        self
    }

    /// Получение из кэша
    pub fn get_cached(mut self, key: &str) -> Self {
        self.builder = self.builder.get(key);
        self
    }

    /// Удаление из кэша
    pub fn invalidate(mut self, key: &str) -> Self {
        self.builder = self.builder.delete(key);
        self
    }

    /// Проверка существования в кэше
    pub fn exists_cached(mut self, key: &str) -> Self {
        self.builder = self.builder.exists(key);
        self
    }

    /// Получение операций
    pub fn operations(&self) -> &[RedisOperation] {
        self.builder.operations()
    }
}

impl Default for CacheQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для очередей
pub struct QueueQueryBuilder {
    builder: RedisQueryBuilder,
}

impl QueueQueryBuilder {
    /// Создание нового Queue Query Builder
    pub fn new() -> Self {
        Self {
            builder: RedisQueryBuilder::new(),
        }
    }

    /// Добавление в очередь
    pub fn enqueue<T>(mut self, queue_name: &str, item: &T) -> Self
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(item)
            .unwrap_or_else(|_| "{}".to_string());
        
        self.builder = self.builder.lpush(queue_name, &serialized);
        self
    }

    /// Получение из очереди
    pub fn dequeue(mut self, queue_name: &str) -> Self {
        self.builder = self.builder.rpop(queue_name);
        self
    }

    /// Получение размера очереди
    pub fn size(mut self, queue_name: &str) -> Self {
        self.builder = self.builder.llen(queue_name);
        self
    }

    /// Получение всех элементов очереди
    pub fn peek_all(mut self, queue_name: &str) -> Self {
        self.builder = self.builder.lrange(queue_name, 0, -1);
        self
    }

    /// Получение операций
    pub fn operations(&self) -> &[RedisOperation] {
        self.builder.operations()
    }
}

impl Default for QueueQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для счетчиков
pub struct CounterQueryBuilder {
    builder: RedisQueryBuilder,
}

impl CounterQueryBuilder {
    /// Создание нового Counter Query Builder
    pub fn new() -> Self {
        Self {
            builder: RedisQueryBuilder::new(),
        }
    }

    /// Инкремент счетчика
    pub fn increment(mut self, counter_name: &str) -> Self {
        self.builder = self.builder.increment(counter_name);
        self
    }

    /// Декремент счетчика
    pub fn decrement(mut self, counter_name: &str) -> Self {
        self.builder = self.builder.decrement(counter_name);
        self
    }

    /// Получение значения счетчика
    pub fn get_value(mut self, counter_name: &str) -> Self {
        self.builder = self.builder.get(counter_name);
        self
    }

    /// Установка значения счетчика
    pub fn set_value(mut self, counter_name: &str, value: i64) -> Self {
        self.builder = self.builder.set(counter_name, &value.to_string());
        self
    }

    /// Получение операций
    pub fn operations(&self) -> &[RedisOperation] {
        self.builder.operations()
    }
}

impl Default for CounterQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Утилиты для работы с ключами
pub struct KeyUtils;

impl KeyUtils {
    /// Создание ключа для цены
    pub fn price_key(symbol: &str) -> String {
        format!("price:{}", symbol)
    }

    /// Создание ключа для сессии
    pub fn session_key(user_id: &str) -> String {
        format!("session:{}", user_id)
    }

    /// Создание ключа для очереди команд
    pub fn command_queue_key(strategy_id: &str) -> String {
        format!("commands:{}", strategy_id)
    }

    /// Создание ключа для блокировки
    pub fn lock_key(resource: &str) -> String {
        format!("lock:{}", resource)
    }

    /// Создание ключа для счетчика
    pub fn counter_key(name: &str) -> String {
        format!("counter:{}", name)
    }

    /// Создание ключа для кэша индикаторов
    pub fn indicator_cache_key(symbol: &str, indicator: &str, timeframe: &str) -> String {
        format!("indicator:{}:{}:{}", symbol, indicator, timeframe)
    }

    /// Создание ключа для последней цены
    pub fn last_price_key(symbol: &str) -> String {
        format!("last_price:{}", symbol)
    }

    /// Создание ключа для торговых сигналов
    pub fn signal_key(strategy_id: &str, symbol: &str) -> String {
        format!("signal:{}:{}", strategy_id, symbol)
    }
}
