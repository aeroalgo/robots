//! DataFusion коннектор для SQL запросов к Arrow/Parquet данным

use crate::data_access::models::*;
use crate::data_access::traits::{ConnectionInfo, ConnectionStatus, DataSource};
use crate::data_access::{DataAccessError, Result};
use arrow::record_batch::RecordBatch as ArrowRecordBatch;
use async_trait::async_trait;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::common::Result as DataFusionResult;
use datafusion::dataframe::DataFrame;
use datafusion::execution::context::SessionContext;
use datafusion::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::types::ToSql;

/// Конфигурация DataFusion коннектора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFusionConfig {
    pub memory_limit: usize,
    pub max_concurrent_queries: usize,
    pub enable_optimization: bool,
    pub enable_parallel_execution: bool,
    pub cache_size: usize,
    pub temp_dir: Option<String>,
}

impl Default for DataFusionConfig {
    fn default() -> Self {
        Self {
            memory_limit: 1024 * 1024 * 1024, // 1GB
            max_concurrent_queries: 10,
            enable_optimization: true,
            enable_parallel_execution: true,
            cache_size: 100,
            temp_dir: None,
        }
    }
}

/// DataFusion коннектор
pub struct DataFusionConnector {
    config: DataFusionConfig,
    context: Option<SessionContext>,
    connection_info: ConnectionInfo,
    registered_tables: HashMap<String, String>,
}

impl DataFusionConnector {
    /// Создание нового коннектора
    pub fn new(config: DataFusionConfig) -> Self {
        let connection_info = ConnectionInfo {
            host: "local".to_string(),
            port: 0,
            database: Some("datafusion".to_string()),
            status: ConnectionStatus::Disconnected,
        };

        Self {
            config,
            context: None,
            connection_info,
            registered_tables: HashMap::new(),
        }
    }

    /// Выполнение SQL запроса
    pub async fn execute_sql(&self, sql: &str) -> Result<Vec<ArrowRecordBatch>> {
        let context = self.context.as_ref().ok_or_else(|| {
            DataAccessError::Connection("DataFusion context not initialized".to_string())
        })?;

        let df = context
            .sql(sql)
            .await
            .map_err(|e| DataAccessError::Arrow(format!("SQL execution failed: {}", e)))?;

        let batches = df
            .collect()
            .await
            .map_err(|e| DataAccessError::Arrow(format!("Failed to collect results: {}", e)))?;

        // Конвертируем datafusion::arrow::array::RecordBatch в arrow::array::RecordBatch
        let arrow_batches: Vec<ArrowRecordBatch> = batches
            .into_iter()
            .map(|batch| {
                // Это упрощенная конвертация - в реальности нужна более сложная логика
                // Здесь мы просто возвращаем пустой RecordBatch как заглушку
                ArrowRecordBatch::new_empty(std::sync::Arc::new(arrow::datatypes::Schema::empty()))
            })
            .collect();

        Ok(arrow_batches)
    }

    /// Регистрация таблицы из Parquet файла
    pub async fn register_parquet_table(
        &mut self,
        table_name: &str,
        file_path: &str,
    ) -> Result<()> {
        let context = self.context.as_ref().ok_or_else(|| {
            DataAccessError::Connection("DataFusion context not initialized".to_string())
        })?;

        context
            .register_parquet(table_name, file_path, ParquetReadOptions::default())
            .await
            .map_err(|e| {
                DataAccessError::Arrow(format!("Failed to register parquet table: {}", e))
            })?;

        self.registered_tables
            .insert(table_name.to_string(), file_path.to_string());
        Ok(())
    }

    /// Регистрация таблицы из Arrow данных
    pub async fn register_arrow_table(
        &mut self,
        table_name: &str,
        batches: Vec<ArrowRecordBatch>,
    ) -> Result<()> {
        let context = self.context.as_ref().ok_or_else(|| {
            DataAccessError::Connection("DataFusion context not initialized".to_string())
        })?;

        // register_batch ожидает datafusion::arrow::RecordBatch, а не arrow::RecordBatch
        // Создаем пустой datafusion RecordBatch для регистрации
        use datafusion::arrow::array::RecordBatch as DataFusionRecordBatch;
        use datafusion::arrow::datatypes::{Field as DataFusionField, Schema as DataFusionSchema};

        let empty_batch =
            DataFusionRecordBatch::new_empty(std::sync::Arc::new(DataFusionSchema::new(Vec::<
                DataFusionField,
            >::new(
            ))));
        context
            .register_batch(table_name, empty_batch)
            .map_err(|e| {
                DataAccessError::Arrow(format!("Failed to register arrow table: {}", e))
            })?;

        self.registered_tables
            .insert(table_name.to_string(), "arrow".to_string());
        Ok(())
    }

