use crate::indicators::{
    base::{
        Indicator, IndicatorBuildRules, IndicatorCompareConfig, NestingConfig,
        OscillatorIndicator, OverboughtOversoldZones, PriceCompareConfig,
        SimpleIndicator, ThresholdType,
    },
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
    impl_::trend::EMA,
};

pub struct RSI {
    parameters: ParameterSet,
}

impl RSI {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета RSI",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        "RSI"
    }
    fn description(&self) -> &str {
        "Relative Strength Index - индекс относительной силы"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillator
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
        let mut period = self.parameters.get_value("period").unwrap() as usize;
        if period == 0 {
            period = 1;
        }

        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }
        period = period.min(len);

        let mut gains = vec![0.0; len];
        let mut losses = vec![0.0; len];

        for i in 1..len {
            let change = data[i] - data[i - 1];
            if change > 0.0 {
                gains[i] = change;
            } else if change < 0.0 {
                losses[i] = -change;
            }
        }

        let ema_gains = EMA::new_unchecked(period as f32).calculate_simple(&gains)?;
        let ema_losses = EMA::new_unchecked(period as f32).calculate_simple(&losses)?;

        let mut rsi_values = vec![0.0; len];

        for i in 0..len {
            let gain = ema_gains[i];
            let loss = ema_losses[i];

            if loss == 0.0 {
                rsi_values[i] = 100.0;
            } else {
                let rs = gain / loss;
                if (rs - 1.0).abs() < f32::EPSILON {
                    rsi_values[i] = 0.0;
                } else {
                    rsi_values[i] = 100.0 - 100.0 / (1.0 + rs);
                }
            }
        }

        Ok(rsi_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        self.calculate_simple(&data.close)
    }

    fn build_rules(&self) -> IndicatorBuildRules {
        IndicatorBuildRules {
            allowed_conditions: &[
                "Above",
                "Below",
                "CrossesAbove",
                "CrossesBelow",
                "RisingTrend",
                "FallingTrend",
            ],
            price_compare: PriceCompareConfig::DISABLED,
            threshold_type: ThresholdType::Absolute,
            indicator_compare: IndicatorCompareConfig::DISABLED,
            nesting: NestingConfig::OSCILLATOR,
            phase_1_allowed: true,
            supports_percent_condition: false,
            can_compare_with_input_source: false,
            can_compare_with_nested_result: true,
            nested_compare_conditions: &[],
        }
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for RSI {}

impl OscillatorIndicator for RSI {
    fn get_overbought_oversold_zones(
        &self,
        data: &[f32],
    ) -> Result<OverboughtOversoldZones, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(current_value) = values.last() {
            Ok(OverboughtOversoldZones::new(70.0, 30.0, *current_value))
        } else {
            Err(IndicatorError::CalculationError(
                "No values calculated".to_string(),
            ))
        }
    }
}

