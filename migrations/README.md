# Система миграций для торговых роботов

Система управления схемами баз данных для проекта торговых роботов.

## Архитектура баз данных

### ClickHouse - Аналитическое хранилище
**Назначение**: Хранение больших объемов временных рядов и аналитических данных

**Основные таблицы**:
- `ohlcv_data` - исторические свечи (OHLCV)
- `tick_data` - тиковые данные в реальном времени
- `indicators` - вычисленные значения индикаторов
- `signals` - торговые сигналы стратегий
- `trades` - история всех сделок
- `strategy_metrics` - метрики производительности стратегий
- `optimization_results` - результаты оптимизации параметров
- `backtest_results` - результаты бэктестов
- `genetic_population` - популяции генетического алгоритма
- `portfolio_snapshots` - снимки состояния портфеля
- `walk_forward_results` - результаты walk-forward тестирования
- `monte_carlo_simulations` - симуляции Монте-Карло

**Преимущества**:
- Колоночное хранение для быстрых аналитических запросов
- Автоматическое партиционирование по времени
- Эффективное сжатие данных
- Материализованные представления для агрегаций

### PostgreSQL - Реляционная БД
**Назначение**: Транзакционные данные, пользователи, конфигурации

**Основные таблицы**:
- `users` - пользователи системы
- `user_sessions` - активные сессии
- `api_keys` - API ключи для интеграций
- `user_permissions` - права доступа
- `audit_logs` - журнал аудита действий
- `strategies` - конфигурации стратегий
- `strategy_runs` - запуски стратегий
- `strategy_access` - доступ к стратегиям
- `trading_accounts` - торговые счета
- `subscriptions` - подписки пользователей
- `notifications` - уведомления
- `alert_rules` - правила для алертов
- `email_queue` - очередь email сообщений

**Преимущества**:
- ACID транзакции
- Relational integrity
- Сложные запросы с JOIN
- JSONB для гибких схем

### MongoDB - NoSQL хранилище
**Назначение**: Конфигурации, метаданные, логи, события

**Основные коллекции**:
- `strategy_configs` - JSON конфигурации стратегий
- `indicator_metadata` - метаданные индикаторов
- `system_logs` - системные логи
- `event_store` - хранилище событий (Event Sourcing)
- `ml_models` - метаданные ML моделей
- `genetic_algorithm_config` - конфигурации генетического алгоритма

**Преимущества**:
- Гибкая схема данных
- Быстрое чтение/запись
- Встроенная валидация схем
- Хорошо подходит для логов и событий

### Redis - Кэш и сессии
**Назначение**: Кэширование горячих данных, сессии, очереди

**Основные структуры**:
- `candles:{symbol}:{timeframe}` - последние свечи (списки)
- `indicators:{strategy_id}:{symbol}` - вычисленные индикаторы (хэши)
- `sessions:{session_id}` - пользовательские сессии (хэши)
- `limits:{user_id}` - rate limiting (счетчики)
- `queue:calculations` - очередь задач (списки)

**Преимущества**:
- Субмиллисекундные задержки
- Встроенные структуры данных
- Pub/Sub для real-time
- TTL для автоочистки

## Структура миграций

```
migrations/
├── migrate.py                  # Python скрипт для применения миграций
├── run.sh                      # Bash обертка
├── clickhouse/
│   ├── 001_initial_schema.sql
│   ├── 002_trading_system_schema.sql
│   └── 003_performance_and_analytics.sql
├── postgres/
│   ├── 001_initial_schema.sql
│   ├── 002_users_and_auth.sql
│   └── 003_notifications_and_alerts.sql
└── mongodb/
    └── 001_collections_schema.js
```

## Формат миграций

Имя файла: `{номер}_{описание}.{расширение}`

Примеры:
- `001_initial_schema.sql` - начальная схема
- `002_add_users_table.sql` - добавление таблицы пользователей
- `003_add_indexes.sql` - добавление индексов

## Использование

### Применить все миграции для всех БД
```bash
python migrations/migrate.py
# или
./migrations/run.sh
```

### Применить только для конкретной БД
```bash
python migrations/migrate.py clickhouse
python migrations/migrate.py postgres
```

### Из виртуального окружения
```bash
cd /path/to/project
source venv/bin/activate
export $(grep -v '^#' docker/env.local | xargs)
python migrations/migrate.py
```

## Переменные окружения

### PostgreSQL
- `POSTGRES_HOST` (default: localhost)
- `POSTGRES_PORT` (default: 5432)
- `POSTGRES_DB` (default: trading_users)
- `POSTGRES_USER` (default: postgres)
- `POSTGRES_PASSWORD` (default: postgres)

### ClickHouse
- `CLICKHOUSE_HOST` (default: localhost)
- `CLICKHOUSE_NATIVE_PORT` (default: 9002) - нативный TCP протокол
- `CLICKHOUSE_DB` (default: default)
- `CLICKHOUSE_USER` (default: default)
- `CLICKHOUSE_PASSWORD` (default: '')

### MongoDB
- `MONGO_HOST` (default: localhost)
- `MONGO_PORT` (default: 27017)
- `MONGO_USER` (default: admin)
- `MONGO_PASSWORD` (default: password)
- `MONGO_DATABASE` (default: trading_meta)

## Версионирование

Система автоматически отслеживает примененные миграции:
- Таблица `migration_history` в ClickHouse и PostgreSQL
- Коллекция `migrations` в MongoDB
- Повторное применение невозможно
- Миграции применяются строго по порядку номеров

## Создание новой миграции

1. Определите номер следующей миграции
2. Создайте файл с описательным именем
3. Напишите SQL/JS код
4. Протестируйте на dev окружении
5. Примените миграцию

Пример:
```bash
touch migrations/clickhouse/004_add_sentiment_analysis.sql
echo "CREATE TABLE sentiment_scores ..." > migrations/clickhouse/004_add_sentiment_analysis.sql
python migrations/migrate.py clickhouse
```

## Best Practices

1. **Одна миграция = одна ответственность**
   - Создавайте отдельные миграции для разных изменений

2. **Обратная совместимость**
   - Не удаляйте колонки сразу, сначала пометьте как deprecated

3. **Тестирование**
   - Тестируйте миграции на копии продакшн данных

4. **Резервные копии**
   - Делайте бэкапы перед применением миграций

5. **Документация**
   - Описывайте назначение изменений в комментариях

## Troubleshooting

### Ошибка подключения к ClickHouse
```bash
# Проверьте что порт 9002 (не 8123!)
export CLICKHOUSE_NATIVE_PORT=9002
```

### База данных не существует
```bash
# Создайте базу вручную или через Docker
docker-compose exec clickhouse clickhouse-client --query "CREATE DATABASE IF NOT EXISTS trading"
```

### Миграция застряла
```bash
# Посмотрите какие миграции применены
docker-compose exec clickhouse clickhouse-client --query "SELECT * FROM migration_history"
```

## Мониторинг

Система логирует:
- Все применяемые миграции
- Время выполнения
- Ошибки и предупреждения
- Статус каждой миграции

Логи доступны через:
- Stdout при запуске
- MongoDB коллекция `system_logs`
- PostgreSQL таблица `audit_logs`
