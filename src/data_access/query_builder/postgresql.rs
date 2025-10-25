//! Query Builder для PostgreSQL

use crate::data_access::models::*;
use crate::data_access::{DataAccessError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Базовый Query Builder для PostgreSQL
#[derive(Clone)]
pub struct PostgreSQLQueryBuilder {
    query_type: QueryType,
    table: String,
    columns: Vec<String>,
    conditions: Vec<Condition>,
    order_by: Vec<OrderClause>,
    group_by: Vec<String>,
    having: Vec<Condition>,
    limit: Option<u32>,
    offset: Option<u32>,
    joins: Vec<JoinClause>,
    values: Vec<Vec<String>>,
    updates: Vec<UpdateClause>,
}

/// Тип запроса
#[derive(Debug, Clone)]
enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
}

/// Условие WHERE
#[derive(Debug, Clone)]
struct Condition {
    column: String,
    operator: ComparisonOperator,
    value: String,
    logical_op: Option<LogicalOperator>,
}

/// Оператор сравнения
#[derive(Debug, Clone)]
enum ComparisonOperator {
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

/// Логический оператор
#[derive(Debug, Clone)]
enum LogicalOperator {
    And,
    Or,
}

/// Упорядочивание
#[derive(Debug, Clone)]
struct OrderClause {
    column: String,
    direction: OrderDirection,
}

/// Направление сортировки
#[derive(Debug, Clone)]
enum OrderDirection {
    Asc,
    Desc,
}

/// JOIN операция
#[derive(Debug, Clone)]
struct JoinClause {
    join_type: JoinType,
    table: String,
    condition: String,
}

/// Тип JOIN
#[derive(Debug, Clone)]
enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// UPDATE операция
#[derive(Debug, Clone)]
struct UpdateClause {
    column: String,
    value: String,
}

impl PostgreSQLQueryBuilder {
    /// Создание нового Query Builder
    pub fn new() -> Self {
        Self {
            query_type: QueryType::Select,
            table: String::new(),
            columns: Vec::new(),
            conditions: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
            values: Vec::new(),
            updates: Vec::new(),
        }
    }

    /// SELECT запрос
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.query_type = QueryType::Select;
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// SELECT * запрос
    pub fn select_all(mut self) -> Self {
        self.query_type = QueryType::Select;
        self.columns = vec!["*".to_string()];
        self
    }

    /// FROM таблица
    pub fn from(mut self, table: &str) -> Self {
        self.table = table.to_string();
        self
    }

    /// WHERE условие
    pub fn where_eq(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::Equal, value, None);
        self
    }

