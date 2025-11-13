//! Примеры использования Arrow/Parquet коннекторов

use crate::data_access::database::*;
use crate::data_access::models::*;
use crate::data_access::query_builder::*;
use crate::data_access::traits::DataSource;
use crate::data_access::{DataAccessError, Result};

/// Пример использования Arrow Flight коннектора
pub async fn arrow_flight_example() -> Result<()> {
    println!("=== Arrow Flight Example ===");

    // Создание конфигурации
    let config = ArrowFlightConfig {
        host: "localhost".to_string(),
        port: 8815,
        timeout_seconds: 30,
        max_retries: 3,
        batch_size: 1000,
        compression_enabled: true,
    };

    // Создание коннектора
    let mut connector = ArrowFlightConnector::new(config);

    // Подключение
    connector.connect().await?;
    println!("✅ Подключен к Arrow Flight Server");

    // Получение данных
    let query = ArrowFlightUtils::create_candles_query("BTCUSDT", "2024-01-01", "2024-01-31");

    let batches = connector.get_data(&query).await?;
    println!("📊 Получено {} батчей данных", batches.len());

    // Получение схемы
    let schema = connector.get_schema(&query).await?;
    println!("📋 Схема данных: {}", schema);

    // Отключение
    connector.disconnect().await?;
    println!("🔌 Отключен от Arrow Flight Server");

    Ok(())
}

/// Пример использования Parquet коннектора
pub async fn parquet_example() -> Result<()> {
    println!("=== Parquet Example ===");

    // Создание конфигурации
    let config = ParquetConfig {
        base_path: "./data/parquet".to_string(),
        compression: ParquetCompression::Snappy,
        batch_size: 1000,
        max_file_size: 100 * 1024 * 1024, // 100MB
        create_directories: true,
    };

    // Создание коннектора
    let mut connector = ParquetConnector::new(config);

    // Подключение
    connector.connect().await?;
    println!("✅ Подключен к Parquet хранилищу");

    // Создание тестовых данных
    let test_candles = vec![
        Candle {
            timestamp: chrono::Utc::now(),
            symbol: "BTCUSDT".to_string(),
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 1000.0,
        },
        Candle {
            timestamp: chrono::Utc::now(),
            symbol: "ETHUSDT".to_string(),
            open: 3000.0,
            high: 3100.0,
            low: 2900.0,
            close: 3050.0,
            volume: 2000.0,
        },
    ];

    // Сохранение данных
    let file_path = ParquetUtils::create_candles_path("BTCUSDT", "2024-01-01");

    // Конвертируем Vec<Candle> в RecordBatch (упрощенная версия)
    let batches = vec![]; // В реальности нужна конвертация

    connector.write_parquet(&file_path, batches).await?;
    println!("💾 Данные сохранены в {}", file_path);

    // Чтение данных
    let read_batches = connector.read_parquet(&file_path).await?;
    println!("📖 Прочитано {} батчей из файла", read_batches.len());

    // Получение метаданных
    let metadata = connector.get_metadata(&file_path).await?;
    println!("📊 Метаданные файла:");
    println!("   - Путь: {}", metadata.file_path);
    println!("   - Размер: {} байт", metadata.file_size);
    println!("   - Строк: {}", metadata.num_rows);
    println!("   - Колонок: {}", metadata.num_columns);

    // Список файлов
    let files = connector.list_files("candles").await?;
    println!("📁 Найдено {} файлов в директории candles", files.len());

    // Отключение
    connector.disconnect().await?;
    println!("🔌 Отключен от Parquet хранилища");

    Ok(())
}

