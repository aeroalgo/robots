use crate::condition::types::ConditionInputData;
use crate::condition::{base::*, helpers::ConditionHelpers, types::*};
use std::time::Instant;

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
            } => {
                let percent_value = percent.ok_or_else(|| {
                    ConditionError::InvalidParameter(
                        "GreaterPercent condition requires percent parameter".to_string(),
                    )
                })?;
                (primary, secondary, percent_value)
            }
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
                "GreaterPercentCondition требует два вектора".to_string(),
            )),
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

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
            } => {
                let percent_value = percent.ok_or_else(|| {
                    ConditionError::InvalidParameter(
                        "LowerPercent condition requires percent parameter".to_string(),
                    )
                })?;
                (primary, secondary, percent_value)
            }
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
                "LowerPercentCondition требует два вектора".to_string(),
            )),
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}
