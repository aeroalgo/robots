use crate::indicators::{
    base::{
        Indicator, SimpleIndicator, TrendDirection, TrendIndicator,
    },
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::{adjust_period, default_trend_direction},
};

pub struct SQWMA {
    parameters: ParameterSet,
}

impl SQWMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SQWMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for SQWMA {
    fn name(&self) -> &str {
        "SQWMA"
    }
    fn description(&self) -> &str {
        "Square Root Weighted Moving Average - квадратично-взвешенное скользящее среднее"
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

        let mut sqwma_values = Vec::with_capacity(len);

        let period_f = period as f32;
        let num = period_f * (period_f - 1.0) / 2.0;
        let num2 = period_f * (period_f - 1.0) * (2.0 * period_f - 1.0) / 6.0;

        for i in 0..len {
            if i < period {
                sqwma_values.push(data[i]);
                continue;
            }

            let mut sum = 0.0;
            let mut weighted_sum = 0.0;

            for j in 0..period {
                let value = data[i - j];
                let j_f = j as f32;
                sum += value;
                weighted_sum += value * j_f;
            }

            let denom = num2 * period_f - num * num;
            let slope = if denom.abs() < f32::EPSILON {
                0.0
            } else {
                (weighted_sum * period_f - num * sum) / denom
            };
            let intercept = (sum - num * slope) / period_f;
            sqwma_values.push(intercept);
        }

        Ok(sqwma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SQWMA {}

impl TrendIndicator for SQWMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}


