use crate::indicators::{
    base::{Indicator, SimpleIndicator, TrendDirection, TrendIndicator},
    impl_::common::default_trend_direction,
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};

pub struct AMA {
    parameters: ParameterSet,
}

impl AMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета AMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for AMA {
    fn name(&self) -> &str {
        "AMA"
    }
    fn description(&self) -> &str {
        "Adaptive Moving Average - адаптивное скользящее среднее"
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
        if len == 0 {
            return Ok(Vec::new());
        }

        if period == 0 {
            return Ok(vec![0.0; len]);
        }

        let mut ama_values = vec![0.0; len];
        let copy_limit = period.saturating_mul(2).min(len.saturating_sub(1));
        for i in 0..=copy_limit {
            ama_values[i] = data[i];
        }

        let mut ama_prev = if len < period || period + 1 >= len {
            0.0
        } else {
            data[period + 1]
        };

        for j in (period + 2)..len {
            if j < period || j >= len {
                continue;
            }

            let diff = (data[j] - data[j - period]).abs();
            let mut denom = 1e-9_f32;

            for k in 0..period {
                let idx = j - k;
                let prev_idx = idx - 1;
                denom += (data[idx] - data[prev_idx]).abs();
            }

            let efficiency = if denom <= 0.0 { 0.0 } else { diff / denom };
            let x = efficiency * 0.60215 + 0.06452;
            let smoothing = x * x;
            ama_prev = ama_prev + smoothing * (data[j] - ama_prev);
            ama_values[j] = ama_prev;
        }

        Ok(ama_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for AMA {}

impl TrendIndicator for AMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}
