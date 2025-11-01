//! Примеры использования ClickHouse коннектора

use crate::data_access::database::{
    BacktestRecord, ClickHouseConnector, OhlcvData, TradeRecord,
};
use crate::data_access::models::*;
use crate::data_access::query_builder::{
    ClickHouseBacktestQueryBuilder, ClickHouseCandleQueryBuilder, ClickHouseQueryBuilder,
    ClickHouseTradeQueryBuilder, ClickHouseUtils,
};
use crate::data_access::traits::{DataSource, Database};
use chrono::{Duration, NaiveDate, Utc};
use std::time::Duration as StdDuration;

/// Пример базового использования ClickHouse коннектора
pub async fn basic_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Пример базового использования ClickHouse ===");

    // Создание коннектора
    let mut connector =
        ClickHouseConnector::new("localhost".to_string(), 9000, "trading".to_string())
            .with_auth("default".to_string(), "".to_string())
            .with_timeouts(StdDuration::from_secs(30), StdDuration::from_secs(300));

    // Подключение
    connector.connect().await?;
    println!("✅ Подключение к ClickHouse установлено");

    // Примечание: Таблицы создаются миграциями
    println!("ℹ️  Таблицы создаются миграциями из migrations/clickhouse/");

    // Проверка подключения
    connector.ping().await?;
    println!("✅ Ping успешен");

    // Отключение
    connector.disconnect().await?;
    println!("✅ Отключение от ClickHouse");

    Ok(())
}

/// Пример работы с OHLCV данными (свечи)
pub async fn candle_data_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Пример работы с OHLCV данными ===");

    let mut connector =
        ClickHouseConnector::new("localhost".to_string(), 9000, "trading".to_string());

    connector.connect().await?;

    // Создание тестовых OHLCV данных
    let ohlcv_data = vec![
        OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now() - Duration::hours(2),
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 1000.0,
        },
        OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now() - Duration::hours(1),
            open: 50500.0,
            high: 51500.0,
            low: 50000.0,
            close: 51200.0,
            volume: 1200.0,
        },
        OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now(),
            open: 51200.0,
            high: 52000.0,
            low: 50800.0,
            close: 51800.0,
            volume: 1500.0,
        },
    ];

    // Вставка данных
    let inserted = connector.insert_ohlcv(&ohlcv_data).await?;
    println!("✅ Вставлено {} свечей", inserted);

    // Получение данных за последние 24 часа
    let start_time = Utc::now() - Duration::days(1);
    let end_time = Utc::now();
    let retrieved_data = connector
        .get_ohlcv("BTCUSDT", "1h", start_time, end_time, Some(10))
        .await?;
    println!("✅ Получено {} свечей", retrieved_data.len());

    for data in &retrieved_data {
        println!(
            "  📊 {}: O={}, H={}, L={}, C={}, V={}",
            data.timestamp.format("%Y-%m-%d %H:%M:%S"),
            data.open,
            data.high,
            data.low,
            data.close,
            data.volume
        );
    }

    connector.disconnect().await?;
    Ok(())
}

/// Пример работы с торговыми сделками
pub async fn trade_data_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Пример работы с торговыми сделками ===");

    let mut connector =
        ClickHouseConnector::new("localhost".to_string(), 9000, "trading".to_string());

    connector.connect().await?;

    // Создание тестовых данных сделок
    let trades = vec![
        TradeRecord {
            trade_id: "trade_1".to_string(),
            strategy_id: "strategy_1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "buy".to_string(),
            quantity: 0.1,
            entry_price: 51000.0,
            exit_price: None,
            entry_time: Utc::now() - Duration::minutes(30),
            exit_time: None,
            pnl: None,
            commission: 5.1,
            status: "open".to_string(),
            metadata: "{}".to_string(),
        },
        TradeRecord {
            trade_id: "trade_2".to_string(),
            strategy_id: "strategy_1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "sell".to_string(),
            quantity: 0.05,
            entry_price: 50000.0,
            exit_price: Some(51500.0),
            entry_time: Utc::now() - Duration::hours(1),
            exit_time: Some(Utc::now() - Duration::minutes(15)),
            pnl: Some(75.0),
            commission: 2.5,
            status: "closed".to_string(),
            metadata: "{}".to_string(),
        },
    ];

    // Вставка данных
    let inserted = connector.insert_trades(&trades).await?;
    println!("✅ Вставлено {} сделок", inserted);

    // Получение сделок по стратегии и символу
    let retrieved_trades = connector
        .get_trades(
            Some("strategy_1"),
            Some("BTCUSDT"),
            Some(Utc::now() - Duration::hours(2)),
            Some(Utc::now()),
            Some("closed"),
            Some(10),
        )
        .await?;

    println!("✅ Получено {} сделок", retrieved_trades.len());

    for trade in &retrieved_trades {
        println!(
            "  💰 {}: {} {} @ {} (PnL: {:?}, Status: {})",
            trade.entry_time.format("%Y-%m-%d %H:%M:%S"),
            trade.side,
            trade.quantity,
            trade.entry_price,
            trade.pnl,
            trade.status
        );
    }

    connector.disconnect().await?;
    Ok(())
}

