//! Query Builder для ClickHouse

use crate::data_access::models::*;
use crate::data_access::{DataAccessError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Базовый Query Builder для ClickHouse
#[derive(Clone)]
pub struct ClickHouseQueryBuilder {
    query_type: QueryType,
    table: Option<String>,
    columns: Vec<String>,
    conditions: Vec<Condition>,
    order_by: Vec<OrderClause>,
    group_by: Vec<String>,
    having: Vec<Condition>,
    limit_value: Option<u32>,
    offset_value: Option<u32>,
    joins: Vec<JoinClause>,
    values: Vec<Vec<String>>,
}

/// Тип запроса
#[derive(Debug, Clone)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    DropTable,
    AlterTable,
}

/// Условие WHERE
#[derive(Debug, Clone)]
pub struct Condition {
    pub column: String,
    pub operator: ComparisonOperator,
    pub value: String,
    pub logical_operator: Option<LogicalOperator>,
}

/// Операторы сравнения
#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    In,
    NotIn,
    Between,
    IsNull,
    IsNotNull,
}

/// Логические операторы
#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

/// Упорядочивание
#[derive(Debug, Clone)]
pub struct OrderClause {
    pub column: String,
    pub direction: OrderDirection,
}

/// Направление сортировки
#[derive(Debug, Clone)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// JOIN клаузула
#[derive(Debug, Clone)]
pub struct JoinClause {
    pub table: String,
    pub join_type: JoinType,
    pub condition: String,
}

