//! Примеры использования PostgreSQL коннектора

use crate::data_access::database::postgresql::{
    PostgreSQLConfig, PostgreSQLConnector, PostgreSQLUtils,
};
use crate::data_access::models::*;
use crate::data_access::query_builder::postgresql::{
    BacktestQueryBuilder, CandleQueryBuilder, PostgreSQLQueryBuilder, StrategyQueryBuilder,
    TradeQueryBuilder, UserQueryBuilder,
};
use crate::data_access::traits::{DataSource, Database};
use chrono::{DateTime, Utc};

/// Пример 1: Базовые операции с PostgreSQL
pub async fn basic_postgresql_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Пример 1: Базовые операции с PostgreSQL");

    // Создание коннектора с дефолтной конфигурацией
    let mut connector = PostgreSQLConnector::new_default();

    // Подключение к базе данных
    connector.connect().await?;
    println!("✅ Подключение к PostgreSQL установлено");

    // Создание таблиц
    connector.create_tables().await?;
    println!("✅ Таблицы созданы");

    // Создание индексов
    connector.create_indexes().await?;
    println!("✅ Индексы созданы");

    // Тестирование подключения
    connector.ping().await?;
    println!("✅ Ping успешен");

    // Получение статистики базы данных
    let stats = connector.get_database_stats().await?;
    println!("📊 Статистика базы данных:");
    for (table, count) in stats {
        println!("  {}: {} записей", table, count);
    }

    // Отключение
    connector.disconnect().await?;
    println!("✅ Отключение от PostgreSQL");

    Ok(())
}

/// Пример 2: CRUD операции с пользователями
pub async fn user_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("👤 Пример 2: CRUD операции с пользователями");

    let mut connector = PostgreSQLConnector::new_default();
    connector.connect().await?;
    connector.create_tables().await?;

    // Создание пользователя
    let user = User {
        id: "user_001".to_string(),
        username: "trader_john".to_string(),
        email: "john@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let insert_query = PostgreSQLQueryBuilder::new()
        .insert_into(
            "users",
            &["id", "username", "email", "created_at", "updated_at"],
        )
        .values(&[
            &user.id,
            &user.username,
            &user.email,
            &user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            &user.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ])
        .build()?;

    println!("📝 Выполнение INSERT запроса: {}", insert_query);
    connector.execute(&insert_query).await?;
    println!("✅ Пользователь создан");

    // Получение пользователя по username
    let select_query = UserQueryBuilder::new().by_username("trader_john").build()?;

    println!("🔍 Выполнение SELECT запроса: {}", select_query);
    let users: Vec<User> = connector.query(&select_query).await?;
    println!("✅ Найдено пользователей: {}", users.len());
    for user in users {
        println!("  👤 {} ({})", user.username, user.email);
    }

    // Обновление пользователя
    let update_query = PostgreSQLQueryBuilder::new()
        .update("users")
        .set("email", "'john.updated@example.com'")
        .where_eq("username", "'trader_john'")
        .build()?;

    println!("🔄 Выполнение UPDATE запроса: {}", update_query);
    let rows_affected = connector.execute(&update_query).await?;
    println!("✅ Обновлено записей: {}", rows_affected);

    // Удаление пользователя
    let delete_query = PostgreSQLQueryBuilder::new()
        .delete_from("users")
        .where_eq("username", "'trader_john'")
        .build()?;

    println!("🗑️ Выполнение DELETE запроса: {}", delete_query);
    let rows_affected = connector.execute(&delete_query).await?;
    println!("✅ Удалено записей: {}", rows_affected);

    connector.disconnect().await?;
    Ok(())
}

