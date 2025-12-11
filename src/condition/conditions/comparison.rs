use crate::condition::types::ConditionInputData;
use crate::condition::{base::*, helpers::ConditionHelpers, types::*};
use std::time::Instant;

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
                ConditionHelpers::calculate_signal_strength(diff)
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(ConditionHelpers::direction_from_signal(signal));
        }

        let metadata =
            ConditionHelpers::create_condition_metadata(start_time.elapsed(), min_len, 0.8);

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
                ConditionHelpers::calculate_signal_strength(diff)
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(ConditionHelpers::direction_from_signal_reverse(signal));
        }

        let metadata =
            ConditionHelpers::create_condition_metadata(start_time.elapsed(), min_len, 0.8);

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