/// Тип JOIN
#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl ClickHouseQueryBuilder {
    /// Создание нового Query Builder
    pub fn new() -> Self {
        Self {
            query_type: QueryType::Select,
            table: None,
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            limit_value: None,
            offset_value: None,
            joins: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Установка типа запроса
    pub fn query_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }

    /// Установка таблицы
    pub fn table(mut self, table: &str) -> Self {
        self.table = Some(table.to_string());
        self
    }

    /// Добавление колонок для SELECT
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query_type = QueryType::Select;
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// SELECT *
    pub fn select_all(mut self) -> Self {
        self.query_type = QueryType::Select;
        self.columns = vec!["*".to_string()];
        self
    }

    /// Добавление условия WHERE
    pub fn where_eq(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::Equal, value, None);
        self
    }

    /// WHERE column != value
    pub fn where_ne(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::NotEqual, value, None);
        self
    }

    /// WHERE column > value
    pub fn where_gt(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::GreaterThan, value, None);
        self
    }

    /// WHERE column >= value
    pub fn where_gte(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::GreaterThanOrEqual, value, None);
        self
    }

    /// WHERE column < value
    pub fn where_lt(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::LessThan, value, None);
        self
    }

    /// WHERE column <= value
    pub fn where_lte(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::LessThanOrEqual, value, None);
        self
    }

    /// WHERE column LIKE pattern
    pub fn where_like(mut self, column: &str, pattern: &str) -> Self {
        self.add_condition(column, ComparisonOperator::Like, pattern, None);
        self
    }

    /// WHERE column IN (values...)
    pub fn where_in(mut self, column: &str, values: &[&str]) -> Self {
        let values_str = format!("({})", values.join(", "));
        self.add_condition(column, ComparisonOperator::In, &values_str, None);
        self
    }

    /// WHERE column BETWEEN start AND end
    pub fn where_between(mut self, column: &str, start: &str, end: &str) -> Self {
        let value = format!("{} AND {}", start, end);
        self.add_condition(column, ComparisonOperator::Between, &value, None);
        self
    }

    /// WHERE column IS NULL
    pub fn where_null(mut self, column: &str) -> Self {
        self.add_condition(column, ComparisonOperator::IsNull, "", None);
        self
    }

    /// WHERE column IS NOT NULL
    pub fn where_not_null(mut self, column: &str) -> Self {
        self.add_condition(column, ComparisonOperator::IsNotNull, "", None);
        self
    }

    /// AND условие
    pub fn and_eq(mut self, column: &str, value: &str) -> Self {
        self.add_condition(
            column,
            ComparisonOperator::Equal,
            value,
            Some(LogicalOperator::And),
        );
        self
    }

    /// OR условие
    pub fn or_eq(mut self, column: &str, value: &str) -> Self {
        self.add_condition(
            column,
            ComparisonOperator::Equal,
            value,
            Some(LogicalOperator::Or),
        );
        self
    }

    /// ORDER BY
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push(OrderClause {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// ORDER BY ASC
    pub fn order_by_asc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Asc)
    }

    /// ORDER BY DESC
    pub fn order_by_desc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Desc)
    }

    /// GROUP BY
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// LIMIT
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit_value = Some(limit);
        self
    }

    /// OFFSET
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset_value = Some(offset);
        self
    }

    /// INNER JOIN
    pub fn inner_join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(JoinClause {
            table: table.to_string(),
            join_type: JoinType::Inner,
            condition: condition.to_string(),
        });
        self
    }

    /// LEFT JOIN
    pub fn left_join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(JoinClause {
            table: table.to_string(),
            join_type: JoinType::Left,
            condition: condition.to_string(),
        });
        self
    }

    /// Добавление условия
    fn add_condition(
        &mut self,
        column: &str,
        operator: ComparisonOperator,
        value: &str,
        logical_op: Option<LogicalOperator>,
    ) {
        self.conditions.push(Condition {
            column: column.to_string(),
            operator,
            value: value.to_string(),
            logical_operator: logical_op,
        });
    }

    /// Построение SQL запроса
    pub fn build(&self) -> Result<String> {
        match self.query_type {
            QueryType::Select => self.build_select(),
            QueryType::Insert => self.build_insert(),
            QueryType::Update => self.build_update(),
            QueryType::Delete => self.build_delete(),
            QueryType::CreateTable => self.build_create_table(),
            QueryType::DropTable => self.build_drop_table(),
            QueryType::AlterTable => self.build_alter_table(),
        }
    }

    /// Построение SELECT запроса
    fn build_select(&self) -> Result<String> {
        let mut query = String::new();

        // SELECT
        query.push_str("SELECT ");
        if self.columns.is_empty() {
            query.push_str("*");
        } else {
            query.push_str(&self.columns.join(", "));
        }

        // FROM
        if let Some(table) = &self.table {
            query.push_str(&format!(" FROM {}", table));
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        // JOIN
        for join in &self.joins {
            let join_type = match join.join_type {
                JoinType::Inner => "INNER JOIN",
                JoinType::Left => "LEFT JOIN",
                JoinType::Right => "RIGHT JOIN",
                JoinType::Full => "FULL JOIN",
            };
            query.push_str(&format!(
                " {} {} ON {}",
                join_type, join.table, join.condition
            ));
        }

        // WHERE
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            for (i, condition) in self.conditions.iter().enumerate() {
                if i > 0 {
                    match condition.logical_operator {
                        Some(LogicalOperator::And) => query.push_str(" AND "),
                        Some(LogicalOperator::Or) => query.push_str(" OR "),
                        None => query.push_str(" AND "),
                    }
                }

                query.push_str(&self.build_condition(condition));
            }
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            query.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING
        if !self.having.is_empty() {
            query.push_str(" HAVING ");
            for (i, condition) in self.having.iter().enumerate() {
                if i > 0 {
                    query.push_str(" AND ");
                }
                query.push_str(&self.build_condition(condition));
            }
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            let order_clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|clause| {
                    let direction = match clause.direction {
                        OrderDirection::Asc => "ASC",
                        OrderDirection::Desc => "DESC",
                    };
                    format!("{} {}", clause.column, direction)
                })
                .collect();
            query.push_str(&order_clauses.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.limit_value {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset_value {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(query)
    }

    /// Построение INSERT запроса
    fn build_insert(&self) -> Result<String> {
        let mut query = String::new();

        query.push_str("INSERT INTO ");
        if let Some(table) = &self.table {
            query.push_str(table);
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        if !self.columns.is_empty() {
            query.push_str(&format!(" ({})", self.columns.join(", ")));
        }

        if !self.values.is_empty() {
            query.push_str(" VALUES ");
            let value_strings: Vec<String> = self
                .values
                .iter()
                .map(|row| format!("({})", row.join(", ")))
                .collect();
            query.push_str(&value_strings.join(", "));
        }

        Ok(query)
    }

    /// Построение UPDATE запроса
    fn build_update(&self) -> Result<String> {
        let mut query = String::new();

        query.push_str("ALTER TABLE ");
        if let Some(table) = &self.table {
            query.push_str(table);
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        // ClickHouse использует ALTER TABLE для обновлений
        if !self.columns.is_empty() && !self.values.is_empty() {
            query.push_str(" UPDATE ");
            let updates: Vec<String> = self
                .columns
                .iter()
                .zip(self.values[0].iter())
                .map(|(col, val)| format!("{} = {}", col, val))
                .collect();
            query.push_str(&updates.join(", "));
        }

        // WHERE
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            for (i, condition) in self.conditions.iter().enumerate() {
                if i > 0 {
                    query.push_str(" AND ");
                }
                query.push_str(&self.build_condition(condition));
            }
        }

        Ok(query)
    }

    /// Построение DELETE запроса
    fn build_delete(&self) -> Result<String> {
        let mut query = String::new();

        query.push_str("ALTER TABLE ");
        if let Some(table) = &self.table {
            query.push_str(table);
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        query.push_str(" DELETE");

        // WHERE
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            for (i, condition) in self.conditions.iter().enumerate() {
                if i > 0 {
                    query.push_str(" AND ");
                }
                query.push_str(&self.build_condition(condition));
            }
        }

        Ok(query)
    }

    /// Построение CREATE TABLE запроса
    fn build_create_table(&self) -> Result<String> {
        let mut query = String::new();

        query.push_str("CREATE TABLE ");
        if let Some(table) = &self.table {
            query.push_str(table);
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        if !self.columns.is_empty() {
            query.push_str(" (");
            query.push_str(&self.columns.join(", "));
            query.push_str(")");
        }

        Ok(query)
    }

    /// Построение DROP TABLE запроса
    fn build_drop_table(&self) -> Result<String> {
        let mut query = String::new();

        query.push_str("DROP TABLE ");
        if let Some(table) = &self.table {
            query.push_str(table);
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        Ok(query)
    }

    /// Построение ALTER TABLE запроса
    fn build_alter_table(&self) -> Result<String> {
        let mut query = String::new();

        query.push_str("ALTER TABLE ");
        if let Some(table) = &self.table {
            query.push_str(table);
        } else {
            return Err(DataAccessError::Query("Table not specified".to_string()));
        }

        if !self.columns.is_empty() {
            query.push_str(&format!(" {}", self.columns.join(", ")));
        }

        Ok(query)
    }

    /// Построение условия
    fn build_condition(&self, condition: &Condition) -> String {
        let operator_str = match condition.operator {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::Like => "LIKE",
            ComparisonOperator::In => "IN",
            ComparisonOperator::NotIn => "NOT IN",
            ComparisonOperator::Between => "BETWEEN",
            ComparisonOperator::IsNull => "IS NULL",
            ComparisonOperator::IsNotNull => "IS NOT NULL",
        };

        match condition.operator {
            ComparisonOperator::IsNull | ComparisonOperator::IsNotNull => {
                format!("{} {}", condition.column, operator_str)
            }
            _ => {
                format!("{} {} {}", condition.column, operator_str, condition.value)
            }
        }
    }
}

impl Default for ClickHouseQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для свечей
pub struct CandleQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl CandleQueryBuilder {
    /// Создание нового Candle Query Builder
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("candles"),
        }
    }

    /// Получение свечей по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    /// Фильтр по времени
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self
            .builder
            .where_gte(
                "timestamp",
                &format!("'{}'", start.format("%Y-%m-%d %H:%M:%S")),
            )
            .and_eq(
                "timestamp",
                &format!("'{}'", end.format("%Y-%m-%d %H:%M:%S")),
            );
        self
    }

    /// Сортировка по времени (новые сначала)
    pub fn order_by_time_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("timestamp");
        self
    }

    /// Сортировка по времени (старые сначала)
    pub fn order_by_time_asc(mut self) -> Self {
        self.builder = self.builder.order_by_asc("timestamp");
        self
    }

    /// Ограничение количества записей
    pub fn limit(mut self, limit: u32) -> Self {
        self.builder = self.builder.limit(limit);
        self
    }

    /// Получение последних N свечей
    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("timestamp").limit(count);
        self
    }

    /// Получение свечей за последние N дней
    pub fn last_days(self, days: u32) -> Self {
        let start_time = Utc::now() - chrono::Duration::days(days as i64);
        self.time_range(start_time, Utc::now())
    }

    /// Построение запроса
    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "timestamp",
                "symbol",
                "open",
                "high",
                "low",
                "close",
                "volume",
            ])
            .build()
    }
}

