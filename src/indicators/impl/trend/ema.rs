use crate::indicators::{
    base::{
        Indicator, IndicatorBuildRules, IndicatorCompareConfig, NestingConfig, PriceCompareConfig,
        SimpleIndicator, ThresholdType, TrendDirection, TrendIndicator,
    },
    impl_::common::default_trend_direction,
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};
use crate::strategy::types::ConditionOperator;

pub struct EMA {
    parameters: ParameterSet,
}

impl EMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета EMA",
        ));
        Self { parameters: params }
    }
}

impl Indicator for EMA {
    fn name(&self) -> &str {
        "EMA"
    }
    fn description(&self) -> &str {
        "Exponential Moving Average - экспоненциальное скользящее среднее"
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
        let period = self.parameters.get_value("period").unwrap();
        if period <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "EMA period must be greater than 0".to_string(),
            ));
        }

        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }

        let multiplier = 2.0 / (1.0 + period);
        let mut ema_values = Vec::with_capacity(len);

        ema_values.push(data[0]);

        for i in 1..len {
            let prev_ema = ema_values[i - 1];
            let price = data[i];
            ema_values.push(prev_ema + multiplier * (price - prev_ema));
        }

        Ok(ema_values)
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

impl SimpleIndicator for EMA {}

impl TrendIndicator for EMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}





