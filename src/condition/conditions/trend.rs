use crate::condition::types::ConditionInputData;
use crate::condition::{base::*, helpers::ConditionHelpers, types::*};
use std::time::Instant;

pub struct RisingTrendCondition {
    period: usize,
    config: ConditionConfig,
}

impl RisingTrendCondition {
    pub fn new(period: f32) -> Result<Self, ConditionError> {
        let period_usize = period as usize;

        let config = ConditionConfig {
            name: "RisingTrend".to_string(),
            description: "Проверяет растущий тренд".to_string(),
            condition_type: ConditionType::Trend,
            category: ConditionCategory::Filter,
            min_data_points: period_usize,
            is_reversible: true,
            required_inputs: vec![
                ConditionInput::Single,
                ConditionInput::Dual,
                ConditionInput::Ohlc,
            ],
        };

        Ok(Self {
            period: period_usize,
            config,
        })
    }

    fn is_rising_trend(&self, data: &[f32]) -> bool {
        if data.len() < 2 {
            return false;
        }

        data.windows(2).all(|w| w[0] < w[1])
    }

    fn calculate_slope(&self, data: &[f32]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let first = data[0];
        let last = data[data.len() - 1];
        let period = data.len() as f32;

        (last - first) / (period - 1.0)
    }

    fn check_optimized(
        &self,
        data: &[f32],
    ) -> (Vec<bool>, Vec<SignalStrength>, Vec<TrendDirection>) {
        let period = self.period;
        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        if period <= 1 || data.len() < period {
            for _ in 0..data.len() {
                signals.push(false);
                strengths.push(SignalStrength::Weak);
                directions.push(TrendDirection::Sideways);
            }
            return (signals, strengths, directions);
        }

        for _ in 0..period - 1 {
            signals.push(false);
            strengths.push(SignalStrength::Weak);
            directions.push(TrendDirection::Sideways);
        }

        let mut rising_count = 0;
        for i in 1..period {
            if data[i - 1] < data[i] {
                rising_count += 1;
            }
        }

        let is_trend = rising_count == period - 1;
        let slope = if period > 1 {
            (data[period - 1] - data[0]) / ((period - 1) as f32)
        } else {
            0.0
        };
        let strength = if is_trend {
            ConditionHelpers::calculate_signal_strength(slope)
        } else {
            SignalStrength::Weak
        };
        signals.push(is_trend);
        strengths.push(strength);
        directions.push(ConditionHelpers::direction_from_signal(is_trend));

        for i in period..data.len() {
            if data[i - period] < data[i - period + 1] {
                rising_count -= 1;
            }
            if data[i - 1] < data[i] {
                rising_count += 1;
            }

            let is_trend = rising_count == period - 1;
            let start_idx = i + 1 - period;
            let slope = if period > 1 {
                (data[i] - data[start_idx]) / ((period - 1) as f32)
            } else {
                0.0
            };
            let strength = if is_trend {
                ConditionHelpers::calculate_signal_strength(slope)
            } else {
                SignalStrength::Weak
            };
            signals.push(is_trend);
            strengths.push(strength);
            directions.push(ConditionHelpers::direction_from_signal(is_trend));
        }

        (signals, strengths, directions)
    }
}

impl Condition for RisingTrendCondition {
    fn name(&self) -> &str {
        "RisingTrend"
    }

    fn description(&self) -> &str {
        "Проверяет растущий тренд"
    }

    fn config(&self) -> &ConditionConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.period
    }

    fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
        self.validate(&input)?;
        let data = match input {
            ConditionInputData::Single { data } => data,
            ConditionInputData::Dual { primary, .. } => primary,
            ConditionInputData::Ohlc { data } => data.close.as_slice(),
            _ => unreachable!("валидация должна была отклонить неподдерживаемый тип входа"),
        };

        let start_time = Instant::now();
        let (signals, strengths, directions) = self.check_optimized(data);

        let metadata =
            ConditionHelpers::create_condition_metadata(start_time.elapsed(), data.len(), 0.75);

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError> {
        let length = match input {
            ConditionInputData::Single { data } => data.len(),
            ConditionInputData::Dual { primary, .. } => primary.len(),
            ConditionInputData::Ohlc { data } => data.close.len(),
            _ => {
                return Err(ConditionError::InvalidParameter(
                    "RisingTrendCondition поддерживает только один основной вектор".to_string(),
                ))
            }
        };

        if length < self.min_data_points() {
            Err(ConditionError::InsufficientData {
                required: self.min_data_points(),
                actual: length,
            })
        } else {
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new(self.period as f32).unwrap())
    }
}

