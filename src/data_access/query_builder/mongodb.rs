//! Query Builder для MongoDB - метаданные и конфигурация

use crate::data_access::models::*;
use crate::data_access::{DataAccessError, Result};
use chrono::{DateTime, Utc};
use mongodb::bson::{doc, Document};

/// Базовый Query Builder для MongoDB
#[derive(Clone)]
pub struct MongoDBQueryBuilder {
    collection: String,
    filter: Document,
    projection: Option<Document>,
    sort: Option<Document>,
    limit: Option<i64>,
    skip: Option<u64>,
    aggregation_pipeline: Vec<Document>,
    update_doc: Option<Document>,
    insert_docs: Vec<Document>,
    operation: QueryOperation,
}

/// Тип операции MongoDB
#[derive(Debug, Clone)]
enum QueryOperation {
    Find,
    FindOne,
    Insert,
    InsertMany,
    Update,
    UpdateMany,
    Delete,
    DeleteMany,
    Aggregate,
    Count,
}

impl MongoDBQueryBuilder {
    /// Создание нового Query Builder
    pub fn new() -> Self {
        Self {
            collection: String::new(),
            filter: doc! {},
            projection: None,
            sort: None,
            limit: None,
            skip: None,
            aggregation_pipeline: Vec::new(),
            update_doc: None,
            insert_docs: Vec::new(),
            operation: QueryOperation::Find,
        }
    }

    /// Выбор коллекции
    pub fn collection(mut self, collection: &str) -> Self {
        self.collection = collection.to_string();
        self
    }

    /// FIND операция
    pub fn find(mut self) -> Self {
        self.operation = QueryOperation::Find;
        self
    }

    /// FINDONE операция
    pub fn find_one(mut self) -> Self {
        self.operation = QueryOperation::FindOne;
        self
    }

    /// INSERT операция
    pub fn insert(mut self) -> Self {
        self.operation = QueryOperation::Insert;
        self
    }

    /// INSERT MANY операция
    pub fn insert_many(mut self) -> Self {
        self.operation = QueryOperation::InsertMany;
        self
    }

    /// UPDATE операция
    pub fn update(mut self) -> Self {
        self.operation = QueryOperation::Update;
        self
    }

    /// UPDATE MANY операция
    pub fn update_many(mut self) -> Self {
        self.operation = QueryOperation::UpdateMany;
        self
    }

    /// DELETE операция
    pub fn delete(mut self) -> Self {
        self.operation = QueryOperation::Delete;
        self
    }

    /// DELETE MANY операция
    pub fn delete_many(mut self) -> Self {
        self.operation = QueryOperation::DeleteMany;
        self
    }

    /// AGGREGATE операция
    pub fn aggregate(mut self) -> Self {
        self.operation = QueryOperation::Aggregate;
        self
    }

    /// COUNT операция
    pub fn count(mut self) -> Self {
        self.operation = QueryOperation::Count;
        self
    }

    /// WHERE условие (равенство)
    pub fn where_eq(mut self, field: &str, value: &str) -> Self {
        self.filter.insert(field, value);
        self
    }

    /// WHERE условие (неравенство)
    pub fn where_ne(mut self, field: &str, value: &str) -> Self {
        self.filter.insert(field, doc! { "$ne": value });
        self
    }

    /// WHERE условие (больше)
    pub fn where_gt(mut self, field: &str, value: &str) -> Self {
        self.filter.insert(field, doc! { "$gt": value });
        self
    }

    /// WHERE условие (больше или равно)
    pub fn where_gte(mut self, field: &str, value: &str) -> Self {
        self.filter.insert(field, doc! { "$gte": value });
        self
    }

    /// WHERE условие (меньше)
    pub fn where_lt(mut self, field: &str, value: &str) -> Self {
        self.filter.insert(field, doc! { "$lt": value });
        self
    }

