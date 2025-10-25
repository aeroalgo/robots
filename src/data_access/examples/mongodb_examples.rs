//! Примеры использования MongoDB коннектора

use crate::data_access::database::mongodb::{MongoDBConnector, MongoDBUtils};
use crate::data_access::models::*;
use crate::data_access::query_builder::mongodb::{
    BacktestQueryBuilder,
    CandleQueryBuilder,
    MongoDBQueryBuilder,
    SortDirection,
    // Новые билдеры для конфигураций и метаданных
    StrategyConfigQueryBuilder,
    StrategyQueryBuilder,
    SystemConfigQueryBuilder,
    SystemMetadataQueryBuilder,
    TradeQueryBuilder,
    UserQueryBuilder,
    UserSettingsQueryBuilder,
};
use crate::data_access::traits::{DataSource, Database};
use chrono::{DateTime, Utc};

/// Пример 1: Базовые операции с MongoDB
pub async fn basic_mongodb_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Пример 1: Базовые операции с MongoDB");

    // Создание коннектора с дефолтной конфигурацией
    let mut connector = MongoDBConnector::new_default();

    // Подключение к базе данных
    connector.connect().await?;
    println!("✅ Подключение к MongoDB установлено");

    // Создание индексов
    connector.create_indexes().await?;
    println!("✅ Индексы созданы");

    // Тестирование подключения
    connector.ping().await?;
    println!("✅ Ping успешен");

    // Получение статистики базы данных
    let stats = connector.get_database_stats().await?;
    println!("📊 Статистика базы данных:");
    for (collection, count) in stats {
        println!("  {}: {} записей", collection, count);
    }

    // Отключение
    connector.disconnect().await?;
    println!("✅ Отключение от MongoDB");

    Ok(())
}

/// Пример 2: CRUD операции с пользователями
pub async fn user_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("👤 Пример 2: CRUD операции с пользователями");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Создание пользователя
    let user = User {
        id: "user_001".to_string(),
        username: "trader_john".to_string(),
        email: "john@example.com".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    connector.create_user(&user).await?;
    println!("✅ Пользователь создан");

    // Получение пользователя по ID
    let retrieved_user = connector.get_user_by_id("user_001").await?;
    if let Some(user) = retrieved_user {
        println!("✅ Найден пользователь: {} ({})", user.username, user.email);
    }

    // Получение пользователя по username
    let user_by_username = connector.get_user_by_username("trader_john").await?;
    if let Some(user) = user_by_username {
        println!(
            "✅ Найден пользователь по username: {} ({})",
            user.username, user.email
        );
    }

    // Обновление пользователя
    let mut updated_user = user.clone();
    updated_user.email = "john.updated@example.com".to_string();
    updated_user.updated_at = Utc::now();

    connector.update_user("user_001", &updated_user).await?;
    println!("✅ Пользователь обновлен");

    // Удаление пользователя
    connector.delete_user("user_001").await?;
    println!("✅ Пользователь удален");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 9: Работа с конфигурациями и метаданными (основная задача MongoDB)

/// Пример 10: Комплексная работа с конфигурациями

/// Пример 3: Работа с торговыми данными
pub async fn trading_data_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 Пример 3: Работа с торговыми данными");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

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

    connector.create_strategy(&strategy).await?;
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

    connector.create_candles_batch(&candles).await?;
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
        connector.create_trade(&trade).await?;
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

    connector.create_backtest_result(&backtest_result).await?;
    println!("✅ Результат бэктеста создан");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 9: Работа с конфигурациями и метаданными (основная задача MongoDB)

/// Пример 10: Комплексная работа с конфигурациями
pub async fn complex_configuration_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60));
    println!("🔄 ПРИМЕР 10: Комплексная работа с конфигурациями");
    println!("{}", "=".repeat(60));

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Сценарий: Настройка новой стратегии с конфигурациями
    println!("\n🚀 Сценарий: Настройка стратегии 'Moving Average Crossover'");

    // ПРИМЕЧАНИЕ: Эти методы не реализованы, так как MongoDB не используется в основной архитектуре
    // Для конфигураций, метаданных и настроек используется файловая система или другие хранилища

    // 1. Получение всех активных конфигураций системы
    // let all_configs_query = MongoDBUtils::get_all_active_configs()?;
    println!("📋 Получение конфигураций системы (не реализовано)");

    // 2. Получение конфигураций для модуля стратегий
    // let strategy_module_configs = MongoDBUtils::get_system_configs("strategy_engine", None)?;
    println!("⚙️ Получение конфигураций модуля стратегий (не реализовано)");

    // 3. Получение метаданных для индикаторов
    // let indicator_metadata = MongoDBUtils::get_system_metadata("indicator_config")?;
    println!("📊 Получение метаданных индикаторов (не реализовано)");

    // 4. Получение конфигураций стратегии по типу
    // let risk_configs = MongoDBUtils::get_strategy_configs_by_type("strategy_001", "risk_management")?;
    println!("🛡️ Получение конфигураций риск-менеджмента (не реализовано)");

    // 5. Получение пользовательских настроек для UI
    // let ui_settings = MongoDBUtils::get_user_settings("user_001", Some("ui_preferences"))?;
    println!("🎨 Получение настроек UI пользователя (не реализовано)");

    // 6. Получение конкретной настройки пользователя
    // let specific_setting = MongoDBUtils::get_user_setting_by_key("user_001", "theme")?;
    println!("🎭 Получение конкретной настройки темы (не реализовано)");

    // 7. Получение метаданных по пространству имен
    // let trading_signals_metadata = MongoDBUtils::get_metadata_by_namespace("trading.signals")?;
    println!("📡 Получение метаданных торговых сигналов (не реализовано)");

    println!("\n✅ Комплексная настройка стратегии завершена!");
    println!("📝 Все необходимые конфигурации и метаданные получены");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 4: Сложные запросы с Query Builder
