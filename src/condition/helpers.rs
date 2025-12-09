use crate::condition::types::{ConditionMetadata, SignalStrength, TrendDirection};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ConditionHelpers;

impl ConditionHelpers {
    pub fn calculate_signal_strength(diff: f32) -> SignalStrength {
        if diff > 0.1 {
            SignalStrength::VeryStrong
        } else if diff > 0.05 {
            SignalStrength::Strong
        } else if diff > 0.02 {
            SignalStrength::Medium
        } else {
            SignalStrength::Weak
        }
    }

    pub fn calculate_signal_strength_absolute(diff: f32) -> SignalStrength {
        let abs_diff = diff.abs();
        if abs_diff > 0.1 {
            SignalStrength::VeryStrong
        } else if abs_diff > 0.05 {
            SignalStrength::Strong
        } else if abs_diff > 0.02 {
            SignalStrength::Medium
        } else {
            SignalStrength::Weak
        }
    }

    pub fn create_condition_metadata(
        execution_time: Duration,
        data_points_processed: usize,
        confidence_score: f32,
    ) -> ConditionMetadata {
        ConditionMetadata {
            execution_time,
            data_points_processed,
            confidence_score,
            additional_info: HashMap::new(),
        }
    }

    pub fn create_condition_metadata_with_info(
        execution_time: Duration,
        data_points_processed: usize,
        confidence_score: f32,
        additional_info: HashMap<String, String>,
    ) -> ConditionMetadata {
        ConditionMetadata {
            execution_time,
            data_points_processed,
            confidence_score,
            additional_info,
        }
    }

    pub fn direction_from_signal(signal: bool) -> TrendDirection {
        if signal {
            TrendDirection::Rising
        } else {
            TrendDirection::Falling
        }
    }

    pub fn direction_from_signal_reverse(signal: bool) -> TrendDirection {
        if signal {
            TrendDirection::Falling
        } else {
            TrendDirection::Rising
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_signal_strength() {
        assert_eq!(
            ConditionHelpers::calculate_signal_strength(0.15),
            SignalStrength::VeryStrong
        );
        assert_eq!(
            ConditionHelpers::calculate_signal_strength(0.06),
            SignalStrength::Strong
        );
        assert_eq!(
            ConditionHelpers::calculate_signal_strength(0.03),
            SignalStrength::Medium
        );
        assert_eq!(
            ConditionHelpers::calculate_signal_strength(0.01),
            SignalStrength::Weak
        );
    }

    #[test]
    fn test_direction_from_signal() {
        assert_eq!(
            ConditionHelpers::direction_from_signal(true),
            TrendDirection::Rising
        );
        assert_eq!(
            ConditionHelpers::direction_from_signal(false),
            TrendDirection::Falling
        );
    }
}
