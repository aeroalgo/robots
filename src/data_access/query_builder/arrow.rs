//! Arrow Query Builder для работы с Arrow/Parquet данными

use crate::data_access::models::*;
use crate::data_access::{DataAccessError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Базовый Arrow Query Builder
#[derive(Debug, Clone)]
pub struct ArrowQueryBuilder {
    table_name: String,
    columns: Vec<String>,
    filters: Vec<FilterCondition>,
    aggregations: Vec<Aggregation>,
    group_by: Vec<String>,
    order_by: Vec<OrderBy>,
    limit: Option<usize>,
    offset: Option<usize>,
}

/// Условие фильтрации
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub column: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
    pub logical_operator: Option<LogicalOperator>,
}

/// Оператор фильтрации
#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    In,
    Between,
    IsNull,
    IsNotNull,
}

/// Значение фильтра
#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Number(f32),
    Integer(i64),
    Boolean(bool),
    List(Vec<FilterValue>),
    Range(Box<FilterValue>, Box<FilterValue>),
    Null,
}

/// Логический оператор
#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

/// Агрегация
#[derive(Debug, Clone)]
pub struct Aggregation {
    pub function: AggregationFunction,
    pub column: String,
    pub alias: Option<String>,
}

/// Функция агрегации
#[derive(Debug, Clone)]
pub enum AggregationFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    StdDev,
    Variance,
    First,
    Last,
    Distinct,
}

/// Сортировка
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub column: String,
    pub direction: SortDirection,
}

