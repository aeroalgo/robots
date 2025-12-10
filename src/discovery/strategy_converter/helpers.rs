use crate::data_model::types::TimeFrame;
use crate::discovery::types::ConditionInfo;
use crate::strategy::types::{DataSeriesSource, IndicatorBindingSpec, PriceField};
use std::collections::{HashMap, HashSet};

pub struct ConverterHelpers;

impl ConverterHelpers {
    pub fn build_alias_to_timeframes_map(
        indicator_bindings: &[IndicatorBindingSpec],
    ) -> HashMap<String, HashSet<TimeFrame>> {
        let mut alias_to_timeframes = HashMap::new();
        for binding in indicator_bindings {
            alias_to_timeframes
                .entry(binding.alias.clone())
                .or_insert_with(HashSet::new)
                .insert(binding.timeframe.clone());
        }
        alias_to_timeframes
    }

    pub fn create_indicator_source(
        alias: &str,
        explicit_timeframe: Option<&TimeFrame>,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> DataSeriesSource {
        if let Some(tf) = explicit_timeframe {
            DataSeriesSource::indicator_with_timeframe(alias.to_string(), tf.clone())
        } else if let Some(timeframes) = alias_to_timeframes.get(alias) {
            if let Some(tf) = timeframes.iter().next() {
                DataSeriesSource::indicator_with_timeframe(alias.to_string(), tf.clone())
            } else {
                DataSeriesSource::indicator(alias)
            }
        } else {
            DataSeriesSource::indicator(alias)
        }
    }

    pub fn create_price_source(
        price_field: PriceField,
        explicit_timeframe: Option<&TimeFrame>,
    ) -> DataSeriesSource {
        if let Some(tf) = explicit_timeframe {
            DataSeriesSource::price_with_timeframe(price_field, tf.clone())
        } else {
            DataSeriesSource::price(price_field)
        }
    }

    pub fn extract_percent_param(condition: &ConditionInfo) -> Option<f32> {
        let percent_param = condition
            .optimization_params
            .iter()
            .find(|p| p.name == "percent" || p.name == "percentage");

        percent_param.map(|_| {
            let range = crate::condition::parameters::ConditionParameterPresets::percentage();
            ((range.min + range.max) / 2.0) as f32
        })
    }

    pub fn parse_price_field_from_string(price_field_str: &str) -> Option<PriceField> {
        match price_field_str {
            "Open" => Some(PriceField::Open),
            "High" => Some(PriceField::High),
            "Low" => Some(PriceField::Low),
            "Close" => Some(PriceField::Close),
            "Volume" => Some(PriceField::Volume),
            _ => None,
        }
    }

    pub fn extract_price_field_from_condition_id(condition_id: &str) -> Option<PriceField> {
        if condition_id.starts_with("ind_price_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 4 {
                match parts[3] {
                    "Open" => Some(PriceField::Open),
                    "High" => Some(PriceField::High),
                    "Low" => Some(PriceField::Low),
                    "Close" => Some(PriceField::Close),
                    "Volume" => Some(PriceField::Volume),
                    _ => Some(PriceField::Close),
                }
            } else {
                Some(PriceField::Close)
            }
        } else {
            Some(PriceField::Close)
        }
    }

    pub fn get_price_field_for_condition(condition: &ConditionInfo) -> PriceField {
        if let Some(ref pf_str) = condition.price_field {
            Self::parse_price_field_from_string(pf_str).unwrap_or_else(|| PriceField::Close)
        } else {
            Self::extract_price_field_from_condition_id(&condition.id)
                .unwrap_or_else(|| PriceField::Close)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price_field_from_string() {
        assert_eq!(
            ConverterHelpers::parse_price_field_from_string("Close"),
            Some(PriceField::Close)
        );
        assert_eq!(
            ConverterHelpers::parse_price_field_from_string("Open"),
            Some(PriceField::Open)
        );
        assert_eq!(
            ConverterHelpers::parse_price_field_from_string("Invalid"),
            None
        );
    }

    #[test]
    fn test_extract_price_field_from_condition_id() {
        assert_eq!(
            ConverterHelpers::extract_price_field_from_condition_id("ind_price_sma_Close"),
            Some(PriceField::Close)
        );
        assert_eq!(
            ConverterHelpers::extract_price_field_from_condition_id("ind_price_ema_Open"),
            Some(PriceField::Open)
        );
        assert_eq!(
            ConverterHelpers::extract_price_field_from_condition_id("ind_price_"),
            Some(PriceField::Close)
        );
        assert_eq!(
            ConverterHelpers::extract_price_field_from_condition_id("other_id"),
            Some(PriceField::Close)
        );
    }
}
