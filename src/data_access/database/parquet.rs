//! Parquet коннектор для работы с колоночными файлами

use crate::data_access::models::*;
use crate::data_access::traits::{ConnectionInfo, ConnectionStatus, DataSource};
use crate::data_access::{DataAccessError, Result};
use arrow::array::{
    Array, ArrayRef, Float32Array, Float32Builder, StringArray, StringBuilder,
    TimestampMicrosecondArray, TimestampMicrosecondBuilder, UInt32Array, UInt32Builder,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;

/// Конфигурация Parquet коннектора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetConfig {
    pub base_path: String,
    pub compression: ParquetCompression,
    pub batch_size: usize,
    pub max_file_size: u64,
    pub create_directories: bool,
}

/// Типы сжатия Parquet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParquetCompression {
    None,
    Snappy,
    Gzip,
    Lz4,
    Zstd,
}

impl Default for ParquetConfig {
    fn default() -> Self {
        Self {
            base_path: "./data/parquet".to_string(),
            compression: ParquetCompression::Snappy,
            batch_size: 1000,
            max_file_size: 100 * 1024 * 1024, // 100MB
            create_directories: true,
        }
    }
}

/// Parquet коннектор
pub struct ParquetConnector {
    config: ParquetConfig,
    connection_info: ConnectionInfo,
}

impl ParquetConnector {
    /// Создание нового коннектора
    pub fn new(config: ParquetConfig) -> Self {
        let connection_info = ConnectionInfo {
            host: "local".to_string(),
            port: 0,
            database: Some(config.base_path.clone()),
            status: ConnectionStatus::Disconnected,
        };

        Self {
            config,
            connection_info,
        }
    }

    /// Чтение Parquet файла
    pub async fn read_parquet(&self, file_path: &str) -> Result<Vec<RecordBatch>> {
        let path = Path::new(&self.config.base_path).join(file_path);

        if !path.exists() {
            return Err(DataAccessError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )));
        }

        let file = File::open(&path).map_err(|e| DataAccessError::Io(e))?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            DataAccessError::Arrow(format!("Failed to create parquet reader: {}", e))
        })?;

        let reader = builder
            .with_batch_size(self.config.batch_size)
            .build()
            .map_err(|e| {
                DataAccessError::Arrow(format!("Failed to build parquet reader: {}", e))
            })?;

        let mut batches = Vec::new();
        for batch_result in reader {
            let batch = batch_result
                .map_err(|e| DataAccessError::Arrow(format!("Failed to read batch: {}", e)))?;
            batches.push(batch);
        }

        Ok(batches)
    }

    /// Запись в Parquet файл
    pub async fn write_parquet(&self, file_path: &str, batches: Vec<RecordBatch>) -> Result<()> {
        if batches.is_empty() {
            return Err(DataAccessError::Arrow(
                "No record batches provided for Parquet write".to_string(),
            ));
        }

        let path = Path::new(&self.config.base_path).join(file_path);

        if self.config.create_directories {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| DataAccessError::Io(e))?;
            }
        }

        let file = File::create(&path).map_err(|e| DataAccessError::Io(e))?;

        let writer_props = WriterProperties::builder()
            .set_compression(self.get_compression_type())
            .set_max_row_group_size(self.config.batch_size)
            .build();

        let mut writer = ArrowWriter::try_new(file, batches[0].schema(), Some(writer_props))
            .map_err(|e| {
                DataAccessError::Arrow(format!("Failed to create parquet writer: {}", e))
            })?;

        for batch in batches {
            writer
                .write(&batch)
                .map_err(|e| DataAccessError::Arrow(format!("Failed to write batch: {}", e)))?;
        }

        writer.close().map_err(|e| {
            DataAccessError::Arrow(format!("Failed to close parquet writer: {}", e))
        })?;

        Ok(())
    }

    /// Получение схемы Parquet файла
    pub async fn get_schema(&self, file_path: &str) -> Result<String> {
        let path = Path::new(&self.config.base_path).join(file_path);

        if !path.exists() {
            return Err(DataAccessError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )));
        }

        let file = File::open(&path).map_err(|e| DataAccessError::Io(e))?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            DataAccessError::Arrow(format!("Failed to create parquet reader: {}", e))
        })?;

        let schema = builder.schema();
        Ok(format!("{:?}", schema))
    }

    /// Получение метаданных Parquet файла
    pub async fn get_metadata(&self, file_path: &str) -> Result<ParquetMetadata> {
        let path = Path::new(&self.config.base_path).join(file_path);

        if !path.exists() {
            return Err(DataAccessError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            )));
        }

        let file = File::open(&path).map_err(|e| DataAccessError::Io(e))?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            DataAccessError::Arrow(format!("Failed to create parquet reader: {}", e))
        })?;

        let metadata = builder.metadata();
        let file_size = fs::metadata(&path)
            .await
            .map_err(|e| DataAccessError::Io(e))?
            .len();

        Ok(ParquetMetadata {
            file_path: file_path.to_string(),
            file_size,
            num_rows: metadata.file_metadata().num_rows() as usize,
            num_columns: metadata.file_metadata().schema().get_fields().len(),
            created_at: Utc::now(),
        })
    }

    /// Список файлов в директории
    pub async fn list_files(&self, directory: &str) -> Result<Vec<String>> {
        let path = Path::new(&self.config.base_path).join(directory);

        if !path.exists() {
            return Ok(vec![]);
        }

        let mut entries = fs::read_dir(&path)
            .await
            .map_err(|e| DataAccessError::Io(e))?;

        let mut files = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DataAccessError::Io(e))?
        {
            if entry
                .file_type()
                .await
                .map_err(|e| DataAccessError::Io(e))?
                .is_file()
            {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".parquet") {
                        files.push(file_name.to_string());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Удаление файла
    pub async fn delete_file(&self, file_path: &str) -> Result<()> {
        let path = Path::new(&self.config.base_path).join(file_path);

        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|e| DataAccessError::Io(e))?;
        }

        Ok(())
    }

    /// Получение типа сжатия для WriterProperties
    fn get_compression_type(&self) -> parquet::basic::Compression {
        match self.config.compression {
            ParquetCompression::None => parquet::basic::Compression::UNCOMPRESSED,
            ParquetCompression::Snappy => parquet::basic::Compression::SNAPPY,
            ParquetCompression::Gzip => {
                parquet::basic::Compression::GZIP(parquet::basic::GzipLevel::default())
            }
            ParquetCompression::Lz4 => parquet::basic::Compression::LZ4,
            ParquetCompression::Zstd => {
                parquet::basic::Compression::ZSTD(parquet::basic::ZstdLevel::default())
            }
        }
    }
}