    /// WHERE условие (меньше или равно)
    pub fn where_lte(mut self, field: &str, value: &str) -> Self {
        self.filter.insert(field, doc! { "$lte": value });
        self
    }

    /// WHERE условие (в массиве)
    pub fn where_in(mut self, field: &str, values: &[&str]) -> Self {
        self.filter.insert(field, doc! { "$in": values });
        self
    }

    /// WHERE условие (не в массиве)
    pub fn where_nin(mut self, field: &str, values: &[&str]) -> Self {
        self.filter.insert(field, doc! { "$nin": values });
        self
    }

    /// WHERE условие (существует)
    pub fn where_exists(mut self, field: &str) -> Self {
        self.filter.insert(field, doc! { "$exists": true });
        self
    }

    /// WHERE условие (не существует)
    pub fn where_not_exists(mut self, field: &str) -> Self {
        self.filter.insert(field, doc! { "$exists": false });
        self
    }

    /// WHERE условие (регулярное выражение)
    pub fn where_regex(mut self, field: &str, pattern: &str) -> Self {
        self.filter.insert(field, doc! { "$regex": pattern });
        self
    }

    /// WHERE условие (диапазон дат)
    pub fn where_date_range(
        mut self,
        field: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        self.filter.insert(
            field,
            doc! {
                "$gte": mongodb::bson::DateTime::from_millis(start.timestamp_millis()),
                "$lte": mongodb::bson::DateTime::from_millis(end.timestamp_millis())
            },
        );
        self
    }

    /// AND условие
    pub fn and_eq(mut self, field: &str, value: &str) -> Self {
        if let Some(and_conditions) = self.filter.get_mut("$and") {
            if let Some(and_array) = and_conditions.as_array_mut() {
                and_array.push(doc! { field: value }.into());
            }
        } else {
            self.filter.insert("$and", vec![doc! { field: value }]);
        }
        self
    }

    /// OR условие
    pub fn or_eq(mut self, field: &str, value: &str) -> Self {
        if let Some(or_conditions) = self.filter.get_mut("$or") {
            if let Some(or_array) = or_conditions.as_array_mut() {
                or_array.push(doc! { field: value }.into());
            }
        } else {
            self.filter.insert("$or", vec![doc! { field: value }]);
        }
        self
    }

    /// PROJECTION (выбор полей)
    pub fn project(mut self, fields: &[&str]) -> Self {
        let mut projection = doc! {};
        for field in fields {
            projection.insert(*field, 1);
        }
        self.projection = Some(projection);
        self
    }

    /// SORT (сортировка)
    pub fn sort(mut self, field: &str, direction: SortDirection) -> Self {
        let sort_doc = match direction {
            SortDirection::Asc => doc! { field: 1 },
            SortDirection::Desc => doc! { field: -1 },
        };
        self.sort = Some(sort_doc);
        self
    }

    /// SORT по нескольким полям
    pub fn sort_multi(mut self, sorts: &[(&str, SortDirection)]) -> Self {
        let mut sort_doc = doc! {};
        for (field, direction) in sorts {
            let value = match direction {
                SortDirection::Asc => 1,
                SortDirection::Desc => -1,
            };
            sort_doc.insert(*field, value);
        }
        self.sort = Some(sort_doc);
        self
    }

    /// LIMIT
    pub fn limit(mut self, count: i64) -> Self {
        self.limit = Some(count);
        self
    }

    /// SKIP
    pub fn skip(mut self, count: u64) -> Self {
        self.skip = Some(count);
        self
    }

    /// SET для UPDATE
    pub fn set(mut self, field: &str, value: &str) -> Self {
        if let Some(update) = &mut self.update_doc {
            update.insert(field, value);
        } else {
            self.update_doc = Some(doc! { "$set": { field: value } });
        }
        self
    }

