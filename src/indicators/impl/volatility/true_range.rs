use crate::indicators::{
    base::{
        Indicator, IndicatorBuildRules, IndicatorCompareConfig, NestingConfig, PriceCompareConfig,
        ThresholdType, VolatilityIndicator,
    },
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};
use crate::strategy::types::{ConditionOperator, PriceField};

pub struct TrueRange {
    parameters: ParameterSet,
}

impl TrueRange {
    pub fn new() -> Result<Self, IndicatorError> {
        Ok(Self {
            parameters: ParameterSet::new(),
        })
    }

    pub fn new_unchecked() -> Self {
        Self {
            parameters: ParameterSet::new(),
        }
    }

    fn series(data: &OHLCData) -> Vec<f32> {
        let mut result = Vec::with_capacity(data.len());
        for idx in 0..data.len() {
            let high_low = data.high[idx] - data.low[idx];
            let high_close_prev = if idx > 0 {
                (data.high[idx] - data.close[idx - 1]).abs()
            } else {
                0.0
            };
            let low_close_prev = if idx > 0 {
                (data.low[idx] - data.close[idx - 1]).abs()
            } else {
                0.0
            };
            result.push(high_low.max(high_close_prev).max(low_close_prev));
        }
        result
    }
}

impl Indicator for TrueRange {
    fn name(&self) -> &str {
        "TrueRange"
    }
    fn description(&self) -> &str {
        "True Range - показатель истинного диапазона"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        1
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        Ok(Self::series(data))
    }

    fn build_rules(&self) -> IndicatorBuildRules {
        IndicatorBuildRules {
            allowed_conditions: &[
                ConditionOperator::Above,
                ConditionOperator::Below,
                ConditionOperator::RisingTrend,
                ConditionOperator::FallingTrend,
            ],
            price_compare: PriceCompareConfig::DISABLED,
            threshold_type: ThresholdType::PercentOfPrice {
                base_price_fields: &[PriceField::Close],
            },
            indicator_compare: IndicatorCompareConfig::DISABLED,
            nesting: NestingConfig::VOLATILITY,
            phase_1_allowed: false,
            supports_percent_condition: false,
            can_compare_with_input_source: false,
            can_compare_with_nested_result: true,
            nested_compare_conditions: &[
                ConditionOperator::Above,
                ConditionOperator::Below,
                ConditionOperator::GreaterPercent,
                ConditionOperator::LowerPercent,
            ],
        }
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

impl VolatilityIndicator for TrueRange {
    fn get_volatility_level(&self, _data: &[f32]) -> Result<f32, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }
}
