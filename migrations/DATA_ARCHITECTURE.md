# Архитектура данных системы торговых роботов

## Обзор

Система использует **полигибридную архитектуру данных** (Polyglot Persistence), где каждая база данных оптимизирована для своего типа нагрузки и паттернов доступа.

## Распределение данных по типам БД

### 1. ClickHouse - Аналитическое OLAP хранилище

**Почему ClickHouse?**
- Колоночное хранение идеально для аналитики временных рядов
- Агрегация миллионов строк за миллисекунды
- Эффективное сжатие (10-100x)
- Автоматическое партиционирование

**Что хранится:**

#### Рыночные данные
- `ohlcv_data` - свечи всех таймфреймов
- `tick_data` - тиковые данные
- `market_events` - важные рыночные события

#### Индикаторы и сигналы
- `indicators` - вычисленные значения индикаторов
- `signals` - торговые сигналы стратегий

#### Торговая активность
- `trades` - все сделки (открытие/закрытие)
- `orders` - история ордеров
- `positions` - текущие позиции

#### Аналитика и метрики
- `strategy_metrics` - метрики производительности (Sharpe, Sortino и т.д.)
- `portfolio_snapshots` - состояние портфеля
- `risk_metrics` - метрики риска (VaR, CVaR и т.д.)
- `performance_attribution` - атрибуция доходности

#### Оптимизация и тестирование
- `optimization_results` - результаты оптимизации параметров
- `backtest_results` - результаты бэктестов
- `walk_forward_results` - walk-forward анализ
- `monte_carlo_simulations` - симуляции Монте-Карло
- `genetic_population` - популяции генетического алгоритма

#### Производительность системы
- `execution_stats` - статистика выполнения
- `correlation_matrix` - корреляции между инструментами
- `feature_importance` - важность признаков для ML

**Материализованные представления:**
- `daily_stats` - дневная статистика по символам
- `strategy_performance_daily` - дневная производительность стратегий
- `hourly_indicators_agg` - агрегация индикаторов по часам

### 2. PostgreSQL - Реляционная OLTP БД

**Почему PostgreSQL?**
- ACID транзакции для критичных данных
- Сложные JOIN и foreign keys
- JSONB для гибких схем
- Отличная поддержка индексов

**Что хранится:**

#### Пользователи и безопасность
- `users` - учетные записи пользователей
- `user_sessions` - активные сессии
- `api_keys` - API ключи для интеграций
- `user_permissions` - гранулярные права доступа

#### Стратегии
- `strategies` - конфигурации стратегий
- `strategy_runs` - история запусков стратегий
- `strategy_access` - доступ пользователей к стратегиям

#### Финансовые аккаунты
- `trading_accounts` - торговые счета пользователей
- `subscriptions` - подписки и тарифы

#### Уведомления и алерты
- `notifications` - пользовательские уведомления
- `alert_rules` - правила для алертов
- `alert_history` - история срабатываний
- `email_queue` - очередь email сообщений

#### Аудит
- `audit_logs` - полный журнал действий пользователей

### 3. MongoDB - NoSQL документное хранилище

**Почему MongoDB?**
- Гибкая схема для конфигураций
- Быстрая запись логов
- Event Sourcing паттерн
- Валидация схем на уровне БД

**Что хранится:**

#### Конфигурации
- `strategy_configs` - полные JSON конфигурации стратегий
- `indicator_metadata` - метаданные индикаторов с параметрами
- `genetic_algorithm_config` - конфигурации GA оптимизации

#### Логи и события
- `system_logs` - системные логи всех сервисов
- `event_store` - хранилище событий (Event Sourcing)

#### Machine Learning
- `ml_models` - метаданные ML моделей
  - Архитектура
  - Гиперпараметры
  - Метрики производительности
  - История версий

### 4. Redis - In-Memory кэш и очереди

**Почему Redis?**
- Субмиллисекундные задержки
- Встроенные структуры данных
- Pub/Sub для real-time
- Автоматический TTL

**Что хранится:**

#### Кэш рыночных данных
- `candles:{symbol}:{timeframe}` - последние 1000 свечей (List)
- `ticks:{symbol}` - последние тики (Sorted Set по времени)
- `orderbook:{symbol}` - стакан заявок (Hash)

#### Кэш индикаторов
- `indicators:{strategy_id}:{symbol}` - вычисленные индикаторы (Hash)
- `signals:{strategy_id}` - последние сигналы (List)

#### Сессии и аутентификация
- `session:{session_id}` - данные сессии (Hash с TTL 24h)
- `user:{user_id}:tokens` - активные токены (Set)

#### Rate Limiting
- `ratelimit:{user_id}:{endpoint}` - счетчики запросов (String с TTL)
- `quota:{user_id}` - квоты пользователя (Hash)

#### Очереди задач
- `queue:calculations` - очередь расчетов индикаторов (List)
- `queue:backtest` - очередь бэктестов (List)
- `queue:optimization` - очередь оптимизаций (List)