    /// INC для UPDATE (инкремент)
    pub fn inc(mut self, field: &str, value: i64) -> Self {
        if let Some(update) = &mut self.update_doc {
            if let Some(inc_doc) = update.get_mut("$inc") {
                if let Some(inc_obj) = inc_doc.as_document_mut() {
                    inc_obj.insert(field, value);
                }
            } else {
                update.insert("$inc", doc! { field: value });
            }
        } else {
            self.update_doc = Some(doc! { "$inc": { field: value } });
        }
        self
    }

    /// PUSH для UPDATE (добавление в массив)
    pub fn push(mut self, field: &str, value: &str) -> Self {
        if let Some(update) = &mut self.update_doc {
            if let Some(push_doc) = update.get_mut("$push") {
                if let Some(push_obj) = push_doc.as_document_mut() {
                    push_obj.insert(field, value);
                }
            } else {
                update.insert("$push", doc! { field: value });
            }
        } else {
            self.update_doc = Some(doc! { "$push": { field: value } });
        }
        self
    }

    /// VALUES для INSERT
    pub fn values(mut self, values: &[(&str, &str)]) -> Self {
        let mut doc = doc! {};
        for (field, value) in values {
            doc.insert(*field, *value);
        }
        self.insert_docs.push(doc);
        self
    }

    /// GROUP для агрегации
    pub fn group(mut self, group_by: &str, aggregations: &[(&str, &str)]) -> Self {
        let mut group_doc = doc! { "_id": format!("${}", group_by) };
        for (field, operation) in aggregations {
            group_doc.insert(*field, doc! { operation: format!("${}", field) });
        }
        self.aggregation_pipeline.push(doc! { "$group": group_doc });
        self
    }

    /// MATCH для агрегации
    pub fn match_agg(mut self, filter: Document) -> Self {
        self.aggregation_pipeline.push(doc! { "$match": filter });
        self
    }

    /// SORT для агрегации
    pub fn sort_agg(mut self, field: &str, direction: SortDirection) -> Self {
        let sort_value = match direction {
            SortDirection::Asc => 1,
            SortDirection::Desc => -1,
        };
        self.aggregation_pipeline
            .push(doc! { "$sort": { field: sort_value } });
        self
    }

    /// LIMIT для агрегации
    pub fn limit_agg(mut self, count: i64) -> Self {
        self.aggregation_pipeline.push(doc! { "$limit": count });
        self
    }

    /// SKIP для агрегации
    pub fn skip_agg(mut self, count: i64) -> Self {
        self.aggregation_pipeline.push(doc! { "$skip": count });
        self
    }

    /// Построение MongoDB запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        if self.collection.is_empty() {
            return Err(DataAccessError::Query(
                "Collection name is required".to_string(),
            ));
        }

        let query = MongoDBQuery {
            collection: self.collection.clone(),
            operation: self.operation.clone(),
            filter: self.filter.clone(),
            projection: self.projection.clone(),
            sort: self.sort.clone(),
            limit: self.limit,
            skip: self.skip,
            aggregation_pipeline: self.aggregation_pipeline.clone(),
            update_doc: self.update_doc.clone(),
            insert_docs: self.insert_docs.clone(),
        };

        Ok(query)
    }
}

/// Направление сортировки
#[derive(Debug, Clone)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// MongoDB запрос
#[derive(Debug, Clone)]
pub struct MongoDBQuery {
    pub collection: String,
    pub operation: QueryOperation,
    pub filter: Document,
    pub projection: Option<Document>,
    pub sort: Option<Document>,
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub aggregation_pipeline: Vec<Document>,
    pub update_doc: Option<Document>,
    pub insert_docs: Vec<Document>,
}

