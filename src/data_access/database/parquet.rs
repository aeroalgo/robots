//! Parquet коннектор для работы с колоночными файлами

use crate::data_access::models::*;
use crate::data_access::traits::{ConnectionInfo, ConnectionStatus, DataSource};
use crate::data_access::{DataAccessError, Result};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

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
            created_at: chrono::Utc::now(),
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
    pub created_at: chrono::DateTime<chrono::Utc>,
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
        let file_path = ParquetUtils::create_candles_path(symbol, date);

        // Конвертируем Vec<Candle> в RecordBatch
        // Это упрощенная версия - в реальности нужна более сложная логика
        let batches = vec![];

        self.base_connector.write_parquet(&file_path, batches).await
    }

    /// Загрузка свечей из Parquet файла
    pub async fn load_candles(&self, symbol: &str, date: &str) -> Result<Vec<Candle>> {
        let file_path = ParquetUtils::create_candles_path(symbol, date);
        let batches = self.base_connector.read_parquet(&file_path).await?;

        // Конвертируем RecordBatch в Vec<Candle>
        // Это упрощенная версия - в реальности нужна более сложная логика
        Ok(vec![])
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
    base_connector: ParquetConnector,
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
        let file_path = ParquetUtils::create_backtest_path(strategy_id, date);

        // Конвертируем Vec<BacktestResult> в RecordBatch
        // Это упрощенная версия - в реальности нужна более сложная логика
        let batches = vec![];

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

        // Конвертируем RecordBatch в Vec<BacktestResult>
        // Это упрощенная версия - в реальности нужна более сложная логика
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parquet_config_default() {
        let config = ParquetConfig::default();
        assert_eq!(config.base_path, "./data/parquet");
        assert!(matches!(config.compression, ParquetCompression::Snappy));
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert!(config.create_directories);
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