/// Метаданные Parquet файла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParquetMetadata {
    pub file_path: String,
    pub file_size: u64,
    pub num_rows: usize,
    pub num_columns: usize,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
impl DataSource for ParquetConnector {
    type Error = DataAccessError;

    async fn connect(&mut self) -> Result<()> {
        if self.config.create_directories {
            fs::create_dir_all(&self.config.base_path)
                .await
                .map_err(|e| DataAccessError::Io(e))?;
        }

        self.connection_info.status = ConnectionStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connection_info.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        matches!(self.connection_info.status, ConnectionStatus::Connected)
    }

    fn connection_info(&self) -> ConnectionInfo {
        self.connection_info.clone()
    }
}

/// Утилиты для работы с Parquet
pub struct ParquetUtils;

impl ParquetUtils {
    /// Создание пути для свечей
    pub fn create_candles_path(symbol: &str, date: &str) -> String {
        format!("candles/{}/{}.parquet", symbol, date)
    }

    /// Создание пути для сделок
    pub fn create_trades_path(symbol: &str, date: &str) -> String {
        format!("trades/{}/{}.parquet", symbol, date)
    }

    /// Создание пути для результатов бэктестов
    pub fn create_backtest_path(strategy_id: &str, date: &str) -> String {
        format!("backtests/{}/{}.parquet", strategy_id, date)
    }

    /// Создание пути для метрик
    pub fn create_metrics_path(strategy_id: &str, date: &str) -> String {
        format!("metrics/{}/{}.parquet", strategy_id, date)
    }

    /// Получение даты из пути файла
    pub fn extract_date_from_path(file_path: &str) -> Option<String> {
        let path = Path::new(file_path);
        if let Some(stem) = path.file_stem() {
            stem.to_str().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Проверка является ли файл Parquet
    pub fn is_parquet_file(file_path: &str) -> bool {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("parquet"))
            .unwrap_or(false)
    }

    /// Создание имени файла с временной меткой
    pub fn create_timestamped_filename(prefix: &str, extension: &str) -> String {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        format!("{}_{}.{}", prefix, timestamp, extension)
    }
}

/// Специализированный коннектор для свечей
pub struct CandleParquetConnector {
    pub base_connector: ParquetConnector,
}

impl CandleParquetConnector {
    pub fn new(config: ParquetConfig) -> Self {
        Self {
            base_connector: ParquetConnector::new(config),
        }
    }

