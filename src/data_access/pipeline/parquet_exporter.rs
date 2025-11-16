use std::sync::Arc;

use arrow::array::{Float32Builder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, Utc};

use crate::data_access::database::clickhouse::{ClickHouseConnector, OhlcvData};
use crate::data_access::database::parquet::ParquetConnector;
use crate::data_access::Result;
use crate::data_model::types::{Symbol, TimeFrame};

pub struct OhlcvParquetExporter<'a> {
    storage: &'a ClickHouseConnector,
    parquet: &'a ParquetConnector,
}

impl<'a> OhlcvParquetExporter<'a> {
    pub fn new(storage: &'a ClickHouseConnector, parquet: &'a ParquetConnector) -> Self {
        Self { storage, parquet }
    }

    pub async fn export_range(
        &self,
        symbol: &Symbol,
        timeframe: &TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<u32>,
        output_relative_path: &str,
    ) -> Result<()> {
        let rows = self
            .storage
            .get_ohlcv_typed(symbol, timeframe, start, end, limit)
            .await?;

        if rows.is_empty() {
            return Ok(());
        }

        let batch = build_ohlcv_batch(&rows)?;
        self.parquet
            .write_parquet(output_relative_path, vec![batch])
            .await
    }
}

fn build_ohlcv_batch(rows: &[OhlcvData]) -> Result<RecordBatch> {
    let mut symbol_builder = StringBuilder::new(rows.len());
    let mut timeframe_builder = StringBuilder::new(rows.len());
    let mut ts_builder = Int64Builder::new(rows.len());
    let mut open_builder = Float32Builder::new(rows.len());
    let mut high_builder = Float32Builder::new(rows.len());
    let mut low_builder = Float32Builder::new(rows.len());
    let mut close_builder = Float32Builder::new(rows.len());
    let mut volume_builder = Float32Builder::new(rows.len());

    for row in rows {
        symbol_builder.append_value(&row.symbol)?;
        timeframe_builder.append_value(&row.timeframe)?;
        ts_builder.append_value(row.timestamp.timestamp_micros())?;
        open_builder.append_value(row.open)?;
        high_builder.append_value(row.high)?;
        low_builder.append_value(row.low)?;
        close_builder.append_value(row.close)?;
        volume_builder.append_value(row.volume)?;
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("symbol", DataType::Utf8, false),
        Field::new("timeframe", DataType::Utf8, false),
        Field::new("timestamp", DataType::Int64, false),
        Field::new("open", DataType::Float32, false),
        Field::new("high", DataType::Float32, false),
        Field::new("low", DataType::Float32, false),
        Field::new("close", DataType::Float32, false),
        Field::new("volume", DataType::Float32, false),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(symbol_builder.finish()),
            Arc::new(timeframe_builder.finish()),
            Arc::new(ts_builder.finish()),
            Arc::new(open_builder.finish()),
            Arc::new(high_builder.finish()),
            Arc::new(low_builder.finish()),
            Arc::new(close_builder.finish()),
            Arc::new(volume_builder.finish()),
        ],
    )?;

    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn builds_record_batch() {
        let row = OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
        };

        let batch = build_ohlcv_batch(&[row]).expect("batch");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 8);
    }
}
