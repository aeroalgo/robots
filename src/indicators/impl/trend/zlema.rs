use crate::indicators::{
    base::{
        Indicator, SimpleIndicator, TrendDirection, TrendIndicator,
    },
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::default_trend_direction,
};

pub struct ZLEMA {
    parameters: ParameterSet,
}

impl ZLEMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ZLEMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for ZLEMA {
    fn name(&self) -> &str {
        "ZLEMA"
    }
    fn description(&self) -> &str {
        "Zero-Lag Exponential Moving Average - экспоненциальное скользящее среднее с нулевым лагом"
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

        let alpha = 2.0 / (period as f32 + 1.0);
        let lag = (period.saturating_sub(1)) / 2;
        let mut zlema_values = vec![0.0; len];

        for i in lag..len {
            let prev_zlema = if i == 0 {
                data[0]
            } else {
                zlema_values[i.saturating_sub(1)]
            };

            let reference_index = if i >= lag { i - lag } else { 0 };
            let current_price = data[i];
            let reference_price = data[reference_index];
            let ema_input = current_price + (current_price - reference_price);

            zlema_values[i] = prev_zlema + alpha * (ema_input - prev_zlema);
        }

        Ok(zlema_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for ZLEMA {}

impl TrendIndicator for ZLEMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}


