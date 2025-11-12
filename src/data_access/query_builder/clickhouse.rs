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

/// Специализированный Query Builder для OHLCV данных
pub struct ClickHouseCandleQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl ClickHouseCandleQueryBuilder {
    /// Создание нового OHLCV Query Builder
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.ohlcv_data"),
        }
    }

    /// Фильтр по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    /// Фильтр по таймфрейму
    pub fn by_timeframe(mut self, timeframe: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("timeframe", &format!("'{}'", timeframe));
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
            .where_lte(
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
                "timeframe",
                "open",
                "high",
                "low",
                "close",
                "volume",
            ])
            .build()
    }
}

impl Default for ClickHouseCandleQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для торговых сделок (обновленный)
pub struct ClickHouseTradeQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl ClickHouseTradeQueryBuilder {
    /// Создание нового Trade Query Builder
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.trades"),
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

    /// Фильтр по стороне сделки
    pub fn by_side(mut self, side: TradeSide) -> Self {
        let side_str = match side {
            TradeSide::Buy => "buy",
            TradeSide::Sell => "sell",
        };
        self.builder = self.builder.where_eq("side", &format!("'{}'", side_str));
        self
    }

    /// Фильтр по статусу
    pub fn by_status(mut self, status: &str) -> Self {
        self.builder = self.builder.where_eq("status", &format!("'{}'", status));
        self
    }

    /// Фильтр по времени входа
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self
            .builder
            .where_gte(
                "entry_time",
                &format!("'{}'", start.format("%Y-%m-%d %H:%M:%S")),
            )
            .where_lte(
                "entry_time",
                &format!("'{}'", end.format("%Y-%m-%d %H:%M:%S")),
            );
        self
    }

    /// Фильтр по диапазону цен
    pub fn price_range(mut self, min_price: f32, max_price: f32) -> Self {
        self.builder = self
            .builder
            .where_gte("entry_price", &min_price.to_string())
            .where_lte("entry_price", &max_price.to_string());
        self
    }

    /// Фильтр по прибыльности
    pub fn profitable_only(mut self) -> Self {
        self.builder = self.builder.where_gt("pnl", "0");
        self
    }

    /// Сортировка по времени (новые сначала)
    pub fn order_by_time_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("entry_time");
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
                "trade_id",
                "strategy_id",
                "symbol",
                "side",
                "quantity",
                "entry_price",
                "exit_price",
                "entry_time",
                "exit_time",
                "pnl",
                "commission",
                "status",
                "metadata",
            ])
            .build()
    }
}

impl Default for ClickHouseTradeQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для результатов бэктестов (обновленный)
pub struct ClickHouseBacktestQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl ClickHouseBacktestQueryBuilder {
    /// Создание нового Backtest Query Builder
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.backtest_results"),
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

    /// Фильтр по таймфрейму
    pub fn by_timeframe(mut self, timeframe: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("timeframe", &format!("'{}'", timeframe));
        self
    }

    /// Минимальная доходность
    pub fn min_return(mut self, min_pnl: f32) -> Self {
        self.builder = self.builder.where_gte("total_pnl", &min_pnl.to_string());
        self
    }

    /// Минимальный Sharpe Ratio
    pub fn min_sharpe(mut self, min_sharpe: f32) -> Self {
        self.builder = self
            .builder
            .where_gte("sharpe_ratio", &min_sharpe.to_string());
        self
    }

    /// Максимальная просадка
    pub fn max_drawdown(mut self, max_dd: f32) -> Self {
        self.builder = self.builder.where_gte("max_drawdown", &max_dd.to_string());
        self
    }

    /// Минимальный win rate
    pub fn min_win_rate(mut self, min_wr: f32) -> Self {
        self.builder = self.builder.where_gte("win_rate", &min_wr.to_string());
        self
    }

    /// Сортировка по Sharpe Ratio (лучшие сначала)
    pub fn order_by_sharpe_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("sharpe_ratio");
        self
    }

    /// Сортировка по доходности (лучшие сначала)
    pub fn order_by_pnl_desc(mut self) -> Self {
        self.builder = self.builder.order_by_desc("total_pnl");
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
                "backtest_id",
                "strategy_id",
                "symbol",
                "timeframe",
                "start_date",
                "end_date",
                "total_trades",
                "winning_trades",
                "losing_trades",
                "total_pnl",
                "max_drawdown",
                "sharpe_ratio",
                "profit_factor",
                "win_rate",
                "avg_win",
                "avg_loss",
                "execution_time_ms",
            ])
            .build()
    }
}