    /// Сохранение свечей в Parquet файл
    pub async fn save_candles(&self, symbol: &str, date: &str, candles: Vec<Candle>) -> Result<()> {
        if candles.is_empty() {
            return Ok(());
        }

        let file_path = ParquetUtils::create_candles_path(symbol, date);
        let batches = candles_to_batches(&candles, self.base_connector.config.batch_size)?;
        self.base_connector.write_parquet(&file_path, batches).await
    }

    /// Загрузка свечей из Parquet файла
    pub async fn load_candles(&self, symbol: &str, date: &str) -> Result<Vec<Candle>> {
        let file_path = ParquetUtils::create_candles_path(symbol, date);
        let batches = self.base_connector.read_parquet(&file_path).await?;
        batches_to_candles(&batches)
    }

    /// Получение списка доступных дат для символа
    pub async fn get_available_dates(&self, symbol: &str) -> Result<Vec<String>> {
        let directory = format!("candles/{}", symbol);
        let files = self.base_connector.list_files(&directory).await?;

        let dates: Vec<String> = files
            .into_iter()
            .filter_map(|file| ParquetUtils::extract_date_from_path(&file))
            .collect();

        Ok(dates)
    }
}

/// Специализированный коннектор для результатов бэктестов
pub struct BacktestParquetConnector {
    pub base_connector: ParquetConnector,
}

impl BacktestParquetConnector {
    pub fn new(config: ParquetConfig) -> Self {
        Self {
            base_connector: ParquetConnector::new(config),
        }
    }

    /// Сохранение результатов бэктеста
    pub async fn save_backtest_results(
        &self,
        strategy_id: &str,
        date: &str,
        results: Vec<BacktestResult>,
    ) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        let file_path = ParquetUtils::create_backtest_path(strategy_id, date);
        let batches = backtests_to_batches(&results, self.base_connector.config.batch_size)?;
        self.base_connector.write_parquet(&file_path, batches).await
    }

    /// Загрузка результатов бэктеста
    pub async fn load_backtest_results(
        &self,
        strategy_id: &str,
        date: &str,
    ) -> Result<Vec<BacktestResult>> {
        let file_path = ParquetUtils::create_backtest_path(strategy_id, date);
        let batches = self.base_connector.read_parquet(&file_path).await?;
        batches_to_backtests(&batches)
    }
}

fn candles_to_batches(candles: &[Candle], batch_size: usize) -> Result<Vec<RecordBatch>> {
    let chunk_size = batch_size.max(1);
    let mut batches = Vec::new();
    for chunk in candles.chunks(chunk_size) {
        batches.push(build_candle_batch(chunk)?);
    }
    Ok(batches)
}

fn build_candle_batch(chunk: &[Candle]) -> Result<RecordBatch> {
    let mut timestamp_builder = TimestampMicrosecondBuilder::with_capacity(chunk.len());
    let mut symbol_builder = StringBuilder::with_capacity(chunk.len(), chunk.len() * 32);
    let mut open_builder = Float32Builder::with_capacity(chunk.len());
    let mut high_builder = Float32Builder::with_capacity(chunk.len());
    let mut low_builder = Float32Builder::with_capacity(chunk.len());
    let mut close_builder = Float32Builder::with_capacity(chunk.len());
    let mut volume_builder = Float32Builder::with_capacity(chunk.len());

    for candle in chunk {
        timestamp_builder.append_value(candle.timestamp.timestamp_micros());
        symbol_builder.append_value(&candle.symbol);
        open_builder.append_value(candle.open);
        high_builder.append_value(candle.high);
        low_builder.append_value(candle.low);
        close_builder.append_value(candle.close);
        volume_builder.append_value(candle.volume);
    }

    let schema = candle_schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(timestamp_builder.finish()) as ArrayRef,
        Arc::new(symbol_builder.finish()) as ArrayRef,
        Arc::new(open_builder.finish()) as ArrayRef,
        Arc::new(high_builder.finish()) as ArrayRef,
        Arc::new(low_builder.finish()) as ArrayRef,
        Arc::new(close_builder.finish()) as ArrayRef,
        Arc::new(volume_builder.finish()) as ArrayRef,
    ];

    Ok(RecordBatch::try_new(schema, columns)?)
}