impl MongoDBQuery {
    /// Получение строкового представления запроса
    pub fn to_string(&self) -> String {
        match self.operation {
            QueryOperation::Find => {
                format!("db.{}.find({})", self.collection, self.filter)
            }
            QueryOperation::FindOne => {
                format!("db.{}.findOne({})", self.collection, self.filter)
            }
            QueryOperation::Insert => {
                if let Some(doc) = self.insert_docs.first() {
                    format!("db.{}.insertOne({})", self.collection, doc)
                } else {
                    format!("db.{}.insertOne({{}})", self.collection)
                }
            }
            QueryOperation::InsertMany => {
                format!("db.{}.insertMany({:?})", self.collection, self.insert_docs)
            }
            QueryOperation::Update => {
                if let Some(update) = &self.update_doc {
                    format!(
                        "db.{}.updateOne({}, {})",
                        self.collection, self.filter, update
                    )
                } else {
                    format!("db.{}.updateOne({}, {{}})", self.collection, self.filter)
                }
            }
            QueryOperation::UpdateMany => {
                if let Some(update) = &self.update_doc {
                    format!(
                        "db.{}.updateMany({}, {})",
                        self.collection, self.filter, update
                    )
                } else {
                    format!("db.{}.updateMany({}, {{}})", self.collection, self.filter)
                }
            }
            QueryOperation::Delete => {
                format!("db.{}.deleteOne({})", self.collection, self.filter)
            }
            QueryOperation::DeleteMany => {
                format!("db.{}.deleteMany({})", self.collection, self.filter)
            }
            QueryOperation::Aggregate => {
                format!(
                    "db.{}.aggregate({:?})",
                    self.collection, self.aggregation_pipeline
                )
            }
            QueryOperation::Count => {
                format!("db.{}.countDocuments({})", self.collection, self.filter)
            }
        }
    }
}

/// Специализированный Query Builder для пользователей
pub struct UserQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl UserQueryBuilder {
    /// Создание нового UserQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("users"),
        }
    }

    /// Фильтр по username
    pub fn by_username(mut self, username: &str) -> Self {
        self.builder = self.builder.where_eq("username", username);
        self
    }

    /// Фильтр по email
    pub fn by_email(mut self, email: &str) -> Self {
        self.builder = self.builder.where_eq("email", email);
        self
    }

    /// Фильтр по дате создания
    pub fn created_after(mut self, date: DateTime<Utc>) -> Self {
        self.builder = self.builder.where_gte("created_at", &date.to_rfc3339());
        self
    }

    /// Сортировка по дате создания
    pub fn order_by_created_at(mut self) -> Self {
        self.builder = self.builder.sort("created_at", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&["id", "username", "email", "created_at", "updated_at"])
            .build()
    }
}

/// Специализированный Query Builder для стратегий
pub struct StrategyQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl StrategyQueryBuilder {
    /// Создание нового StrategyQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("strategies"),
        }
    }

    /// Фильтр по enabled статусу
    pub fn enabled_only(mut self) -> Self {
        self.builder = self.builder.where_eq("enabled", "true");
        self
    }

    /// Фильтр по названию стратегии
    pub fn by_name(mut self, name: &str) -> Self {
        self.builder = self.builder.where_regex("name", &format!(".*{}.*", name));
        self
    }

    /// Сортировка по дате создания
    pub fn order_by_created_at(mut self) -> Self {
        self.builder = self.builder.sort("created_at", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
                "id",
                "name",
                "description",
                "parameters",
                "enabled",
                "created_at",
                "updated_at",
            ])
            .build()
    }
}