impl Default for CandleQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для сделок
pub struct TradeQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl TradeQueryBuilder {
    /// Создание нового Trade Query Builder
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trades"),
        }
    }

    /// Фильтр по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    /// Фильтр по стороне сделки
    pub fn by_side(mut self, side: TradeSide) -> Self {
        let side_str = match side {
            TradeSide::Buy => "Buy",
            TradeSide::Sell => "Sell",
        };
        self.builder = self.builder.where_eq("side", &format!("'{}'", side_str));
        self
    }

    /// Фильтр по времени
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self
            .builder
            .where_gte(
                "timestamp",
                &format!("'{}'", start.format("%Y-%m-%d %H:%M:%S")),
            )
            .and_eq(
                "timestamp",
                &format!("'{}'", end.format("%Y-%m-%d %H:%M:%S")),
            );
        self
    }

    /// Фильтр по цене
    pub fn price_range(mut self, min_price: f64, max_price: f64) -> Self {
        self.builder = self
            .builder
            .where_gte("price", &min_price.to_string())
            .and_eq("price", &max_price.to_string());
        self
    }

    /// Сортировка по времени
    pub fn order_by_time_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("timestamp");
        self
    }

    /// Ограничение количества записей
    pub fn limit(mut self, limit: u32) -> Self {
        self.builder = self.builder.limit(limit);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "id",
                "timestamp",
                "symbol",
                "price",
                "quantity",
                "side",
                "order_id",
            ])
            .build()
    }
}

