use crate::data_model::types::TimeFrame;
use std::collections::{HashMap, HashSet};

pub struct ConditionId;

impl ConditionId {
    pub fn indicator_price(prefix: &str, alias: &str, random: u32) -> String {
        format!("{}_{}_{}", prefix, alias, random)
    }

    pub fn indicator_price_with_timeframes(
        prefix: &str,
        alias: &str,
        price_field: &str,
        operator: &str,
        primary_tf: &TimeFrame,
        secondary_tf: &TimeFrame,
    ) -> String {
        format!(
            "{}_{}_{}_{}_{:?}_{:?}",
            prefix, alias, price_field, operator, primary_tf, secondary_tf
        )
    }

    pub fn indicator_constant(prefix: &str, alias: &str, random: u32) -> String {
        format!("{}_{}_{}", prefix, alias, random)
    }

    pub fn indicator_constant_with_timeframe(
        prefix: &str,
        alias: &str,
        operator: &str,
        constant: f32,
        tf: &TimeFrame,
    ) -> String {
        format!("{}_{}_{}_{}_tf{:?}", prefix, alias, operator, constant, tf)
    }

    pub fn indicator_indicator(
        prefix: &str,
        primary_alias: &str,
        secondary_alias: &str,
        random: u32,
    ) -> String {
        format!(
            "{}_{}::{}_{}",
            prefix, primary_alias, secondary_alias, random
        )
    }

    pub fn indicator_indicator_with_timeframes(
        prefix: &str,
        primary_alias: &str,
        secondary_alias: &str,
        operator: &str,
        primary_tf: &TimeFrame,
        secondary_tf: &TimeFrame,
    ) -> String {
        format!(
            "{}_{}::{}_{}_tf{:?}_tf{:?}",
            prefix, primary_alias, secondary_alias, operator, primary_tf, secondary_tf
        )
    }

    pub fn trend_condition(
        prefix: &str,
        alias: &str,
        trend_type: TrendType,
        random: u32,
    ) -> String {
        format!("{}_{}_{}_{}", prefix, alias, trend_type.as_str(), random)
    }

    pub fn exit_wrapper(condition_id: &str) -> String {
        format!("exit_{}", condition_id)
    }

