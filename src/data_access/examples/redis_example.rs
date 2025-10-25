//! Пример использования Redis коннектора

use crate::data_access::{
    database::RedisConnector,
    query_builder::{CacheQueryBuilder, QueueQueryBuilder, CounterQueryBuilder, KeyUtils},
    models::*,
    Cache, DataSource,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Запуск примера Redis коннектора");
    
    // Создание Redis коннектора
    let mut redis = RedisConnector::new("localhost".to_string(), 6379);
    
    // Подключение
    println!("📡 Подключение к Redis...");
    redis.connect().await?;
    println!("✅ Подключение успешно!");
    
    // Пример 1: Кэширование цены
    println!("\n📊 Пример 1: Кэширование цены");
    let ticker = Ticker {
        symbol: "BTCUSDT".to_string(),
        price: 45000.0,
        bid: 44999.0,
        ask: 45001.0,
        volume: 1000.0,
        timestamp: chrono::Utc::now(),
    };
    
    let price_key = KeyUtils::price_key("BTCUSDT");
    redis.set(&price_key, &ticker, Some(60)).await?;
    println!("💾 Цена BTCUSDT сохранена в кэш");
    
    let cached_ticker: Option<Ticker> = redis.get(&price_key).await?;
    if let Some(ticker) = cached_ticker {
        println!("📈 Получена цена: ${}", ticker.price);
    }
    
    // Пример 2: Работа с очередями
    println!("\n📨 Пример 2: Работа с очередями");
    let command_queue = KeyUtils::command_queue_key("rsi_strategy");
    
    // Добавление команд в очередь
    redis.lpush(&command_queue, &"BUY:BTCUSDT:100").await?;
    redis.lpush(&command_queue, &"SELL:ETHUSDT:50").await?;
    redis.lpush(&command_queue, &"HOLD:ADAUSDT:0").await?;
    
    println!("📝 Добавлено 3 команды в очередь");
    
    // Получение размера очереди
    let queue_size = redis.llen(&command_queue).await?;
    println!("📊 Размер очереди: {}", queue_size);
    
    // Получение всех команд
    let commands: Vec<String> = redis.lrange(&command_queue, 0, -1).await?;
    println!("📋 Все команды: {:?}", commands);
    
    // Получение команды из очереди
    let command: Option<String> = redis.rpop(&command_queue).await?;
    if let Some(cmd) = command {
        println!("🎯 Обработана команда: {}", cmd);
    }
    
    // Пример 3: Работа со счетчиками
    println!("\n🔢 Пример 3: Работа со счетчиками");
    let trades_counter = KeyUtils::counter_key("trades");
    
    // Инкремент счетчика
    let count1 = redis.increment(&trades_counter).await?;
    let count2 = redis.increment(&trades_counter).await?;
    let count3 = redis.increment(&trades_counter).await?;
    
    println!("📈 Счетчик сделок: {} -> {} -> {}", count1, count2, count3);
    
    // Пример 4: Кэширование торгового сигнала
    println!("\n📊 Пример 4: Кэширование торгового сигнала");
    let signal = TradingSignal {
        id: "signal_001".to_string(),
        strategy_id: "rsi_strategy".to_string(),
        symbol: "BTCUSDT".to_string(),
        signal_type: SignalType::Buy,
        confidence: 0.85,
        price: 45000.0,
        timestamp: chrono::Utc::now(),
        metadata: Some(serde_json::json!({
            "rsi_value": 25.5,
            "volume": 100.5
        })),
    };
    
    let signal_key = KeyUtils::signal_key("rsi_strategy", "BTCUSDT");
    redis.set(&signal_key, &signal, Some(300)).await?; // TTL 5 минут
    println!("💾 Торговый сигнал сохранен");
    
    let cached_signal: Option<TradingSignal> = redis.get(&signal_key).await?;
    if let Some(signal) = cached_signal {
        println!("📈 Получен сигнал: {:?} с уверенностью {:.2}", 
                signal.signal_type, signal.confidence);
    }
    
    // Пример 5: Использование Query Builder
    println!("\n🔧 Пример 5: Использование Query Builder");
    
    // Cache Query Builder
    let cache_query = CacheQueryBuilder::new()
        .cache("user:123", &"John Doe", Some(3600))
        .get_cached("user:123")
        .exists_cached("user:123");
    
    println!("📋 Cache Query Builder создан с {} операциями", cache_query.operations().len());
    
    // Queue Query Builder
    let queue_query = QueueQueryBuilder::new()
        .enqueue("notifications", &"Новый сигнал!")
        .enqueue("notifications", &"Ордер исполнен!")
        .size("notifications");
    
    println!("📋 Queue Query Builder создан с {} операциями", queue_query.operations().len());
    
    // Counter Query Builder
    let counter_query = CounterQueryBuilder::new()
        .increment("api_calls")
        .increment("api_calls")
        .get_value("api_calls");
    
    println!("📋 Counter Query Builder создан с {} операциями", counter_query.operations().len());
    
    // Пример 6: Работа с сессиями
    println!("\n👤 Пример 6: Работа с сессиями");
    let session_key = KeyUtils::session_key("user123");
    let session_data = serde_json::json!({
        "user_id": "user123",
        "username": "trader",
        "last_login": chrono::Utc::now(),
        "permissions": ["read", "write", "trade"]
    });
    
    redis.set(&session_key, &session_data, Some(3600)).await?; // TTL 1 час
    println!("🔐 Сессия пользователя сохранена");
    
    let cached_session: Option<serde_json::Value> = redis.get(&session_key).await?;
    if let Some(session) = cached_session {
        println!("👤 Получена сессия: {}", session["username"]);
    }
    
    // Пример 7: Блокировки
    println!("\n🔒 Пример 7: Блокировки");
    let lock_key = KeyUtils::lock_key("strategy_1");
    
    // Попытка установить блокировку
    let lock_set = redis.set_string(&lock_key, "locked", Some(10)).await?;
    println!("🔒 Блокировка установлена: {:?}", lock_set);
    
    // Проверка блокировки
    let is_locked = redis.exists(&lock_key).await?;
    println!("🔍 Блокировка активна: {}", is_locked);
    
    // Отключение
    println!("\n🔌 Отключение от Redis...");
    redis.disconnect().await?;
    println!("✅ Отключение успешно!");
    
    println!("\n🎉 Все примеры выполнены успешно!");
    Ok(())
}