pub struct FallingTrendCondition {
    period: usize,
    config: ConditionConfig,
}

impl FallingTrendCondition {
    pub fn new(period: f32) -> Result<Self, ConditionError> {
        let period_usize = period as usize;

        let config = ConditionConfig {
            name: "FallingTrend".to_string(),
            description: "Проверяет падающий тренд".to_string(),
            condition_type: ConditionType::Trend,
            category: ConditionCategory::Filter,
            min_data_points: period_usize,
            is_reversible: true,
            required_inputs: vec![
                ConditionInput::Single,
                ConditionInput::Dual,
                ConditionInput::Ohlc,
            ],
        };

        Ok(Self {
            period: period_usize,
            config,
        })
    }

    fn is_falling_trend(&self, data: &[f32]) -> bool {
        if data.len() < 2 {
            return false;
        }

        data.windows(2).all(|w| w[0] > w[1])
    }

    fn calculate_slope(&self, data: &[f32]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let first = data[0];
        let last = data[data.len() - 1];
        let period = data.len() as f32;

        (first - last) / (period - 1.0)
    }

    fn check_optimized(
        &self,
        data: &[f32],
    ) -> (Vec<bool>, Vec<SignalStrength>, Vec<TrendDirection>) {
        let period = self.period;
        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        if period <= 1 || data.len() < period {
            for _ in 0..data.len() {
                signals.push(false);
                strengths.push(SignalStrength::Weak);
                directions.push(TrendDirection::Sideways);
            }
            return (signals, strengths, directions);
        }

        for _ in 0..period - 1 {
            signals.push(false);
            strengths.push(SignalStrength::Weak);
            directions.push(TrendDirection::Sideways);
        }

        let mut falling_count = 0;
        for i in 1..period {
            if data[i - 1] > data[i] {
                falling_count += 1;
            }
        }

        let is_trend = falling_count == period - 1;
        let slope = if period > 1 {
            (data[0] - data[period - 1]) / ((period - 1) as f32)
        } else {
            0.0
        };
        let strength = if is_trend {
            ConditionHelpers::calculate_signal_strength(slope)
        } else {
            SignalStrength::Weak
        };
        signals.push(is_trend);
        strengths.push(strength);
        directions.push(ConditionHelpers::direction_from_signal_reverse(is_trend));

        for i in period..data.len() {
            if data[i - period] > data[i - period + 1] {
                falling_count -= 1;
            }
            if data[i - 1] > data[i] {
                falling_count += 1;
            }

            let is_trend = falling_count == period - 1;
            let start_idx = i + 1 - period;
            let slope = if period > 1 {
                (data[start_idx] - data[i]) / ((period - 1) as f32)
            } else {
                0.0
            };
            let strength = if is_trend {
                ConditionHelpers::calculate_signal_strength(slope)
            } else {
                SignalStrength::Weak
            };
            signals.push(is_trend);
            strengths.push(strength);
            directions.push(ConditionHelpers::direction_from_signal_reverse(is_trend));
        }

        (signals, strengths, directions)
    }
}

impl Condition for FallingTrendCondition {
    fn name(&self) -> &str {
        "FallingTrend"
    }

    fn description(&self) -> &str {
        "Проверяет падающий тренд"
    }

    fn config(&self) -> &ConditionConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        self.period
    }

    fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
        self.validate(&input)?;
        let data = match input {
            ConditionInputData::Single { data } => data,
            ConditionInputData::Dual { primary, .. } => primary,
            ConditionInputData::Ohlc { data } => data.close.as_slice(),
            _ => unreachable!("валидация должна была отклонить неподдерживаемый тип входа"),
        };

        let start_time = Instant::now();
        let (signals, strengths, directions) = self.check_optimized(data);

        let metadata =
            ConditionHelpers::create_condition_metadata(start_time.elapsed(), data.len(), 0.75);

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError> {
        let length = match input {
            ConditionInputData::Single { data } => data.len(),
            ConditionInputData::Dual { primary, .. } => primary.len(),
            ConditionInputData::Ohlc { data } => data.close.len(),
            _ => {
                return Err(ConditionError::InvalidParameter(
                    "FallingTrendCondition поддерживает только один основной вектор".to_string(),
                ))
            }
        };

        if length < self.min_data_points() {
            Err(ConditionError::InsufficientData {
                required: self.min_data_points(),
                actual: length,
            })
        } else {
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new(self.period as f32).unwrap())
    }
}