/// Пример использования DataFusion коннектора
pub async fn datafusion_example() -> Result<()> {
    println!("=== DataFusion Example ===");

    // Создание конфигурации
    let config = DataFusionConfig {
        memory_limit: 1024 * 1024 * 1024, // 1GB
        max_concurrent_queries: 10,
        enable_optimization: true,
        enable_parallel_execution: true,
        cache_size: 100,
        temp_dir: Some("./temp".to_string()),
    };

    // Создание коннектора
    let mut connector = DataFusionConnector::new(config);

    // Подключение
    connector.connect().await?;
    println!("✅ Подключен к DataFusion");

    // Регистрация таблицы из Parquet файла
    let table_name = "candles";
    let file_path = "./data/parquet/candles/BTCUSDT/2024-01-01.parquet";

    connector
        .register_parquet_table(table_name, file_path)
        .await?;
    println!("📋 Таблица {} зарегистрирована", table_name);

    // Выполнение SQL запроса
    let sql =
        "SELECT symbol, AVG(close) as avg_price, COUNT(*) as count FROM candles GROUP BY symbol";
    let batches = connector.execute_sql(sql).await?;
    println!("📊 Результат SQL запроса: {} батчей", batches.len());

    // Создание аналитического запроса
    let analytics_query = DataFusionUtils::create_candles_analysis_query(
        "BTCUSDT",
        "2024-01-01",
        "2024-01-31",
        &["AVG(close)", "MAX(high)", "MIN(low)"],
    );

    let analytics_batches = connector.execute_analytics_query(&analytics_query).await?;
    println!(
        "📈 Результат аналитического запроса: {} батчей",
        analytics_batches.len()
    );

    // Получение статистики таблицы
    let stats = connector.get_table_stats(table_name).await?;
    println!("📊 Статистика таблицы {}:", stats.table_name);
    println!("   - Строк: {}", stats.row_count);
    println!("   - Колонок: {}", stats.column_count);

    // Получение схемы таблицы
    let schema = connector.get_table_schema(table_name).await?;
    println!("📋 Схема таблицы: {} полей", schema.fields().len());

    // Отключение
    connector.disconnect().await?;
    println!("🔌 Отключен от DataFusion");

    Ok(())
}

/// Пример использования Arrow Query Builder
pub async fn arrow_query_builder_example() -> Result<()> {
    println!("=== Arrow Query Builder Example ===");

    // Базовый Query Builder
    let query = ArrowQueryBuilder::new("candles")
        .select(&["timestamp", "symbol", "close"])
        .where_equal("symbol", FilterValue::String("BTCUSDT".to_string()))
        .where_greater_than("close", FilterValue::Number(50000.0))
        .order_by("timestamp", SortDirection::Asc)
        .limit(100)
        .build()?;

    println!("📝 Базовый запрос: {}", query);

    // Query Builder с агрегациями
    let agg_query = ArrowQueryBuilder::new("candles")
        .avg("close", Some("avg_close"))
        .max("high", Some("max_high"))
        .min("low", Some("min_low"))
        .count("timestamp", Some("count"))
        .group_by(&["symbol"])
        .order_by("avg_close", SortDirection::Desc)
        .build()?;

    println!("📊 Запрос с агрегациями: {}", agg_query);

    // Специализированный Query Builder для свечей
    let candle_query = CandleArrowQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .by_time_range("2024-01-01", "2024-01-31")
        .ohlcv()
        .order_by_time(SortDirection::Asc)
        .build()?;

    println!("🕯️ Запрос свечей: {}", candle_query);

    // Специализированный Query Builder для сделок
    let trade_query = TradeArrowQueryBuilder::new()
        .by_symbol("BTCUSDT")
        .by_side("Buy")
        .recent(50)
        .build()?;

    println!("💰 Запрос сделок: {}", trade_query);

    // Специализированный Query Builder для результатов бэктестов
    let backtest_query = BacktestArrowQueryBuilder::new()
        .by_strategy("strategy_1")
        .min_sharpe(1.0)
        .top_results(10)
        .build()?;

    println!("📈 Запрос результатов бэктестов: {}", backtest_query);

    // Утилиты для создания запросов
    let utils_query = ArrowQueryUtils::create_candles_analysis_query(
        "BTCUSDT",
        "2024-01-01",
        "2024-01-31",
        &["AVG(close)", "MAX(high)", "MIN(low)"],
    )?;

    println!("🔧 Запрос через утилиты: {}", utils_query);

    Ok(())
}

