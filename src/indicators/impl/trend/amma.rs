use crate::data_model::vector_ops::unsafe_ops;
use crate::indicators::{
    base::{
        Indicator, IndicatorBuildRules, IndicatorCompareConfig, NestingConfig, PriceCompareConfig,
        SimpleIndicator, ThresholdType, TrendDirection, TrendIndicator,
    },
    impl_::common::{adjust_period, default_trend_direction},
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};
use crate::strategy::types::ConditionOperator;

pub struct AMMA {
    parameters: ParameterSet,
}

impl AMMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета AMMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for AMMA {
    fn name(&self) -> &str {
        "AMMA"
    }
    fn description(&self) -> &str {
        "Arithmetic Mean of Moving Averages - арифметическое среднее скользящих средних"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let window_double = (period.saturating_mul(2)).max(1).min(len);
        let mut amma_values = Vec::with_capacity(len);

        for _ in 0..window_double.saturating_sub(1) {
            amma_values.push(0.0);
        }

        for i in window_double - 1..len {
            let current_window = (period.saturating_mul(2)).min(i + 1);
            let start = i + 1 - current_window;
            let slice = &data[start..=i];

            let sma1_window = period.min(slice.len());
            let sma1_start = slice.len().saturating_sub(sma1_window);
            let sma1_sum = unsafe_ops::sum_f32_fast(&slice[sma1_start..]);
            let sma1_value = sma1_sum / sma1_window as f32;

            let sma2_window = (period.saturating_mul(2)).min(slice.len());
            let sma2_start = slice.len().saturating_sub(sma2_window);
            let sma2_sum = unsafe_ops::sum_f32_fast(&slice[sma2_start..]);
            let sma2_value = sma2_sum / sma2_window as f32;

            let amma = (sma1_value + sma2_value) / 2.0;
            amma_values.push(amma);
        }

        Ok(amma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn build_rules(&self) -> IndicatorBuildRules {
        IndicatorBuildRules {
            allowed_conditions: &[
                ConditionOperator::Above,
                ConditionOperator::Below,
                ConditionOperator::RisingTrend,
                ConditionOperator::FallingTrend,
                ConditionOperator::GreaterPercent,
                ConditionOperator::LowerPercent,
            ],
            price_compare: PriceCompareConfig::CLOSE_ONLY,
            threshold_type: ThresholdType::None,
            indicator_compare: IndicatorCompareConfig::TREND_AND_CHANNEL,
            nesting: NestingConfig::TREND,
            phase_1_allowed: true,
            supports_percent_condition: true,
            can_compare_with_input_source: true,
            can_compare_with_nested_result: true,
            nested_compare_conditions: &[],
        }
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for AMMA {}

impl TrendIndicator for AMMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}