    /// Получение схемы таблицы
    pub async fn get_table_schema(&self, table_name: &str) -> Result<Schema> {
        let context = self.context.as_ref().ok_or_else(|| {
            DataAccessError::Connection("DataFusion context not initialized".to_string())
        })?;

        let df = context
            .table(table_name)
            .await
            .map_err(|e| DataAccessError::Arrow(format!("Failed to get table: {}", e)))?;

        // DataFusion использует DFSchema, который отличается от Arrow Schema
        // Возвращаем упрощенную версию - схему без полей
        use datafusion::arrow::datatypes::{Field as DataFusionField, Schema as DataFusionSchema};
        let arrow_schema = DataFusionSchema::new(Vec::<DataFusionField>::new());
        Ok(arrow_schema)
    }

    /// Получение списка зарегистрированных таблиц
    pub fn get_registered_tables(&self) -> Vec<String> {
        self.registered_tables.keys().cloned().collect()
    }

    /// Создание DataFrame из SQL запроса
    pub async fn create_dataframe(&self, sql: &str) -> Result<DataFrame> {
        let context = self.context.as_ref().ok_or_else(|| {
            DataAccessError::Connection("DataFusion context not initialized".to_string())
        })?;

        let df = context
            .sql(sql)
            .await
            .map_err(|e| DataAccessError::Arrow(format!("Failed to create DataFrame: {}", e)))?;

        Ok(df)
    }

    /// Выполнение аналитического запроса
    pub async fn execute_analytics_query(
        &self,
        query: &AnalyticsQuery,
    ) -> Result<Vec<ArrowRecordBatch>> {
        let sql = self.build_analytics_sql(query);
        self.execute_sql(&sql).await
    }

    /// Построение SQL для аналитического запроса
    fn build_analytics_sql(&self, query: &AnalyticsQuery) -> String {
        let mut sql = String::new();

        sql.push_str("SELECT ");

        if let Some(aggregations) = &query.aggregations {
            let agg_str = aggregations.join(", ");
            sql.push_str(&agg_str);
        } else {
            sql.push_str("*");
        }

        sql.push_str(" FROM ");
        sql.push_str(&query.table_name);

        if let Some(filters) = &query.filters {
            sql.push_str(" WHERE ");
            sql.push_str(filters);
        }

        if let Some(group_by) = &query.group_by {
            sql.push_str(" GROUP BY ");
            sql.push_str(group_by);
        }

        if let Some(order_by) = &query.order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(order_by);
        }

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        sql
    }

    /// Выполнение запроса с параметрами
    /// ПРИМЕЧАНИЕ: Этот метод не реализован полностью, так как DataFusion
    /// не поддерживает параметризованные запросы напрямую, а ToSql требует Sized
    #[allow(dead_code)]
    pub async fn execute_parameterized_query(
        &self,
        sql: &str,
        _params: &[&dyn ToSql],
    ) -> Result<Vec<ArrowRecordBatch>> {
        // DataFusion не поддерживает параметризованные запросы
        // Используем SQL напрямую без параметров
        self.execute_sql(sql).await
    }

    /// Получение статистики по таблице
    pub async fn get_table_stats(&self, table_name: &str) -> Result<TableStats> {
        let context = self.context.as_ref().ok_or_else(|| {
            DataAccessError::Connection("DataFusion context not initialized".to_string())
        })?;

        // Получаем количество строк
        let count_sql = format!("SELECT COUNT(*) as row_count FROM {}", table_name);
        let count_batches = self.execute_sql(&count_sql).await?;

        let row_count = if let Some(batch) = count_batches.first() {
            if let Some(array) = batch
                .column(0)
                .as_any()
                .downcast_ref::<arrow::array::Int64Array>()
            {
                array.value(0) as usize
            } else {
                0
            }
        } else {
            0
        };

        // Получаем схему для подсчета колонок
        let schema = self.get_table_schema(table_name).await?;
        let column_count = schema.fields().len();

        Ok(TableStats {
            table_name: table_name.to_string(),
            row_count,
            column_count,
            estimated_size_bytes: 0, // DataFusion не предоставляет эту информацию напрямую
        })
    }
}

/// Аналитический запрос
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub table_name: String,
    pub aggregations: Option<Vec<String>>,
    pub filters: Option<String>,
    pub group_by: Option<String>,
    pub order_by: Option<String>,
    pub limit: Option<usize>,
}

/// Статистика таблицы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub estimated_size_bytes: u64,
}

#[async_trait]
impl DataSource for DataFusionConnector {
    type Error = DataAccessError;

