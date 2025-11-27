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

pub struct SMA {
    parameters: ParameterSet,
}

impl SMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета SMA",
        ));
        Self { parameters: params }
    }
}

impl Indicator for SMA {
    fn name(&self) -> &str {
        "SMA"
    }
    fn description(&self) -> &str {
        "Simple Moving Average - простое скользящее среднее"
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

        let mut sma_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            sma_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let sum: f32 = unsafe_ops::sum_f32_fast(&data[start..=i]);
            sma_values.push(sum / current_window as f32);
        }

        Ok(sma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn build_rules(&self) -> IndicatorBuildRules {
        IndicatorBuildRules {
            allowed_conditions: &[
                "Above",
                "Below",
                "CrossesAbove",
                "CrossesBelow",
                "RisingTrend",
                "FallingTrend",
                "GreaterPercent",
                "LowerPercent",
            ],
            price_compare: PriceCompareConfig::STANDARD,
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

impl SimpleIndicator for SMA {}

impl TrendIndicator for SMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}
