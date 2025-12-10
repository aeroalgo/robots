use crate::optimization::condition_id::{ConditionId, TrendType};
use rand::Rng;

pub struct ConditionIdGenerator;

impl ConditionIdGenerator {
    pub fn indicator_price(prefix: &str, alias: &str, rng: &mut impl Rng) -> String {
        ConditionId::indicator_price(prefix, alias, rng.gen::<u32>())
    }

    pub fn indicator_constant(prefix: &str, alias: &str, rng: &mut impl Rng) -> String {
        ConditionId::indicator_constant(prefix, alias, rng.gen::<u32>())
    }

    pub fn indicator_indicator(
        prefix: &str,
        primary_alias: &str,
        secondary_alias: &str,
        rng: &mut impl Rng,
    ) -> String {
        ConditionId::indicator_indicator(prefix, primary_alias, secondary_alias, rng.gen::<u32>())
    }

    pub fn trend_condition(
        prefix: &str,
        alias: &str,
        trend_type: TrendType,
        rng: &mut impl Rng,
    ) -> String {
        ConditionId::trend_condition(prefix, alias, trend_type, rng.gen::<u32>())
    }

    pub fn prefix_for(is_entry: bool) -> &'static str {
        ConditionId::prefix_for(is_entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_indicator_price() {
        let mut rng = thread_rng();
        let id = ConditionIdGenerator::indicator_price("entry", "sma", &mut rng);
        assert!(id.starts_with("entry_sma_"));
    }

    #[test]
    fn test_indicator_indicator() {
        let mut rng = thread_rng();
        let id = ConditionIdGenerator::indicator_indicator("entry", "sma", "ema", &mut rng);
        assert!(id.starts_with("entry_sma::ema_"));
    }

    #[test]
    fn test_trend_condition() {
        let mut rng = thread_rng();
        let id = ConditionIdGenerator::trend_condition("entry", "sma", TrendType::Rising, &mut rng);
        assert!(id.starts_with("entry_sma_risingtrend_"));
    }
}
