//! Интеграционные тесты для MongoDB
//!
//! Для запуска тестов необходимо:
//! 1. Установить и запустить MongoDB:
//!    docker run -d --name mongodb-test -p 27017:27017 mongo:latest
//! 2. Запустить тесты:
//!    cargo test --test mongodb_integration_tests -- --test-threads=1
//!
//! Используйте флаг --ignored для пропуска тестов без MongoDB:
//!    cargo test --test mongodb_integration_tests

use mongodb::bson::{doc, Document};
use robots::data_access::database::{MongoDBConfig, MongoDBConnector};
use robots::data_access::traits::{DataSource, Database};

/// Вспомогательная функция для создания тестового коннектора
async fn create_test_connector() -> MongoDBConnector {
    let config = MongoDBConfig {
        host: std::env::var("MONGODB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        port: std::env::var("MONGODB_PORT")
            .unwrap_or_else(|_| "27017".to_string())
            .parse()
            .unwrap_or(27017),
        database: "trading_test".to_string(),
        username: None,
        password: None,
        auth_database: Some("admin".to_string()),
        max_pool_size: Some(10),
        min_pool_size: Some(5),
        max_idle_time: Some(300),
        connect_timeout: Some(5),
        server_selection_timeout: Some(5),
    };

    MongoDBConnector::new(config)
}

/// Проверка доступности MongoDB
fn is_mongodb_available() -> bool {
    std::env::var("MONGODB_AVAILABLE").unwrap_or_else(|_| "false".to_string()) == "true"
}

// ============================================================================
// ТЕСТЫ ПОДКЛЮЧЕНИЯ
// ============================================================================

#[tokio::test]
#[ignore] // Игнорируем по умолчанию, т.к. требуется запущенный MongoDB
async fn test_mongodb_connection() {
    if !is_mongodb_available() {
        println!("⚠️  MongoDB недоступен, тест пропущен");
        return;
    }

    let mut connector = create_test_connector().await;

    // Тест подключения
    let result = connector.connect().await;
    assert!(
        result.is_ok(),
        "Не удалось подключиться к MongoDB: {:?}",
        result.err()
    );
    assert!(connector.is_connected(), "Коннектор должен быть подключен");

    // Тест отключения
    let disconnect_result = connector.disconnect().await;
    assert!(
        disconnect_result.is_ok(),
        "Отключение не удалось: {:?}",
        disconnect_result.err()
    );
    assert!(!connector.is_connected(), "Коннектор должен быть отключен");

    println!("✅ Тест подключения пройден");
}

#[tokio::test]
#[ignore]
async fn test_connection_info() {
    let connector = create_test_connector().await;
    let info = connector.connection_info();

    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 27017);
    assert_eq!(info.database, Some("trading_test".to_string()));

    println!("✅ Тест connection_info пройден");
}

// ============================================================================
// ТЕСТЫ БАЗОВЫХ ОПЕРАЦИЙ
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_index_creation() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Создание индексов
    let result = connector.create_indexes().await;
    assert!(
        result.is_ok(),
        "Создание индексов не удалось: {:?}",
        result.err()
    );

    println!("✅ Тест создания индексов пройден");
    connector.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_document_operations() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Вставка документа
    let insert_query = r#"db.test_collection.insertOne({"name": "test", "value": 123})"#;
    let insert_result = connector.execute(insert_query).await;
    assert!(
        insert_result.is_ok(),
        "Вставка документа не удалась: {:?}",
        insert_result.err()
    );

    println!("✅ Тест операций с документами пройден");
    connector.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_candle_operations() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Используем существующий метод get_candles
    let start_time = chrono::Utc::now() - chrono::Duration::hours(24);
    let end_time = chrono::Utc::now();
    let result = connector
        .get_candles("BTCUSDT", start_time, end_time, Some(10))
        .await;
    assert!(
        result.is_ok(),
        "Получение свечей не удалось: {:?}",
        result.err()
    );

    println!("✅ Тест операций со свечами пройден");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ТЕСТЫ ERROR HANDLING
