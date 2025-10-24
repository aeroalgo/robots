-- ============================================================================
-- Инициализация ClickHouse для торговых роботов
-- ============================================================================

-- Создание базы данных
CREATE DATABASE IF NOT EXISTS trading;

USE trading;

-- Таблица для исторических данных OHLCV
CREATE TABLE IF NOT EXISTS ohlcv_data (
    symbol String,
    timeframe String,
    timestamp DateTime64(3),
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (symbol, timeframe, timestamp)
SETTINGS index_granularity = 8192;

-- Таблица для тиковых данных
CREATE TABLE IF NOT EXISTS tick_data (
    symbol String,
    timestamp DateTime64(3),
    bid Float64,
    ask Float64,
    last_price Float64,
    volume Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (symbol, timestamp)
SETTINGS index_granularity = 8192;

-- Таблица для индикаторов
CREATE TABLE IF NOT EXISTS indicators (
    symbol String,
    timeframe String,
    indicator_name String,
    timestamp DateTime64(3),
    value Float64,
    parameters String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (symbol, timeframe, indicator_name, timestamp)
SETTINGS index_granularity = 8192;

-- Таблица для сигналов
CREATE TABLE IF NOT EXISTS signals (
    strategy_id String,
    symbol String,
    timeframe String,
    timestamp DateTime64(3),
    signal_type String,
    signal_strength Float64,
    price Float64,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (strategy_id, symbol, timestamp)
SETTINGS index_granularity = 8192;

-- Таблица для сделок
CREATE TABLE IF NOT EXISTS trades (
    trade_id String,
    strategy_id String,
    symbol String,
    side String,
    quantity Float64,
    entry_price Float64,
    exit_price Nullable(Float64),
    entry_time DateTime64(3),
    exit_time Nullable(DateTime64(3)),
    pnl Nullable(Float64),
    commission Float64,
    status String,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(entry_time)
ORDER BY (strategy_id, symbol, entry_time)
SETTINGS index_granularity = 8192;

-- Таблица для метрик стратегий
CREATE TABLE IF NOT EXISTS strategy_metrics (
    strategy_id String,
    metric_name String,
    metric_value Float64,
    calculation_date Date,
    period_start Date,
    period_end Date,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(calculation_date)
ORDER BY (strategy_id, metric_name, calculation_date)
SETTINGS index_granularity = 8192;

-- Таблица для оптимизации параметров
CREATE TABLE IF NOT EXISTS optimization_results (
    optimization_id String,
    strategy_id String,
    parameter_name String,
    parameter_value Float64,
    metric_name String,
    metric_value Float64,
    iteration Int32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(created_at)
ORDER BY (optimization_id, strategy_id, iteration)
SETTINGS index_granularity = 8192;

-- Создание материализованных представлений для агрегации
CREATE MATERIALIZED VIEW IF NOT EXISTS daily_stats
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (symbol, date)
AS SELECT
    symbol,
    toDate(timestamp) as date,
    count() as bars_count,
    min(low) as daily_low,
    max(high) as daily_high,
    argMin(open, timestamp) as daily_open,
    argMax(close, timestamp) as daily_close,
    sum(volume) as daily_volume
FROM ohlcv_data
GROUP BY symbol, date;

-- Создание материализованного представления для статистики сделок
CREATE MATERIALIZED VIEW IF NOT EXISTS trade_stats
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(entry_time)
ORDER BY (strategy_id, symbol, toDate(entry_time))
AS SELECT
    strategy_id,
    symbol,
    toDate(entry_time) as trade_date,
    count() as trades_count,
    sumIf(pnl, pnl > 0) as total_profit,
    sumIf(pnl, pnl < 0) as total_loss,
    sum(commission) as total_commission,
    avg(pnl) as avg_pnl
FROM trades
WHERE status = 'closed'
GROUP BY strategy_id, symbol, trade_date;

-- Создание пользователей с правами доступа
CREATE USER IF NOT EXISTS trading_user IDENTIFIED BY 'trading_password_2024';
CREATE USER IF NOT EXISTS monitoring IDENTIFIED BY 'monitoring_password_2024';

-- Предоставление прав доступа
GRANT ALL ON trading.* TO trading_user;
GRANT SELECT ON trading.* TO monitoring;

-- Создание роли для чтения
CREATE ROLE IF NOT EXISTS reader;
GRANT SELECT ON trading.* TO reader;

-- Создание роли для записи
CREATE ROLE IF NOT EXISTS writer;
GRANT INSERT, UPDATE, DELETE ON trading.* TO writer;

-- Назначение ролей пользователям
GRANT reader TO monitoring;
GRANT writer TO trading_user;
GRANT ALL TO trading_user;