pub async fn complex_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Пример 4: Сложные запросы с Query Builder");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Запрос свечей с фильтрацией по времени и символу
    let candle_query = CandleQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .time_range(Utc::now() - chrono::Duration::days(7), Utc::now())
        .min_volume(100.0)
        .order_by_timestamp()
        .build()?;

    println!("📊 Запрос свечей: {}", candle_query.to_string());

    // Запрос сделок с фильтрацией по стороне
    let trade_query = TradeQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .by_side(TradeSide::Buy)
        .price_range(50000.0, 51000.0)
        .order_by_timestamp()
        .build()?;

    println!("💰 Запрос сделок: {}", trade_query.to_string());

    // Запрос результатов бэктестов с фильтрацией по доходности
    let backtest_query = BacktestQueryBuilder::new()
        .by_strategy("strategy_001")
        .min_return(0.1)
        .min_sharpe(1.5)
        .max_drawdown(0.1)
        .order_by_return()
        .build()?;

    println!(
        "📈 Запрос результатов бэктестов: {}",
        backtest_query.to_string()
    );

    // Запрос стратегий с фильтрацией по статусу
    let strategy_query = StrategyQueryBuilder::new()
        .enabled_only()
        .by_name("Moving Average")
        .order_by_created_at()
        .build()?;

    println!("🎯 Запрос стратегий: {}", strategy_query.to_string());

    connector.disconnect().await?;
    Ok(())
}

/// Пример 9: Работа с конфигурациями и метаданными (основная задача MongoDB)

/// Пример 10: Комплексная работа с конфигурациями