    /// WHERE NOT EQUAL условие
    pub fn where_ne(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::NotEqual, value, None);
        self
    }

    /// WHERE GREATER THAN условие
    pub fn where_gt(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::GreaterThan, value, None);
        self
    }

    /// WHERE GREATER THAN OR EQUAL условие
    pub fn where_gte(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::GreaterThanOrEqual, value, None);
        self
    }

    /// WHERE LESS THAN условие
    pub fn where_lt(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::LessThan, value, None);
        self
    }

    /// WHERE LESS THAN OR EQUAL условие
    pub fn where_lte(mut self, column: &str, value: &str) -> Self {
        self.add_condition(column, ComparisonOperator::LessThanOrEqual, value, None);
        self
    }

    /// WHERE LIKE условие
    pub fn where_like(mut self, column: &str, pattern: &str) -> Self {
        self.add_condition(column, ComparisonOperator::Like, pattern, None);
        self
    }

    /// WHERE IN условие
    pub fn where_in(mut self, column: &str, values: &[&str]) -> Self {
        let value_str = format!("({})", values.join(", "));
        self.add_condition(column, ComparisonOperator::In, &value_str, None);
        self
    }

    /// WHERE BETWEEN условие
    pub fn where_between(mut self, column: &str, start: &str, end: &str) -> Self {
        let value_str = format!("{} AND {}", start, end);
        self.add_condition(column, ComparisonOperator::Between, &value_str, None);
        self
    }

    /// WHERE IS NULL условие
    pub fn where_is_null(mut self, column: &str) -> Self {
        self.add_condition(column, ComparisonOperator::IsNull, "", None);
        self
    }

    /// WHERE IS NOT NULL условие
    pub fn where_is_not_null(mut self, column: &str) -> Self {
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

    /// ORDER BY ASC
    pub fn order_by_asc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Asc)
    }

    /// ORDER BY DESC
    pub fn order_by_desc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Desc)
    }

    /// ORDER BY
    fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push(OrderClause {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// GROUP BY
    pub fn group_by(mut self, columns: &[&str]) -> Self {
        self.group_by = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// HAVING условие
    pub fn having(mut self, column: &str, operator: &str, value: &str) -> Self {
        self.having.push(Condition {
            column: column.to_string(),
            operator: ComparisonOperator::Equal, // Упрощение для примера
            value: value.to_string(),
            logical_op: None,
        });
        self
    }

    /// LIMIT
    pub fn limit(mut self, count: u32) -> Self {
        self.limit = Some(count);
        self
    }

    /// OFFSET
    pub fn offset(mut self, count: u32) -> Self {
        self.offset = Some(count);
        self
    }

    /// INNER JOIN
    pub fn inner_join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// LEFT JOIN
    pub fn left_join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// RIGHT JOIN
    pub fn right_join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Right,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// FULL JOIN
    pub fn full_join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Full,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// INSERT запрос
    pub fn insert_into(mut self, table: &str, columns: &[&str]) -> Self {
        self.query_type = QueryType::Insert;
        self.table = table.to_string();
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// VALUES для INSERT
    pub fn values(mut self, values: &[&str]) -> Self {
        self.values
            .push(values.iter().map(|s| s.to_string()).collect());
        self
    }

    /// UPDATE запрос
    pub fn update(mut self, table: &str) -> Self {
        self.query_type = QueryType::Update;
        self.table = table.to_string();
        self
    }

    /// SET для UPDATE
    pub fn set(mut self, column: &str, value: &str) -> Self {
        self.updates.push(UpdateClause {
            column: column.to_string(),
            value: value.to_string(),
        });
        self
    }

    /// DELETE запрос
    pub fn delete_from(mut self, table: &str) -> Self {
        self.query_type = QueryType::Delete;
        self.table = table.to_string();
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
            logical_op,
        });
    }

    /// Построение SQL запроса
    pub fn build(&self) -> Result<String> {
        match self.query_type {
            QueryType::Select => self.build_select(),
            QueryType::Insert => self.build_insert(),
            QueryType::Update => self.build_update(),
            QueryType::Delete => self.build_delete(),
        }
    }

    /// Построение SELECT запроса
    fn build_select(&self) -> Result<String> {
        if self.table.is_empty() {
            return Err(DataAccessError::Query(
                "Table name is required for SELECT query".to_string(),
            ));
        }

        let mut query = String::new();

        // SELECT
        query.push_str("SELECT ");
        if self.columns.is_empty() {
            query.push_str("*");
        } else {
            query.push_str(&self.columns.join(", "));
        }

        // FROM
        query.push_str(&format!(" FROM {}", self.table));

        // JOINs
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
            let mut first = true;
            for condition in &self.conditions {
                if !first {
                    match condition.logical_op {
                        Some(LogicalOperator::And) => query.push_str(" AND "),
                        Some(LogicalOperator::Or) => query.push_str(" OR "),
                        None => {}
                    }
                }
                first = false;

                query.push_str(&self.build_condition(condition));
            }
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            query.push_str(" GROUP BY ");
            query.push_str(&self.group_by.join(", "));
        }

        // HAVING
        if !self.having.is_empty() {
            query.push_str(" HAVING ");
            let mut first = true;
            for condition in &self.having {
                if !first {
                    query.push_str(" AND ");
                }
                first = false;
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
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(query)
    }

    /// Построение INSERT запроса
    fn build_insert(&self) -> Result<String> {
        if self.table.is_empty() {
            return Err(DataAccessError::Query(
                "Table name is required for INSERT query".to_string(),
            ));
        }

        if self.columns.is_empty() {
            return Err(DataAccessError::Query(
                "Columns are required for INSERT query".to_string(),
            ));
        }

        if self.values.is_empty() {
            return Err(DataAccessError::Query(
                "Values are required for INSERT query".to_string(),
            ));
        }

        let mut query = String::new();

        // INSERT INTO
        query.push_str(&format!("INSERT INTO {} (", self.table));
        query.push_str(&self.columns.join(", "));
        query.push_str(")");

        // VALUES
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
        if self.table.is_empty() {
            return Err(DataAccessError::Query(
                "Table name is required for UPDATE query".to_string(),
            ));
        }

        if self.updates.is_empty() {
            return Err(DataAccessError::Query(
                "SET clauses are required for UPDATE query".to_string(),
            ));
        }

        let mut query = String::new();

        // UPDATE
        query.push_str(&format!("UPDATE {}", self.table));

        // SET
        if !self.columns.is_empty() && !self.values.is_empty() {
            query.push_str(" SET ");
            let updates: Vec<String> = self
                .columns
                .iter()
                .zip(self.values[0].iter())
                .map(|(col, val)| format!("{} = {}", col, val))
                .collect();
            query.push_str(&updates.join(", "));
        } else if !self.updates.is_empty() {
            query.push_str(" SET ");
            let updates: Vec<String> = self
                .updates
                .iter()
                .map(|update| format!("{} = {}", update.column, update.value))
                .collect();
            query.push_str(&updates.join(", "));
        }

        // WHERE
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let mut first = true;
            for condition in &self.conditions {
                if !first {
                    match condition.logical_op {
                        Some(LogicalOperator::And) => query.push_str(" AND "),
                        Some(LogicalOperator::Or) => query.push_str(" OR "),
                        None => {}
                    }
                }
                first = false;
                query.push_str(&self.build_condition(condition));
            }
        }

        Ok(query)
    }

    /// Построение DELETE запроса
    fn build_delete(&self) -> Result<String> {
        if self.table.is_empty() {
            return Err(DataAccessError::Query(
                "Table name is required for DELETE query".to_string(),
            ));
        }

        let mut query = String::new();

        // DELETE FROM
        query.push_str(&format!("DELETE FROM {}", self.table));

        // WHERE
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let mut first = true;
            for condition in &self.conditions {
                if !first {
                    match condition.logical_op {
                        Some(LogicalOperator::And) => query.push_str(" AND "),
                        Some(LogicalOperator::Or) => query.push_str(" OR "),
                        None => {}
                    }
                }
                first = false;
                query.push_str(&self.build_condition(condition));
            }
        }

        Ok(query)
    }

    /// Построение условия
    fn build_condition(&self, condition: &Condition) -> String {
        match condition.operator {
            ComparisonOperator::Equal => format!("{} = {}", condition.column, condition.value),
            ComparisonOperator::NotEqual => format!("{} != {}", condition.column, condition.value),
            ComparisonOperator::GreaterThan => {
                format!("{} > {}", condition.column, condition.value)
            }
            ComparisonOperator::GreaterThanOrEqual => {
                format!("{} >= {}", condition.column, condition.value)
            }
            ComparisonOperator::LessThan => format!("{} < {}", condition.column, condition.value),
            ComparisonOperator::LessThanOrEqual => {
                format!("{} <= {}", condition.column, condition.value)
            }
            ComparisonOperator::Like => format!("{} LIKE {}", condition.column, condition.value),
            ComparisonOperator::In => format!("{} IN {}", condition.column, condition.value),
            ComparisonOperator::Between => {
                format!("{} BETWEEN {}", condition.column, condition.value)
            }
            ComparisonOperator::IsNull => format!("{} IS NULL", condition.column),
            ComparisonOperator::IsNotNull => format!("{} IS NOT NULL", condition.column),
        }
    }
}