/// Специализированный Query Builder для свечей
pub struct CandleQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl CandleQueryBuilder {
    /// Создание нового CandleQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("candles"),
        }
    }

    /// Фильтр по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", symbol);
        self
    }

    /// Фильтр по времени
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self.builder.where_date_range("timestamp", start, end);
        self
    }

    /// Фильтр по объему
    pub fn min_volume(mut self, min_volume: f64) -> Self {
        self.builder = self.builder.where_gte("volume", &min_volume.to_string());
        self
    }

    /// Сортировка по времени
    pub fn order_by_timestamp(mut self) -> Self {
        self.builder = self.builder.sort("timestamp", SortDirection::Asc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
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

/// Специализированный Query Builder для сделок
pub struct TradeQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl TradeQueryBuilder {
    /// Создание нового TradeQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("trades"),
        }
    }

    /// Фильтр по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", symbol);
        self
    }

    /// Фильтр по стороне сделки
    pub fn by_side(mut self, side: TradeSide) -> Self {
        let side_str = match side {
            TradeSide::Buy => "Buy",
            TradeSide::Sell => "Sell",
        };
        self.builder = self.builder.where_eq("side", side_str);
        self
    }

    /// Фильтр по времени
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.builder = self.builder.where_date_range("timestamp", start, end);
        self
    }

    /// Фильтр по цене
    pub fn price_range(mut self, min_price: f64, max_price: f64) -> Self {
        self.builder = self
            .builder
            .where_gte("price", &min_price.to_string())
            .where_lte("price", &max_price.to_string());
        self
    }

    /// Сортировка по времени
    pub fn order_by_timestamp(mut self) -> Self {
        self.builder = self.builder.sort("timestamp", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
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

/// Специализированный Query Builder для результатов бэктестов
pub struct BacktestQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl BacktestQueryBuilder {
    /// Создание нового BacktestQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("backtest_results"),
        }
    }

    /// Фильтр по стратегии
    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self.builder.where_eq("strategy_id", strategy_id);
        self
    }

    /// Фильтр по символу
    pub fn by_symbol(mut self, symbol: &str) -> Self {
        self.builder = self.builder.where_eq("symbol", symbol);
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
        self.builder = self.builder.sort("total_return", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
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
            .build()
    }
}

/// Специализированный Query Builder для конфигураций стратегий
pub struct StrategyConfigQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl StrategyConfigQueryBuilder {
    /// Создание нового StrategyConfigQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("strategy_configs"),
        }
    }

    /// Фильтр по стратегии
    pub fn by_strategy(mut self, strategy_id: &str) -> Self {
        self.builder = self.builder.where_eq("strategy_id", strategy_id);
        self
    }

    /// Фильтр по типу конфигурации
    pub fn by_config_type(mut self, config_type: &str) -> Self {
        self.builder = self.builder.where_eq("config_type", config_type);
        self
    }

    /// Фильтр по активным конфигурациям
    pub fn active_only(mut self) -> Self {
        self.builder = self.builder.where_eq("active", "true");
        self
    }

    /// Фильтр по версии
    pub fn by_version(mut self, version: &str) -> Self {
        self.builder = self.builder.where_eq("version", version);
        self
    }

    /// Сортировка по дате создания
    pub fn order_by_created_at(mut self) -> Self {
        self.builder = self.builder.sort("created_at", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
                "strategy_id",
                "config_type",
                "config_data",
                "version",
                "active",
                "created_at",
                "updated_at",
            ])
            .build()
    }
}

/// Специализированный Query Builder для системных метаданных
pub struct SystemMetadataQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl SystemMetadataQueryBuilder {
    /// Создание нового SystemMetadataQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("system_metadata"),
        }
    }

    /// Фильтр по типу метаданных
    pub fn by_metadata_type(mut self, metadata_type: &str) -> Self {
        self.builder = self.builder.where_eq("metadata_type", metadata_type);
        self
    }

    /// Фильтр по ключу
    pub fn by_key(mut self, key: &str) -> Self {
        self.builder = self.builder.where_eq("key", key);
        self
    }

    /// Фильтр по пространству имен
    pub fn by_namespace(mut self, namespace: &str) -> Self {
        self.builder = self.builder.where_eq("namespace", namespace);
        self
    }

    /// Сортировка по дате обновления
    pub fn order_by_updated_at(mut self) -> Self {
        self.builder = self.builder.sort("updated_at", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
                "metadata_type",
                "key",
                "namespace",
                "value",
                "description",
                "created_at",
                "updated_at",
            ])
            .build()
    }
}