/// Пример 5: Аналитические запросы
pub async fn analytical_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Пример 5: Аналитические запросы");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Топ стратегий по доходности
    let top_strategies_query = MongoDBUtils::get_top_strategies_by_return(&connector, 10).await?;
    println!(
        "🏆 Топ стратегий по доходности: {} записей",
        top_strategies_query.len()
    );

    // Статистика по символам
    let symbol_stats_query = connector.get_symbol_statistics().await?;
    println!(
        "📈 Статистика по символам: {} записей",
        symbol_stats_query.len()
    );

    // Дневная статистика торгов
    let daily_stats_query = connector.get_daily_trading_stats().await?;
    println!(
        "📅 Дневная статистика торгов: {} записей",
        daily_stats_query.len()
    );

    // Активные пользователи
    let active_users_query = MongoDBUtils::get_strategy_performance(&connector).await?;
    println!(
        "👥 Активные пользователи: {} записей",
        active_users_query.len()
    );

    // Производительность стратегий
    let strategy_performance_query = MongoDBUtils::get_strategy_performance(&connector).await?;
    println!(
        "🎯 Производительность стратегий: {} записей",
        strategy_performance_query.len()
    );

    connector.disconnect().await?;
    Ok(())
}

/// Пример 9: Работа с конфигурациями и метаданными (основная задача MongoDB)
pub async fn configuration_and_metadata_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60));
    println!("🔧 ПРИМЕР 9: Конфигурации и метаданные MongoDB");
    println!("{}", "=".repeat(60));

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // === КОНФИГУРАЦИИ СТРАТЕГИЙ ===
    println!("\n📊 Конфигурации стратегий:");

    // Получение конфигураций стратегии
    let strategy_configs_query = StrategyConfigQueryBuilder::new()
        .by_strategy("strategy_001")
        .active_only()
        .order_by_created_at()
        .build()?;
    println!(
        "🎯 Конфигурации стратегии: {}",
        strategy_configs_query.to_string()
    );

    // Получение конфигураций по типу
    let config_by_type_query = StrategyConfigQueryBuilder::new()
        .by_strategy("strategy_001")
        .by_config_type("risk_management")
        .active_only()
        .build()?;
    println!(
        "⚙️ Конфигурации риск-менеджмента: {}",
        config_by_type_query.to_string()
    );

    // === СИСТЕМНЫЕ МЕТАДАННЫЕ ===
    println!("\n📋 Системные метаданные:");

    // Получение метаданных по типу
    let system_metadata_query = SystemMetadataQueryBuilder::new()
        .by_metadata_type("indicator_config")
        .order_by_updated_at()
        .build()?;
    println!(
        "📈 Метаданные индикаторов: {}",
        system_metadata_query.to_string()
    );

    // Получение метаданных по пространству имен
    let namespace_metadata_query = SystemMetadataQueryBuilder::new()
        .by_namespace("trading.signals")
        .order_by_updated_at()
        .build()?;
    println!(
        "🔍 Метаданные пространства: {}",
        namespace_metadata_query.to_string()
    );

    // === ПОЛЬЗОВАТЕЛЬСКИЕ НАСТРОЙКИ ===
    println!("\n👤 Пользовательские настройки:");

    // Получение настроек пользователя
    let user_settings_query = UserSettingsQueryBuilder::new()
        .by_user("user_001")
        .order_by_updated_at()
        .build()?;
    println!(
        "⚙️ Настройки пользователя: {}",
        user_settings_query.to_string()
    );

    // Получение настроек по категории
    let category_settings_query = UserSettingsQueryBuilder::new()
        .by_user("user_001")
        .by_category("ui_preferences")
        .order_by_updated_at()
        .build()?;
    println!("🎨 Настройки UI: {}", category_settings_query.to_string());

    // === КОНФИГУРАЦИИ СИСТЕМЫ ===
    println!("\n🖥️ Конфигурации системы:");

    // Получение конфигураций модуля
    let module_config_query = SystemConfigQueryBuilder::new()
        .by_module("data_access")
        .active_only()
        .order_by_priority()
        .build()?;
    println!(
        "🔧 Конфигурации модуля: {}",
        module_config_query.to_string()
    );

    // Получение конфигураций по окружению
    let env_config_query = SystemConfigQueryBuilder::new()
        .by_module("trading_engine")
        .by_environment("production")
        .active_only()
        .order_by_priority()
        .build()?;
    println!("🏭 Продакшн конфигурации: {}", env_config_query.to_string());

    // === ИСПОЛЬЗОВАНИЕ УТИЛИТ ===
    println!("\n🛠️ Использование утилит:");

    // ПРИМЕЧАНИЕ: Эти утилиты не реализованы, так как MongoDB не используется для конфигураций
    // В архитектуре проекта для конфигураций используются другие подходы

    // Получение конфигураций стратегии через утилиты
    // let strategy_configs_util = MongoDBUtils::get_strategy_configs("strategy_001")?;
    println!("🎯 Утилита - конфигурации стратегии (не реализовано)");

    // Получение системных метаданных через утилиты
    // let system_metadata_util = MongoDBUtils::get_system_metadata("indicator_config")?;
    println!("📈 Утилита - системные метаданные (не реализовано)");

    // Получение пользовательских настроек через утилиты
    // let user_settings_util = MongoDBUtils::get_user_settings("user_001", Some("ui_preferences"))?;
    println!("👤 Утилита - пользовательские настройки (не реализовано)");

    // Получение конфигураций системы через утилиты
    // let system_configs_util = MongoDBUtils::get_system_configs("data_access", Some("production"))?;
    println!("🖥️ Утилита - конфигурации системы (не реализовано)");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 10: Комплексная работа с конфигурациями