/// Пример работы с результатами бэктестов
pub async fn backtest_results_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Пример работы с результатами бэктестов ===");

    let mut connector =
        ClickHouseConnector::new("localhost".to_string(), 9000, "trading".to_string());

    connector.connect().await?;

    // Создание тестовых результатов бэктеста
    let backtest_result = BacktestRecord {
        backtest_id: "bt_1".to_string(),
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

    // Вставка результата
    let inserted = connector.insert_backtest_result(&backtest_result).await?;
    println!("✅ Вставлен результат бэктеста ({} записей)", inserted);

    // Получение результатов по стратегии
    let results = connector
        .get_backtest_results(Some("strategy_1"), Some("BTCUSDT"), Some("1h"), Some(5))
        .await?;

    println!("✅ Получено {} результатов бэктестов", results.len());

    for result in &results {
        println!(
            "  📈 Strategy {}: PnL={}, Sharpe={}, DD={}%, Trades={}",
            result.strategy_id,
            result.total_pnl,
            result.sharpe_ratio,
            result.max_drawdown,
            result.total_trades
        );
    }

    connector.disconnect().await?;
    Ok(())
}

/// Пример использования Query Builder
pub async fn query_builder_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Пример использования Query Builder ===");

    // Базовый Query Builder
    let query = ClickHouseQueryBuilder::new()
        .table("trading.ohlcv_data")
        .select(&["timestamp", "symbol", "timeframe", "close", "volume"])
        .where_eq("symbol", "'BTCUSDT'")
        .where_eq("timeframe", "'1h'")
        .where_gte("timestamp", "'2024-01-01 00:00:00'")
        .order_by_desc("timestamp")
        .limit(100)
        .build()?;

    println!("📝 Сгенерированный SQL запрос:");
    println!("{}", query);

    // Специализированный Candle Query Builder
    let candle_query = ClickHouseCandleQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .last_days(7)
        .latest(50)
        .build()?;

    println!("\n📊 Запрос для свечей:");
    println!("{}", candle_query);

    // Специализированный Trade Query Builder
    let trade_query = ClickHouseTradeQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .by_side(TradeSide::Buy)
        .time_range(Utc::now() - Duration::days(1), Utc::now())
        .price_range(50000.0, 52000.0)
        .order_by_time_desc()
        .limit(20)
        .build()?;

    println!("\n💰 Запрос для сделок:");
    println!("{}", trade_query);

    // Специализированный Backtest Query Builder
    let backtest_query = ClickHouseBacktestQueryBuilder::new()
        .by_strategy("strategy_1")
        .min_return(10.0)
        .min_sharpe(1.5)
        .max_drawdown(-10.0)
        .order_by_sharpe_desc()
        .limit(10)
        .build()?;

    println!("\n📈 Запрос для результатов бэктестов:");
    println!("{}", backtest_query);

    Ok(())
}

/// Пример аналитических запросов
pub async fn analytics_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Пример аналитических запросов ===");

    let mut connector =
        ClickHouseConnector::new("localhost".to_string(), 9000, "trading".to_string());

    connector.connect().await?;

    // Статистика по символу
    let stats = connector.get_symbol_stats("BTCUSDT", "1h").await?;
    println!("📊 Статистика по символу BTCUSDT:");
    for (key, value) in &stats {
        println!("  {}: {}", key, value);
    }

    // Статистика стратегии
    let strategy_stats = connector.get_strategy_stats("strategy_1").await?;
    println!("\n🏆 Статистика стратегии:");
    for (key, value) in &strategy_stats {
        println!("  {}: {}", key, value);
    }

    // Примеры аналитических запросов через Utils
    let volatility_query = ClickHouseUtils::volatility_analysis_query("BTCUSDT", "1h", 30);
    println!("\n📈 Запрос анализа волатильности:");
    println!("{}", volatility_query);

    let correlation_query = ClickHouseUtils::correlation_query("BTCUSDT", "ETHUSDT", "1h", 30);
    println!("\n🔗 Запрос корреляционного анализа:");
    println!("{}", correlation_query);

    connector.disconnect().await?;
    Ok(())
}

/// Пример работы с транзакциями (заглушка для ClickHouse)
pub async fn transaction_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Пример работы с транзакциями ===");

    let mut connector =
        ClickHouseConnector::new("localhost".to_string(), 9000, "trading".to_string());

    connector.connect().await?;

    // ClickHouse не поддерживает традиционные транзакции
    // Но мы можем использовать "транзакцию" для группировки операций
    let transaction = connector.begin_transaction().await?;

    // Выполнение операций в "транзакции"
    transaction
        .execute("INSERT INTO test_table (id, value) VALUES (1, 'test')")
        .await?;

    // Подтверждение (в ClickHouse это просто освобождение ресурсов)
    transaction.commit().await?;

    println!("✅ Операции в 'транзакции' выполнены");

    connector.disconnect().await?;
    Ok(())
}

/// Запуск всех примеров
pub async fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск примеров ClickHouse коннектора\n");

    basic_usage_example().await?;
    candle_data_example().await?;
    trade_data_example().await?;
    backtest_results_example().await?;
    query_builder_example().await?;
    analytics_example().await?;
    transaction_example().await?;

    println!("\n✅ Все примеры выполнены успешно!");
    Ok(())
}