/// Специализированный Query Builder для пользовательских настроек
pub struct UserSettingsQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl UserSettingsQueryBuilder {
    /// Создание нового UserSettingsQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("user_settings"),
        }
    }

    /// Фильтр по пользователю
    pub fn by_user(mut self, user_id: &str) -> Self {
        self.builder = self.builder.where_eq("user_id", user_id);
        self
    }

    /// Фильтр по категории настроек
    pub fn by_category(mut self, category: &str) -> Self {
        self.builder = self.builder.where_eq("category", category);
        self
    }

    /// Фильтр по ключу настройки
    pub fn by_setting_key(mut self, key: &str) -> Self {
        self.builder = self.builder.where_eq("setting_key", key);
        self
    }

    /// Сортировка по дате обновления
    pub fn order_by_updated_at(mut self) -> Self {
        self.builder = self.builder.sort("updated_at", SortDirection::Desc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
                "user_id",
                "category",
                "setting_key",
                "setting_value",
                "data_type",
                "created_at",
                "updated_at",
            ])
            .build()
    }
}

/// Специализированный Query Builder для конфигураций системы
pub struct SystemConfigQueryBuilder {
    builder: MongoDBQueryBuilder,
}

impl SystemConfigQueryBuilder {
    /// Создание нового SystemConfigQueryBuilder
    pub fn new() -> Self {
        Self {
            builder: MongoDBQueryBuilder::new().collection("system_configs"),
        }
    }

    /// Фильтр по модулю системы
    pub fn by_module(mut self, module: &str) -> Self {
        self.builder = self.builder.where_eq("module", module);
        self
    }

    /// Фильтр по окружению
    pub fn by_environment(mut self, environment: &str) -> Self {
        self.builder = self.builder.where_eq("environment", environment);
        self
    }

    /// Фильтр по активным конфигурациям
    pub fn active_only(mut self) -> Self {
        self.builder = self.builder.where_eq("active", "true");
        self
    }

    /// Сортировка по приоритету
    pub fn order_by_priority(mut self) -> Self {
        self.builder = self.builder.sort("priority", SortDirection::Asc);
        self
    }

    /// Построение запроса
    pub fn build(&self) -> Result<MongoDBQuery> {
        self.builder
            .clone()
            .find()
            .project(&[
                "module",
                "environment",
                "config_key",
                "config_value",
                "priority",
                "active",
                "created_at",
                "updated_at",
            ])
            .build()
    }
}

/// Утилиты для MongoDB запросов
pub struct MongoDBUtils;

impl MongoDBUtils {
    /// Получение топ стратегий по доходности
    pub fn top_strategies_by_return(limit: i64) -> Result<MongoDBQuery> {
        MongoDBQueryBuilder::new()
            .collection("backtest_results")
            .aggregate()
            .group(
                "strategy_id",
                &[
                    ("avg_return", "$avg"),
                    ("avg_sharpe", "$avg"),
                    ("avg_drawdown", "$avg"),
                    ("test_count", "$sum"),
                ],
            )
            .sort_agg("avg_return", SortDirection::Desc)
            .limit_agg(limit)
            .build()
    }

    /// Получение статистики по символам
    pub fn symbol_statistics() -> Result<MongoDBQuery> {
        MongoDBQueryBuilder::new()
            .collection("trades")
            .aggregate()
            .group(
                "symbol",
                &[
                    ("trade_count", "$sum"),
                    ("avg_price", "$avg"),
                    ("total_volume", "$sum"),
                    ("min_price", "$min"),
                    ("max_price", "$max"),
                ],
            )
            .sort_agg("trade_count", SortDirection::Desc)
            .build()
    }

    /// Получение дневной статистики торгов
    pub fn daily_trading_stats() -> Result<MongoDBQuery> {
        MongoDBQueryBuilder::new()
            .collection("trades")
            .aggregate()
            .group(
                "timestamp",
                &[
                    ("trade_count", "$sum"),
                    ("total_volume", "$sum"),
                    ("unique_symbols", "$addToSet"),
                ],
            )
            .sort_agg("timestamp", SortDirection::Desc)
            .build()
    }

