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

pub struct TPBF {
    parameters: ParameterSet,
}

impl TPBF {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета TPBF",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    fn filter(values: &[f32], period: usize) -> Vec<f32> {
        let len = values.len();
        if len == 0 {
            return Vec::new();
        }
        if period == 0 {
            return values.to_vec();
        }

        let pi = std::f32::consts::PI;
        let period_f = period as f32;
        let exp_term = (-pi / period_f).exp();
        let exp_term_sq = exp_term * exp_term;
        let cos_term = (pi * 3.0_f32.sqrt() / period_f).cos();

        let coef1 = (1.0 - 2.0 * exp_term * cos_term + exp_term_sq) * (1.0 - exp_term_sq) / 8.0;
        let coef2 = 2.0 * exp_term * cos_term + exp_term_sq;
        let coef3 = -(exp_term_sq + 2.0 * exp_term.powi(3) * cos_term);
        let coef4 = exp_term_sq * exp_term_sq;

        let mut result = vec![0.0; len];

        for i in 0..len {
            if i < 4 {
                result[i] = values[i];
            } else {
                let price_i = values[i];
                let price_1 = values[i - 1];
                let price_2 = values[i - 2];
                let price_3 = values[i - 3];

                result[i] = coef1 * (price_i + 3.0 * (price_1 + price_2) + price_3)
                    + coef2 * result[i - 1]
                    + coef3 * result[i - 2]
                    + coef4 * result[i - 3];
            }
        }

        result
    }
}

impl Indicator for TPBF {
    fn name(&self) -> &str {
        "TPBF"
    }
    fn description(&self) -> &str {
        "Three Pole Butterworth Filter - трехполюсный фильтр Баттерворта"
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
        Ok(Self::filter(data, period))
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let median_prices = data
            .high
            .iter()
            .zip(&data.low)
            .map(|(&h, &l)| (h + l) / 2.0)
            .collect::<Vec<_>>();
        Ok(Self::filter(&median_prices, period))
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

impl SimpleIndicator for TPBF {}

impl TrendIndicator for TPBF {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}





