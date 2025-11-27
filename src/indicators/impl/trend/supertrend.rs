use crate::indicators::{
    base::{Indicator, TrendDirection, TrendIndicator},
    impl_::common::{adjust_period, default_trend_direction},
    impl_::volatility::{ATR, WATR},
    parameters::{create_multiplier_parameter, create_period_parameter},
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};

pub struct SuperTrend {
    parameters: ParameterSet,
}

impl SuperTrend {
    pub fn new(period: f32, coeff_atr: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "coeff_atr",
                coeff_atr,
                "Коэффициент ATR для SuperTrend",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    fn calculate_median_price(&self, data: &OHLCData) -> Vec<f32> {
        data.get_median_price()
    }
}

impl Indicator for SuperTrend {
    fn name(&self) -> &str {
        "SuperTrend"
    }
    fn description(&self) -> &str {
        "SuperTrend - трендовый индикатор с полосами ATR (поддерживает Simple и OHLC данные)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Universal
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let atr_indicator = ATR::new(period as f32)?;
        let atr_values = atr_indicator.calculate_simple(data)?;
        let mut supertrend_values = vec![0.0; len];

        for i in 2..len {
            let atr = atr_values[i];
            let current_price = data[i];

            let upper_band = current_price + (coeff_atr * atr);
            let lower_band = current_price - (coeff_atr * atr);

            let prev_supertrend = supertrend_values[i - 1];
            let supertrend = if current_price >= prev_supertrend {
                if i > 0 && data[i - 1] < prev_supertrend {
                    lower_band
                } else {
                    if lower_band > prev_supertrend {
                        lower_band
                    } else {
                        prev_supertrend
                    }
                }
            } else if current_price < prev_supertrend {
                if i > 0 && data[i - 1] > prev_supertrend {
                    upper_band
                } else {
                    if upper_band < prev_supertrend {
                        upper_band
                    } else {
                        prev_supertrend
                    }
                }
            } else {
                prev_supertrend
            };

            supertrend_values[i] = supertrend;
        }

        Ok(supertrend_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let watr_indicator = WATR::new_unchecked(period as f32);
        let atr_values = watr_indicator.calculate_ohlc(data)?;

        let median_prices = self.calculate_median_price(data);
        let mut supertrend_values = vec![0.0; len];

        for i in 2..len {
            let atr = atr_values[i];
            let median_price = median_prices[i];

            let upper_band = median_price + (coeff_atr * atr);
            let lower_band = median_price - (coeff_atr * atr);

            let prev_supertrend = supertrend_values[i - 1];
            let current_close = data.close[i];
            let prev_close = data.close[i - 1];

            let supertrend = if current_close >= prev_supertrend {
                if prev_close < prev_supertrend {
                    lower_band
                } else if lower_band > prev_supertrend {
                    lower_band
                } else {
                    prev_supertrend
                }
            } else if current_close < prev_supertrend {
                if prev_close > prev_supertrend {
                    upper_band
                } else if upper_band < prev_supertrend {
                    upper_band
                } else {
                    prev_supertrend
                }
            } else {
                prev_supertrend
            };

            supertrend_values[i] = supertrend;
        }

        Ok(supertrend_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();
        Box::new(Self::new(period, coeff_atr).unwrap())
    }
}

impl TrendIndicator for SuperTrend {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        default_trend_direction(values)
    }
}