fn candle_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
        Field::new("symbol", DataType::Utf8, false),
        Field::new("open", DataType::Float32, false),
        Field::new("high", DataType::Float32, false),
        Field::new("low", DataType::Float32, false),
        Field::new("close", DataType::Float32, false),
        Field::new("volume", DataType::Float32, false),
    ]))
}

fn batches_to_candles(batches: &[RecordBatch]) -> Result<Vec<Candle>> {
    let mut result = Vec::new();
    for batch in batches {
        result.extend(batch_to_candles(batch)?);
    }
    Ok(result)
}

fn batch_to_candles(batch: &RecordBatch) -> Result<Vec<Candle>> {
    let timestamps = column_as::<TimestampMicrosecondArray>(batch, "timestamp")?;
    let symbols = column_as::<StringArray>(batch, "symbol")?;
    let open = column_as::<Float32Array>(batch, "open")?;
    let high = column_as::<Float32Array>(batch, "high")?;
    let low = column_as::<Float32Array>(batch, "low")?;
    let close = column_as::<Float32Array>(batch, "close")?;
    let volume = column_as::<Float32Array>(batch, "volume")?;

    let mut rows = Vec::with_capacity(batch.num_rows());
    for i in 0..batch.num_rows() {
        rows.push(Candle {
            timestamp: timestamp_from_micros(timestamps.value(i))?,
            symbol: symbols.value(i).to_string(),
            open: open.value(i),
            high: high.value(i),
            low: low.value(i),
            close: close.value(i),
            volume: volume.value(i),
        });
    }

    Ok(rows)
}

fn backtests_to_batches(results: &[BacktestResult], batch_size: usize) -> Result<Vec<RecordBatch>> {
    let chunk_size = batch_size.max(1);
    let mut batches = Vec::new();
    for chunk in results.chunks(chunk_size) {
        batches.push(build_backtest_batch(chunk)?);
    }
    Ok(batches)
}

fn build_backtest_batch(chunk: &[BacktestResult]) -> Result<RecordBatch> {
    let mut strategy_builder = StringBuilder::with_capacity(chunk.len(), chunk.len() * 32);
    let mut symbol_builder = StringBuilder::with_capacity(chunk.len(), chunk.len() * 32);
    let mut start_builder = TimestampMicrosecondBuilder::with_capacity(chunk.len());
    let mut end_builder = TimestampMicrosecondBuilder::with_capacity(chunk.len());
    let mut total_return_builder = Float32Builder::with_capacity(chunk.len());
    let mut sharpe_builder = Float32Builder::with_capacity(chunk.len());
    let mut drawdown_builder = Float32Builder::with_capacity(chunk.len());
    let mut total_trades_builder = UInt32Builder::with_capacity(chunk.len());
    let mut winning_builder = UInt32Builder::with_capacity(chunk.len());
    let mut losing_builder = UInt32Builder::with_capacity(chunk.len());
    let mut win_rate_builder = Float32Builder::with_capacity(chunk.len());
    let mut created_builder = TimestampMicrosecondBuilder::with_capacity(chunk.len());

    for result in chunk {
        strategy_builder.append_value(&result.strategy_id);
        symbol_builder.append_value(&result.symbol);
        start_builder.append_value(result.start_date.timestamp_micros());
        end_builder.append_value(result.end_date.timestamp_micros());
        total_return_builder.append_value(result.total_return);
        sharpe_builder.append_value(result.sharpe_ratio);
        drawdown_builder.append_value(result.max_drawdown);
        total_trades_builder.append_value(result.total_trades);
        winning_builder.append_value(result.winning_trades);
        losing_builder.append_value(result.losing_trades);
        win_rate_builder.append_value(result.win_rate);
        created_builder.append_value(result.created_at.timestamp_micros());
    }

    let schema = backtest_schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(strategy_builder.finish()) as ArrayRef,
        Arc::new(symbol_builder.finish()) as ArrayRef,
        Arc::new(start_builder.finish()) as ArrayRef,
        Arc::new(end_builder.finish()) as ArrayRef,
        Arc::new(total_return_builder.finish()) as ArrayRef,
        Arc::new(sharpe_builder.finish()) as ArrayRef,
        Arc::new(drawdown_builder.finish()) as ArrayRef,
        Arc::new(total_trades_builder.finish()) as ArrayRef,
        Arc::new(winning_builder.finish()) as ArrayRef,
        Arc::new(losing_builder.finish()) as ArrayRef,
        Arc::new(win_rate_builder.finish()) as ArrayRef,
        Arc::new(created_builder.finish()) as ArrayRef,
    ];

    Ok(RecordBatch::try_new(schema, columns)?)
}

