use arrow::record_batch::RecordBatch;
use chrono::{DateTime, Utc};

use crate::data_access::database::clickhouse::ClickHouseConnector;
use crate::data_access::database::parquet::ParquetConnector;
use crate::data_access::pipeline::OhlcvParquetExporter;
use crate::data_access::Result;
use crate::data_model::types::{Symbol, TimeFrame};

pub struct AnalyticsRepository<'a> {
    exporter: OhlcvParquetExporter<'a>,
    parquet: &'a ParquetConnector,
}

impl<'a> AnalyticsRepository<'a> {
    pub fn new(storage: &'a ClickHouseConnector, parquet: &'a ParquetConnector) -> Self {
        Self {
            exporter: OhlcvParquetExporter::new(storage, parquet),
            parquet,
        }
    }

    pub async fn export_ohlcv_segment(
        &self,
        symbol: &Symbol,
        timeframe: &TimeFrame,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<u32>,
        output_path: &str,
    ) -> Result<()> {
        self.exporter
            .export_range(symbol, timeframe, start, end, limit, output_path)
            .await
    }

    pub async fn list_parquet_segments(&self, directory: &str) -> Result<Vec<String>> {
        self.parquet.list_files(directory).await
    }

    pub async fn load_segment(&self, path: &str) -> Result<Vec<RecordBatch>> {
        self.parquet.read_parquet(path).await
    }
}