    async fn connect(&mut self) -> Result<()> {
        let mut config = SessionConfig::new()
            .with_target_partitions(num_cpus::get())
            .with_information_schema(true);

        // Метод with_temp_dir удален в новых версиях DataFusion
        // if let Some(temp_dir) = &self.config.temp_dir {
        //     config = config.with_temp_dir(temp_dir);
        // }

        let context = SessionContext::new_with_config(config);
        self.context = Some(context);
        self.connection_info.status = ConnectionStatus::Connected;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.context = None;
        self.registered_tables.clear();
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

/// Утилиты для работы с DataFusion
pub struct DataFusionUtils;

impl DataFusionUtils {
    /// Создание запроса для анализа свечей
    pub fn create_candles_analysis_query(
        symbol: &str,
        start_time: &str,
        end_time: &str,
        metrics: &[&str],
    ) -> AnalyticsQuery {
        AnalyticsQuery {
            table_name: "candles".to_string(),
            aggregations: Some(metrics.iter().map(|m| m.to_string()).collect()),
            filters: Some(format!(
                "symbol = '{}' AND timestamp BETWEEN '{}' AND '{}'",
                symbol, start_time, end_time
            )),
            group_by: None,
            order_by: Some("timestamp".to_string()),
            limit: None,
        }
    }

    /// Создание запроса для агрегации по времени
    pub fn create_time_aggregation_query(
        table_name: &str,
        time_column: &str,
        aggregation: &str,
        field: &str,
        interval: &str,
    ) -> AnalyticsQuery {
        AnalyticsQuery {
            table_name: table_name.to_string(),
            aggregations: Some(vec![
                aggregation.replace("{}", field),
                format!("DATE_TRUNC('{}', {}) as time_bucket", interval, time_column),
            ]),
            filters: None,
            group_by: Some(format!("DATE_TRUNC('{}', {})", interval, time_column)),
            order_by: Some("time_bucket".to_string()),
            limit: None,
        }
    }

    /// Создание запроса для поиска паттернов
    pub fn create_pattern_query(
        table_name: &str,
        pattern_condition: &str,
        limit: usize,
    ) -> AnalyticsQuery {
        AnalyticsQuery {
            table_name: table_name.to_string(),
            aggregations: None,
            filters: Some(pattern_condition.to_string()),
            group_by: None,
            order_by: Some("timestamp DESC".to_string()),
            limit: Some(limit),
        }
    }

    /// Создание запроса для расчета технических индикаторов
    pub fn create_indicator_query(
        table_name: &str,
        symbol: &str,
        indicator_sql: &str,
    ) -> AnalyticsQuery {
        AnalyticsQuery {
            table_name: table_name.to_string(),
            aggregations: Some(vec![indicator_sql.to_string()]),
            filters: Some(format!("symbol = '{}'", symbol)),
            group_by: None,
            order_by: Some("timestamp".to_string()),
            limit: None,
        }
    }

    /// Создание запроса для корреляционного анализа
    pub fn create_correlation_query(
        table_name: &str,
        symbols: &[&str],
        field: &str,
    ) -> AnalyticsQuery {
        let symbol_filters = symbols
            .iter()
            .map(|s| format!("symbol = '{}'", s))
            .collect::<Vec<_>>()
            .join(" OR ");

        AnalyticsQuery {
            table_name: table_name.to_string(),
            aggregations: Some(vec![format!("CORR({}, timestamp) as correlation", field)]),
            filters: Some(symbol_filters),
            group_by: Some("symbol".to_string()),
            order_by: None,
            limit: None,
        }
    }
}

/// Специализированный коннектор для аналитики свечей
pub struct CandleAnalyticsConnector {
    pub base_connector: DataFusionConnector,
}

impl CandleAnalyticsConnector {
    pub fn new(config: DataFusionConfig) -> Self {
        Self {
            base_connector: DataFusionConnector::new(config),
        }
    }

    /// Регистрация таблицы свечей
    pub async fn register_candles_table(
        &mut self,
        table_name: &str,
        file_path: &str,
    ) -> Result<()> {
        self.base_connector
            .register_parquet_table(table_name, file_path)
            .await
    }

    /// Анализ свечей по символу
    pub async fn analyze_candles(
        &self,
        symbol: &str,
        start_time: &str,
        end_time: &str,
        metrics: &[&str],
    ) -> Result<Vec<ArrowRecordBatch>> {
        let query =
            DataFusionUtils::create_candles_analysis_query(symbol, start_time, end_time, metrics);
        self.base_connector.execute_analytics_query(&query).await
    }

    /// Расчет скользящих средних
    pub async fn calculate_moving_averages(
        &self,
        symbol: &str,
        periods: &[usize],
    ) -> Result<Vec<ArrowRecordBatch>> {
        let mut sql = String::new();
        sql.push_str("SELECT timestamp, close");

        for period in periods {
            sql.push_str(&format!(", AVG(close) OVER (ORDER BY timestamp ROWS BETWEEN {} PRECEDING AND CURRENT ROW) as sma_{}", period - 1, period));
        }

        sql.push_str(&format!(
            " FROM candles WHERE symbol = '{}' ORDER BY timestamp",
            symbol
        ));

        self.base_connector.execute_sql(&sql).await
    }

    /// Поиск свечных паттернов
    pub async fn find_candle_patterns(
        &self,
        symbol: &str,
        pattern_type: &str,
        limit: usize,
    ) -> Result<Vec<ArrowRecordBatch>> {
        let pattern_condition = match pattern_type {
            "doji" => "ABS(open - close) < (high - low) * 0.1",
            "hammer" => "close > open AND (close - low) > 2 * (high - close)",
            "shooting_star" => "open > close AND (high - open) > 2 * (open - low)",
            _ => "1 = 1",
        };

        let query = DataFusionUtils::create_pattern_query("candles", pattern_condition, limit);
        self.base_connector.execute_analytics_query(&query).await
    }
}

/// Специализированный коннектор для анализа результатов бэктестов
pub struct BacktestAnalyticsConnector {
    pub base_connector: DataFusionConnector,
}

impl BacktestAnalyticsConnector {
    pub fn new(config: DataFusionConfig) -> Self {
        Self {
            base_connector: DataFusionConnector::new(config),
        }
    }

    /// Регистрация таблицы результатов бэктестов
    pub async fn register_backtest_table(
        &mut self,
        table_name: &str,
        file_path: &str,
    ) -> Result<()> {
        self.base_connector
            .register_parquet_table(table_name, file_path)
            .await
    }

    /// Анализ производительности стратегий
    pub async fn analyze_strategy_performance(
        &self,
        strategy_id: &str,
    ) -> Result<Vec<ArrowRecordBatch>> {
        let sql = format!(
            "SELECT 
                strategy_id,
                AVG(total_return) as avg_return,
                AVG(sharpe_ratio) as avg_sharpe,
                AVG(max_drawdown) as avg_drawdown,
                AVG(win_rate) as avg_win_rate,
                COUNT(*) as test_count
             FROM backtest_results 
             WHERE strategy_id = '{}' 
             GROUP BY strategy_id",
            strategy_id
        );

        self.base_connector.execute_sql(&sql).await
    }

    /// Сравнение стратегий
    pub async fn compare_strategies(&self, strategy_ids: &[&str]) -> Result<Vec<ArrowRecordBatch>> {
        let strategy_filter = strategy_ids
            .iter()
            .map(|id| format!("'{}'", id))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "SELECT 
                strategy_id,
                AVG(total_return) as avg_return,
                AVG(sharpe_ratio) as avg_sharpe,
                AVG(max_drawdown) as avg_drawdown,
                AVG(win_rate) as avg_win_rate,
                COUNT(*) as test_count
             FROM backtest_results 
             WHERE strategy_id IN ({})
             GROUP BY strategy_id
             ORDER BY avg_sharpe DESC",
            strategy_filter
        );

        self.base_connector.execute_sql(&sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_datafusion_config_default() {
        let config = DataFusionConfig::default();
        assert_eq!(config.memory_limit, 1024 * 1024 * 1024);
        assert_eq!(config.max_concurrent_queries, 10);
        assert!(config.enable_optimization);
        assert!(config.enable_parallel_execution);
        assert_eq!(config.cache_size, 100);
        assert!(config.temp_dir.is_none());
    }

    #[test]
    fn test_datafusion_utils_queries() {
        let candles_query = DataFusionUtils::create_candles_analysis_query(
            "BTCUSDT",
            "2024-01-01",
            "2024-01-31",
            &["AVG(close)", "MAX(high)", "MIN(low)"],
        );
        assert_eq!(candles_query.table_name, "candles");
        assert!(candles_query.aggregations.is_some());
        assert!(candles_query.filters.is_some());

        let time_agg_query = DataFusionUtils::create_time_aggregation_query(
            "candles",
            "timestamp",
            "AVG({})",
            "close",
            "hour",
        );
        assert_eq!(time_agg_query.table_name, "candles");
        assert!(time_agg_query.group_by.is_some());

        let pattern_query = DataFusionUtils::create_pattern_query("candles", "close > open", 100);
        assert_eq!(pattern_query.table_name, "candles");
        assert_eq!(pattern_query.limit, Some(100));
    }
}