/// Пример 3: Работа с торговыми данными
pub async fn trading_data_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 Пример 3: Работа с торговыми данными");

    let mut connector = PostgreSQLConnector::new_default();
    connector.connect().await?;
    connector.create_tables().await?;

    // Создание стратегии
    let strategy = Strategy {
        id: "strategy_001".to_string(),
        name: "Moving Average Crossover".to_string(),
        description: Some("Простая стратегия пересечения скользящих средних".to_string()),
        parameters: serde_json::json!({
            "short_period": 10,
            "long_period": 30,
            "threshold": 0.01
        }),
        enabled: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let strategy_query = PostgreSQLQueryBuilder::new()
        .insert_into(
            "strategies",
            &[
                "id",
                "name",
                "description",
                "parameters",
                "enabled",
                "created_at",
                "updated_at",
            ],
        )
        .values(&[
            &strategy.id,
            &strategy.name,
            &strategy.description.as_ref().unwrap(),
            &strategy.parameters.to_string(),
            "true",
            &strategy.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            &strategy.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ])
        .build()?;

    connector.execute(&strategy_query).await?;
    println!("✅ Стратегия создана");

    // Создание свечей
    let candles = vec![
        Candle {
            timestamp: Utc::now(),
            symbol: "BTCUSDT".to_string(),
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 100.5,
        },
        Candle {
            timestamp: Utc::now(),
            symbol: "ETHUSDT".to_string(),
            open: 3000.0,
            high: 3100.0,
            low: 2950.0,
            close: 3050.0,
            volume: 500.0,
        },
    ];

    for candle in candles {
        let candle_query = PostgreSQLQueryBuilder::new()
            .insert_into(
                "candles",
                &[
                    "timestamp",
                    "symbol",
                    "open",
                    "high",
                    "low",
                    "close",
                    "volume",
                ],
            )
            .values(&[
                &candle.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                &candle.symbol,
                &candle.open.to_string(),
                &candle.high.to_string(),
                &candle.low.to_string(),
                &candle.close.to_string(),
                &candle.volume.to_string(),
            ])
            .build()?;

        connector.execute(&candle_query).await?;
    }
    println!("✅ Свечи созданы");

    // Создание сделок
    let trades = vec![
        Trade {
            id: "trade_001".to_string(),
            timestamp: Utc::now(),
            symbol: "BTCUSDT".to_string(),
            price: 50500.0,
            quantity: 0.1,
            side: TradeSide::Buy,
            order_id: Some("order_001".to_string()),
        },
        Trade {
            id: "trade_002".to_string(),
            timestamp: Utc::now(),
            symbol: "ETHUSDT".to_string(),
            price: 3050.0,
            quantity: 1.0,
            side: TradeSide::Sell,
            order_id: Some("order_002".to_string()),
        },
    ];

    for trade in trades {
        let trade_query = PostgreSQLQueryBuilder::new()
            .insert_into(
                "trades",
                &[
                    "id",
                    "timestamp",
                    "symbol",
                    "price",
                    "quantity",
                    "side",
                    "order_id",
                ],
            )
            .values(&[
                &trade.id,
                &trade.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                &trade.symbol,
                &trade.price.to_string(),
                &trade.quantity.to_string(),
                &format!("'{:?}'", trade.side),
                &trade.order_id.as_ref().unwrap(),
            ])
            .build()?;

        connector.execute(&trade_query).await?;
    }
    println!("✅ Сделки созданы");

    // Создание результата бэктеста
    let backtest_result = BacktestResult {
        strategy_id: "strategy_001".to_string(),
        symbol: "BTCUSDT".to_string(),
        start_date: Utc::now(),
        end_date: Utc::now(),
        total_return: 0.15,
        sharpe_ratio: 1.8,
        max_drawdown: 0.05,
        total_trades: 100,
        winning_trades: 65,
        losing_trades: 35,
        win_rate: 0.65,
        created_at: Utc::now(),
    };

    let backtest_query = PostgreSQLQueryBuilder::new()
        .insert_into(
            "backtest_results",
            &[
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
            ],
        )
        .values(&[
            &backtest_result.strategy_id,
            &backtest_result.symbol,
            &backtest_result
                .start_date
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            &backtest_result
                .end_date
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            &backtest_result.total_return.to_string(),
            &backtest_result.sharpe_ratio.to_string(),
            &backtest_result.max_drawdown.to_string(),
            &backtest_result.total_trades.to_string(),
            &backtest_result.winning_trades.to_string(),
            &backtest_result.losing_trades.to_string(),
            &backtest_result.win_rate.to_string(),
            &backtest_result
                .created_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ])
        .build()?;

    connector.execute(&backtest_query).await?;
    println!("✅ Результат бэктеста создан");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 4: Сложные запросы с Query Builder
pub async fn complex_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Пример 4: Сложные запросы с Query Builder");

    let mut connector = PostgreSQLConnector::new_default();
    connector.connect().await?;
    connector.create_tables().await?;

    // Запрос свечей с фильтрацией по времени и символу
    let candle_query = CandleQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .time_range(Utc::now() - chrono::Duration::days(7), Utc::now())
        .min_volume(100.0)
        .order_by_timestamp()
        .build()?;

    println!("📊 Запрос свечей: {}", candle_query);

    // Запрос сделок с фильтрацией по стороне
    let trade_query = TradeQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .by_side(TradeSide::Buy)
        .price_range(50000.0, 51000.0)
        .order_by_timestamp()
        .build()?;

    println!("💰 Запрос сделок: {}", trade_query);

    // Запрос результатов бэктестов с фильтрацией по доходности
    let backtest_query = BacktestQueryBuilder::new()
        .by_strategy("strategy_001")
        .min_return(0.1)
        .min_sharpe(1.5)
        .max_drawdown(0.1)
        .order_by_return()
        .build()?;

    println!("📈 Запрос результатов бэктестов: {}", backtest_query);

    // Запрос стратегий с фильтрацией по статусу
    let strategy_query = StrategyQueryBuilder::new()
        .enabled_only()
        .by_name("Moving Average")
        .order_by_created_at()
        .build()?;

    println!("🎯 Запрос стратегий: {}", strategy_query);

    connector.disconnect().await?;
    Ok(())
}

