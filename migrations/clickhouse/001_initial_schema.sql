CREATE DATABASE IF NOT EXISTS trading;

CREATE TABLE IF NOT EXISTS trading.ohlcv_data (
    symbol String,
    timeframe String,
    timestamp Int64,
    open Float32,
    high Float32,
    low Float32,
    close Float32,
    volume Float32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(timestamp / 1000))
ORDER BY (symbol, timeframe, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.tick_data (
    symbol String,
    timestamp Int64,
    bid Float32,
    ask Float32,
    last_price Float32,
    volume Float32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(timestamp / 1000))
ORDER BY (symbol, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.symbol_info (
    code String,
    name String,
    exchange String,
    created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (exchange, code)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.indicators (
    symbol String,
    timeframe String,
    indicator_name String,
    timestamp Int64,
    value Float32,
    parameters String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(timestamp / 1000))
ORDER BY (symbol, timeframe, indicator_name, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.signals (
    strategy_id String,
    symbol String,
    timeframe String,
    timestamp Int64,
    signal_type String,
    signal_strength Float32,
    price Float32,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(timestamp / 1000))
ORDER BY (strategy_id, symbol, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.trades (
    trade_id String,
    strategy_id String,
    symbol String,
    side String,
    quantity Float32,
    entry_price Float32,
    exit_price Nullable(Float32),
    entry_time Int64,
    exit_time Nullable(Int64),
    pnl Nullable(Float32),
    commission Float32,
    status String,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(entry_time / 1000))
ORDER BY (strategy_id, symbol, entry_time)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.strategy_metrics (
    strategy_id String,
    metric_name String,
    metric_value Float32,
    calculation_date Date,
    period_start Date,
    period_end Date,
    metadata String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(calculation_date)
ORDER BY (strategy_id, metric_name, calculation_date)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.strategies (
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

CREATE TABLE IF NOT EXISTS trading.backtest_results (
    backtest_id String,
    strategy_id String,
    symbol String,
    timeframe String,
    start_date Date,
    end_date Date,
    total_trades Int32,
    winning_trades Int32,
    losing_trades Int32,
    total_pnl Float32,
    max_drawdown Float32,
    sharpe_ratio Float32,
    profit_factor Float32,
    win_rate Float32,
    avg_win Float32,
    avg_loss Float32,
    execution_time_ms Int32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(start_date)
ORDER BY (strategy_id, backtest_id, start_date)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.positions (
    position_id String,
    strategy_id String,
    symbol String,
    side String,
    quantity Float32,
    entry_price Float32,
    current_price Float32,
    unrealized_pnl Float32,
    stop_loss Nullable(Float32),
    take_profit Nullable(Float32),
    opened_at Int64,
    updated_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(updated_at)
ORDER BY (position_id)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.orders (
    order_id String,
    position_id String,
    strategy_id String,
    symbol String,
    order_type String,
    side String,
    quantity Float32,
    price Float32,
    status String,
    filled_quantity Float32,
    avg_fill_price Nullable(Float32),
    commission Float32,
    created_at Int64,
    filled_at Nullable(Int64),
    cancelled_at Nullable(Int64)
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(created_at / 1000))
ORDER BY (strategy_id, symbol, created_at)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.genetic_population (
    generation Int32,
    individual_id String,
    strategy_id String,
    fitness_score Float32,
    sharpe_ratio Float32,
    max_drawdown Float32,
    win_rate Float32,
    profit_factor Float32,
    genes String,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY generation
ORDER BY (generation, fitness_score)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.optimization_results (
    optimization_id String,
    strategy_id String,
    parameter_name String,
    parameter_value Float32,
    metric_name String,
    metric_value Float32,
    iteration Int32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(created_at)
ORDER BY (optimization_id, strategy_id, iteration)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.portfolio_snapshots (
    snapshot_id String,
    user_id String,
    timestamp Int64,
    total_value Float32,
    cash Float32,
    positions_value Float32,
    unrealized_pnl Float32,
    realized_pnl Float32,
    daily_return Float32,
    total_return Float32,
    sharpe_ratio Float32,
    max_drawdown Float32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(timestamp / 1000))
ORDER BY (user_id, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.walk_forward_results (
    wf_id String,
    strategy_id String,
    window_number Int32,
    in_sample_start Date,
    in_sample_end Date,
    out_sample_start Date,
    out_sample_end Date,
    is_sharpe Float32,
    oos_sharpe Float32,
    is_profit Float32,
    oos_profit Float32,
    is_drawdown Float32,
    oos_drawdown Float32,
    efficiency_ratio Float32,
    overfitting_score Float32,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY window_number
ORDER BY (strategy_id, window_number)
SETTINGS index_granularity = 8192;

CREATE MATERIALIZED VIEW IF NOT EXISTS trading.daily_stats
ENGINE = SummingMergeTree()
PARTITION BY toYear(date)
ORDER BY (symbol, date)
AS SELECT
    symbol,
    toDate(toDateTime(timestamp / 1000)) as date,
    count() as bars_count,
    min(low) as daily_low,
    max(high) as daily_high,
    argMin(open, timestamp) as daily_open,
    argMax(close, timestamp) as daily_close,
    sum(volume) as daily_volume
FROM trading.ohlcv_data
GROUP BY symbol, date;

CREATE MATERIALIZED VIEW IF NOT EXISTS trading.strategy_performance_daily
ENGINE = AggregatingMergeTree()
PARTITION BY toYear(trade_date)
ORDER BY (strategy_id, trade_date)
AS SELECT
    strategy_id,
    toDate(toDateTime(entry_time / 1000)) as trade_date,
    count() as trades_count,
    sumIf(pnl, pnl > 0) as total_profit,
    sumIf(pnl, pnl < 0) as total_loss,
    sum(pnl) as net_pnl,
    avgIf(pnl, pnl > 0) as avg_win,
    avgIf(pnl, pnl < 0) as avg_loss
FROM trading.trades
WHERE status = 'closed'
GROUP BY strategy_id, trade_date;

