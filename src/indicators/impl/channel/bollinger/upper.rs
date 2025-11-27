use crate::data_model::vector_ops::unsafe_ops;
use crate::indicators::{
    base::{Indicator, SimpleIndicator},
    parameters::{create_period_parameter, create_multiplier_parameter},
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::adjust_period,
};

pub struct BBUpper {
    parameters: ParameterSet,
}

impl BBUpper {
    pub fn new(period: f32, deviation: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "deviation",
                deviation,
                "Стандартное отклонение множитель",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for BBUpper {
    fn name(&self) -> &str {
        "BBUpper"
    }
    fn description(&self) -> &str {
        "Bollinger Bands Upper Line (SMA + deviation)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
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
        let deviation = self.parameters.get_value("deviation").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut upper_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            upper_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let end = i + 1;
            let window = &data[start..end];
            let sma: f32 = unsafe_ops::sum_f32_fast(window) / current_window as f32;

            let variance: f32 =
                unsafe_ops::sum_sq_diff_f32_fast(window, sma) / current_window as f32;
            let std_dev = variance.sqrt();

            let upper = sma + (deviation * std_dev);
            upper_values.push(upper);
        }

        Ok(upper_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let deviation = self.parameters.get_value("deviation").unwrap();
        Box::new(Self::new(period, deviation).unwrap())
    }
}

impl SimpleIndicator for BBUpper {}

