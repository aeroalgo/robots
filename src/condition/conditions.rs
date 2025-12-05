use crate::condition::types::{ConditionInput, ConditionInputData};
use crate::condition::{base::*, types::*};
use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// Условия сравнения
// ============================================================================

/// Условие "выше другого вектора"
pub struct AboveCondition {
    config: ConditionConfig,
}

impl AboveCondition {
    pub fn new() -> Result<Self, ConditionError> {
        let config = ConditionConfig {
            name: "Above".to_string(),
            description: "Проверяет, что первый вектор выше второго".to_string(),
            condition_type: ConditionType::Comparison,
            category: ConditionCategory::Filter,
            min_data_points: 2,
            is_reversible: true,
            required_inputs: vec![ConditionInput::Dual],
        };

        Ok(Self { config })
    }
}

impl Condition for AboveCondition {
    fn name(&self) -> &str {
        "Above"
    }

    fn description(&self) -> &str {
        "Проверяет, что первый вектор выше второго"
    }

    fn config(&self) -> &ConditionConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        2
    }

    fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
        self.validate(&input)?;
        let (data1, data2) = match input {
            ConditionInputData::Dual {
                primary, secondary, ..
            } => (primary, secondary),
            _ => unreachable!("валидация должна была отклонить неподдерживаемый тип входа"),
        };

        let start_time = Instant::now();
        let min_len = std::cmp::min(data1.len(), data2.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let signal = data1[i] > data2[i];
            signals.push(signal);

            let strength = if signal {
                let diff = (data1[i] - data2[i]) / data2[i];
                if diff > 0.1 {
                    SignalStrength::VeryStrong
                } else if diff > 0.05 {
                    SignalStrength::Strong
                } else if diff > 0.02 {
                    SignalStrength::Medium
                } else {
                    SignalStrength::Weak
                }
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(if signal {
                TrendDirection::Rising
            } else {
                TrendDirection::Falling
            });
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: min_len,
            confidence_score: 0.8,
            additional_info: HashMap::new(),
        };

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError> {
        match input {
            ConditionInputData::Dual {
                primary, secondary, ..
            } => {
                if primary.len() < self.min_data_points()
                    || secondary.len() < self.min_data_points()
                {
                    Err(ConditionError::InsufficientData {
                        required: self.min_data_points(),
                        actual: std::cmp::min(primary.len(), secondary.len()),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Err(ConditionError::InvalidParameter(
                "AboveCondition требует два вектора".to_string(),
            )),
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

/// Условие "выше на процент"
pub struct GreaterPercentCondition {
    config: ConditionConfig,
}

impl GreaterPercentCondition {
    pub fn new() -> Result<Self, ConditionError> {
        let config = ConditionConfig {
            name: "GreaterPercent".to_string(),
            description: "Проверяет, что первый вектор выше второго на указанный процент"
                .to_string(),
            condition_type: ConditionType::Percentage,
            category: ConditionCategory::Filter,
            min_data_points: 2,
            is_reversible: true,
            required_inputs: vec![ConditionInput::DualWithPercent],
        };

        Ok(Self { config })
    }
}

impl Condition for GreaterPercentCondition {
    fn name(&self) -> &str {
        "GreaterPercent"
    }

    fn description(&self) -> &str {
        "Проверяет, что первый вектор выше второго на указанный процент"
    }

    fn config(&self) -> &ConditionConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        2
    }

    fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
        self.validate(&input)?;
        let (data1, data2, percent) = match input {
            ConditionInputData::Dual {
                primary,
                secondary,
                percent,
            } => (primary, secondary, percent.unwrap_or(0.0)),
            _ => unreachable!("валидация должна была отклонить неподдерживаемый тип входа"),
        };

        let start_time = Instant::now();
        let min_len = std::cmp::min(data1.len(), data2.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let target = data2[i] * (1.0 + percent / 100.0);
            let signal = data1[i] > target;
            signals.push(signal);

            let strength = if signal {
                let diff = (data1[i] - target) / target;
                if diff > 0.1 {
                    SignalStrength::VeryStrong
                } else if diff > 0.05 {
                    SignalStrength::Strong
                } else if diff > 0.02 {
                    SignalStrength::Medium
                } else {
                    SignalStrength::Weak
                }
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(if signal {
                TrendDirection::Rising
            } else {
                TrendDirection::Falling
            });
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: min_len,
            confidence_score: 0.8,
            additional_info: HashMap::new(),
        };

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError> {
        match input {
            ConditionInputData::Dual {
                primary, secondary, ..
            } => {
                if primary.len() < self.min_data_points()
                    || secondary.len() < self.min_data_points()
                {
                    Err(ConditionError::InsufficientData {
                        required: self.min_data_points(),
                        actual: std::cmp::min(primary.len(), secondary.len()),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Err(ConditionError::InvalidParameter(
                "GreaterPercentCondition требует два вектора".to_string(),
            )),
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

/// Условие "ниже на процент"
pub struct LowerPercentCondition {
    config: ConditionConfig,
}

impl LowerPercentCondition {
    pub fn new() -> Result<Self, ConditionError> {
        let config = ConditionConfig {
            name: "LowerPercent".to_string(),
            description: "Проверяет, что первый вектор ниже второго на указанный процент"
                .to_string(),
            condition_type: ConditionType::Percentage,
            category: ConditionCategory::Filter,
            min_data_points: 2,
            is_reversible: true,
            required_inputs: vec![ConditionInput::DualWithPercent],
        };

        Ok(Self { config })
    }
}

impl Condition for LowerPercentCondition {
    fn name(&self) -> &str {
        "LowerPercent"
    }

    fn description(&self) -> &str {
        "Проверяет, что первый вектор ниже второго на указанный процент"
    }

    fn config(&self) -> &ConditionConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        2
    }

    fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
        self.validate(&input)?;
        let (data1, data2, percent) = match input {
            ConditionInputData::Dual {
                primary,
                secondary,
                percent,
            } => (primary, secondary, percent.unwrap_or(0.0)),
            _ => unreachable!("валидация должна была отклонить неподдерживаемый тип входа"),
        };

        let start_time = Instant::now();
        let min_len = std::cmp::min(data1.len(), data2.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let target = data2[i] * (1.0 - percent / 100.0);
            let signal = data1[i] < target;
            signals.push(signal);

            let strength = if signal {
                let diff = (target - data1[i]) / target;
                if diff > 0.1 {
                    SignalStrength::VeryStrong
                } else if diff > 0.05 {
                    SignalStrength::Strong
                } else if diff > 0.02 {
                    SignalStrength::Medium
                } else {
                    SignalStrength::Weak
                }
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(if signal {
                TrendDirection::Falling
            } else {
                TrendDirection::Rising
            });
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: min_len,
            confidence_score: 0.8,
            additional_info: HashMap::new(),
        };

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError> {
        match input {
            ConditionInputData::Dual {
                primary, secondary, ..
            } => {
                if primary.len() < self.min_data_points()
                    || secondary.len() < self.min_data_points()
                {
                    Err(ConditionError::InsufficientData {
                        required: self.min_data_points(),
                        actual: std::cmp::min(primary.len(), secondary.len()),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Err(ConditionError::InvalidParameter(
                "LowerPercentCondition требует два вектора".to_string(),
            )),
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

// ============================================================================
// Трендовые условия
// ============================================================================

/// Условие "растущий тренд"
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
        let period = self.period;
        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        if period <= 1 {
            for _ in 0..data.len() {
                signals.push(false);
                strengths.push(SignalStrength::Weak);
                directions.push(TrendDirection::Sideways);
            }
        } else {
            for _ in 0..period - 1 {
                signals.push(false);
                strengths.push(SignalStrength::Weak);
                directions.push(TrendDirection::Sideways);
            }

            for i in period - 1..data.len() {
                let start = i + 1 - period;
                let trend_slice = &data[start..=i];
                let signal = self.is_rising_trend(trend_slice);
                signals.push(signal);

                let strength = if signal {
                    let slope = self.calculate_slope(trend_slice);
                    if slope > 0.1 {
                        SignalStrength::VeryStrong
                    } else if slope > 0.05 {
                        SignalStrength::Strong
                    } else if slope > 0.02 {
                        SignalStrength::Medium
                    } else {
                        SignalStrength::Weak
                    }
                } else {
                    SignalStrength::Weak
                };
                strengths.push(strength);

                directions.push(if signal {
                    TrendDirection::Rising
                } else {
                    TrendDirection::Falling
                });
            }
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: data.len(),
            confidence_score: 0.75,
            additional_info: HashMap::new(),
        };

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

/// Условие "ниже другого вектора"
pub struct BelowCondition {
    config: ConditionConfig,
}

impl BelowCondition {
    pub fn new() -> Result<Self, ConditionError> {
        let config = ConditionConfig {
            name: "Below".to_string(),
            description: "Проверяет, что первый вектор ниже второго".to_string(),
            condition_type: ConditionType::Comparison,
            category: ConditionCategory::Filter,
            min_data_points: 2,
            is_reversible: true,
            required_inputs: vec![ConditionInput::Dual],
        };

        Ok(Self { config })
    }
}

impl Condition for BelowCondition {
    fn name(&self) -> &str {
        "Below"
    }

    fn description(&self) -> &str {
        "Проверяет, что первый вектор ниже второго"
    }

    fn config(&self) -> &ConditionConfig {
        &self.config
    }

    fn min_data_points(&self) -> usize {
        2
    }

    fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
        self.validate(&input)?;
        let (data1, data2) = match input {
            ConditionInputData::Dual {
                primary, secondary, ..
            } => (primary, secondary),
            _ => unreachable!("валидация должна была отклонить неподдерживаемый тип входа"),
        };

        let start_time = Instant::now();
        let min_len = std::cmp::min(data1.len(), data2.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let signal = data1[i] < data2[i];
            signals.push(signal);

            let strength = if signal {
                let diff = (data2[i] - data1[i]) / data2[i];
                if diff > 0.1 {
                    SignalStrength::VeryStrong
                } else if diff > 0.05 {
                    SignalStrength::Strong
                } else if diff > 0.02 {
                    SignalStrength::Medium
                } else {
                    SignalStrength::Weak
                }
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(if signal {
                TrendDirection::Falling
            } else {
                TrendDirection::Rising
            });
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: min_len,
            confidence_score: 0.8,
            additional_info: HashMap::new(),
        };

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError> {
        match input {
            ConditionInputData::Dual {
                primary, secondary, ..
            } => {
                if primary.len() < self.min_data_points()
                    || secondary.len() < self.min_data_points()
                {
                    Err(ConditionError::InsufficientData {
                        required: self.min_data_points(),
                        actual: std::cmp::min(primary.len(), secondary.len()),
                    })
                } else {
                    Ok(())
                }
            }
            _ => Err(ConditionError::InvalidParameter(
                "BelowCondition требует два вектора".to_string(),
            )),
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

/// Условие "падающий тренд"
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
        let period = self.period;
        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        if period <= 1 {
            for _ in 0..data.len() {
                signals.push(false);
                strengths.push(SignalStrength::Weak);
                directions.push(TrendDirection::Sideways);
            }
        } else {
            for _ in 0..period - 1 {
                signals.push(false);
                strengths.push(SignalStrength::Weak);
                directions.push(TrendDirection::Sideways);
            }

            for i in period - 1..data.len() {
                let start = i + 1 - period;
                let trend_slice = &data[start..=i];
                let signal = self.is_falling_trend(trend_slice);
                signals.push(signal);

                let strength = if signal {
                    let slope = self.calculate_slope(trend_slice);
                    if slope > 0.1 {
                        SignalStrength::VeryStrong
                    } else if slope > 0.05 {
                        SignalStrength::Strong
                    } else if slope > 0.02 {
                        SignalStrength::Medium
                    } else {
                        SignalStrength::Weak
                    }
                } else {
                    SignalStrength::Weak
                };
                strengths.push(strength);

                directions.push(if signal {
                    TrendDirection::Falling
                } else {
                    TrendDirection::Rising
                });
            }
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: data.len(),
            confidence_score: 0.75,
            additional_info: HashMap::new(),
        };

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