/// Направление сортировки
#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl ArrowQueryBuilder {
    /// Создание нового Query Builder
    pub fn new(table_name: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            columns: Vec::new(),
            filters: Vec::new(),
            aggregations: Vec::new(),
            group_by: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Добавление колонок для SELECT
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Добавление всех колонок
    pub fn select_all(mut self) -> Self {
        self.columns.clear();
        self
    }

    /// Добавление условия фильтрации
    pub fn filter(mut self, condition: FilterCondition) -> Self {
        self.filters.push(condition);
        self
    }

    /// Добавление условия равенства
    pub fn where_equal(mut self, column: &str, value: FilterValue) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator: FilterOperator::Equal,
            value,
            logical_operator: None,
        });
        self
    }

    /// Добавление условия больше
    pub fn where_greater_than(mut self, column: &str, value: FilterValue) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator: FilterOperator::GreaterThan,
            value,
            logical_operator: None,
        });
        self
    }

    /// Добавление условия меньше
    pub fn where_less_than(mut self, column: &str, value: FilterValue) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator: FilterOperator::LessThan,
            value,
            logical_operator: None,
        });
        self
    }

    /// Добавление условия BETWEEN
    pub fn where_between(mut self, column: &str, start: FilterValue, end: FilterValue) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator: FilterOperator::Between,
            value: FilterValue::Range(Box::new(start), Box::new(end)),
            logical_operator: None,
        });
        self
    }

    /// Добавление условия IN
    pub fn where_in(mut self, column: &str, values: Vec<FilterValue>) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator: FilterOperator::In,
            value: FilterValue::List(values),
            logical_operator: None,
        });
        self
    }

    /// Добавление агрегации
    pub fn aggregate(mut self, aggregation: Aggregation) -> Self {
        self.aggregations.push(aggregation);
        self
    }

    /// Добавление COUNT агрегации
    pub fn count(mut self, column: &str, alias: Option<&str>) -> Self {
        self.aggregations.push(Aggregation {
            function: AggregationFunction::Count,
            column: column.to_string(),
            alias: alias.map(|a| a.to_string()),
        });
        self
    }

    /// Добавление SUM агрегации
    pub fn sum(mut self, column: &str, alias: Option<&str>) -> Self {
        self.aggregations.push(Aggregation {
            function: AggregationFunction::Sum,
            column: column.to_string(),
            alias: alias.map(|a| a.to_string()),
        });
        self
    }

    /// Добавление AVG агрегации
    pub fn avg(mut self, column: &str, alias: Option<&str>) -> Self {
        self.aggregations.push(Aggregation {
            function: AggregationFunction::Avg,
            column: column.to_string(),
            alias: alias.map(|a| a.to_string()),
        });
        self
    }

    /// Добавление MIN агрегации
    pub fn min(mut self, column: &str, alias: Option<&str>) -> Self {
        self.aggregations.push(Aggregation {
            function: AggregationFunction::Min,
            column: column.to_string(),
            alias: alias.map(|a| a.to_string()),
        });
        self
    }

    /// Добавление MAX агрегации
    pub fn max(mut self, column: &str, alias: Option<&str>) -> Self {
        self.aggregations.push(Aggregation {
            function: AggregationFunction::Max,
            column: column.to_string(),
            alias: alias.map(|a| a.to_string()),
        });
        self
    }

    /// Добавление GROUP BY
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Добавление ORDER BY
    pub fn order_by(mut self, column: &str, direction: SortDirection) -> Self {
        self.order_by.push(OrderBy {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// Добавление LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Добавление OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Построение SQL запроса
    pub fn build(&self) -> Result<String> {
        let mut sql = String::new();

        // SELECT clause
        sql.push_str("SELECT ");
        if !self.aggregations.is_empty() {
            let agg_parts: Vec<String> = self
                .aggregations
                .iter()
                .map(|agg| {
                    let func_name = match agg.function {
                        AggregationFunction::Count => "COUNT",
                        AggregationFunction::Sum => "SUM",
                        AggregationFunction::Avg => "AVG",
                        AggregationFunction::Min => "MIN",
                        AggregationFunction::Max => "MAX",
                        AggregationFunction::StdDev => "STDDEV",
                        AggregationFunction::Variance => "VARIANCE",
                        AggregationFunction::First => "FIRST",
                        AggregationFunction::Last => "LAST",
                        AggregationFunction::Distinct => "DISTINCT",
                    };

                    let alias = if let Some(alias) = &agg.alias {
                        format!("{} AS {}", func_name, alias)
                    } else {
                        func_name.to_string()
                    };

                    format!("{}({})", alias, agg.column)
                })
                .collect();
            sql.push_str(&agg_parts.join(", "));
        } else if self.columns.is_empty() {
            sql.push_str("*");
        } else {
            sql.push_str(&self.columns.join(", "));
        }

        // FROM clause
        sql.push_str(&format!(" FROM {}", self.table_name));

        // WHERE clause
        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            let where_clause = self.build_where_clause()?;
            sql.push_str(&where_clause);
        }

        // GROUP BY clause
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // ORDER BY clause
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self
                .order_by
                .iter()
                .map(|order| {
                    let direction = match order.direction {
                        SortDirection::Asc => "ASC",
                        SortDirection::Desc => "DESC",
                    };
                    format!("{} {}", order.column, direction)
                })
                .collect();
            sql.push_str(&order_parts.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(sql)
    }

    /// Построение WHERE клаузулы
    fn build_where_clause(&self) -> Result<String> {
        let mut conditions = Vec::new();

        for (i, filter) in self.filters.iter().enumerate() {
            let condition = self.build_filter_condition(filter)?;

            if i > 0 {
                let logical_op = match filter.logical_operator {
                    Some(LogicalOperator::And) => " AND ",
                    Some(LogicalOperator::Or) => " OR ",
                    None => " AND ", // По умолчанию AND
                };
                conditions.push(logical_op.to_string());
            }

            conditions.push(condition);
        }

        Ok(conditions.join(""))
    }

    /// Построение условия фильтрации
    fn build_filter_condition(&self, filter: &FilterCondition) -> Result<String> {
        let mut condition = String::new();
        condition.push_str(&filter.column);

        match &filter.operator {
            FilterOperator::Equal => {
                condition.push_str(" = ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::NotEqual => {
                condition.push_str(" != ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::GreaterThan => {
                condition.push_str(" > ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::GreaterThanOrEqual => {
                condition.push_str(" >= ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::LessThan => {
                condition.push_str(" < ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::LessThanOrEqual => {
                condition.push_str(" <= ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::Like => {
                condition.push_str(" LIKE ");
                condition.push_str(&self.format_filter_value(&filter.value)?);
            }
            FilterOperator::In => {
                condition.push_str(" IN (");
                if let FilterValue::List(values) = &filter.value {
                    let formatted_values: Vec<String> = values
                        .iter()
                        .map(|v| self.format_filter_value(v))
                        .collect::<Result<Vec<_>>>()?;
                    condition.push_str(&formatted_values.join(", "));
                }
                condition.push_str(")");
            }
            FilterOperator::Between => {
                condition.push_str(" BETWEEN ");
                if let FilterValue::Range(start, end) = &filter.value {
                    condition.push_str(&self.format_filter_value(start)?);
                    condition.push_str(" AND ");
                    condition.push_str(&self.format_filter_value(end)?);
                }
            }
            FilterOperator::IsNull => {
                condition.push_str(" IS NULL");
            }
            FilterOperator::IsNotNull => {
                condition.push_str(" IS NOT NULL");
            }
        }

        Ok(condition)
    }

    /// Форматирование значения фильтра
    fn format_filter_value(&self, value: &FilterValue) -> Result<String> {
        match value {
            FilterValue::String(s) => Ok(format!("'{}'", s.replace("'", "''"))),
            FilterValue::Number(n) => Ok(n.to_string()),
            FilterValue::Integer(i) => Ok(i.to_string()),
            FilterValue::Boolean(b) => Ok(b.to_string()),
            FilterValue::List(_) => Err(DataAccessError::Query(
                "List values should be handled in IN operator".to_string(),
            )),
            FilterValue::Range(_, _) => Err(DataAccessError::Query(
                "Range values should be handled in BETWEEN operator".to_string(),
            )),
            FilterValue::Null => Ok("NULL".to_string()),
        }
    }

    /// Получение параметров запроса
    pub fn get_parameters(&self) -> Vec<FilterValue> {
        self.filters.iter().map(|f| f.value.clone()).collect()
    }

    /// Клонирование builder'а
    pub fn clone_builder(&self) -> Self {
        self.clone()
    }
}

/// Специализированный Query Builder для свечей
pub struct CandleArrowQueryBuilder {
    base_builder: ArrowQueryBuilder,
}

impl CandleArrowQueryBuilder {
    /// Создание нового builder'а для свечей
    pub fn new() -> Self {
        Self {
            base_builder: ArrowQueryBuilder::new("candles"),
        }
    }

    /// Получение свечей по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.base_builder = self
            .base_builder
            .where_equal("symbol", FilterValue::String(symbol.to_string()));
        self
    }

    /// Получение свечей по временному диапазону
    pub fn by_time_range(mut self, start_time: &str, end_time: &str) -> Self {
        self.base_builder = self.base_builder.where_between(
            "timestamp",
            FilterValue::String(start_time.to_string()),
            FilterValue::String(end_time.to_string()),
        );
        self
    }

    /// Получение свечей за последние N дней
    pub fn last_days(mut self, days: u32) -> Self {
        // Это упрощенная версия - в реальности нужна более сложная логика с датами
        self.base_builder = self.base_builder.where_greater_than(
            "timestamp",
            FilterValue::String(format!("NOW() - INTERVAL {} DAY", days)),
        );
        self
    }

    /// Получение OHLCV данных
    pub fn ohlcv(mut self) -> Self {
        self.base_builder = self.base_builder.select(&[
            "timestamp",
            "symbol",
            "open",
            "high",
            "low",
            "close",
            "volume",
        ]);
        self
    }

    /// Получение только цен закрытия
    pub fn close_prices(mut self) -> Self {
        self.base_builder = self.base_builder.select(&["timestamp", "close"]);
        self
    }

    /// Получение объемов
    pub fn volumes(mut self) -> Self {
        self.base_builder = self.base_builder.select(&["timestamp", "volume"]);
        self
    }

    /// Сортировка по времени
    pub fn order_by_time(mut self, direction: SortDirection) -> Self {
        self.base_builder = self.base_builder.order_by("timestamp", direction);
        self
    }

    /// Построение запроса
    pub fn build(self) -> Result<String> {
        self.base_builder.build()
    }
}

/// Специализированный Query Builder для сделок
pub struct TradeArrowQueryBuilder {
    base_builder: ArrowQueryBuilder,
}

impl TradeArrowQueryBuilder {
    /// Создание нового builder'а для сделок
    pub fn new() -> Self {
        Self {
            base_builder: ArrowQueryBuilder::new("trades"),
        }
    }

    /// Получение сделок по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.base_builder = self
            .base_builder
            .where_equal("symbol", FilterValue::String(symbol.to_string()));
        self
    }

    /// Получение сделок по стороне
    pub fn by_side(mut self, side: &str) -> Self {
        self.base_builder = self
            .base_builder
            .where_equal("side", FilterValue::String(side.to_string()));
        self
    }

    /// Получение сделок по ценовому диапазону
    pub fn by_price_range(mut self, min_price: f32, max_price: f32) -> Self {
        self.base_builder = self.base_builder.where_between(
            "price",
            FilterValue::Number(min_price),
            FilterValue::Number(max_price),
        );
        self
    }

    /// Получение последних сделок
    pub fn recent(mut self, limit: usize) -> Self {
        self.base_builder = self
            .base_builder
            .order_by("timestamp", SortDirection::Desc)
            .limit(limit);
        self
    }

    /// Получение агрегированных данных по сделкам
    pub fn aggregated(mut self) -> Self {
        self.base_builder = self
            .base_builder
            .sum("quantity", Some("total_quantity"))
            .avg("price", Some("avg_price"))
            .count("id", Some("trade_count"));
        self
    }

    /// Построение запроса
    pub fn build(self) -> Result<String> {
        self.base_builder.build()
    }
}

/// Специализированный Query Builder для результатов бэктестов
pub struct BacktestArrowQueryBuilder {
    base_builder: ArrowQueryBuilder,
}

impl BacktestArrowQueryBuilder {
    /// Создание нового builder'а для результатов бэктестов
    pub fn new() -> Self {
        Self {
            base_builder: ArrowQueryBuilder::new("backtest_results"),
        }
    }

    /// Получение результатов по стратегии
    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.base_builder = self
            .base_builder
            .where_equal("strategy_id", FilterValue::String(strategy_id.to_string()));
        self
    }

    /// Получение результатов по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.base_builder = self
            .base_builder
            .where_equal("symbol", FilterValue::String(symbol.to_string()));
        self
    }

    /// Получение результатов с минимальной доходностью
    pub fn min_return(mut self, min_return: f32) -> Self {
        self.base_builder = self
            .base_builder
            .where_greater_than("total_return", FilterValue::Number(min_return));
        self
    }

    /// Получение результатов с минимальным Sharpe ratio
    pub fn min_sharpe(mut self, min_sharpe: f32) -> Self {
        self.base_builder = self
            .base_builder
            .where_greater_than("sharpe_ratio", FilterValue::Number(min_sharpe));
        self
    }

    /// Получение топ результатов
    pub fn top_results(mut self, limit: usize) -> Self {
        self.base_builder = self
            .base_builder
            .order_by("sharpe_ratio", SortDirection::Desc)
            .limit(limit);
        self
    }

    /// Получение агрегированных метрик
    pub fn metrics_summary(mut self) -> Self {
        self.base_builder = self
            .base_builder
            .avg("total_return", Some("avg_return"))
            .avg("sharpe_ratio", Some("avg_sharpe"))
            .avg("max_drawdown", Some("avg_drawdown"))
            .avg("win_rate", Some("avg_win_rate"))
            .count("strategy_id", Some("test_count"));
        self
    }

    /// Построение запроса
    pub fn build(self) -> Result<String> {
        self.base_builder.build()
    }
}

/// Утилиты для Arrow Query Builder
pub struct ArrowQueryUtils;

impl ArrowQueryUtils {
    /// Создание запроса для анализа свечей
    pub fn create_candles_analysis_query(
        symbol: &str,
        start_time: &str,
        end_time: &str,
        metrics: &[&str],
    ) -> Result<String> {
        let mut builder = CandleArrowQueryBuilder::new()
            .by_symbol(symbol)
            .by_time_range(start_time, end_time);

        if !metrics.is_empty() {
            builder.base_builder = builder.base_builder.select(metrics);
        }

        builder.order_by_time(SortDirection::Asc).build()
    }

    /// Создание запроса для агрегации данных
    pub fn create_aggregation_query(
        table_name: &str,
        group_by: &[&str],
        aggregations: &[(&str, AggregationFunction)],
    ) -> Result<String> {
        let mut builder = ArrowQueryBuilder::new(table_name);

        for (column, function) in aggregations {
            builder = builder.aggregate(Aggregation {
                function: function.clone(),
                column: column.to_string(),
                alias: None,
            });
        }

        builder.group_by(group_by).build()
    }

    /// Создание запроса для поиска паттернов
    pub fn create_pattern_query(
        table_name: &str,
        pattern_condition: &str,
        limit: usize,
    ) -> Result<String> {
        ArrowQueryBuilder::new(table_name)
            .select_all()
            .limit(limit)
            .build()
    }

    /// Создание запроса для корреляционного анализа
    pub fn create_correlation_query(
        table_name: &str,
        symbols: &[&str],
        field: &str,
    ) -> Result<String> {
        let symbol_filter = symbols
            .iter()
            .map(|s| format!("symbol = '{}'", s))
            .collect::<Vec<_>>()
            .join(" OR ");

        ArrowQueryBuilder::new(table_name).select_all().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_query_builder_basic() {
        let query = ArrowQueryBuilder::new("candles")
            .select(&["timestamp", "close"])
            .where_equal("symbol", FilterValue::String("BTCUSDT".to_string()))
            .order_by("timestamp", SortDirection::Asc)
            .limit(100)
            .build()
            .unwrap();

        assert!(query.contains("SELECT timestamp, close"));
        assert!(query.contains("FROM candles"));
        assert!(query.contains("WHERE symbol = 'BTCUSDT'"));
        assert!(query.contains("ORDER BY timestamp ASC"));
        assert!(query.contains("LIMIT 100"));
    }

    #[test]
    fn test_arrow_query_builder_aggregations() {
        let query = ArrowQueryBuilder::new("candles")
            .avg("close", Some("avg_close"))
            .max("high", Some("max_high"))
            .min("low", Some("min_low"))
            .count("timestamp", Some("count"))
            .group_by(&["symbol"])
            .build()
            .unwrap();

        assert!(query.contains("AVG(close) AS avg_close"));
        assert!(query.contains("MAX(high) AS max_high"));
        assert!(query.contains("MIN(low) AS min_low"));
        assert!(query.contains("COUNT(timestamp) AS count"));
        assert!(query.contains("GROUP BY symbol"));
    }

    #[test]
    fn test_candle_arrow_query_builder() {
        let query = CandleArrowQueryBuilder::new()
            .by_symbol("BTCUSDT")
            .by_time_range("2024-01-01", "2024-01-31")
            .ohlcv()
            .order_by_time(SortDirection::Asc)
            .build()
            .unwrap();

        assert!(query.contains("symbol = 'BTCUSDT'"));
        assert!(query.contains("timestamp BETWEEN '2024-01-01' AND '2024-01-31'"));
        assert!(query.contains("SELECT timestamp, symbol, open, high, low, close, volume"));
    }

    #[test]
    fn test_trade_arrow_query_builder() {
        let query = TradeArrowQueryBuilder::new()
            .by_symbol("BTCUSDT")
            .by_side("Buy")
            .recent(50)
            .build()
            .unwrap();

        assert!(query.contains("symbol = 'BTCUSDT'"));
        assert!(query.contains("side = 'Buy'"));
        assert!(query.contains("ORDER BY timestamp DESC"));
        assert!(query.contains("LIMIT 50"));
    }

    #[test]
    fn test_backtest_arrow_query_builder() {
        let query = BacktestArrowQueryBuilder::new()
            .by_strategy("strategy_1")
            .min_sharpe(1.0)
            .top_results(10)
            .build()
            .unwrap();

        assert!(query.contains("strategy_id = 'strategy_1'"));
        assert!(query.contains("sharpe_ratio > 1"));
        assert!(query.contains("ORDER BY sharpe_ratio DESC"));
        assert!(query.contains("LIMIT 10"));
    }
}