impl Default for ClickHouseBacktestQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для сигналов
pub struct SignalQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl SignalQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.signals"),
        }
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    pub fn by_type(mut self, signal_type: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("signal_type", &format!("'{}'", signal_type));
        self
    }

    pub fn min_strength(mut self, min: f32) -> Self {
        self.builder = self.builder.where_gte("signal_strength", &min.to_string());
        self
    }

    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self
            .builder
            .where_gte(
                "timestamp",
                &format!("'{}'", start.format("%Y-%m-%d %H:%M:%S")),
            )
            .where_lte(
                "timestamp",
                &format!("'{}'", end.format("%Y-%m-%d %H:%M:%S")),
            );
        self
    }

    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("timestamp").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "strategy_id",
                "symbol",
                "timeframe",
                "timestamp",
                "signal_type",
                "signal_strength",
                "price",
                "metadata",
            ])
            .build()
    }
}

impl Default for SignalQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для индикаторов
pub struct IndicatorQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl IndicatorQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.indicators"),
        }
    }

    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    pub fn by_timeframe(mut self, timeframe: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("timeframe", &format!("'{}'", timeframe));
        self
    }

    pub fn by_name(mut self, indicator_name: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("indicator_name", &format!("'{}'", indicator_name));
        self
    }

    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self
            .builder
            .where_gte(
                "timestamp",
                &format!("'{}'", start.format("%Y-%m-%d %H:%M:%S")),
            )
            .where_lte(
                "timestamp",
                &format!("'{}'", end.format("%Y-%m-%d %H:%M:%S")),
            );
        self
    }

    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("timestamp").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "symbol",
                "timeframe",
                "indicator_name",
                "timestamp",
                "value",
                "parameters",
            ])
            .build()
    }
}

impl Default for IndicatorQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для стратегий
pub struct StrategyQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl StrategyQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.strategies"),
        }
    }

    pub fn by_id(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_type(mut self, strategy_type: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_type", &format!("'{}'", strategy_type));
        self
    }

    pub fn by_creator(mut self, created_by: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("created_by", &format!("'{}'", created_by));
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "strategy_id",
                "strategy_name",
                "strategy_type",
                "indicators",
                "entry_conditions",
                "exit_conditions",
                "parameters",
                "created_by",
            ])
            .build()
    }
}

impl Default for StrategyQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для метрик стратегий
pub struct StrategyMetricQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl StrategyMetricQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.strategy_metrics"),
        }
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_metric_name(mut self, metric_name: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("metric_name", &format!("'{}'", metric_name));
        self
    }

    pub fn date_range(mut self, start_date: &str, end_date: &str) -> Self {
        self.builder = self
            .builder
            .where_gte("calculation_date", &format!("'{}'", start_date))
            .where_lte("calculation_date", &format!("'{}'", end_date));
        self
    }

    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("calculation_date").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "strategy_id",
                "metric_name",
                "metric_value",
                "calculation_date",
                "period_start",
                "period_end",
                "metadata",
            ])
            .build()
    }
}

impl Default for StrategyMetricQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для позиций
pub struct PositionQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl PositionQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.positions"),
        }
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    pub fn by_side(mut self, side: &str) -> Self {
        self.builder = self.builder.where_eq("side", &format!("'{}'", side));
        self
    }

    pub fn profitable_only(mut self) -> Self {
        self.builder = self.builder.where_gt("unrealized_pnl", "0");
        self
    }

    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("updated_at").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "position_id",
                "strategy_id",
                "symbol",
                "side",
                "quantity",
                "entry_price",
                "current_price",
                "unrealized_pnl",
                "stop_loss",
                "take_profit",
                "opened_at",
            ])
            .build()
    }
}

impl Default for PositionQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для ордеров
pub struct OrderQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl OrderQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.orders"),
        }
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", &format!("'{}'", symbol));
        self
    }

    pub fn by_status(mut self, status: &str) -> Self {
        self.builder = self.builder.where_eq("status", &format!("'{}'", status));
        self
    }

    pub fn by_order_type(mut self, order_type: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("order_type", &format!("'{}'", order_type));
        self
    }

    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("created_at").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "order_id",
                "position_id",
                "strategy_id",
                "symbol",
                "order_type",
                "side",
                "quantity",
                "price",
                "status",
                "filled_quantity",
                "avg_fill_price",
                "commission",
                "created_at",
                "filled_at",
                "cancelled_at",
            ])
            .build()
    }
}

