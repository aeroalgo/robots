use crate::indicators::{
    base::{
        Indicator, IndicatorBuildRules, IndicatorCompareConfig, NestingConfig, OscillatorIndicator,
        OverboughtOversoldZones, PriceCompareConfig, ThresholdType,
    },
    impl_::common::adjust_period,
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};
use crate::strategy::types::ConditionOperator;

pub struct Stochastic {
    parameters: ParameterSet,
}

impl Stochastic {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета Stochastic",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for Stochastic {
    fn name(&self) -> &str {
        "Stochastic"
    }
    fn description(&self) -> &str {
        "Stochastic - стохастический осциллятор"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillator
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
        let mut stochastic_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            stochastic_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let start_idx = i + 1 - period;
            let end_idx = i + 1;

            let highest_high = data.high[start_idx..end_idx]
                .iter()
                .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            let lowest_low = data.low[start_idx..end_idx]
                .iter()
                .fold(f32::INFINITY, |a, &b| a.min(b));

            let current_close = data.close[i];

            let stochastic = if highest_high == lowest_low {
                50.0
            } else {
                ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
            };

            stochastic_values.push(stochastic);
        }

        Ok(stochastic_values)
    }

    fn build_rules(&self) -> IndicatorBuildRules {
        IndicatorBuildRules {
            allowed_conditions: &[
                ConditionOperator::Above,
                ConditionOperator::Below,
                ConditionOperator::RisingTrend,
                ConditionOperator::FallingTrend,
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

impl OscillatorIndicator for Stochastic {
    fn get_overbought_oversold_zones(
        &self,
        data: &[f32],
    ) -> Result<OverboughtOversoldZones, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(current_value) = values.last() {
            Ok(OverboughtOversoldZones::new(80.0, 20.0, *current_value))
        } else {
            Err(IndicatorError::CalculationError(
                "No values calculated".to_string(),
            ))
        }
    }
}