fn backtest_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("strategy_id", DataType::Utf8, false),
        Field::new("symbol", DataType::Utf8, false),
        Field::new(
            "start_date",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
        Field::new(
            "end_date",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
        Field::new("total_return", DataType::Float32, false),
        Field::new("sharpe_ratio", DataType::Float32, false),
        Field::new("max_drawdown", DataType::Float32, false),
        Field::new("total_trades", DataType::UInt32, false),
        Field::new("winning_trades", DataType::UInt32, false),
        Field::new("losing_trades", DataType::UInt32, false),
        Field::new("win_rate", DataType::Float32, false),
        Field::new(
            "created_at",
            DataType::Timestamp(TimeUnit::Microsecond, None),
            false,
        ),
    ]))
}

fn batches_to_backtests(batches: &[RecordBatch]) -> Result<Vec<BacktestResult>> {
    let mut result = Vec::new();
    for batch in batches {
        result.extend(batch_to_backtests(batch)?);
    }
    Ok(result)
}

fn batch_to_backtests(batch: &RecordBatch) -> Result<Vec<BacktestResult>> {
    let strategy = column_as::<StringArray>(batch, "strategy_id")?;
    let symbol = column_as::<StringArray>(batch, "symbol")?;
    let start_date = column_as::<TimestampMicrosecondArray>(batch, "start_date")?;
    let end_date = column_as::<TimestampMicrosecondArray>(batch, "end_date")?;
    let total_return = column_as::<Float32Array>(batch, "total_return")?;
    let sharpe = column_as::<Float32Array>(batch, "sharpe_ratio")?;
    let drawdown = column_as::<Float32Array>(batch, "max_drawdown")?;
    let total_trades = column_as::<UInt32Array>(batch, "total_trades")?;
    let winning_trades = column_as::<UInt32Array>(batch, "winning_trades")?;
    let losing_trades = column_as::<UInt32Array>(batch, "losing_trades")?;
    let win_rate = column_as::<Float32Array>(batch, "win_rate")?;
    let created_at = column_as::<TimestampMicrosecondArray>(batch, "created_at")?;

    let mut rows = Vec::with_capacity(batch.num_rows());
    for i in 0..batch.num_rows() {
        rows.push(BacktestResult {
            strategy_id: strategy.value(i).to_string(),
            symbol: symbol.value(i).to_string(),
            start_date: timestamp_from_micros(start_date.value(i))?,
            end_date: timestamp_from_micros(end_date.value(i))?,
            total_return: total_return.value(i),
            sharpe_ratio: sharpe.value(i),
            max_drawdown: drawdown.value(i),
            total_trades: total_trades.value(i),
            winning_trades: winning_trades.value(i),
            losing_trades: losing_trades.value(i),
            win_rate: win_rate.value(i),
            created_at: timestamp_from_micros(created_at.value(i))?,
        });
    }

    Ok(rows)
}

fn timestamp_from_micros(value: i64) -> Result<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp_micros(value)
        .ok_or_else(|| DataAccessError::Arrow(format!("Invalid timestamp: {}", value)))
}

