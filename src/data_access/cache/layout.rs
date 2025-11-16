pub const TTL_ACTIVE_POSITION_SECONDS: u64 = 24 * 60 * 60;
pub const TTL_SIGNAL_SECONDS: u64 = 15 * 60;
pub const TTL_INDICATOR_SECONDS: u64 = 60;

pub fn active_position_key(strategy_id: &str, position_id: &str) -> String {
    format!("strategy:{}:positions:{}", strategy_id, position_id)
}

pub fn active_position_pattern(strategy_id: &str) -> String {
    format!("strategy:{}:positions:*", strategy_id)
}

pub fn signal_queue_key(strategy_id: &str) -> String {
    format!("strategy:{}:signals", strategy_id)
}

pub fn indicator_cache_key(alias: &str, timeframe: &str) -> String {
    format!("indicator_cache:{}:{}", alias, timeframe)
}

pub fn signal_score_zset(strategy_id: &str) -> String {
    format!("strategy:{}:signals:z", strategy_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_keys() {
        assert_eq!(
            active_position_key("alpha", "pos1"),
            "strategy:alpha:positions:pos1"
        );
        assert_eq!(
            active_position_pattern("alpha"),
            "strategy:alpha:positions:*"
        );
        assert_eq!(signal_queue_key("alpha"), "strategy:alpha:signals");
        assert_eq!(indicator_cache_key("rsi", "1h"), "indicator_cache:rsi:1h");
        assert_eq!(signal_score_zset("alpha"), "strategy:alpha:signals:z");
    }
}
