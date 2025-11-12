use chrono::{DateTime, Utc};

use crate::data_access::database::clickhouse::{ClickHouseConnector, OhlcvData};
use crate::data_access::Result;

use super::types::Symbol;
use super::types::TimeFrame;

impl ClickHouseConnector {
    pub async fn get_ohlcv_typed(
        &self,
        symbol: &Symbol,
        timeframe: &TimeFrame,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<u32>,
    ) -> Result<Vec<OhlcvData>> {
        let symbol_descriptor = symbol.descriptor();
        let timeframe_identifier = timeframe.identifier();

        self.get_ohlcv(
            &symbol_descriptor,
            &timeframe_identifier,
            start_time,
            end_time,
            limit,
        )
        .await
    }
}
