use crate::indicators::base::Indicator;
use crate::indicators::base::{IndicatorBuildRules, NestingConfig, ThresholdType};
use crate::indicators::implementations::*;
use std::collections::HashMap;
use std::sync::OnceLock;

static BUILD_RULES_CACHE: OnceLock<HashMap<String, IndicatorBuildRules>> = OnceLock::new();

fn init_build_rules_cache() -> HashMap<String, IndicatorBuildRules> {
    let mut cache = HashMap::new();

    if let Ok(ind) = SMA::new(20.0) {
        cache.insert("SMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = EMA::new(20.0) {
        cache.insert("EMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = WMA::new(20.0) {
        cache.insert("WMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = AMA::new(20.0) {
        cache.insert("AMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = AMMA::new(20.0) {
        cache.insert("AMMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = ZLEMA::new(20.0) {
        cache.insert("ZLEMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = SINEWMA::new(20.0) {
        cache.insert("SINEWMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = SQWMA::new(20.0) {
        cache.insert("SQWMA".to_string(), ind.build_rules());
    }
    if let Ok(ind) = GEOMEAN::new(20.0) {
        cache.insert("GEOMEAN".to_string(), ind.build_rules());
    }
    if let Ok(ind) = TPBF::new(20.0) {
        cache.insert("TPBF".to_string(), ind.build_rules());
    }
    if let Ok(ind) = SuperTrend::new(10.0, 3.0) {
        cache.insert("SuperTrend".to_string(), ind.build_rules());
    }
    if let Ok(ind) = VTRAND::new(14.0) {
        cache.insert("VTRAND".to_string(), ind.build_rules());
    }

    if let Ok(ind) = RSI::new(14.0) {
        cache.insert("RSI".to_string(), ind.build_rules());
    }
    if let Ok(ind) = Stochastic::new(14.0) {
        cache.insert("Stochastic".to_string(), ind.build_rules());
    }

    if let Ok(ind) = ATR::new(14.0) {
        cache.insert("ATR".to_string(), ind.build_rules());
    }
    if let Ok(ind) = WATR::new(14.0) {
        cache.insert("WATR".to_string(), ind.build_rules());
    }
    if let Ok(ind) = TrueRange::new() {
        cache.insert("TrueRange".to_string(), ind.build_rules());
    }

    cache
}

pub fn get_build_rules(indicator_name: &str) -> Option<&'static IndicatorBuildRules> {
    let cache = BUILD_RULES_CACHE.get_or_init(init_build_rules_cache);
    cache.get(indicator_name)
}

pub fn get_build_rules_or_default(indicator_name: &str) -> IndicatorBuildRules {
    get_build_rules(indicator_name)
        .cloned()
        .unwrap_or(IndicatorBuildRules::TREND)
}

pub fn is_phase_1_allowed(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.phase_1_allowed)
        .unwrap_or(true)
}

pub fn has_absolute_threshold(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| matches!(r.threshold_type, ThresholdType::Absolute))
        .unwrap_or(false)
}

pub fn has_percent_of_price_threshold(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| matches!(r.threshold_type, ThresholdType::PercentOfPrice { .. }))
        .unwrap_or(false)
}

pub fn has_no_threshold(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| matches!(r.threshold_type, ThresholdType::None))
        .unwrap_or(false)
}

pub fn can_accept_nested_input(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.nesting.accepts_input)
        .unwrap_or(false)
}

pub fn can_be_nested_input(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.nesting.can_be_input)
        .unwrap_or(false)
}

pub fn supports_percent_condition(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.supports_percent_condition)
        .unwrap_or(false)
}

pub fn can_compare_with_input_source(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.can_compare_with_input_source)
        .unwrap_or(false)
}

pub fn can_compare_with_nested_result(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.can_compare_with_nested_result)
        .unwrap_or(false)
}

pub fn indicator_compare_enabled(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.indicator_compare.enabled)
        .unwrap_or(false)
}

pub fn price_compare_enabled(indicator_name: &str) -> bool {
    get_build_rules(indicator_name)
        .map(|r| r.price_compare.enabled)
        .unwrap_or(true)
}

pub fn is_oscillator_like(indicator_name: &str) -> bool {
    has_absolute_threshold(indicator_name) && !indicator_compare_enabled(indicator_name)
}

pub fn get_allowed_conditions(
    indicator_name: &str,
) -> &'static [crate::strategy::types::ConditionOperator] {
    get_build_rules(indicator_name)
        .map(|r| r.allowed_conditions)
        .unwrap_or(&[
            crate::strategy::types::ConditionOperator::Above,
            crate::strategy::types::ConditionOperator::Below,
        ])
}