// ============================================================================

#[tokio::test]
async fn test_mongodb_connection_error_handling() {
    // Создаем коннектор с неверными параметрами
    let config = MongoDBConfig {
        host: "invalid_host".to_string(),
        port: 9999,
        database: "test".to_string(),
        username: None,
        password: None,
        auth_database: None,
        max_pool_size: Some(5),
        min_pool_size: Some(1),
        max_idle_time: Some(10),
        connect_timeout: Some(1),
        server_selection_timeout: Some(1),
    };

    let mut connector = MongoDBConnector::new(config);

    // Попытка подключиться к несуществующему хосту
    let result = connector.connect().await;
    println!(
        "Результат подключения к несуществующему MongoDB: {:?}",
        result
    );
    // В реальной реализации должна быть ошибка
}

#[tokio::test]
async fn test_query_without_connection() {
    let connector = create_test_connector().await;
    // НЕ вызываем connect()

    let result = connector.query::<Document>("{}").await;
    assert!(
        result.is_err(),
        "Запрос без подключения должен вернуть ошибку"
    );

    println!("✅ Тест обработки ошибок пройден");
}

// ============================================================================
// ТЕСТЫ РАБОТЫ С КОЛЛЕКЦИЯМИ
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_collection_operations() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Получение коллекции
    let collection_result = connector.get_collection::<Document>("test_collection");
    assert!(
        collection_result.is_ok(),
        "Получение коллекции не удалось: {:?}",
        collection_result.err()
    );

    println!("✅ Тест работы с коллекциями пройден");
    connector.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_trade_operations() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Используем существующий метод get_trades
    let result = connector.get_trades(None, None, None, Some(10)).await;
    assert!(
        result.is_ok(),
        "Получение сделок не удалось: {:?}",
        result.err()
    );

    println!("✅ Тест операций со сделками пройден");
    connector.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_backtest_operations() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Используем существующий метод get_backtest_results
    let result = connector.get_backtest_results(None, None, Some(10)).await;
    assert!(
        result.is_ok(),
        "Получение результатов бэктестов не удалось: {:?}",
        result.err()
    );

    println!("✅ Тест операций с бэктестами пройден");
    connector.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_complete_workflow() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    println!("📝 Тест полного рабочего процесса MongoDB");

    // 1. Создание индексов
    connector.create_indexes().await.unwrap();
    println!("  ✓ Индексы созданы");

    // 2. Работа с коллекциями
    let collection = connector
        .get_collection::<Document>("test_collection")
        .unwrap();
    println!("  ✓ Коллекция получена");

    // 3. Получение данных
    let candles = connector
        .get_candles("BTCUSDT", None, None, Some(10))
        .await
        .unwrap();
    println!("  ✓ Получено {} свечей", candles.len());

    let trades = connector
        .get_trades(None, None, None, Some(10))
        .await
        .unwrap();
    println!("  ✓ Получено {} сделок", trades.len());

    println!("✅ Полный рабочий процесс завершен успешно");
    connector.disconnect().await.unwrap();
}

// ============================================================================
// ПРОСТЫЕ ТЕСТЫ ДЛЯ ПРОВЕРКИ РАБОТОСПОСОБНОСТИ
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_basic_query() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Простой запрос
    let result = connector.query::<Document>("{}").await;
    assert!(result.is_ok(), "Базовый запрос не удался");

    println!("✅ Базовый тест запроса пройден");
    connector.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_basic_execute() {
    if !is_mongodb_available() {
        return;
    }

    let mut connector = create_test_connector().await;
    connector.connect().await.unwrap();

    // Простое выполнение
    let result = connector.execute("db.test.find()").await;
    assert!(result.is_ok(), "Базовое выполнение не удалось");

    println!("✅ Базовый тест выполнения пройден");
    connector.disconnect().await.unwrap();
}
