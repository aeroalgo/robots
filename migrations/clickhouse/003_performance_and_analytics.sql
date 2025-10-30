CREATE TABLE IF NOT EXISTS trading.portfolio_snapshots (
    snapshot_id String,
    user_id String,
    timestamp DateTime64(3),
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
PARTITION BY toYYYYMM(timestamp)
ORDER BY (user_id, timestamp)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.performance_attribution (
    attribution_id String,
    strategy_id String,
    period_start Date,
    period_end Date,
    selection_effect Float64,
    timing_effect Float64,
    interaction_effect Float64,
    total_alpha Float64,
    systematic_risk Float64,
    specific_risk Float64,
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(period_start)
ORDER BY (strategy_id, period_start)
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

CREATE TABLE IF NOT EXISTS trading.monte_carlo_simulations (
    simulation_id String,
    strategy_id String,
    iteration Int32,
    final_balance Float64,
    max_drawdown Float64,
    total_return Float64,
    sharpe_ratio Float64,
    trades_sequence Array(Float64),
    created_at DateTime DEFAULT now()
) ENGINE = MergeTree()
PARTITION BY intDiv(iteration, 1000)
ORDER BY (strategy_id, simulation_id, iteration)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.correlation_matrix (
    matrix_id String,
    calculation_date Date,
    symbol_pairs Array(Tuple(String, String)),
    correlations Array(Float64),
    timeframe String,
    lookback_period Int32,
    created_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(created_at)
ORDER BY (calculation_date, timeframe)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.feature_importance (
    feature_id String,
    strategy_id String,
    feature_name String,
    importance_score Float64,
    method String,
    calculation_date Date,
    created_at DateTime DEFAULT now()
) ENGINE = ReplacingMergeTree(created_at)
ORDER BY (strategy_id, calculation_date, importance_score)
SETTINGS index_granularity = 8192;

CREATE TABLE IF NOT EXISTS trading.execution_stats (
    exec_id String,
    component String,
    operation String,
    execution_time_ms Int32,
    memory_used_mb Float64,
    cpu_usage_percent Float64,
    timestamp DateTime64(3),
    metadata String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (component, timestamp)
SETTINGS index_granularity = 8192;

CREATE MATERIALIZED VIEW IF NOT EXISTS hourly_indicators_agg
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(hour)
ORDER BY (symbol, timeframe, indicator_name, hour)
AS SELECT
    symbol,
    timeframe,
    indicator_name,
    toStartOfHour(timestamp) as hour,
    avg(value) as avg_value,
    min(value) as min_value,
    max(value) as max_value,
    stddevPop(value) as stddev_value
FROM trading.indicators
GROUP BY symbol, timeframe, indicator_name, hour;

CREATE MATERIALIZED VIEW IF NOT EXISTS strategy_correlation
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (strategy_id, date)
AS SELECT
    strategy_id,
    toDate(entry_time) as date,
    groupArray(pnl) as returns,
    corr(pnl, entry_price) as price_correlation
FROM trading.trades
WHERE status = 'closed'
GROUP BY strategy_id, date;