    /// Получение производительности стратегий
    pub fn strategy_performance() -> Result<MongoDBQuery> {
        MongoDBQueryBuilder::new()
            .collection("backtest_results")
            .aggregate()
            .group(
                "strategy_id",
                &[
                    ("avg_return", "$avg"),
                    ("avg_sharpe", "$avg"),
                    ("avg_drawdown", "$avg"),
                    ("test_count", "$sum"),
                    ("symbols", "$addToSet"),
                ],
            )
            .sort_agg("avg_return", SortDirection::Desc)
            .build()
    }

    /// Получение активных пользователей
    pub fn active_users(days: u32) -> Result<MongoDBQuery> {
        let start_date = Utc::now() - chrono::Duration::days(days as i64);
        MongoDBQueryBuilder::new()
            .collection("trades")
            .aggregate()
            .match_agg(doc! {
                "timestamp": { "$gte": mongodb::bson::DateTime::from_millis(start_date.timestamp_millis()) }
            })
            .group(
                "user_id",
                &[("trade_count", "$sum"), ("total_volume", "$sum")],
            )
            .sort_agg("trade_count", SortDirection::Desc)
            .build()
    }

    // === УТИЛИТЫ ДЛЯ КОНФИГУРАЦИЙ И МЕТАДАННЫХ ===

    /// Получение конфигураций стратегии
    pub fn get_strategy_configs(strategy_id: &str) -> Result<MongoDBQuery> {
        StrategyConfigQueryBuilder::new()
            .by_strategy(strategy_id)
            .active_only()
            .order_by_created_at()
            .build()
    }

    /// Получение системных метаданных по типу
    pub fn get_system_metadata(metadata_type: &str) -> Result<MongoDBQuery> {
        SystemMetadataQueryBuilder::new()
            .by_metadata_type(metadata_type)
            .order_by_updated_at()
            .build()
    }

    /// Получение пользовательских настроек
    pub fn get_user_settings(user_id: &str, category: Option<&str>) -> Result<MongoDBQuery> {
        let mut builder = UserSettingsQueryBuilder::new().by_user(user_id);
        if let Some(cat) = category {
            builder = builder.by_category(cat);
        }
        builder.order_by_updated_at().build()
    }

    /// Получение конфигураций системы по модулю
    pub fn get_system_configs(module: &str, environment: Option<&str>) -> Result<MongoDBQuery> {
        let mut builder = SystemConfigQueryBuilder::new()
            .by_module(module)
            .active_only();
        if let Some(env) = environment {
            builder = builder.by_environment(env);
        }
        builder.order_by_priority().build()
    }

    /// Получение всех активных конфигураций системы
    pub fn get_all_active_configs() -> Result<MongoDBQuery> {
        SystemConfigQueryBuilder::new()
            .active_only()
            .order_by_priority()
            .build()
    }

    /// Получение метаданных по пространству имен
    pub fn get_metadata_by_namespace(namespace: &str) -> Result<MongoDBQuery> {
        SystemMetadataQueryBuilder::new()
            .by_namespace(namespace)
            .order_by_updated_at()
            .build()
    }

    /// Получение конфигураций стратегии по типу
    pub fn get_strategy_configs_by_type(
        strategy_id: &str,
        config_type: &str,
    ) -> Result<MongoDBQuery> {
        StrategyConfigQueryBuilder::new()
            .by_strategy(strategy_id)
            .by_config_type(config_type)
            .active_only()
            .order_by_created_at()
            .build()
    }

    /// Получение пользовательских настроек по ключу
    pub fn get_user_setting_by_key(user_id: &str, setting_key: &str) -> Result<MongoDBQuery> {
        UserSettingsQueryBuilder::new()
            .by_user(user_id)
            .by_setting_key(setting_key)
            .order_by_updated_at()
            .build()
    }
}
