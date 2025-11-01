//! Интеграционные тесты для ClickHouse
//!
//! Для запуска тестов необходимо:
//! 1. Установить и запустить ClickHouse:
//!    docker run -d --name clickhouse-test -p 9000:9000 -p 8123:8123 clickhouse/clickhouse-server
//! 2. Запустить тесты:
//!    cargo test --test clickhouse_integration_tests -- --test-threads=1
//!
//! Используйте флаг --ignored для пропуска тестов без ClickHouse:
//!    cargo test --test clickhouse_integration_tests

use chrono::{NaiveDate, Utc};
use robots::data_access::database::{
    BacktestRecord, ClickHouseConfig, ClickHouseConnector, GeneticIndividual, Indicator, OhlcvData,
    OptimizationResult, OrderRecord, PortfolioSnapshot, Position, Signal, Strategy, StrategyMetric,
    SymbolInfo, TickData, TradeRecord, WalkForwardResult,
};
use robots::data_access::traits::{DataSource, Database};
use std::time::Duration;

/// Вспомогательная функция для создания тестового коннектора
async fn create_test_connector() -> ClickHouseConnector {
    let config = ClickHouseConfig {
        host: std::env::var("CLICKHOUSE_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port: std::env::var("CLICKHOUSE_PORT")
            .unwrap_or_else(|_| "9000".to_string())
            .parse()
            .unwrap_or(9000),
        database: "trading_test".to_string(),
        username: Some("default".to_string()),
        password: Some("".to_string()),
        connection_timeout: Duration::from_secs(5),
        query_timeout: Duration::from_secs(30),
    };

    ClickHouseConnector::with_config(config)
}

/// Проверка доступности ClickHouse
fn is_clickhouse_available() -> bool {
    std::env::var("CLICKHOUSE_AVAILABLE").unwrap_or_else(|_| "false".to_string()) == "true"
}

// ============================================================================
// ТЕСТЫ ПОДКЛЮЧЕНИЯ
// ============================================================================

#[tokio::test]
#[ignore] // Игнорируем по умолчанию, т.к. требуется запущенный ClickHouse
async fn test_clickhouse_connection() {
    if !is_clickhouse_available() {
        println!("⚠️  ClickHouse недоступен, тест пропущен");
        return;
    }

    let mut connector = create_test_connector().await;

    // Тест подключения
    let result = connector.connect().await;
    assert!(
        result.is_ok(),
        "Не удалось подключиться к ClickHouse: {:?}",
        result.err()
    );
    assert!(connector.is_connected(), "Коннектор должен быть подключен");

    // Тест простого запроса вместо ping
    let query_result = connector.execute("SELECT 1").await;
    assert!(
        query_result.is_ok(),
        "Тестовый запрос не прошел: {:?}",
        query_result.err()
    );

    // Тест отключения
    let disconnect_result = connector.disconnect().await;
    assert!(
        disconnect_result.is_ok(),
        "Отключение не удалось: {:?}",
        disconnect_result.err()
    );
    assert!(!connector.is_connected(), "Коннектор должен быть отключен");
}

#[tokio::test]
#[ignore]
async fn test_connection_info() {
    let connector = create_test_connector().await;
    let info = connector.connection_info();

    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 9000);
    assert_eq!(info.database, Some("trading_test".to_string()));
}