#### Real-time данные
- `pubsub:ticks:{symbol}` - стрим тиков (Pub/Sub)
- `pubsub:signals` - стрим сигналов (Pub/Sub)

## Потоки данных

### 1. Загрузка исторических данных
```
Source (Exchange API) 
  → Python loader 
  → ClickHouse (ohlcv_data) 
  → Redis cache (последние 1000 баров)
```

### 2. Real-time обработка
```
WebSocket (Exchange) 
  → Redis (pubsub:ticks) 
  → Strategy Engine 
  → Redis (indicators cache)
  → ClickHouse (signals, trades)
```

### 3. Бэктестинг
```
ClickHouse (исторические данные) 
  → Arrow/Parquet (in-memory) 
  → Strategy Engine 
  → ClickHouse (backtest_results)
  → PostgreSQL (strategy_runs)
```

### 4. Оптимизация
```
MongoDB (strategy_configs) 
  → Genetic Algorithm 
  → ClickHouse (genetic_population) 
  → Best individuals 
  → MongoDB (updated configs)
```

## Стратегии кэширования

### L1 Cache (Redis) - Горячие данные
- TTL: 5 минут - 24 часа
- Размер: последние 1000 записей
- Hit rate: 90%+

### L2 Cache (Arrow in-memory) - Теплые данные
- TTL: 1 час
- Размер: данные текущего дня
- Hit rate: 70%+

### L3 Storage (ClickHouse) - Холодные данные
- TTL: вечное хранение
- Партиционирование по месяцам
- Архивирование старых партиций в S3

## Паттерны доступа к данным

### Read Pattern (Чтение)
```python
def get_candles(symbol, timeframe, limit):
    # 1. Проверить Redis
    cached = redis.lrange(f"candles:{symbol}:{timeframe}", 0, limit)
    if len(cached) >= limit:
        return cached
    
    # 2. Проверить Arrow in-memory
    if arrow_cache.has(symbol, timeframe):
        return arrow_cache.get(symbol, timeframe, limit)
    
    # 3. Загрузить из ClickHouse
    data = clickhouse.query(f"SELECT * FROM ohlcv_data WHERE symbol='{symbol}' ORDER BY timestamp DESC LIMIT {limit}")
    
    # 4. Закэшировать
    redis.lpush(f"candles:{symbol}:{timeframe}", data)
    redis.expire(f"candles:{symbol}:{timeframe}", 3600)
    
    return data
```

### Write Pattern (Запись)
```python
def save_trade(trade):
    # 1. Сохранить в ClickHouse (источник истины)
    clickhouse.insert("trades", trade)
    
    # 2. Обновить Redis кэш
    redis.lpush(f"trades:{trade.strategy_id}", trade.to_json())
    redis.ltrim(f"trades:{trade.strategy_id}", 0, 999)
    
    # 3. Pub/Sub уведомление
    redis.publish("trades:new", trade.to_json())
    
    # 4. Обновить метрики (async)
    queue.enqueue("calculate_metrics", trade.strategy_id)
```

## Резервное копирование

### ClickHouse
- Инкрементальные бэкапы каждые 6 часов
- Полные бэкапы раз в день
- Хранение: 30 дней локально + S3

### PostgreSQL
- WAL архивирование (Point-in-Time Recovery)
- Полные бэкапы раз в день
- Хранение: 30 дней локально + S3

### MongoDB
- Бэкапы каждые 12 часов
- Хранение: 14 дней локально + S3

### Redis
- RDB snapshots каждые 15 минут
- AOF для критичных данных
- Реплики для high availability

## Масштабирование

### Вертикальное
- ClickHouse: до 1TB RAM, 128 cores
- PostgreSQL: до 256GB RAM, 64 cores
- MongoDB: до 128GB RAM, 32 cores
- Redis: до 64GB RAM, 16 cores

### Горизонтальное
- ClickHouse: шардирование по symbol
- PostgreSQL: read replicas для аналитики
- MongoDB: шардирование по strategy_id
- Redis: Redis Cluster для распределения нагрузки

## Мониторинг

### Метрики
- Latency: p50, p95, p99
- Throughput: QPS, TPS
- Storage: использование диска, рост данных
- Cache: hit rate, miss rate

### Алерты
- Превышение latency > 100ms
- Cache hit rate < 80%
- Disk usage > 80%
- Replication lag > 10s

## Best Practices

1. **Используйте правильную БД для задачи**
   - Временные ряды → ClickHouse
   - Транзакции → PostgreSQL
   - Конфигурации → MongoDB
   - Кэш → Redis

2. **Партиционируйте данные**
   - ClickHouse: по месяцам
   - PostgreSQL: по годам для audit_logs
   - MongoDB: по датам для system_logs

3. **Индексируйте правильно**
   - Только используемые колонки
   - Composite indexes для частых запросов
   - Регулярно обновляйте статистику

4. **Кэшируйте разумно**
   - Горячие данные в Redis
   - TTL по типу данных
   - Предварительная загрузка (warm-up)

5. **Мониторьте всё**
   - Slow queries
   - Index usage
   - Cache efficiency
   - Disk I/O

