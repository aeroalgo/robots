use chrono::{DateTime, Utc};

use crate::data_access::database::clickhouse::{
    ClickHouseConnector, OhlcvData, Signal, TradeRecord,
};
use crate::data_access::Result;
use crate::data_model::types::{Symbol, TimeFrame};

pub struct StorageRepository<'a> {
    clickhouse: &'a ClickHouseConnector,
}

impl<'a> StorageRepository<'a> {
    pub fn new(clickhouse: &'a ClickHouseConnector) -> Self {
        Self { clickhouse }
    }

    pub async fn fetch_ohlcv(
        &self,
        symbol: &Symbol,
        timeframe: &TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<u32>,
    ) -> Result<Vec<OhlcvData>> {
        self.clickhouse
            .get_ohlcv_typed(symbol, timeframe, start, end, limit)
            .await
    }

    pub async fn fetch_signals(
        &self,
        strategy_id: &str,
        symbol: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<u32>,
    ) -> Result<Vec<Signal>> {
        self.clickhouse
            .get_signals(strategy_id, symbol, start, end, limit)
            .await
    }

    pub async fn persist_trades(&self, records: &[TradeRecord]) -> Result<u64> {
        self.clickhouse.insert_trades(records).await
    }
}
