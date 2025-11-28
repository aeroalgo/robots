use crate::indicators::{
    base::{
        Indicator, SimpleIndicator, TrendDirection, TrendIndicator,
    },
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::{adjust_period, default_trend_direction},
};

pub struct SINEWMA {
    parameters: ParameterSet,
}

impl SINEWMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SINEWMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for SINEWMA {
    fn name(&self) -> &str {
        "SINEWMA"
    }
    fn description(&self) -> &str {
        "Sine Weighted Moving Average - синусоидально-взвешенное скользящее среднее"
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

        let mut sinewma_values = Vec::with_capacity(len);

        for i in 0..len {
            if i < period {
                sinewma_values.push(data[i]);
                continue;
            }

            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            let denom = period as f32 + 1.0;

            for j in 0..period.saturating_sub(1) {
                let weight = (std::f32::consts::PI * (j as f32 + 1.0) / denom).sin();
                let value = data[i - j];
                weighted_sum += value * weight;
                weight_sum += weight;
            }

            if weight_sum > 0.0 {
                sinewma_values.push(weighted_sum / weight_sum);
            } else {
                sinewma_values.push(0.0);
            }
        }

        Ok(sinewma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SINEWMA {}

impl TrendIndicator for SINEWMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}


