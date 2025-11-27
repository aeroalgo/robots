use crate::indicators::{
    base::{Indicator, OHLCIndicator},
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::adjust_period,
};

pub struct KCMiddle {
    parameters: ParameterSet,
}

impl KCMiddle {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for KCMiddle {
    fn name(&self) -> &str {
        "KCMiddle"
    }
    fn description(&self) -> &str {
        "Keltner Channel Middle Line (EMA)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
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

        let mut ema_values = Vec::with_capacity(len);
        let mut ema = 0.0;

        for i in 0..len {
            let high = data.high[i];
            let low = data.low[i];
            let close = data.close[i];

            let typical_price = (high + low + close) / 3.0;

            if i == 0 {
                ema = typical_price;
            } else {
                let multiplier = 2.0 / (period as f32 + 1.0);
                ema = (typical_price * multiplier) + (ema * (1.0 - multiplier));
            }

            ema_values.push(ema);
        }

        Ok(ema_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        Box::new(Self::new(period).unwrap())
    }
}

impl OHLCIndicator for KCMiddle {}
