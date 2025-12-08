use crate::data_model::vector_ops::unsafe_ops;
use crate::indicators::{
    base::{
        Indicator, IndicatorBuildRules, IndicatorCompareConfig, NestingConfig, PriceCompareConfig,
        ThresholdType, VolatilityIndicator,
    },
    impl_::common::adjust_period,
    parameters::create_period_parameter,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};
use crate::strategy::types::{ConditionOperator, PriceField};

pub struct ATR {
    parameters: ParameterSet,
}

impl ATR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    fn true_range_simple(&self, data: &[f32], period: usize, bar_num: usize) -> f32 {
        if period == 0 {
            return 0.0;
        }

        let available = bar_num + 1;
        let window = available.min(period);
        let start = bar_num + 1 - window;

        let mut sum = 0.0;
        for i in start..=bar_num {
            if i > 0 {
                let true_range = (data[i] - data[i - 1]).abs();
                sum += true_range;
            }
        }

        sum / window.max(1) as f32
    }

    fn true_range_ohlc(&self, data: &OHLCData, j: usize) -> f32 {
        let high_low = data.high[j] - data.low[j];
        let high_close_prev = if j > 0 {
            (data.high[j] - data.close[j - 1]).abs()
        } else {
            0.0
        };
        let low_close_prev = if j > 0 {
            (data.low[j] - data.close[j - 1]).abs()
        } else {
            0.0
        };

        high_low.max(high_close_prev).max(low_close_prev)
    }
}

impl Indicator for ATR {
    fn name(&self) -> &str {
        "ATR"
    }
    fn description(&self) -> &str {
        "Average True Range - средний истинный диапазон (поддерживает Simple и OHLC данные)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
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
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut atr_values = Vec::with_capacity(len);

        for i in 0..len {
            let atr = self.true_range_simple(data, period, i);
            atr_values.push(atr);
        }

        Ok(atr_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut atr_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            atr_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let start = i + 1 - period;
            let mut sum = 0.0;

            for j in start..=i {
                let true_range = self.true_range_ohlc(data, j);
                sum += true_range;
            }

            let atr = sum / period as f32;
            atr_values.push(atr);
        }

        Ok(atr_values)
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
            threshold_type: ThresholdType::PercentOfPrice {
                base_price_fields: &[PriceField::Close],
            },
            indicator_compare: IndicatorCompareConfig::DISABLED,
            nesting: NestingConfig::VOLATILITY,
            phase_1_allowed: false,
            supports_percent_condition: false,
            can_compare_with_input_source: false,
            can_compare_with_nested_result: true,
            nested_compare_conditions: &[
                ConditionOperator::Above,
                ConditionOperator::Below,
                ConditionOperator::GreaterPercent,
                ConditionOperator::LowerPercent,
            ],
        }
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl VolatilityIndicator for ATR {
    fn get_volatility_level(&self, data: &[f32]) -> Result<f32, IndicatorError> {
        let values = self.calculate_simple(data)?;
        Ok(values.last().copied().unwrap_or(0.0))
    }
}
