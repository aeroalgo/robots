use crate::data_model::vector_ops::unsafe_ops;
use crate::indicators::{
    base::Indicator,
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::adjust_period,
};

pub struct MINFOR {
    parameters: ParameterSet,
}

impl MINFOR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета MINFOR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета MINFOR",
        ));
        Self { parameters: params }
    }
}

impl Indicator for MINFOR {
    fn name(&self) -> &str {
        "MINFOR"
    }
    fn description(&self) -> &str {
        "Минимальное значение за период"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut min_values = Vec::with_capacity(len);

        for i in 0..len {
            let window = period.min(i + 1);
            let start_idx = i + 1 - window;
            let end_idx = i + 1;
            let min_value =
                unsafe_ops::min_f32_fast(&data.low[start_idx..end_idx]).unwrap_or(f32::INFINITY);
            min_values.push(min_value);
        }

        Ok(min_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}






