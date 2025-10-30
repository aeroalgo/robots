CREATE TABLE IF NOT EXISTS strategies (
    strategy_id String,
    strategy_name String,
    strategy_type String,
    indicators Array(String),
    entry_conditions String,
    exit_conditions String,
    parameters String,
    created_at DateTime DEFAULT now(),
    created_by String
) ENGINE = ReplacingMergeTree(created_at)
ORDER BY (strategy_id)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS backtest_results (
    backtest_id String,
    strategy_id String,
    symbol String,
    timeframe String,
    start_date Date,
    end_date Date,
    total_trades Int32,
    winning_trades Int32,
    losing_trades Int32,
    total_pnl Float64,
    max_drawdown Float64,
    sharpe_ratio Float64,
    profit_factor Float64,
    win_rate Float64,
    avg_win Float64,
    avg_loss Float64,
    execution_time_ms Int32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(start_date)
ORDER BY (strategy_id, backtest_id, start_date)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS positions (
    position_id String,
    strategy_id String,
    symbol String,
    side String,
    quantity Float64,
    entry_price Float64,
    current_price Float64,
    unrealized_pnl Float64,
    stop_loss Nullable(Float64),
    take_profit Nullable(Float64),
    opened_at DateTime64(3),
    updated_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (position_id)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS orders (
    order_id String,
    position_id String,
    strategy_id String,
    symbol String,
    order_type String,
    side String,
    quantity Float64,
    price Float64,
    status String,
    filled_quantity Float64,
    avg_fill_price Nullable(Float64),
    commission Float64,
    created_at DateTime64(3),
    filled_at Nullable(DateTime64(3)),
    cancelled_at Nullable(DateTime64(3))
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(created_at)
ORDER BY (strategy_id, symbol, created_at)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS genetic_population (
    generation Int32,
    individual_id String,
    strategy_id String,
    fitness_score Float64,
    sharpe_ratio Float64,
    max_drawdown Float64,
    win_rate Float64,
    profit_factor Float64,
    genes String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY generation
ORDER BY (generation, fitness_score)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS optimization_runs (
    optimization_id String,
    strategy_id String,
    algorithm String,
    population_size Int32,
    generations Int32,
    best_fitness Float64,
    best_individual_id String,
    started_at DateTime64(3),
    completed_at Nullable(DateTime64(3)),
    status String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(started_at)
ORDER BY (optimization_id, started_at)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS market_events (
    event_id String,
    event_type String,
    symbol String,
    timestamp DateTime64(3),
    price Float64,
    volume Float64,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (symbol, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS risk_metrics (
    strategy_id String,
    timestamp DateTime64(3),
    var_95 Float64,
    var_99 Float64,
    cvar_95 Float64,
    cvar_99 Float64,
    beta Float64,
    alpha Float64,
    information_ratio Float64,
    sortino_ratio Float64,
    calmar_ratio Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (strategy_id, timestamp)
SETTINGS index_granularity = 8192;

CREATE MATERIALIZED VIEW IF NOT EXISTS strategy_performance_daily
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(trade_date)
ORDER BY (strategy_id, trade_date)
AS SELECT
    strategy_id,
    toDate(entry_time) as trade_date,
    count() as trades_count,
    sumIf(pnl, pnl > 0) as total_profit,
    sumIf(pnl, pnl < 0) as total_loss,
    sum(pnl) as net_pnl,
    avgIf(pnl, pnl > 0) as avg_win,
    avgIf(pnl, pnl < 0) as avg_loss
FROM trading.trades
WHERE status = 'closed'
GROUP BY strategy_id, trade_date;

