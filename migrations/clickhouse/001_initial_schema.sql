CREATE DATABASE IF NOT EXISTS trading;

CREATE TABLE IF NOT EXISTS trading.ohlcv_data (
    symbol String,
    timeframe String,
    timestamp Int64,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYear(toDateTime(timestamp / 1000))
ORDER BY (symbol, timeframe, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.tick_data (
    symbol String,
    timestamp Int64,
    bid Float64,
    ask Float64,
    last_price Float64,
    volume Float64,
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
    value Float64,
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
    signal_strength Float64,
    price Float64,
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
    quantity Float64,
    entry_price Float64,
    exit_price Nullable(Float64),
    entry_time Int64,
    exit_time Nullable(Int64),
    pnl Nullable(Float64),
    commission Float64,
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
    metric_value Float64,
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
PARTITION BY toYear(start_date)
ORDER BY (strategy_id, backtest_id, start_date)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.positions (
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
    quantity Float64,
    price Float64,
    status String,
    filled_quantity Float64,
    avg_fill_price Nullable(Float64),
    commission Float64,
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

CREATE TABLE IF NOT EXISTS trading.optimization_results (
    optimization_id String,
    strategy_id String,
    parameter_name String,
    parameter_value Float64,
    metric_name String,
    metric_value Float64,
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
    total_value Float64,
    cash Float64,
    positions_value Float64,
    unrealized_pnl Float64,
    realized_pnl Float64,
    daily_return Float64,
    total_return Float64,
    sharpe_ratio Float64,
    max_drawdown Float64,
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
    is_sharpe Float64,
    oos_sharpe Float64,
    is_profit Float64,
    oos_profit Float64,
    is_drawdown Float64,
    oos_drawdown Float64,
    efficiency_ratio Float64,
    overfitting_score Float64,
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

