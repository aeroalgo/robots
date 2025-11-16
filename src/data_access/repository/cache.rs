use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::data_access::cache::layout::*;
use crate::data_access::database::redis::RedisConnector;
use crate::data_access::{DataAccessError, Result};

pub struct CacheRepository<'a> {
    redis: &'a RedisConnector,
}

impl<'a> CacheRepository<'a> {
    pub fn new(redis: &'a RedisConnector) -> Self {
        Self { redis }
    }

    pub async fn upsert_active_position(
        &self,
        snapshot: &CachedPositionSnapshot,
    ) -> Result<()> {
        let key = active_position_key(&snapshot.strategy_id, &snapshot.position_id);
        self.redis
            .set(&key, snapshot, Some(TTL_ACTIVE_POSITION_SECONDS))
            .await
    }

    pub async fn remove_active_position(&self, strategy_id: &str, position_id: &str) -> Result<()> {
        let key = active_position_key(strategy_id, position_id);
        self.redis.delete(&key).await
    }

    pub async fn list_active_positions(
        &self,
        strategy_id: &str,
    ) -> Result<Vec<CachedPositionSnapshot>> {
        let pattern = active_position_pattern(strategy_id);
        let keys = self.redis.keys(&pattern).await?;
        let mut snapshots = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(snapshot) = self.redis.get::<CachedPositionSnapshot>(&key).await? {
                snapshots.push(snapshot);
            }
        }
        Ok(snapshots)
    }

    pub async fn cache_indicator(&self, entry: &IndicatorCacheEntry) -> Result<()> {
        let key = indicator_cache_key(&entry.alias, &entry.timeframe);
        self.redis
            .set(&key, entry, Some(entry.ttl_override.unwrap_or(TTL_INDICATOR_SECONDS)))
            .await
    }

    pub async fn get_indicator(&self, alias: &str, timeframe: &str) -> Result<Option<IndicatorCacheEntry>> {
        let key = indicator_cache_key(alias, timeframe);
        self.redis.get(&key).await
    }

    pub async fn enqueue_signal(&self, envelope: &SignalEnvelope) -> Result<()> {
        let payload = serde_json::to_string(envelope).map_err(|e| {
            DataAccessError::Serialization(format!("Failed to serialize signal envelope: {}", e))
        })?;

        let queue_key = signal_queue_key(&envelope.strategy_id);
        let zset_key = signal_score_zset(&envelope.strategy_id);
        let ttl = TTL_SIGNAL_SECONDS as i64;

        {
            let mut conn = self.redis.connection()?;
            redis::cmd("ZADD")
                .arg(&zset_key)
                .arg(envelope.priority)
                .arg(&payload)
                .query::<()>(&mut conn)
                .map_err(|e| DataAccessError::Cache(format!("Redis ZADD error: {}", e)))?;

            redis::cmd("LPUSH")
                .arg(&queue_key)
                .arg(&payload)
                .query::<()>(&mut conn)
                .map_err(|e| DataAccessError::Cache(format!("Redis LPUSH error: {}", e)))?;

            redis::cmd("EXPIRE")
                .arg(&queue_key)
                .arg(ttl)
                .query::<()>(&mut conn)
                .map_err(|e| DataAccessError::Cache(format!("Redis EXPIRE error: {}", e)))?;

            redis::cmd("EXPIRE")
                .arg(&zset_key)
                .arg(ttl)
                .query::<()>(&mut conn)
                .map_err(|e| DataAccessError::Cache(format!("Redis EXPIRE error: {}", e)))?;
        }

        Ok(())
    }

    pub async fn drain_signal_batch(
        &self,
        strategy_id: &str,
        count: usize,
    ) -> Result<Vec<SignalEnvelope>> {
        let zset_key = signal_score_zset(strategy_id);
        let mut conn = self.redis.connection()?;
        let entries: Vec<(String, f64)> = redis::cmd("ZPOPMIN")
            .arg(&zset_key)
            .arg(count)
            .query(&mut conn)
            .map_err(|e| DataAccessError::Cache(format!("Redis ZPOPMIN error: {}", e)))?;

        let mut result = Vec::with_capacity(entries.len());
        for (payload, _) in entries {
            let envelope: SignalEnvelope = serde_json::from_str(&payload).map_err(|e| {
                DataAccessError::Serialization(format!("Failed to deserialize signal envelope: {}", e))
            })?;
            result.push(envelope);
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPositionSnapshot {
    pub strategy_id: String,
    pub position_id: String,
    pub entry_rule_id: Option<String>,
    pub direction: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub updated_at: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorCacheEntry {
    pub alias: String,
    pub timeframe: String,
    pub value: Value,
    pub computed_at: DateTime<Utc>,
    pub ttl_override: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEnvelope {
    pub strategy_id: String,
    pub signal_id: String,
    pub priority: f64,
    pub created_at: DateTime<Utc>,
    pub payload: Value,
}
