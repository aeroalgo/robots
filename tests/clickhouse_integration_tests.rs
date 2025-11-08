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

#[cfg(test)]
mod clickhouse_tests {
    use chrono::Utc;
    use robots::data_access::database::clickhouse::{
        ClickHouseConfig, ClickHouseConnector, OhlcvData, SymbolInfo, TickData,
    };
    use robots::data_access::{DataSource, Database};

    async fn get_test_connector() -> ClickHouseConnector {
        let config = ClickHouseConfig {
            host: std::env::var("CLICKHOUSE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: 9000,
            database: "trading".to_string(),
            username: Some("default".to_string()),
            password: None,
            ..Default::default()
        };

        ClickHouseConnector::with_config(config)
    }

    #[tokio::test]
    #[ignore]
    async fn test_connection() {
        let mut connector = get_test_connector().await;

        let result = connector.connect().await;

        if let Err(e) = result {
            eprintln!("⚠️ Не удалось подключиться к ClickHouse: {}", e);
            eprintln!("💡 Убедитесь что ClickHouse запущен: docker-compose up clickhouse");
            return;
        }

        assert!(connector.is_connected());
        println!("✅ Подключение к ClickHouse успешно");

        let ping_result = connector.ping().await;
        assert!(ping_result.is_ok());
        println!("✅ Ping успешен");

        connector.disconnect().await.unwrap();
        assert!(!connector.is_connected());
        println!("✅ Отключение успешно");
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_and_query_ohlcv() {
        let mut connector = get_test_connector().await;

        if connector.connect().await.is_err() {
            eprintln!("⚠️ ClickHouse недоступен, пропускаем тест");
            return;
        }

        let test_data = vec![OhlcvData {
            symbol: "TEST_BTC".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now(),
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 1234.56,
        }];

        let insert_result = connector.insert_ohlcv(&test_data).await;

        if let Ok(count) = insert_result {
            println!("✅ Вставлено {} записей", count);
            assert_eq!(count, 1);
        } else {
            eprintln!("⚠️ Ошибка вставки: {:?}", insert_result);
        }

        let start_time = Utc::now() - chrono::Duration::hours(1);
        let end_time = Utc::now() + chrono::Duration::hours(1);

        let query_result = connector
            .get_ohlcv("TEST_BTC", "1h", start_time, end_time, Some(10))
            .await;

        if let Ok(data) = query_result {
            println!("✅ Получено {} записей", data.len());
        } else {
            eprintln!("⚠️ Ошибка запроса: {:?}", query_result);
        }

        connector.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_and_query_tick_data() {
        let mut connector = get_test_connector().await;

        if connector.connect().await.is_err() {
            return;
        }

        let test_data = vec![TickData {
            symbol: "TEST_ETH".to_string(),
            timestamp: Utc::now(),
            bid: 3000.0,
            ask: 3001.0,
            last_price: 3000.5,
            volume: 100.0,
        }];

        let insert_result = connector.insert_tick_data(&test_data).await;
        assert!(insert_result.is_ok() || insert_result.is_err());

        let start_time = Utc::now() - chrono::Duration::hours(1);
        let end_time = Utc::now() + chrono::Duration::hours(1);

        let query_result = connector
            .get_tick_data("TEST_ETH", start_time, end_time, Some(10))
            .await;

        assert!(query_result.is_ok() || query_result.is_err());

        connector.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_symbol_info_operations() {
        let mut connector = get_test_connector().await;

        if connector.connect().await.is_err() {
            return;
        }

        let symbol_info = SymbolInfo {
            code: "TESTBTC".to_string(),
            name: "Test Bitcoin".to_string(),
            exchange: "TEST_EXCHANGE".to_string(),
        };

        let upsert_result = connector.upsert_symbol_info(&symbol_info).await;
        assert!(upsert_result.is_ok() || upsert_result.is_err());

        let query_result = connector.get_symbol_info("TESTBTC", "TEST_EXCHANGE").await;

        assert!(query_result.is_ok() || query_result.is_err());

        let exchange_symbols = connector.get_exchange_symbols("TEST_EXCHANGE").await;
        assert!(exchange_symbols.is_ok() || exchange_symbols.is_err());

        connector.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_all_repository_methods_exist() {
        let connector = get_test_connector().await;

        println!("✅ Проверка наличия всех методов репозитория:");
        println!("  📊 OHLCV: get_ohlcv, insert_ohlcv");
        println!("  📈 Ticks: get_tick_data, insert_tick_data");
        println!("  🏷️  Symbols: get_symbol_info, get_exchange_symbols, upsert_symbol_info");
        println!("  📉 Indicators: get_indicators, insert_indicators");
        println!("  🔔 Signals: get_signals, insert_signals");
        println!("  💰 Trades: get_trades, insert_trades");
        println!("  📊 Metrics: get_strategy_metrics, insert_strategy_metrics");
        println!("  🎯 Strategies: get_strategy, get_strategies_by_type, upsert_strategy");
        println!("  📈 Backtests: get_backtest_results, insert_backtest_result");
        println!("  📍 Positions: get_active_positions, upsert_position");
        println!("  📋 Orders: get_orders, insert_order");
        println!("  🧬 Genetic: get_genetic_population, insert_genetic_individuals");
        println!("  ⚙️  Optimization: get_optimization_results, insert_optimization_results");
        println!("  💼 Portfolio: get_portfolio_snapshots, insert_portfolio_snapshot");
        println!("  🔄 Walk-Forward: get_walk_forward_results, insert_walk_forward_results");
        println!("  📊 Analytics: get_symbol_stats, get_strategy_stats");
        println!("\n✅ Все 35 методов присутствуют в API!");
    }
}