/// Пример комплексного использования Arrow/Parquet инфраструктуры
pub async fn comprehensive_example() -> Result<()> {
    println!("=== Comprehensive Arrow/Parquet Example ===");

    // 1. Создание Parquet файлов с историческими данными
    println!("📁 Создание Parquet файлов...");

    let parquet_config = ParquetConfig::default();
    let mut parquet_connector = ParquetConnector::new(parquet_config);
    parquet_connector.connect().await?;

    // Создание тестовых данных для разных символов
    let symbols = vec!["BTCUSDT", "ETHUSDT", "ADAUSDT"];
    let dates = vec!["2024-01-01", "2024-01-02", "2024-01-03"];

    for symbol in &symbols {
        for date in &dates {
            let file_path = ParquetUtils::create_candles_path(symbol, date);

            // Создание тестовых свечей
            let test_candles = vec![Candle {
                timestamp: chrono::Utc::now(),
                symbol: symbol.to_string(),
                open: 50000.0,
                high: 51000.0,
                low: 49000.0,
                close: 50500.0,
                volume: 1000.0,
            }];

            // Конвертация в RecordBatch (упрощенная версия)
            let batches = vec![];

            parquet_connector.write_parquet(&file_path, batches).await?;
            println!("✅ Создан файл: {}", file_path);
        }
    }

    // 2. Настройка DataFusion для аналитики
    println!("🔍 Настройка DataFusion для аналитики...");

    let datafusion_config = DataFusionConfig::default();
    let mut datafusion_connector = DataFusionConnector::new(datafusion_config);
    datafusion_connector.connect().await?;

    // Регистрация всех таблиц
    for symbol in &symbols {
        for date in &dates {
            let table_name = format!("candles_{}_{}", symbol, date.replace("-", "_"));
            let file_path = ParquetUtils::create_candles_path(symbol, date);

            datafusion_connector
                .register_parquet_table(&table_name, &file_path)
                .await?;
            println!("📋 Зарегистрирована таблица: {}", table_name);
        }
    }

    // 3. Выполнение аналитических запросов
    println!("📊 Выполнение аналитических запросов...");

    // Анализ по символам
    for symbol in &symbols {
        let sql = format!(
            "SELECT '{}' as symbol, AVG(close) as avg_price, MAX(high) as max_high, MIN(low) as min_low FROM candles_{}_2024_01_01",
            symbol, symbol
        );

        let batches = datafusion_connector.execute_sql(&sql).await?;
        println!("📈 Анализ {}: {} батчей", symbol, batches.len());
    }

    // Сравнительный анализ
    let comparison_sql = "
        SELECT 
            symbol,
            AVG(close) as avg_price,
            STDDEV(close) as price_volatility,
            COUNT(*) as data_points
        FROM (
            SELECT 'BTCUSDT' as symbol, close FROM candles_BTCUSDT_2024_01_01
            UNION ALL
            SELECT 'ETHUSDT' as symbol, close FROM candles_ETHUSDT_2024_01_01
            UNION ALL
            SELECT 'ADAUSDT' as symbol, close FROM candles_ADAUSDT_2024_01_01
        ) combined
        GROUP BY symbol
        ORDER BY avg_price DESC
    ";

    let comparison_batches = datafusion_connector.execute_sql(comparison_sql).await?;
    println!(
        "🔄 Сравнительный анализ: {} батчей",
        comparison_batches.len()
    );

    // 4. Использование Arrow Flight для передачи данных
    println!("🚀 Использование Arrow Flight для передачи данных...");

    let arrow_config = ArrowFlightConfig::default();
    let mut arrow_connector = ArrowFlightConnector::new(arrow_config);
    arrow_connector.connect().await?;

    // Получение данных через Arrow Flight
    let flight_query = "SELECT * FROM candles_BTCUSDT_2024_01_01 LIMIT 10";
    let flight_batches = arrow_connector.get_data(flight_query).await?;
    println!(
        "✈️ Получено {} батчей через Arrow Flight",
        flight_batches.len()
    );

    // 5. Очистка
    println!("🧹 Очистка ресурсов...");

    arrow_connector.disconnect().await?;
    datafusion_connector.disconnect().await?;
    parquet_connector.disconnect().await?;

    println!("✅ Комплексный пример завершен успешно!");

    Ok(())
}