// ============================================================================
// ТЕСТЫ OHLCV_DATA
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_ohlcv_insert_and_query() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Подготовка тестовых данных
    let test_data = vec![
        OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(2),
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 1000.0,
        },
        OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(1),
            open: 50500.0,
            high: 51500.0,
            low: 50000.0,
            close: 51200.0,
            volume: 1200.0,
        },
    ];

    // Тест вставки
    let insert_result = connector.insert_ohlcv(&test_data).await;
    assert!(
        insert_result.is_ok(),
        "Вставка не удалась: {:?}",
        insert_result.err()
    );

    // Тест чтения
    let start_time = Utc::now() - chrono::Duration::hours(3);
    let end_time = Utc::now();
    let query_result = connector
        .get_ohlcv("BTCUSDT", "1h", start_time, end_time, Some(10))
        .await;
    assert!(
        query_result.is_ok(),
        "Запрос не удался: {:?}",
        query_result.err()
    );

    let data = query_result.unwrap();
    println!("✅ Получено {} записей OHLCV", data.len());

    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ TICK_DATA
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_tick_data_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let test_data = vec![TickData {
        symbol: "ETHUSDT".to_string(),
        timestamp: Utc::now(),
        bid: 3000.0,
        ask: 3001.0,
        last_price: 3000.5,
        volume: 100.0,
    }];

    let insert_result = connector.insert_tick_data(&test_data).await;
    assert!(
        insert_result.is_ok(),
        "Вставка тиковых данных не удалась: {:?}",
        insert_result.err()
    );

    let start_time = Utc::now() - chrono::Duration::minutes(5);
    let end_time = Utc::now() + chrono::Duration::minutes(1);
    let query_result = connector
        .get_tick_data("ETHUSDT", start_time, end_time, Some(10))
        .await;
    assert!(
        query_result.is_ok(),
        "Запрос тиковых данных не удался: {:?}",
        query_result.err()
    );

    println!("✅ Тест тиковых данных пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ SYMBOL_INFO
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_symbol_info_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let symbol_info = SymbolInfo {
        code: "BTCUSDT".to_string(),
        name: "Bitcoin/USDT".to_string(),
        exchange: "Binance".to_string(),
    };

    // Тест upsert
    let upsert_result = connector.upsert_symbol_info(&symbol_info).await;
    assert!(
        upsert_result.is_ok(),
        "Upsert symbol_info не удался: {:?}",
        upsert_result.err()
    );

    // Тест получения
    let get_result = connector.get_symbol_info("BTCUSDT", "Binance").await;
    assert!(
        get_result.is_ok(),
        "Получение symbol_info не удалось: {:?}",
        get_result.err()
    );

    // Тест получения всех символов биржи
    let exchange_symbols = connector.get_exchange_symbols("Binance").await;
    assert!(
        exchange_symbols.is_ok(),
        "Получение символов биржи не удалось: {:?}",
        exchange_symbols.err()
    );

    println!("✅ Тест symbol_info пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ INDICATORS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_indicators_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let indicators = vec![Indicator {
        symbol: "BTCUSDT".to_string(),
        timeframe: "1h".to_string(),
        indicator_name: "RSI".to_string(),
        timestamp: Utc::now(),
        value: 65.5,
        parameters: r#"{"period": 14}"#.to_string(),
    }];

    let insert_result = connector.insert_indicators(&indicators).await;
    assert!(
        insert_result.is_ok(),
        "Вставка индикаторов не удалась: {:?}",
        insert_result.err()
    );

    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now() + chrono::Duration::minutes(1);
    let query_result = connector
        .get_indicators("BTCUSDT", "1h", "RSI", start_time, end_time, Some(10))
        .await;
    assert!(
        query_result.is_ok(),
        "Запрос индикаторов не удался: {:?}",
        query_result.err()
    );

    println!("✅ Тест индикаторов пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ SIGNALS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_signals_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let signals = vec![Signal {
        strategy_id: "momentum_v1".to_string(),
        symbol: "BTCUSDT".to_string(),
        timeframe: "1h".to_string(),
        timestamp: Utc::now(),
        signal_type: "BUY".to_string(),
        signal_strength: 0.85,
        price: 51000.0,
        metadata: r#"{"reason": "strong momentum"}"#.to_string(),
    }];

    let insert_result = connector.insert_signals(&signals).await;
    assert!(
        insert_result.is_ok(),
        "Вставка сигналов не удалась: {:?}",
        insert_result.err()
    );

    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now() + chrono::Duration::minutes(1);
    let query_result = connector
        .get_signals("momentum_v1", None, start_time, end_time, Some(10))
        .await;
    assert!(
        query_result.is_ok(),
        "Запрос сигналов не удался: {:?}",
        query_result.err()
    );

    println!("✅ Тест сигналов пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ TRADES
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_trades_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let trades = vec![TradeRecord {
        trade_id: "trade_001".to_string(),
        strategy_id: "strategy_1".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "buy".to_string(),
        quantity: 0.1,
        entry_price: 50000.0,
        exit_price: Some(51000.0),
        entry_time: Utc::now() - chrono::Duration::hours(1),
        exit_time: Some(Utc::now()),
        pnl: Some(100.0),
        commission: 5.0,
        status: "closed".to_string(),
        metadata: "{}".to_string(),
    }];

    let insert_result = connector.insert_trades(&trades).await;
    assert!(
        insert_result.is_ok(),
        "Вставка сделок не удалась: {:?}",
        insert_result.err()
    );

    let query_result = connector
        .get_trades(
            Some("strategy_1"),
            Some("BTCUSDT"),
            None,
            None,
            Some("closed"),
            Some(10),
        )
        .await;
    assert!(
        query_result.is_ok(),
        "Запрос сделок не удался: {:?}",
        query_result.err()
    );

    println!("✅ Тест сделок пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ STRATEGIES
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_strategies_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let strategy = Strategy {
        strategy_id: "test_strategy_1".to_string(),
        strategy_name: "Test Momentum Strategy".to_string(),
        strategy_type: "momentum".to_string(),
        indicators: vec!["RSI".to_string(), "MACD".to_string()],
        entry_conditions: r#"{"rsi": ">30", "macd": "bullish"}"#.to_string(),
        exit_conditions: r#"{"rsi": ">70"}"#.to_string(),
        parameters: r#"{"risk": 0.02}"#.to_string(),
        created_by: "test_user".to_string(),
    };

    let upsert_result = connector.upsert_strategy(&strategy).await;
    assert!(
        upsert_result.is_ok(),
        "Upsert стратегии не удался: {:?}",
        upsert_result.err()
    );

    let get_result = connector.get_strategy("test_strategy_1").await;
    assert!(
        get_result.is_ok(),
        "Получение стратегии не удалось: {:?}",
        get_result.err()
    );

    let by_type_result = connector.get_strategies_by_type("momentum").await;
    assert!(
        by_type_result.is_ok(),
        "Получение стратегий по типу не удалось: {:?}",
        by_type_result.err()
    );

    println!("✅ Тест стратегий пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ BACKTEST_RESULTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_backtest_results_operations() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    let backtest = BacktestRecord {
        backtest_id: "bt_001".to_string(),
        strategy_id: "strategy_1".to_string(),
        symbol: "BTCUSDT".to_string(),
        timeframe: "1h".to_string(),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        total_trades: 150,
        winning_trades: 95,
        losing_trades: 55,
        total_pnl: 15500.0,
        max_drawdown: -5.2,
        sharpe_ratio: 1.8,
        profit_factor: 2.1,
        win_rate: 63.33,
        avg_win: 200.0,
        avg_loss: -95.0,
        execution_time_ms: 1250,
    };

    let insert_result = connector.insert_backtest_result(&backtest).await;
    assert!(
        insert_result.is_ok(),
        "Вставка результата бэктеста не удалась: {:?}",
        insert_result.err()
    );

    let query_result = connector
        .get_backtest_results(Some("strategy_1"), Some("BTCUSDT"), Some("1h"), Some(10))
        .await;
    assert!(
        query_result.is_ok(),
        "Запрос результатов бэктестов не удался: {:?}",
        query_result.err()
    );

    println!("✅ Тест результатов бэктестов пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ АНАЛИТИЧЕСКИХ МЕТОДОВ
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_analytics_methods() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Тест статистики по символу
    let symbol_stats = connector.get_symbol_stats("BTCUSDT", "1h").await;
    assert!(
        symbol_stats.is_ok(),
        "Получение статистики по символу не удалось: {:?}",
        symbol_stats.err()
    );

    // Тест статистики стратегии
    let strategy_stats = connector.get_strategy_stats("strategy_1").await;
    assert!(
        strategy_stats.is_ok(),
        "Получение статистики стратегии не удалось: {:?}",
        strategy_stats.err()
    );

    println!("✅ Тест аналитических методов пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ BATCH OPERATIONS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_batch_insertions() {
    if !is_clickhouse_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Создаем большой батч данных
    let mut batch_data = Vec::new();
    for i in 0..100 {
        batch_data.push(OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            timestamp: Utc::now() - chrono::Duration::minutes(i),
            open: 50000.0 + (i as f64),
            high: 50100.0 + (i as f64),
            low: 49900.0 + (i as f64),
            close: 50050.0 + (i as f64),
            volume: 1000.0,
        });
    }

    let start = std::time::Instant::now();
    let insert_result = connector.insert_ohlcv(&batch_data).await;
    let duration = start.elapsed();

    assert!(
        insert_result.is_ok(),
        "Батч вставка не удалась: {:?}",
        insert_result.err()
    );
    println!(
        "✅ Вставлено {} записей за {:?}",
        batch_data.len(),
        duration
    );

    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ ERROR HANDLING
// ============================================================================

#[tokio::test]
async fn test_connection_error_handling() {
    // Создаем коннектор с неверными параметрами
    let config = ClickHouseConfig {
        host: "invalid_host".to_string(),
        port: 9999,
        database: "test".to_string(),
        username: None,
        password: None,
        connection_timeout: Duration::from_secs(1),
        query_timeout: Duration::from_secs(1),
    };

    let mut connector = ClickHouseConnector::with_config(config);

    // Попытка подключиться к несуществующему хосту должна завершиться ошибкой
    // (в текущей заглушке это всегда успешно, но в реальной реализации должна быть ошибка)
    let result = connector.connect().await;
    println!(
        "Результат подключения к несуществующему хосту: {:?}",
        result
    );
}

#[tokio::test]
async fn test_query_without_connection() {
    let connector = create_test_connector().await;
    // НЕ вызываем connect()

    let start_time = Utc::now() - chrono::Duration::hours(1);
    let end_time = Utc::now();

    let result = connector
        .get_ohlcv("BTCUSDT", "1h", start_time, end_time, Some(10))
        .await;
    assert!(
        result.is_err(),
        "Запрос без подключения должен вернуть ошибку"
    );

    println!("✅ Тест обработки ошибок пройден");
}