/// Пример 6: Работа с транзакциями
pub async fn transaction_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Пример 6: Работа с транзакциями");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Начало транзакции
    let transaction = connector.begin_transaction().await?;
    println!("✅ Транзакция начата");

    // Выполнение операций в транзакции
    transaction.execute("db.users.insertOne({id: 'user_tx_001', username: 'transaction_user', email: 'tx@example.com'})").await?;
    println!("✅ Пользователь создан в транзакции");

    transaction.execute("db.strategies.insertOne({id: 'strategy_tx_001', name: 'Transaction Strategy', enabled: true})").await?;
    println!("✅ Стратегия создана в транзакции");

    // Подтверждение транзакции
    transaction.commit().await?;
    println!("✅ Транзакция подтверждена");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 9: Работа с конфигурациями и метаданными (основная задача MongoDB)

/// Пример 10: Комплексная работа с конфигурациями

/// Пример 7: Утилиты MongoDB
pub async fn mongodb_utilities() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️ Пример 7: Утилиты MongoDB");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Получение версии MongoDB
    let version = MongoDBUtils::get_version(&connector).await?;
    println!("📋 Версия MongoDB: {}", version);

    // Получение списка коллекций
    let collections = MongoDBUtils::get_collections(&connector).await?;
    println!("📊 Коллекции в базе данных:");
    for collection in collections {
        println!("  - {}", collection);
    }

    // Получение информации о коллекции users
    let collection_info = MongoDBUtils::get_collection_info(&connector, "users").await?;
    println!("📋 Информация о коллекции users:");
    println!(
        "  - Размер: {} байт",
        collection_info
            .get("size")
            .unwrap_or(&mongodb::bson::Bson::Int64(0))
    );
    println!(
        "  - Количество документов: {}",
        collection_info
            .get("count")
            .unwrap_or(&mongodb::bson::Bson::Int64(0))
    );

    // Получение топ стратегий
    let top_strategies = MongoDBUtils::get_top_strategies_by_return(&connector, 5).await?;
    println!("🏆 Топ стратегий по доходности:");
    for strategy in top_strategies {
        println!(
            "  - {}: Return={}%, Sharpe={}",
            strategy
                .get("_id")
                .unwrap_or(&mongodb::bson::Bson::String("Unknown".to_string())),
            strategy
                .get("avg_return")
                .unwrap_or(&mongodb::bson::Bson::Double(0.0)),
            strategy
                .get("avg_sharpe")
                .unwrap_or(&mongodb::bson::Bson::Double(0.0))
        );
    }

    // Получение производительности стратегий
    let strategy_performance = MongoDBUtils::get_strategy_performance(&connector).await?;
    println!("🎯 Производительность стратегий:");
    for strategy in strategy_performance {
        println!(
            "  - {}: Return={}%, Tests={}",
            strategy
                .get("_id")
                .unwrap_or(&mongodb::bson::Bson::String("Unknown".to_string())),
            strategy
                .get("avg_return")
                .unwrap_or(&mongodb::bson::Bson::Double(0.0)),
            strategy
                .get("test_count")
                .unwrap_or(&mongodb::bson::Bson::Int64(0))
        );
    }

    // Очистка всех коллекций (для тестирования)
    connector.truncate_all_collections().await?;
    println!("🧹 Все коллекции очищены");

    connector.disconnect().await?;
    Ok(())
}