fn column_as<'a, T>(batch: &'a RecordBatch, name: &str) -> Result<&'a T>
where
    T: Array + 'static,
{
    let idx = batch
        .schema()
        .index_of(name)
        .map_err(|_| DataAccessError::Arrow(format!("Column not found: {}", name)))?;
    batch
        .column(idx)
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| DataAccessError::Arrow(format!("Column has unexpected type: {}", name)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_parquet_config_default() {
        let config = ParquetConfig::default();
        assert_eq!(config.base_path, "./data/parquet");
        assert!(matches!(config.compression, ParquetCompression::Snappy));
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert!(config.create_directories);
    }

    #[tokio::test]
    async fn candle_parquet_roundtrip() {
        let dir = tempdir().unwrap();
        let config = ParquetConfig {
            base_path: dir.path().to_string_lossy().into_owned(),
            compression: ParquetCompression::Snappy,
            batch_size: 2,
            max_file_size: 1024 * 1024,
            create_directories: true,
        };
        let mut connector = CandleParquetConnector::new(config);
        connector.base_connector.connect().await.unwrap();

        let candles = vec![
            Candle {
                timestamp: chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                symbol: "BTCUSDT".to_string(),
                open: 1.0,
                high: 2.0,
                low: 0.5,
                close: 1.5,
                volume: 10.0,
            },
            Candle {
                timestamp: chrono::Utc.with_ymd_and_hms(2024, 1, 1, 1, 0, 0).unwrap(),
                symbol: "BTCUSDT".to_string(),
                open: 2.0,
                high: 3.0,
                low: 1.5,
                close: 2.5,
                volume: 12.0,
            },
        ];

        connector
            .save_candles("BTCUSDT", "2024-01-01", candles.clone())
            .await
            .unwrap();
        let loaded = connector
            .load_candles("BTCUSDT", "2024-01-01")
            .await
            .unwrap();
        assert_eq!(loaded.len(), candles.len());
        assert!((loaded[0].close - candles[0].close).abs() < f32::EPSILON);
        assert!((loaded[1].high - candles[1].high).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn backtest_parquet_roundtrip() {
        let dir = tempdir().unwrap();
        let config = ParquetConfig {
            base_path: dir.path().to_string_lossy().into_owned(),
            compression: ParquetCompression::None,
            batch_size: 1,
            max_file_size: 1024 * 1024,
            create_directories: true,
        };
        let mut connector = BacktestParquetConnector::new(config);
        connector.base_connector.connect().await.unwrap();

        let results = vec![
            BacktestResult {
                strategy_id: "strat".to_string(),
                symbol: "BTCUSDT".to_string(),
                start_date: chrono::Utc.with_ymd_and_hms(2023, 12, 1, 0, 0, 0).unwrap(),
                end_date: chrono::Utc.with_ymd_and_hms(2023, 12, 31, 0, 0, 0).unwrap(),
                total_return: 0.12,
                sharpe_ratio: 1.5,
                max_drawdown: -0.05,
                total_trades: 25,
                winning_trades: 15,
                losing_trades: 10,
                win_rate: 0.6,
                created_at: chrono::Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
            },
            BacktestResult {
                strategy_id: "strat".to_string(),
                symbol: "ETHUSDT".to_string(),
                start_date: chrono::Utc.with_ymd_and_hms(2023, 11, 1, 0, 0, 0).unwrap(),
                end_date: chrono::Utc.with_ymd_and_hms(2023, 11, 30, 0, 0, 0).unwrap(),
                total_return: 0.2,
                sharpe_ratio: 2.0,
                max_drawdown: -0.07,
                total_trades: 30,
                winning_trades: 18,
                losing_trades: 12,
                win_rate: 0.6,
                created_at: chrono::Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(),
            },
        ];

        connector
            .save_backtest_results("strat", "2024-01-01", results.clone())
            .await
            .unwrap();
        let loaded = connector
            .load_backtest_results("strat", "2024-01-01")
            .await
            .unwrap();
        assert_eq!(loaded.len(), results.len());
        assert!((loaded[0].sharpe_ratio - results[0].sharpe_ratio).abs() < f32::EPSILON);
        assert_eq!(loaded[1].symbol, results[1].symbol);
    }

    #[test]
    fn test_parquet_utils_paths() {
        let candles_path = ParquetUtils::create_candles_path("BTCUSDT", "2024-01-01");
        assert_eq!(candles_path, "candles/BTCUSDT/2024-01-01.parquet");

        let trades_path = ParquetUtils::create_trades_path("ETHUSDT", "2024-01-01");
        assert_eq!(trades_path, "trades/ETHUSDT/2024-01-01.parquet");

        let backtest_path = ParquetUtils::create_backtest_path("strategy_1", "2024-01-01");
        assert_eq!(backtest_path, "backtests/strategy_1/2024-01-01.parquet");

        let metrics_path = ParquetUtils::create_metrics_path("strategy_1", "2024-01-01");
        assert_eq!(metrics_path, "metrics/strategy_1/2024-01-01.parquet");
    }

    #[test]
    fn test_parquet_utils_helpers() {
        let date = ParquetUtils::extract_date_from_path("candles/BTCUSDT/2024-01-01.parquet");
        assert_eq!(date, Some("2024-01-01".to_string()));

        assert!(ParquetUtils::is_parquet_file("test.parquet"));
        assert!(ParquetUtils::is_parquet_file("test.PARQUET"));
        assert!(!ParquetUtils::is_parquet_file("test.txt"));

        let filename = ParquetUtils::create_timestamped_filename("test", "parquet");
        assert!(filename.starts_with("test_"));
        assert!(filename.ends_with(".parquet"));
    }
}