/// Пример 5: Аналитические запросы
pub async fn analytical_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Пример 5: Аналитические запросы");

    let mut connector = PostgreSQLConnector::new_default();
    connector.connect().await?;
    connector.create_tables().await?;

    // Топ стратегий по доходности
    let top_strategies_query =
        crate::data_access::query_builder::postgresql::PostgreSQLUtils::top_strategies_by_return(
            10,
        )?;
    println!("🏆 Топ стратегий по доходности: {}", top_strategies_query);

    // Статистика по символам
    let symbol_stats_query =
        crate::data_access::query_builder::postgresql::PostgreSQLUtils::symbol_statistics()?;
    println!("📈 Статистика по символам: {}", symbol_stats_query);

    // Дневная статистика торгов
    let daily_stats_query =
        crate::data_access::query_builder::postgresql::PostgreSQLUtils::daily_trading_stats()?;
    println!("📅 Дневная статистика торгов: {}", daily_stats_query);

    // Активные пользователи
    let active_users_query =
        crate::data_access::query_builder::postgresql::PostgreSQLUtils::active_users(30)?;
    println!("👥 Активные пользователи: {}", active_users_query);

    // Производительность стратегий
    let strategy_performance_query =
        crate::data_access::query_builder::postgresql::PostgreSQLUtils::strategy_performance()?;
    println!(
        "🎯 Производительность стратегий: {}",
        strategy_performance_query
    );

    connector.disconnect().await?;
    Ok(())
}

/// Пример 6: Работа с транзакциями
pub async fn transaction_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Пример 6: Работа с транзакциями");

    let mut connector = PostgreSQLConnector::new_default();
    connector.connect().await?;
    connector.create_tables().await?;

    // Начало транзакции
    let transaction = connector.begin_transaction().await?;
    println!("✅ Транзакция начата");

    // Выполнение операций в транзакции
    let insert_user_query = PostgreSQLQueryBuilder::new()
        .insert_into(
            "users",
            &["id", "username", "email", "created_at", "updated_at"],
        )
        .values(&[
            "user_tx_001",
            "transaction_user",
            "tx@example.com",
            &Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            &Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ])
        .build()?;

    transaction.execute(&insert_user_query).await?;
    println!("✅ Пользователь создан в транзакции");

    let insert_strategy_query = PostgreSQLQueryBuilder::new()
        .insert_into(
            "strategies",
            &[
                "id",
                "name",
                "description",
                "enabled",
                "created_at",
                "updated_at",
            ],
        )
        .values(&[
            "strategy_tx_001",
            "Transaction Strategy",
            "Стратегия для тестирования транзакций",
            "true",
            &Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            &Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        ])
        .build()?;

    transaction.execute(&insert_strategy_query).await?;
    println!("✅ Стратегия создана в транзакции");

    // Подтверждение транзакции
    transaction.commit().await?;
    println!("✅ Транзакция подтверждена");

    // Проверка результатов
    let check_query = PostgreSQLQueryBuilder::new()
        .select_all()
        .from("users")
        .where_eq("id", "'user_tx_001'")
        .build()?;

    let users: Vec<User> = connector.query(&check_query).await?;
    println!("✅ Найдено пользователей после транзакции: {}", users.len());

    connector.disconnect().await?;
    Ok(())
}

/// Пример 7: Утилиты PostgreSQL
pub async fn postgresql_utilities() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️ Пример 7: Утилиты PostgreSQL");

    let mut connector = PostgreSQLConnector::new_default();
    connector.connect().await?;
    connector.create_tables().await?;

    // Получение версии PostgreSQL
    let version = PostgreSQLUtils::get_version(&connector).await?;
    println!("📋 Версия PostgreSQL: {}", version);

    // Получение списка таблиц
    let tables = PostgreSQLUtils::get_tables(&connector).await?;
    println!("📊 Таблицы в базе данных:");
    for table in tables {
        println!("  - {}", table);
    }

    // Получение информации о таблице users
    let table_info = PostgreSQLUtils::get_table_info(&connector, "users").await?;
    println!("📋 Информация о таблице users:");
    for (column, data_type, is_nullable) in table_info {
        println!("  - {}: {} (nullable: {})", column, data_type, is_nullable);
    }

    // Очистка всех таблиц (для тестирования)
    PostgreSQLUtils::truncate_all_tables(&connector).await?;
    println!("🧹 Все таблицы очищены");

    connector.disconnect().await?;
    Ok(())
}

/// Запуск всех примеров PostgreSQL
pub async fn run_all_postgresql_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск всех примеров PostgreSQL коннектора");
    println!("{}", "=".repeat(60));

    basic_postgresql_operations().await?;
    println!();

    user_crud_operations().await?;
    println!();

    trading_data_operations().await?;
    println!();

    complex_queries().await?;
    println!();

    analytical_queries().await?;
    println!();

    transaction_operations().await?;
    println!();

    postgresql_utilities().await?;
    println!();

    println!("✅ Все примеры PostgreSQL коннектора выполнены успешно!");
    Ok(())
}
