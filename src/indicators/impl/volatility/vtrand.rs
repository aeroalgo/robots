use crate::indicators::{
    base::Indicator,
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::auxiliary::{MAXFOR, MINFOR},
};

pub struct VTRAND {
    parameters: ParameterSet,
}

impl VTRAND {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета VTRAND",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for VTRAND {
    fn name(&self) -> &str {
        "VTRAND"
    }
    fn description(&self) -> &str {
        "VTRAND - среднее между максимальным и минимальным значением за период"
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

        let max_indicator = MAXFOR::new_unchecked(period as f32);
        let min_indicator = MINFOR::new_unchecked(period as f32);

        let max_result = max_indicator.calculate_ohlc(data)?;
        let min_result = min_indicator.calculate_ohlc(data)?;

        let vtrand_values: Vec<f32> = max_result
            .into_iter()
            .zip(min_result)
            .map(|(max_val, min_val)| (max_val + min_val) / 2.0)
            .collect();

        Ok(vtrand_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}





