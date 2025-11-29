use crate::indicators::{
    base::{Indicator, VolatilityIndicator},
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::adjust_period,
    impl_::volatility::TrueRange,
    impl_::trend::WMA,
};

pub struct WATR {
    parameters: ParameterSet,
}

impl WATR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета WATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета WATR",
        ));
        Self { parameters: params }
    }
}

impl Indicator for WATR {
    fn name(&self) -> &str {
        "WATR"
    }
    fn description(&self) -> &str {
        "Weighted Average True Range - средний истинный диапазон на основе WMA"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
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

        let tr_indicator = TrueRange::new_unchecked();
        let true_ranges = tr_indicator.calculate_ohlc(data)?;
        let wma_indicator = WMA::new_unchecked(period as f32);
        wma_indicator.calculate_simple(&true_ranges)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl VolatilityIndicator for WATR {
    fn get_volatility_level(&self, _data: &[f32]) -> Result<f32, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }
}




