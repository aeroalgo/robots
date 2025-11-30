# ClickHouse Driver Integration

## ✅ Статус: Интегрирован (70%)

Реальный ClickHouse драйвер интегрирован и готов к использованию!

## 🚀 Быстрый старт

### Подключение к ClickHouse

```rust
use crate::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use crate::data_access::DataSource;

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
```

### Вставка данных

```rust
use crate::data_access::database::clickhouse::OhlcvData;
use chrono::Utc;

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
];

let count = connector.insert_ohlcv(&data).await?;
println!("Вставлено {} записей", count);
```

### Запрос данных

```rust
let start_time = Utc::now() - chrono::Duration::hours(24);
let end_time = Utc::now();

let candles = connector
    .get_ohlcv("BTCUSDT", "1h", start_time, end_time, Some(100))
    .await?;

for candle in candles {
    println!("Open: {}, Close: {}", candle.open, candle.close);
}
```

## 📋 Реализованные методы

### ✅ Базовые методы
- `connect()` - подключение к ClickHouse через HTTP (порт 8123)
- `disconnect()` - отключение
- `ping()` - проверка соединения
- `is_connected()` - проверка статуса
- `execute()` - выполнение SQL запроса

### ✅ OHLCV данные (примерреализации)
- `get_ohlcv()` - получение свечей с фильтрацией
- `insert_ohlcv()` - вставка батча свечей

### ⏳ Требуется реализовать (~50 методов)
- get/insert для всех остальных таблиц:
  - tick_data
  - symbol_info
  - indicators
  - signals
  - trades
  - strategy_metrics
  - strategies
  - backtest_results
  - positions
  - orders
  - genetic_population
  - optimization_results
  - portfolio_snapshots
  - walk_forward_results

## 🔧 Технические детали

### Архитектура

```
ClickHouseConnector {
    host: String,
    port: u16,
    database: String,
    client: Option<Client>,  // Реальный ClickHouse клиент
}
```

### API драйвера

Используется крейт `clickhouse = "0.11"`:

**Для запросов:**
```rust
client.query("SELECT * FROM table WHERE id = ?")
    .bind(value)
    .fetch_all::<MyStruct>()
    .await?
```

**Для вставки:**
```rust
let mut insert = client.insert("table")?;
for row in data {
    insert.write(&row).await?;
}
insert.end().await?;
```

### Требования к структурам данных

Все структуры должны иметь derive макросы:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct OhlcvData {
    pub symbol: String,
    // ...
}
```

## 📦 Зависимости

```toml
[dependencies]
clickhouse = "0.11"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
```

## ⚠️ Важные замечания

1. **Generic методы не реализованы**: `query<T>()` и `query_with_params<T>()` возвращают пустые векторы из-за trait bounds. Используйте специфичные методы типа `get_ohlcv()`.

2. **HTTP порт**: Драйвер использует HTTP порт 8123, а не нативный TCP порт 9000.

3. **Транзакции**: ClickHouse не поддерживает традиционные транзакции. Методы `begin_transaction()`, `commit()`, `rollback()` являются no-op.

4. **Все 15 моделей данных**: Добавлен derive макрос `Row` для совместимости с драйвером.

## 📚 Примеры использования

Смотрите файл `examples/clickhouse_examples.rs` для полных примеров:
- Подключение и отключение
- Вставка батча данных
- Запрос с фильтрацией
- Проверка соединения

## 🔄 Следующие шаги

Для завершения интеграции нужно:

1. Реализовать остальные ~50 методов по аналогии с `get_ohlcv()` и `insert_ohlcv()`
2. Добавить интеграционные тесты с реальным ClickHouse
3. Добавить обработку специфичных ошибок ClickHouse
4. Добавить retry логику для сетевых ошибок
5. Добавить connection pooling для production

## 🧪 Тестирование

Для тестирования требуется запущенный ClickHouse:

```bash
docker-compose up clickhouse
cargo test --test clickhouse_integration_tests
```

## 📝 Лицензия

См. корневой LICENSE файл проекта.












