impl Default for OrderQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для генетической оптимизации
pub struct GeneticPopulationQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl GeneticPopulationQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.genetic_population"),
        }
    }

    pub fn by_generation(mut self, generation: i32) -> Self {
        self.builder = self.builder.where_eq("generation", &generation.to_string());
        self
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn min_fitness(mut self, min_score: f32) -> Self {
        self.builder = self
            .builder
            .where_gte("fitness_score", &min_score.to_string());
        self
    }

    pub fn top_performers(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("fitness_score").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "generation",
                "individual_id",
                "strategy_id",
                "fitness_score",
                "sharpe_ratio",
                "max_drawdown",
                "win_rate",
                "profit_factor",
                "genes",
            ])
            .build()
    }
}

impl Default for GeneticPopulationQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для результатов оптимизации
pub struct OptimizationResultQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl OptimizationResultQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.optimization_results"),
        }
    }

    pub fn by_optimization_id(mut self, optimization_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("optimization_id", &format!("'{}'", optimization_id));
        self
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_parameter(mut self, parameter_name: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("parameter_name", &format!("'{}'", parameter_name));
        self
    }

    pub fn best_results(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("metric_value").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "optimization_id",
                "strategy_id",
                "parameter_name",
                "parameter_value",
                "metric_name",
                "metric_value",
                "iteration",
            ])
            .build()
    }
}

impl Default for OptimizationResultQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для снимков портфеля
pub struct PortfolioSnapshotQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl PortfolioSnapshotQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.portfolio_snapshots"),
        }
    }

    pub fn by_user(mut self, user_id: &str) -> Self {
        self.builder = self.builder.where_eq("user_id", &format!("'{}'", user_id));
        self
    }

    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self
            .builder
            .where_gte(
                "timestamp",
                &format!("'{}'", start.format("%Y-%m-%d %H:%M:%S")),
            )
            .where_lte(
                "timestamp",
                &format!("'{}'", end.format("%Y-%m-%d %H:%M:%S")),
            );
        self
    }

    pub fn positive_return_only(mut self) -> Self {
        self.builder = self.builder.where_gt("daily_return", "0");
        self
    }

    pub fn latest(mut self, count: u32) -> Self {
        self.builder = self.builder.order_by_desc("timestamp").limit(count);
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "snapshot_id",
                "user_id",
                "timestamp",
                "total_value",
                "cash",
                "positions_value",
                "unrealized_pnl",
                "realized_pnl",
                "daily_return",
                "total_return",
                "sharpe_ratio",
                "max_drawdown",
            ])
            .build()
    }
}

impl Default for PortfolioSnapshotQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Специализированный Query Builder для Walk-Forward анализа
pub struct WalkForwardQueryBuilder {
    builder: ClickHouseQueryBuilder,
}

impl WalkForwardQueryBuilder {
    pub fn new() -> Self {
        Self {
            builder: ClickHouseQueryBuilder::new().table("trading.walk_forward_results"),
        }
    }

    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("strategy_id", &format!("'{}'", strategy_id));
        self
    }

    pub fn by_window(mut self, window_number: i32) -> Self {
        self.builder = self
            .builder
            .where_eq("window_number", &window_number.to_string());
        self
    }

    pub fn min_efficiency(mut self, min_ratio: f32) -> Self {
        self.builder = self
            .builder
            .where_gte("efficiency_ratio", &min_ratio.to_string());
        self
    }

    pub fn max_overfitting(mut self, max_score: f32) -> Self {
        self.builder = self
            .builder
            .where_lte("overfitting_score", &max_score.to_string());
        self
    }

    pub fn order_by_window(mut self) -> Self {
        self.builder = self.builder.order_by_asc("window_number");
        self
    }

    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "wf_id",
                "strategy_id",
                "window_number",
                "in_sample_start",
                "in_sample_end",
                "out_sample_start",
                "out_sample_end",
                "is_sharpe",
                "oos_sharpe",
                "is_profit",
                "oos_profit",
                "is_drawdown",
                "oos_drawdown",
                "efficiency_ratio",
                "overfitting_score",
            ])
            .build()
    }
}

impl Default for WalkForwardQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Утилиты для работы с ClickHouse запросами
pub struct ClickHouseUtils;

impl ClickHouseUtils {
    /// Запрос статистики по символу
    pub fn symbol_stats_query(symbol: &str, timeframe: &str) -> String {
        format!(
            "SELECT 
                symbol,
                timeframe,
                COUNT(*) as bars_count,
                MIN(low) as min_price,
                MAX(high) as max_price,
                AVG(close) as avg_price,
                AVG(volume) as avg_volume,
                SUM(volume) as total_volume,
                stddevPop(close) as price_volatility
            FROM trading.ohlcv_data
            WHERE symbol = '{}' AND timeframe = '{}'
            GROUP BY symbol, timeframe",
            symbol, timeframe
        )
    }