/// Специализированный Query Builder для пользователей
pub struct UserQueryBuilder {
    builder: PostgreSQLQueryBuilder,
}

impl UserQueryBuilder {
    /// Создание нового UserQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: PostgreSQLQueryBuilder::new(),
        }
    }

    /// Фильтр по username
    pub fn by_username(mut self, username: &str) -> Self {
        self.builder = self
            .builder
            .where_eq("username", &format!("'{}'", username));
        self
    }

    /// Фильтр по email
    pub fn by_email(mut self, email: &str) -> Self {
        self.builder = self.builder.where_eq("email", &format!("'{}'", email));
        self
    }

    /// Фильтр по дате создания
    pub fn created_after(mut self, date: DateTime<Utc>) -> Self {
        self.builder = self.builder.where_gte(
            "created_at",
            &format!("'{}'", date.format("%Y-%m-%d %H:%M:%S")),
        );
        self
    }

    /// Сортировка по дате создания
    pub fn order_by_created_at(mut self) -> Self {
        self.builder = self.builder.order_by_desc("created_at");
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&["id", "username", "email", "created_at", "updated_at"])
            .from("users")
            .build()
    }
}

/// Специализированный Query Builder для стратегий
pub struct StrategyQueryBuilder {
    builder: PostgreSQLQueryBuilder,
}

impl StrategyQueryBuilder {
    /// Создание нового StrategyQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: PostgreSQLQueryBuilder::new(),
        }
    }

    /// Фильтр по enabled статусу
    pub fn enabled_only(mut self) -> Self {
        self.builder = self.builder.where_eq("enabled", "true");
        self
    }

    /// Фильтр по названию стратегии
    pub fn by_name(mut self, name: &str) -> Self {
        self.builder = self.builder.where_like("name", &format!("'%{}%'", name));
        self
    }

    /// Сортировка по дате создания
    pub fn order_by_created_at(mut self) -> Self {
        self.builder = self.builder.order_by_desc("created_at");
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "id",
                "name",
                "description",
                "parameters",
                "enabled",
                "created_at",
                "updated_at",
            ])
            .from("strategies")
            .build()
    }
}

