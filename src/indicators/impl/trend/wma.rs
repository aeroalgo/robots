use crate::indicators::{
    base::{Indicator, SimpleIndicator, TrendDirection, TrendIndicator},
    impl_::common::{adjust_period, default_trend_direction},
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};

pub struct WMA {
    parameters: ParameterSet,
}

impl WMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета WMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета WMA",
        ));
        Self { parameters: params }
    }
}

impl Indicator for WMA {
    fn name(&self) -> &str {
        "WMA"
    }
    fn description(&self) -> &str {
        "Weighted Moving Average - взвешенное скользящее среднее"
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

        let mut wma_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            wma_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            for (offset, value) in data[start..=i].iter().enumerate() {
                let weight = (offset + 1) as f32;
                weighted_sum += value * weight;
                weight_sum += weight;
            }

            wma_values.push(weighted_sum / weight_sum.max(1.0));
        }

        Ok(wma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for WMA {}

impl TrendIndicator for WMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}
