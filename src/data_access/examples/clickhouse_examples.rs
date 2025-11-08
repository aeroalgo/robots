//! Примеры использования ClickHouse репозитория с реальной интеграцией
//!
//! Демонстрирует основные операции CRUD с реальным ClickHouse драйвером

use crate::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector, OhlcvData};
use crate::data_access::{DataSource, Database};
use chrono::Utc;

/// Пример подключения к ClickHouse
#[allow(dead_code)]
pub async fn example_connect() -> Result<ClickHouseConnector, Box<dyn std::error::Error>> {
    let config = ClickHouseConfig {
        host: "localhost".to_string(),
        port: 9000,
        database: "trading".to_string(),
        username: Some("default".to_string()),
        password: None,
        ..Default::default()
    };

    let mut connector = ClickHouseConnector::with_config(config);
    connector.connect().await?;

    println!("✅ Подключено к ClickHouse");
    println!(
        "📊 База данных: {}",
        connector.connection_info().database.unwrap()
    );

    Ok(connector)
}

/// Пример вставки OHLCV данных
#[allow(dead_code)]
pub async fn example_insert_ohlcv() -> Result<(), Box<dyn std::error::Error>> {
    let mut connector = example_connect().await?;

    let data = vec![
        OhlcvData {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now(),
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 1234.56,
        },
        OhlcvData {
            symbol: "ETHUSDT".to_string(),
            timeframe: "1h".to_string(),
            timestamp: Utc::now(),
            open: 3000.0,
            high: 3100.0,
            low: 2950.0,
            close: 3050.0,
            volume: 5678.90,
        },
    ];

    let count = connector.insert_ohlcv(&data).await?;
    println!("✅ Вставлено {} записей OHLCV", count);

    connector.disconnect().await?;
    Ok(())
}

/// Пример запроса OHLCV данных
#[allow(dead_code)]
pub async fn example_query_ohlcv() -> Result<(), Box<dyn std::error::Error>> {
    let mut connector = example_connect().await?;

    let start_time = Utc::now() - chrono::Duration::hours(24);
    let end_time = Utc::now();

    let data = connector
        .get_ohlcv("BTCUSDT", "1h", start_time, end_time, Some(100))
        .await?;

    println!("✅ Получено {} свечей OHLCV", data.len());

    for (i, candle) in data.iter().take(5).enumerate() {
        println!(
            "  {}. {} | O: {} H: {} L: {} C: {} V: {}",
            i + 1,
            candle.timestamp.format("%Y-%m-%d %H:%M"),
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume
        );
    }

    connector.disconnect().await?;
    Ok(())
}

/// Пример проверки подключения
#[allow(dead_code)]
pub async fn example_ping() -> Result<(), Box<dyn std::error::Error>> {
    let mut connector = example_connect().await?;

    connector.ping().await?;
    println!("✅ Ping успешен - ClickHouse отвечает");

    connector.disconnect().await?;
    Ok(())
}

/// Демонстрация всех примеров
#[allow(dead_code)]
pub async fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск примеров ClickHouse драйвера\n");

    println!("1️⃣ Пример подключения:");
    example_connect().await?;
    println!();

    println!("2️⃣ Пример ping:");
    example_ping().await?;
    println!();

    println!("3️⃣ Пример вставки данных:");
    example_insert_ohlcv().await?;
    println!();

    println!("4️⃣ Пример запроса данных:");
    example_query_ohlcv().await?;
    println!();

    println!("✅ Все примеры выполнены успешно!");
    Ok(())
}