/// Специализированный Query Builder для свечей
pub struct CandleQueryBuilder {
    builder: PostgreSQLQueryBuilder,
}

impl CandleQueryBuilder {
    /// Создание нового CandleQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: PostgreSQLQueryBuilder::new(),
        }
    }

    /// Фильтр по символу
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

    /// Фильтр по объему
    pub fn min_volume(mut self, min_volume: f64) -> Self {
        self.builder = self.builder.where_gte("volume", &min_volume.to_string());
        self
    }

    /// Сортировка по времени
    pub fn order_by_timestamp(mut self) -> Self {
        self.builder = self.builder.order_by_asc("timestamp");
        self
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
            .from("candles")
            .build()
    }
}

/// Специализированный Query Builder для сделок
pub struct TradeQueryBuilder {
    builder: PostgreSQLQueryBuilder,
}

impl TradeQueryBuilder {
    /// Создание нового TradeQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: PostgreSQLQueryBuilder::new(),
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
    pub fn order_by_timestamp(mut self) -> Self {
        self.builder = self.builder.order_by_desc("timestamp");
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
            .from("trades")
            .build()
    }
}

/// Специализированный Query Builder для результатов бэктестов
pub struct BacktestQueryBuilder {
    builder: PostgreSQLQueryBuilder,
}

impl BacktestQueryBuilder {
    /// Создание нового BacktestQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: PostgreSQLQueryBuilder::new(),
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
    pub fn order_by_return(mut self) -> Self {
        self.builder = self.builder.order_by_desc("total_return");
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<String> {
        self.builder
            .clone()
            .select(&[
                "id",
                "strategy_id",
                "symbol",
                "start_date",
                "end_date",
                "total_return",
                "sharpe_ratio",
                "max_drawdown",
                "total_trades",
                "winning_trades",
                "losing_trades",
                "win_rate",
                "created_at",
            ])
            .from("backtest_results")
            .build()
    }
}

/// Утилиты для PostgreSQL запросов
pub struct PostgreSQLUtils;

impl PostgreSQLUtils {
    /// Получение топ стратегий по доходности
    pub fn top_strategies_by_return(limit: u32) -> Result<String> {
        PostgreSQLQueryBuilder::new()
            .select(&[
                "strategy_id",
                "symbol",
                "total_return",
                "sharpe_ratio",
                "max_drawdown",
            ])
            .from("backtest_results")
            .order_by_desc("total_return")
            .limit(limit)
            .build()
    }

    /// Получение статистики по символам
    pub fn symbol_statistics() -> Result<String> {
        PostgreSQLQueryBuilder::new()
            .select(&[
                "symbol",
                "COUNT(*) as trade_count",
                "AVG(price) as avg_price",
                "SUM(quantity) as total_volume",
            ])
            .from("trades")
            .group_by(&["symbol"])
            .order_by_desc("trade_count")
            .build()
    }

    /// Получение дневной статистики торгов
    pub fn daily_trading_stats() -> Result<String> {
        PostgreSQLQueryBuilder::new()
            .select(&[
                "DATE(timestamp) as date",
                "COUNT(*) as trade_count",
                "SUM(quantity * price) as total_volume",
            ])
            .from("trades")
            .group_by(&["DATE(timestamp)"])
            .order_by_desc("date")
            .build()
    }

    /// Получение активных пользователей
    pub fn active_users(days: u32) -> Result<String> {
        PostgreSQLQueryBuilder::new()
            .select(&[
                "u.id",
                "u.username",
                "u.email",
                "COUNT(t.id) as trade_count",
            ])
            .from("users")
            .inner_join("trades", "u.id = t.user_id")
            .where_gte("t.timestamp", &format!("NOW() - INTERVAL '{} days'", days))
            .group_by(&["u.id", "u.username", "u.email"])
            .having("trade_count", ">", "0")
            .order_by_desc("trade_count")
            .build()
    }

    /// Получение производительности стратегий
    pub fn strategy_performance() -> Result<String> {
        PostgreSQLQueryBuilder::new()
            .select(&[
                "s.name",
                "AVG(br.total_return) as avg_return",
                "AVG(br.sharpe_ratio) as avg_sharpe",
                "COUNT(br.id) as backtest_count",
            ])
            .from("strategies")
            .inner_join("backtest_results", "s.id = br.strategy_id")
            .group_by(&["s.id", "s.name"])
            .order_by_desc("avg_return")
            .build()
    }
}