    pub fn entry_prefix() -> &'static str {
        "entry"
    }

    pub fn exit_prefix() -> &'static str {
        "exit"
    }

    pub fn prefix_for(is_entry: bool) -> &'static str {
        if is_entry {
            "entry"
        } else {
            "exit"
        }
    }

    pub fn parse(condition_id: &str) -> Option<ParsedConditionId> {
        let (prefix, rest) = Self::extract_prefix(condition_id)?;

        if rest.contains("::") {
            return Self::parse_indicator_indicator(prefix, rest);
        }

        if let Some(pos) = rest.find("_risingtrend_") {
            return Self::parse_trend_condition(prefix, rest, pos, TrendType::Rising);
        }
        if let Some(pos) = rest.find("_fallingtrend_") {
            return Self::parse_trend_condition(prefix, rest, pos, TrendType::Falling);
        }

        Self::parse_simple(prefix, rest)
    }

    pub fn is_indicator_indicator(condition_id: &str) -> bool {
        condition_id.contains("::")
    }

    pub fn is_trend_condition(condition_id: &str) -> bool {
        condition_id.contains("_risingtrend_") || condition_id.contains("_fallingtrend_")
    }

    /// Собирает требуемые таймфреймы для всех индикаторов из списка условий
    /// Возвращает HashMap: alias -> HashSet<TimeFrame>
    pub fn collect_required_timeframes(
        conditions: &[&dyn ConditionInfoTrait],
        base_timeframe: &TimeFrame,
    ) -> HashMap<String, HashSet<TimeFrame>> {
        let mut required_timeframes: HashMap<String, HashSet<TimeFrame>> = HashMap::new();

        for condition in conditions {
            if let Some(alias) = condition.primary_indicator_alias() {
                let tf = condition.primary_timeframe()
                    .cloned()
                    .unwrap_or_else(|| base_timeframe.clone());
                required_timeframes.entry(alias).or_default().insert(tf);
            }

            if condition.condition_type() == "indicator_indicator" {
                if let Some(secondary_alias) = condition.secondary_indicator_alias() {
                    let secondary_tf = condition.secondary_timeframe()
                        .cloned()
                        .unwrap_or_else(|| base_timeframe.clone());
                    required_timeframes.entry(secondary_alias).or_default().insert(secondary_tf);
                }
            }
        }

        required_timeframes
    }

    fn extract_prefix(condition_id: &str) -> Option<(ConditionPrefix, &str)> {
        if condition_id.starts_with("entry_") {
            Some((ConditionPrefix::Entry, condition_id.strip_prefix("entry_")?))
        } else if condition_id.starts_with("exit_") {
            Some((ConditionPrefix::Exit, condition_id.strip_prefix("exit_")?))
        } else if condition_id.starts_with("ind_price_") {
            Some((
                ConditionPrefix::IndPrice,
                condition_id.strip_prefix("ind_price_")?,
            ))
        } else if condition_id.starts_with("ind_const_") {
            Some((
                ConditionPrefix::IndConst,
                condition_id.strip_prefix("ind_const_")?,
            ))
        } else if condition_id.starts_with("ind_ind_") {
            Some((
                ConditionPrefix::IndInd,
                condition_id.strip_prefix("ind_ind_")?,
            ))
        } else {
            None
        }
    }

    fn parse_indicator_indicator(prefix: ConditionPrefix, rest: &str) -> Option<ParsedConditionId> {
        let separator_pos = rest.find("::")?;
        let primary_alias = rest[..separator_pos].to_string();
        let after_separator = &rest[separator_pos + 2..];
        let last_underscore = after_separator.rfind('_')?;
        let secondary_alias = after_separator[..last_underscore].to_string();

        Some(ParsedConditionId {
            prefix,
            condition_type: ConditionIdType::IndicatorIndicator,
            primary_alias,
            secondary_alias: Some(secondary_alias),
            trend_type: None,
        })
    }

    fn parse_trend_condition(
        prefix: ConditionPrefix,
        rest: &str,
        pos: usize,
        trend_type: TrendType,
    ) -> Option<ParsedConditionId> {
        let primary_alias = rest[..pos].to_string();

        Some(ParsedConditionId {
            prefix,
            condition_type: ConditionIdType::TrendCondition,
            primary_alias,
            secondary_alias: None,
            trend_type: Some(trend_type),
        })
    }

    fn parse_simple(prefix: ConditionPrefix, rest: &str) -> Option<ParsedConditionId> {
        let last_underscore = rest.rfind('_')?;
        let primary_alias = rest[..last_underscore].to_string();

        Some(ParsedConditionId {
            prefix,
            condition_type: ConditionIdType::Simple,
            primary_alias,
            secondary_alias: None,
            trend_type: None,
        })
    }
}