impl Default for TradeQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для результатов бэктестов
pub struct BacktestQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl BacktestQueryBuilder {
    /// Создание нового Backtest Query Builder
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("backtest_results"),
        }
    }

    /// Фильтр по стратегии
    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    /// Фильтр по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    /// Фильтр по доходности
    pub fn min_return(mut self, min_return: f64) -> Self {
        self.builder = self
            .builder
            .where_gte("total_return", &min_return.to_string());
        self
    }

    /// Фильтр по Sharpe ratio
    pub fn min_sharpe(mut self, min_sharpe: f64) -> Self {
        self.builder = self
            .builder
            .where_gte("sharpe_ratio", &min_sharpe.to_string());
        self
    }

    /// Фильтр по максимальной просадке
    pub fn max_drawdown(mut self, max_drawdown: f64) -> Self {
        self.builder = self
            .builder
            .where_lte("max_drawdown", &max_drawdown.to_string());
        self
    }

    /// Сортировка по доходности
    pub fn order_by_return_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("total_return");
        self
    }

    /// Сортировка по Sharpe ratio
    pub fn order_by_sharpe_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("sharpe_ratio");
        self
    }

    /// Ограничение количества записей
    pub fn limit(mut self, limit: u32) -> Self {
        self.builder = self.builder.limit(limit);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<String> {
        self.builder.clone().select_all().build()
    }
}

impl Default for BacktestQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Утилиты для работы с ClickHouse запросами
pub struct ClickHouseUtils;

impl ClickHouseUtils {
    /// Создание запроса для получения статистики по символу
    pub fn symbol_stats_query(symbol: &str) -> String {
        format!(
            "SELECT 
                COUNT(*) as total_candles,
                MIN(timestamp) as first_candle,
                MAX(timestamp) as last_candle,
                AVG(volume) as avg_volume,
                MAX(high) as max_price,
                MIN(low) as min_price,
                AVG(close) as avg_price,
                STDDEV(close) as price_volatility
            FROM candles 
            WHERE symbol = '{}'
            GROUP BY symbol",
            symbol
        )
    }

    /// Создание запроса для получения топ стратегий
    pub fn top_strategies_query(limit: u32) -> String {
        format!(
            "SELECT 
                strategy_id,
                symbol,
                AVG(total_return) as avg_return,
                AVG(sharpe_ratio) as avg_sharpe,
                AVG(max_drawdown) as avg_drawdown,
                COUNT(*) as backtest_count
            FROM backtest_results 
            GROUP BY strategy_id, symbol
            HAVING backtest_count >= 3
            ORDER BY avg_sharpe DESC
            LIMIT {}",
            limit
        )
    }

    /// Создание запроса для анализа волатильности
    pub fn volatility_analysis_query(symbol: &str, days: u32) -> String {
        format!(
            "SELECT 
                toDate(timestamp) as date,
                AVG(close) as avg_price,
                STDDEV(close) as daily_volatility,
                MAX(high) - MIN(low) as daily_range,
                SUM(volume) as total_volume
            FROM candles 
            WHERE symbol = '{}' 
                AND timestamp >= now() - INTERVAL {} DAY
            GROUP BY toDate(timestamp)
            ORDER BY date DESC",
            symbol, days
        )
    }

    /// Создание запроса для корреляционного анализа
    pub fn correlation_query(symbol1: &str, symbol2: &str, days: u32) -> String {
        format!(
            "SELECT 
                corr(c1.close, c2.close) as correlation,
                COUNT(*) as data_points
            FROM candles c1
            INNER JOIN candles c2 ON c1.timestamp = c2.timestamp
            WHERE c1.symbol = '{}' 
                AND c2.symbol = '{}'
                AND c1.timestamp >= now() - INTERVAL {} DAY",
            symbol1, symbol2, days
        )
    }
}
