# Data Access Layer Implementation Task

## 📋 Общая информация
- **Задача**: Реализация data_access слоя для работы с источниками данных
- **Дата создания**: 2024-01-15
- **Статус**: В процессе
- **Приоритет**: Высокий

## 🎯 Цели задачи
Создать удобную архитектуру для работы с различными источниками данных:
- Redis (кэш/сессии)
- Arrow Flight (высокопроизводительная передача данных)
- Arrow Tables (in-memory обработка)
- Parquet Files (архивное хранение)
- DataFusion (SQL запросы)
- ClickHouse (исторические данные)
- PostgreSQL (пользователи/транзакции)
- MongoDB (метаданные/конфигурация)
- Kafka (сообщения/события)

## 🏗️ Архитектура

### Структура проекта
```
src/
├── data_access/
│   ├── mod.rs
│   ├── api/                    # API коннекторы
│   │   ├── mod.rs
│   │   ├── exchange/           # Коннекторы к биржам
│   │   │   ├── mod.rs
│   │   │   ├── binance.rs
│   │   │   ├── bybit.rs
│   │   │   └── base.rs
│   │   └── data_provider/      # Провайдеры данных
│   │       ├── mod.rs
│   │       ├── yahoo.rs
│   │       └── alpha_vantage.rs
│   ├── database/               # Database коннекторы
│   │   ├── mod.rs
│   │   ├── redis.rs
│   │   ├── clickhouse.rs
│   │   ├── postgresql.rs
│   │   ├── mongodb.rs
│   │   ├── kafka.rs
│   │   ├── arrow_flight.rs
│   │   ├── parquet.rs
│   │   └── datafusion.rs
│   ├── models/                 # Модели данных
│   │   ├── mod.rs
│   │   ├── candle.rs
│   │   ├── trade.rs
│   │   ├── order.rs
│   │   └── user.rs
│   ├── query_builder/          # Query Builder
│   │   ├── mod.rs
│   │   ├── sql.rs
│   │   ├── redis.rs
│   │   └── mongodb.rs
│   └── traits/                 # Трейты для унификации
│       ├── mod.rs
│       ├── database.rs
│       ├── cache.rs
│       └── message_queue.rs
```

## 📊 План реализации

### Этап 1: Базовая инфраструктура ✅ ЗАВЕРШЕН
- [x] Создать структуру папок
- [x] Реализовать базовые трейты
- [x] Создать модели данных
- [x] Настроить зависимости в Cargo.toml

### Этап 2: Database коннекторы
- [x] Redis коннектор ✅ ЗАВЕРШЕН
- [x] ClickHouse коннектор ✅ ЗАВЕРШЕН
- [x] PostgreSQL коннектор ✅ ЗАВЕРШЕН
- [ ] MongoDB коннектор
- [ ] Kafka коннектор

### Этап 3: Arrow/Parquet инфраструктура
- [ ] Arrow Flight коннектор
- [ ] Parquet файлы
- [ ] DataFusion коннектор
- [ ] Arrow Tables интеграция

### Этап 4: Query Builder
- [x] SQL Query Builder (базовый)
- [x] Redis Query Builder ✅ ЗАВЕРШЕН
- [x] ClickHouse Query Builder ✅ ЗАВЕРШЕН
- [x] PostgreSQL Query Builder ✅ ЗАВЕРШЕН
- [ ] MongoDB Query Builder
- [ ] Arrow Query Builder

### Этап 5: API коннекторы
- [ ] Базовый трейт для бирж
- [ ] Binance коннектор
- [ ] Bybit коннектор
- [ ] Провайдеры данных

### Этап 6: Интеграция и тестирование
- [ ] Интеграционные тесты
- [ ] Документация
- [ ] Примеры использования

## 🔧 Технические требования

### Зависимости
```toml
[dependencies]
# Database drivers
redis = "0.24"
clickhouse-rs = "0.11"
tokio-postgres = "0.7"
mongodb = "2.8"
kafka = "0.9"

# Arrow/Parquet
arrow = "50.0"
parquet = "50.0"
datafusion = "37.0"

# HTTP clients
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Async runtime
tokio = { version = "1.0", features = ["full"] }
```

### Трейты для унификации
```rust
// Базовый трейт для всех источников данных
pub trait DataSource {
    type Error;
    async fn connect(&mut self) -> Result<(), Self::Error>;
    async fn disconnect(&mut self) -> Result<(), Self::Error>;
    fn is_connected(&self) -> bool;
}

// Трейт для кэширования
pub trait Cache {
    type Error;
    async fn get<T>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> Result<(), Self::Error>;
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
}

// Трейт для базы данных
pub trait Database {
    type Error;
    async fn query<T>(&self, query: &str) -> Result<Vec<T>, Self::Error>;
    async fn execute(&self, query: &str) -> Result<u64, Self::Error>;
}
```

## 📈 Метрики прогресса
- **Общий прогресс**: 5/6 этапов (83%)
- **Текущий этап**: Этап 2 - Database коннекторы (Redis, ClickHouse и PostgreSQL завершены, MongoDB в процессе)
- **Время выполнения**: 2.5 дня из 2-3 дней
- **Сложность**: Средняя