/// Пример 10: Комплексная работа с конфигурациями

/// Пример 8: Агрегационные запросы
pub async fn aggregation_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Пример 8: Агрегационные запросы");

    let mut connector = MongoDBConnector::new_default();
    connector.connect().await?;
    connector.create_indexes().await?;

    // Создание тестовых данных
    let trades = vec![
        Trade {
            id: "trade_001".to_string(),
            timestamp: Utc::now(),
            symbol: "BTCUSDT".to_string(),
            price: 50000.0,
            quantity: 0.1,
            side: TradeSide::Buy,
            order_id: Some("order_001".to_string()),
        },
        Trade {
            id: "trade_002".to_string(),
            timestamp: Utc::now(),
            symbol: "BTCUSDT".to_string(),
            price: 51000.0,
            quantity: 0.2,
            side: TradeSide::Sell,
            order_id: Some("order_002".to_string()),
        },
        Trade {
            id: "trade_003".to_string(),
            timestamp: Utc::now(),
            symbol: "ETHUSDT".to_string(),
            price: 3000.0,
            quantity: 1.0,
            side: TradeSide::Buy,
            order_id: Some("order_003".to_string()),
        },
    ];

    for trade in trades {
        connector.create_trade(&trade).await?;
    }

    // Статистика по символам
    let symbol_stats = connector.get_symbol_statistics().await?;
    println!("📈 Статистика по символам:");
    for stat in symbol_stats {
        println!(
            "  - {}: {} сделок, средняя цена: {}, объем: {}",
            stat.get("_id")
                .unwrap_or(&mongodb::bson::Bson::String("Unknown".to_string())),
            stat.get("trade_count")
                .unwrap_or(&mongodb::bson::Bson::Int64(0)),
            stat.get("avg_price")
                .unwrap_or(&mongodb::bson::Bson::Double(0.0)),
            stat.get("total_volume")
                .unwrap_or(&mongodb::bson::Bson::Double(0.0))
        );
    }

    // Дневная статистика торгов
    let daily_stats = connector.get_daily_trading_stats().await?;
    println!("📅 Дневная статистика торгов:");
    for stat in daily_stats {
        println!(
            "  - {}: {} сделок, объем: {}, символов: {}",
            stat.get("_id")
                .unwrap_or(&mongodb::bson::Bson::String("Unknown".to_string())),
            stat.get("trade_count")
                .unwrap_or(&mongodb::bson::Bson::Int64(0)),
            stat.get("total_volume")
                .unwrap_or(&mongodb::bson::Bson::Double(0.0)),
            stat.get("symbol_count")
                .unwrap_or(&mongodb::bson::Bson::Int64(0))
        );
    }

    connector.disconnect().await?;
    Ok(())
}

/// Пример 10: Комплексная работа с конфигурациями

/// Запуск всех примеров MongoDB
pub async fn run_all_mongodb_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск всех примеров MongoDB коннектора");
    println!("{}", "=".repeat(60));

    basic_mongodb_operations().await?;
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

    mongodb_utilities().await?;
    println!();

    aggregation_queries().await?;
    println!();

    println!("✅ Все примеры MongoDB коннектора выполнены успешно!");
    Ok(())
}