    /// Запрос топ стратегий по Sharpe Ratio
    pub fn top_strategies_query(limit: u32) -> String {
        format!(
            "SELECT 
                strategy_id,
                symbol,
                timeframe,
                AVG(sharpe_ratio) as avg_sharpe,
                AVG(total_pnl) as avg_pnl,
                AVG(win_rate) as avg_win_rate,
                COUNT(*) as backtest_count
            FROM trading.backtest_results
            GROUP BY strategy_id, symbol, timeframe
            HAVING backtest_count >= 3
            ORDER BY avg_sharpe DESC
            LIMIT {}",
            limit
        )
    }

    /// Запрос анализа волатильности
    pub fn volatility_analysis_query(symbol: &str, timeframe: &str, days: u32) -> String {
        format!(
            "SELECT 
                toDate(timestamp) as date,
                stddevPop(close) as daily_volatility,
                (MAX(high) - MIN(low)) / MIN(low) * 100 as daily_range_pct,
                SUM(volume) as daily_volume
            FROM trading.ohlcv_data
            WHERE symbol = '{}' 
                AND timeframe = '{}'
                AND timestamp >= now() - INTERVAL {} DAY
            GROUP BY date
            ORDER BY date DESC",
            symbol, timeframe, days
        )
    }

    /// Запрос корреляции между двумя символами
    pub fn correlation_query(symbol1: &str, symbol2: &str, timeframe: &str, days: u32) -> String {
        format!(
            "SELECT 
                corr(a.close, b.close) as price_correlation,
                corr(a.volume, b.volume) as volume_correlation
            FROM 
                (SELECT timestamp, close, volume FROM trading.ohlcv_data 
                 WHERE symbol = '{}' AND timeframe = '{}' 
                 AND timestamp >= now() - INTERVAL {} DAY) a
            INNER JOIN 
                (SELECT timestamp, close, volume FROM trading.ohlcv_data 
                 WHERE symbol = '{}' AND timeframe = '{}'
                 AND timestamp >= now() - INTERVAL {} DAY) b
            ON a.timestamp = b.timestamp",
            symbol1, timeframe, days, symbol2, timeframe, days
        )
    }

    /// Запрос производительности стратегии по периодам
    pub fn strategy_performance_by_period(strategy_id: &str) -> String {
        format!(
            "SELECT 
                toStartOfMonth(entry_time) as month,
                COUNT(*) as total_trades,
                SUM(pnl) as total_pnl,
                AVG(pnl) as avg_pnl,
                countIf(pnl > 0) as winning_trades,
                countIf(pnl < 0) as losing_trades,
                countIf(pnl > 0) / COUNT(*) * 100 as win_rate
            FROM trading.trades
            WHERE strategy_id = '{}' AND status = 'closed'
            GROUP BY month
            ORDER BY month DESC",
            strategy_id
        )
    }

    /// Запрос распределения сделок по часам
    pub fn trades_by_hour_distribution(symbol: &str, days: u32) -> String {
        format!(
            "SELECT 
                toHour(entry_time) as hour,
                COUNT(*) as trades_count,
                AVG(pnl) as avg_pnl,
                SUM(volume) as total_volume
            FROM trading.trades
            WHERE symbol = '{}' 
                AND entry_time >= now() - INTERVAL {} DAY
                AND status = 'closed'
            GROUP BY hour
            ORDER BY hour",
            symbol, days
        )
    }

    /// Запрос лучших параметров оптимизации
    pub fn best_optimization_parameters(
        optimization_id: &str,
        metric_name: &str,
        top: u32,
    ) -> String {
        format!(
            "SELECT 
                parameter_name,
                parameter_value,
                AVG(metric_value) as avg_metric,
                MAX(metric_value) as max_metric,
                COUNT(*) as sample_count
            FROM trading.optimization_results
            WHERE optimization_id = '{}' AND metric_name = '{}'
            GROUP BY parameter_name, parameter_value
            ORDER BY avg_metric DESC
            LIMIT {}",
            optimization_id, metric_name, top
        )
    }

    /// Запрос эффективности Walk-Forward анализа
    pub fn walk_forward_efficiency(strategy_id: &str) -> String {
        format!(
            "SELECT 
                AVG(efficiency_ratio) as avg_efficiency,
                AVG(overfitting_score) as avg_overfitting,
                AVG(oos_sharpe / is_sharpe) as sharpe_degradation,
                AVG(oos_profit / is_profit) as profit_degradation,
                COUNT(*) as windows_count
            FROM trading.walk_forward_results
            WHERE strategy_id = '{}'
                AND is_sharpe > 0
                AND is_profit > 0",
            strategy_id
        )
    }
}