/// Трейт для работы с информацией об условиях
pub trait ConditionInfoTrait {
    fn condition_id(&self) -> &str;
    fn condition_type(&self) -> &str;
    fn primary_timeframe(&self) -> Option<&TimeFrame>;
    fn secondary_timeframe(&self) -> Option<&TimeFrame>;
    fn primary_indicator_alias(&self) -> Option<String>;
    fn secondary_indicator_alias(&self) -> Option<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionPrefix {
    Entry,
    Exit,
    IndPrice,
    IndConst,
    IndInd,
}

impl ConditionPrefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionPrefix::Entry => "entry",
            ConditionPrefix::Exit => "exit",
            ConditionPrefix::IndPrice => "ind_price",
            ConditionPrefix::IndConst => "ind_const",
            ConditionPrefix::IndInd => "ind_ind",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendType {
    Rising,
    Falling,
}

impl TrendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrendType::Rising => "risingtrend",
            TrendType::Falling => "fallingtrend",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TrendType::Rising => "RisingTrend",
            TrendType::Falling => "FallingTrend",
        }
    }

    pub fn from_operator_name(name: &str) -> Option<Self> {
        match name {
            "RisingTrend" | "risingtrend" => Some(TrendType::Rising),
            "FallingTrend" | "fallingtrend" => Some(TrendType::Falling),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionIdType {
    Simple,
    IndicatorIndicator,
    TrendCondition,
}

#[derive(Debug, Clone)]
pub struct ParsedConditionId {
    pub prefix: ConditionPrefix,
    pub condition_type: ConditionIdType,
    pub primary_alias: String,
    pub secondary_alias: Option<String>,
    pub trend_type: Option<TrendType>,
}

impl ParsedConditionId {
    pub fn is_indicator_indicator(&self) -> bool {
        self.condition_type == ConditionIdType::IndicatorIndicator
    }

    pub fn is_trend_condition(&self) -> bool {
        self.condition_type == ConditionIdType::TrendCondition
    }

    pub fn is_entry(&self) -> bool {
        self.prefix == ConditionPrefix::Entry
    }

    pub fn is_exit(&self) -> bool {
        self.prefix == ConditionPrefix::Exit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_indicator_price() {
        let id = ConditionId::indicator_price("entry", "sma", 123);
        assert_eq!(id, "entry_sma_123");
    }

    #[test]
    fn test_create_indicator_indicator() {
        let id = ConditionId::indicator_indicator("entry", "vtrand", "zlema", 456);
        assert_eq!(id, "entry_vtrand::zlema_456");
    }

    #[test]
    fn test_create_trend_condition() {
        let id = ConditionId::trend_condition("entry", "amma", TrendType::Rising, 789);
        assert_eq!(id, "entry_amma_risingtrend_789");
    }

    #[test]
    fn test_parse_indicator_price() {
        let parsed = ConditionId::parse("entry_sma_123").unwrap();
        assert_eq!(parsed.primary_alias, "sma");
        assert!(parsed.secondary_alias.is_none());
        assert_eq!(parsed.condition_type, ConditionIdType::Simple);
    }

    #[test]
    fn test_parse_indicator_indicator() {
        let parsed = ConditionId::parse("entry_vtrand::zlema_456").unwrap();
        assert_eq!(parsed.primary_alias, "vtrand");
        assert_eq!(parsed.secondary_alias, Some("zlema".to_string()));
        assert!(parsed.is_indicator_indicator());
    }

    #[test]
    fn test_parse_trend_condition() {
        let parsed = ConditionId::parse("entry_amma_risingtrend_789").unwrap();
        assert_eq!(parsed.primary_alias, "amma");
        assert!(parsed.is_trend_condition());
        assert_eq!(parsed.trend_type, Some(TrendType::Rising));
    }

    #[test]
    fn test_parse_nested_alias() {
        let parsed = ConditionId::parse("entry_geomean_on_rsi_risingtrend_111").unwrap();
        assert_eq!(parsed.primary_alias, "geomean_on_rsi");
        assert!(parsed.is_trend_condition());
    }


    #[test]
    fn test_prefix_for() {
        assert_eq!(ConditionId::prefix_for(true), "entry");
        assert_eq!(ConditionId::prefix_for(false), "exit");
    }

    #[test]
    fn test_is_indicator_indicator() {
        assert!(ConditionId::is_indicator_indicator("entry_sma::ema_123"));
        assert!(!ConditionId::is_indicator_indicator("entry_sma_123"));
    }

    #[test]
    fn test_is_trend_condition() {
        assert!(ConditionId::is_trend_condition("entry_sma_risingtrend_123"));
        assert!(ConditionId::is_trend_condition("exit_rsi_fallingtrend_456"));
        assert!(!ConditionId::is_trend_condition("entry_sma_123"));
    }
}