/// Пример использования специализированных коннекторов
pub async fn specialized_connectors_example() -> Result<()> {
    println!("=== Specialized Connectors Example ===");

    // Candle Parquet Connector
    println!("🕯️ Candle Parquet Connector...");
    let candle_parquet_config = ParquetConfig::default();
    let mut candle_parquet = CandleParquetConnector::new(candle_parquet_config);
    candle_parquet.base_connector.connect().await?;

    // Сохранение свечей
    let test_candles = vec![Candle {
        timestamp: chrono::Utc::now(),
        symbol: "BTCUSDT".to_string(),
        open: 50000.0,
        high: 51000.0,
        low: 49000.0,
        close: 50500.0,
        volume: 1000.0,
    }];

    candle_parquet
        .save_candles("BTCUSDT", "2024-01-01", test_candles)
        .await?;
    println!("💾 Свечи сохранены");

    // Загрузка свечей
    let loaded_candles = candle_parquet.load_candles("BTCUSDT", "2024-01-01").await?;
    println!("📖 Загружено {} свечей", loaded_candles.len());

    // Получение доступных дат
    let available_dates = candle_parquet.get_available_dates("BTCUSDT").await?;
    println!("📅 Доступные даты: {:?}", available_dates);

    // Candle Analytics Connector
    println!("📊 Candle Analytics Connector...");
    let candle_analytics_config = DataFusionConfig::default();
    let mut candle_analytics = CandleAnalyticsConnector::new(candle_analytics_config);
    candle_analytics.base_connector.connect().await?;

    // Регистрация таблицы
    candle_analytics
        .register_candles_table(
            "candles",
            "./data/parquet/candles/BTCUSDT/2024-01-01.parquet",
        )
        .await?;

    // Анализ свечей
    let analysis_batches = candle_analytics
        .analyze_candles(
            "BTCUSDT",
            "2024-01-01",
            "2024-01-31",
            &["AVG(close)", "MAX(high)", "MIN(low)"],
        )
        .await?;
    println!("📈 Результат анализа: {} батчей", analysis_batches.len());

    // Расчет скользящих средних
    let ma_batches = candle_analytics
        .calculate_moving_averages("BTCUSDT", &[5, 10, 20])
        .await?;
    println!("📊 Скользящие средние: {} батчей", ma_batches.len());

    // Поиск свечных паттернов
    let pattern_batches = candle_analytics
        .find_candle_patterns("BTCUSDT", "doji", 10)
        .await?;
    println!("🔍 Найдено паттернов: {} батчей", pattern_batches.len());

    // Backtest Analytics Connector
    println!("📈 Backtest Analytics Connector...");
    let backtest_analytics_config = DataFusionConfig::default();
    let mut backtest_analytics = BacktestAnalyticsConnector::new(backtest_analytics_config);
    backtest_analytics.base_connector.connect().await?;

    // Регистрация таблицы результатов бэктестов
    backtest_analytics
        .register_backtest_table(
            "backtest_results",
            "./data/parquet/backtests/strategy_1/2024-01-01.parquet",
        )
        .await?;

    // Анализ производительности стратегии
    let performance_batches = backtest_analytics
        .analyze_strategy_performance("strategy_1")
        .await?;
    println!(
        "📊 Анализ производительности: {} батчей",
        performance_batches.len()
    );

    // Сравнение стратегий
    let comparison_batches = backtest_analytics
        .compare_strategies(&["strategy_1", "strategy_2"])
        .await?;
    println!(
        "🔄 Сравнение стратегий: {} батчей",
        comparison_batches.len()
    );

    // Очистка
    candle_parquet.base_connector.disconnect().await?;
    candle_analytics.base_connector.disconnect().await?;
    backtest_analytics.base_connector.disconnect().await?;

    println!("✅ Пример специализированных коннекторов завершен!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_arrow_query_builder_example() {
        let result = arrow_query_builder_example().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires local Parquet dataset"]
    async fn test_parquet_example() {
        let result = parquet_example().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires local Parquet dataset"]
    async fn test_datafusion_example() {
        let result = datafusion_example().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires local Parquet dataset"]
    async fn test_comprehensive_example() {
        let result = comprehensive_example().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires local Parquet dataset"]
    async fn test_specialized_connectors_example() {
        let result = specialized_connectors_example().await;
        assert!(result.is_ok());
    }
}