## 🎯 Критерии успеха
- [ ] Все коннекторы работают стабильно
- [ ] Query Builder упрощает работу с данными
- [ ] Хорошая производительность
- [ ] Понятная архитектура
- [ ] Полная документация
- [ ] Покрытие тестами >80%

## 📝 Заметки
- Начать с простых коннекторов (Redis, PostgreSQL)
- Arrow/Parquet интегрировать после базовых БД
- Query Builder должен быть интуитивным
- Обратить внимание на обработку ошибок
- Использовать async/await везде где возможно

## ✅ Что реализовано

### Этап 1: Базовая инфраструктура ✅ ЗАВЕРШЕН
- ✅ Создана структура папок data_access
- ✅ Реализованы базовые трейты (DataSource, Cache, Database, Transaction, MessageQueue)
- ✅ Созданы модели данных (Candle, Trade, Order, User, Ticker, TradingSignal и др.)
- ✅ Настроены зависимости в Cargo.toml (redis, tokio-postgres, reqwest, anyhow)
- ✅ Добавлена поддержка serde для chrono

### Этап 2: Database коннекторы (частично)
- ✅ Redis коннектор полностью реализован
  - Подключение/отключение
  - Кэширование (get, set, delete, exists, expire, keys)
  - Работа с очередями (lpush, rpop, llen, lrange)
  - Счетчики (increment, decrement)
  - Строковые операции
- ✅ ClickHouse коннектор полностью реализован
  - Подключение/отключение
  - Выполнение SQL запросов
  - Работа с параметрами запросов
  - CRUD операции для свечей, сделок, результатов бэктестов
  - Создание таблиц для торговых данных
  - Аналитические запросы
  - Транзакции (заглушка для ClickHouse)
- ✅ PostgreSQL коннектор полностью реализован
  - Подключение/отключение с пулом соединений
  - Выполнение SQL запросов
  - Работа с параметрами запросов
  - CRUD операции для пользователей, стратегий, свечей, сделок, результатов бэктестов
  - Создание таблиц для торговых данных с индексами
  - Аналитические запросы и статистика
  - Транзакции (упрощенная реализация)
  - Утилиты для работы с базой данных
- ⏳ MongoDB коннектор (заглушка)
- ⏳ Kafka коннектор (заглушка)

### Этап 4: Query Builder (частично)
- ✅ Redis Query Builder полностью реализован
  - Базовый RedisQueryBuilder
  - Специализированные: CacheQueryBuilder, QueueQueryBuilder, CounterQueryBuilder
  - Утилиты для ключей (KeyUtils)
- ✅ ClickHouse Query Builder полностью реализован
  - Базовый ClickHouseQueryBuilder с поддержкой SELECT, INSERT, UPDATE, DELETE
  - Специализированные: CandleQueryBuilder, TradeQueryBuilder, BacktestQueryBuilder
  - Поддержка WHERE условий, ORDER BY, GROUP BY, LIMIT, OFFSET
  - JOIN операции (INNER, LEFT, RIGHT, FULL)
  - Утилиты для аналитических запросов (ClickHouseUtils)
- ✅ PostgreSQL Query Builder полностью реализован
  - Базовый PostgreSQLQueryBuilder с поддержкой SELECT, INSERT, UPDATE, DELETE
  - Специализированные: UserQueryBuilder, StrategyQueryBuilder, CandleQueryBuilder, TradeQueryBuilder, BacktestQueryBuilder
  - Поддержка WHERE условий, ORDER BY, GROUP BY, LIMIT, OFFSET
  - JOIN операции (INNER, LEFT, RIGHT, FULL)
  - Утилиты для аналитических запросов (PostgreSQLUtils)
- ⏳ MongoDB Query Builder (заглушка)
- ⏳ Arrow Query Builder (заглушка)

### Дополнительно
- ✅ Создан lib.rs для библиотеки
- ✅ Написаны примеры использования Redis коннектора
- ✅ Написаны примеры использования ClickHouse коннектора
- ✅ Написаны примеры использования PostgreSQL коннектора
- ✅ Созданы тесты для проверки работы
- ✅ Проект успешно компилируется
- ✅ Тесты проходят успешно

## 🔄 Следующие действия
1. ✅ Создать структуру папок
2. ✅ Настроить зависимости
3. ✅ Реализовать базовые трейты
4. ✅ Реализовать Redis коннектор
5. ✅ Создать Redis Query Builder
6. ✅ Реализовать ClickHouse коннектор
7. ✅ Создать ClickHouse Query Builder
8. ✅ Реализовать PostgreSQL коннектор
9. ✅ Создать PostgreSQL Query Builder
10. ⏳ Реализовать MongoDB коннектор
11. ⏳ Реализовать Kafka коннектор
12. ⏳ Создать Arrow/Parquet коннекторы
13. ⏳ Создать API коннекторы к биржам
14. ⏳ Написать интеграционные тесты
15. ⏳ Создать документацию
