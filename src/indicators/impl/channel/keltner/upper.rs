use crate::indicators::{
    base::{Indicator, OHLCIndicator},
    parameters::{create_period_parameter, create_multiplier_parameter},
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::common::adjust_period,
    impl_::channel::keltner::KCMiddle,
    impl_::volatility::ATR,
};

pub struct KCUpper {
    parameters: ParameterSet,
}

impl KCUpper {
    pub fn new(period: f32, atr_period: f32, atr_multiplier: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_period_parameter(
                "atr_period",
                atr_period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "atr_multiplier",
                atr_multiplier,
                "Множитель ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for KCUpper {
    fn name(&self) -> &str {
        "KCUpper"
    }
    fn description(&self) -> &str {
        "Keltner Channel Upper Line (EMA + ATR * multiplier)"
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
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        std::cmp::max(period, atr_period)
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();
        let len = data.len();
        let Some(ema_period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let Some(atr_period) = adjust_period(atr_period, len) else {
            return Ok(Vec::new());
        };

        let middle_indicator = KCMiddle::new(ema_period as f32).unwrap();
        let middle_values = middle_indicator.calculate_ohlc(data)?;

        let atr_indicator = ATR::new(atr_period as f32).unwrap();
        let atr_values = atr_indicator.calculate_ohlc(data)?;

        let mut upper_values = Vec::with_capacity(len);
        for i in 0..len {
            let middle = middle_values.get(i).copied().unwrap_or(0.0);
            let atr = atr_values.get(i).copied().unwrap_or(0.0);
            upper_values.push(middle + (atr * atr_multiplier));
        }

        Ok(upper_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let atr_period = self.parameters.get_value("atr_period").unwrap();
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();
        Box::new(Self::new(period, atr_period, atr_multiplier).unwrap())
    }
}

impl OHLCIndicator for KCUpper {}

