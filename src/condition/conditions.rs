use crate::condition::{base::*, types::*};
use async_trait::async_trait;
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
        };

        Ok(Self { config })
    }
}

#[async_trait]
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

    async fn check_simple(&self, _data: &[f32]) -> ConditionResult<ConditionResultData> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    async fn check_ohlc(
        &self,
        _data: &crate::indicators::types::OHLCData,
    ) -> ConditionResult<ConditionResultData> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    async fn check_dual(
        &self,
        data1: &[f32],
        data2: &[f32],
    ) -> ConditionResult<ConditionResultData> {
        // Сравниваем data1 с data2 как порогом
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

    async fn check_single(&self, _index: usize, _data: &[f32]) -> ConditionResult<bool> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    fn validate_input_data(&self, data: &[f32]) -> Result<(), ConditionError> {
        if data.len() < self.min_data_points() {
            Err(ConditionError::InsufficientData {
                required: self.min_data_points(),
                actual: data.len(),
            })
        } else {
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

// ============================================================================
// Процентные условия
// ============================================================================

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
        };

        Ok(Self { config })
    }
}

#[async_trait]
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

    async fn check_simple(&self, _data: &[f32]) -> ConditionResult<ConditionResultData> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    async fn check_ohlc(
        &self,
        _data: &crate::indicators::types::OHLCData,
    ) -> ConditionResult<ConditionResultData> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    async fn check_dual(
        &self,
        data1: &[f32],
        data2: &[f32],
    ) -> ConditionResult<ConditionResultData> {
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

    async fn check_single(&self, _index: usize, _data: &[f32]) -> ConditionResult<bool> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    fn validate_input_data(&self, data: &[f32]) -> Result<(), ConditionError> {
        if data.len() < self.min_data_points() {
            Err(ConditionError::InsufficientData {
                required: self.min_data_points(),
                actual: data.len(),
            })
        } else {
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

#[async_trait]
impl PercentageCondition for GreaterPercentCondition {
    async fn greater_percent(
        &self,
        data1: &[f32],
        data2: &[f32],
        percent: f32,
    ) -> ConditionResult<ConditionResultData> {
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

    async fn lower_percent(
        &self,
        data1: &[f32],
        data2: &[f32],
        percent: f32,
    ) -> ConditionResult<ConditionResultData> {
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
}

#[async_trait]
impl ComparisonCondition for AboveCondition {
    async fn above(&self, data1: &[f32], data2: &[f32]) -> ConditionResult<ConditionResultData> {
        self.check_dual(data1, data2).await
    }

    async fn below(&self, data1: &[f32], data2: &[f32]) -> ConditionResult<ConditionResultData> {
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

    async fn equals(
        &self,
        data1: &[f32],
        data2: &[f32],
        tolerance: f32,
    ) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        let min_len = std::cmp::min(data1.len(), data2.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let signal = (data1[i] - data2[i]).abs() <= tolerance;
            signals.push(signal);

            let strength = if signal {
                let diff = (data1[i] - data2[i]).abs() / data2[i];
                if diff < 0.01 {
                    SignalStrength::VeryStrong
                } else if diff < 0.02 {
                    SignalStrength::Strong
                } else if diff < 0.05 {
                    SignalStrength::Medium
                } else {
                    SignalStrength::Weak
                }
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(if signal {
                TrendDirection::Sideways
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

    async fn in_range(
        &self,
        data: &[f32],
        min_data: &[f32],
        max_data: &[f32],
    ) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        let min_len = std::cmp::min(std::cmp::min(data.len(), min_data.len()), max_data.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let signal = data[i] >= min_data[i] && data[i] <= max_data[i];
            signals.push(signal);

            let strength = if signal {
                let range = max_data[i] - min_data[i];
                let position = (data[i] - min_data[i]) / range;
                if position > 0.4 && position < 0.6 {
                    SignalStrength::VeryStrong
                } else if position > 0.3 && position < 0.7 {
                    SignalStrength::Strong
                } else if position > 0.2 && position < 0.8 {
                    SignalStrength::Medium
                } else {
                    SignalStrength::Weak
                }
            } else {
                SignalStrength::Weak
            };
            strengths.push(strength);

            directions.push(if signal {
                TrendDirection::Sideways
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
}

// ============================================================================
// Условия пересечения
// ============================================================================

/// Условие "пересечение выше"
pub struct CrossesAboveCondition {
    config: ConditionConfig,
}

impl CrossesAboveCondition {
    pub fn new() -> Result<Self, ConditionError> {
        let config = ConditionConfig {
            name: "CrossesAbove".to_string(),
            description: "Проверяет пересечение первого вектора выше второго".to_string(),
            condition_type: ConditionType::Crossover,
            category: ConditionCategory::Entry,
            min_data_points: 2,
            is_reversible: false,
        };

        Ok(Self { config })
    }
}

#[async_trait]
impl Condition for CrossesAboveCondition {
    fn name(&self) -> &str {
        "CrossesAbove"
    }
    fn description(&self) -> &str {
        "Проверяет пересечение линии выше"
    }
    fn config(&self) -> &ConditionConfig {
        &self.config
    }
    fn min_data_points(&self) -> usize {
        2
    }

    async fn check_simple(&self, _data: &[f32]) -> ConditionResult<ConditionResultData> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    async fn check_ohlc(
        &self,
        data: &crate::indicators::types::OHLCData,
    ) -> ConditionResult<ConditionResultData> {
        self.check_simple(&data.close).await
    }

    async fn check_dual(
        &self,
        data1: &[f32],
        data2: &[f32],
    ) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        let min_len = std::cmp::min(data1.len(), data2.len());
        let mut signals = Vec::with_capacity(min_len);
        let mut strengths = Vec::with_capacity(min_len);
        let mut directions = Vec::with_capacity(min_len);

        // Первый элемент не может быть пересечением
        signals.push(false);
        strengths.push(SignalStrength::Weak);
        directions.push(TrendDirection::Sideways);

        for i in 1..min_len {
            let signal = data1[i] > data2[i] && data1[i - 1] <= data2[i - 1];
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
                TrendDirection::Sideways
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

    async fn check_single(&self, _index: usize, _data: &[f32]) -> ConditionResult<bool> {
        Err(ConditionError::InsufficientData {
            required: 2,
            actual: 1,
        })
    }

    fn validate_input_data(&self, data: &[f32]) -> Result<(), ConditionError> {
        if data.len() < self.min_data_points() {
            Err(ConditionError::InsufficientData {
                required: self.min_data_points(),
                actual: data.len(),
            })
        } else {
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

#[async_trait]
impl CrossoverCondition for CrossesAboveCondition {
    async fn crosses_above(&self, data1: &[f32], data2: &[f32], index: usize) -> bool {
        if index < 1 || index >= data1.len() || index >= data2.len() {
            return false;
        }
        data1[index] > data2[index] && data1[index - 1] <= data2[index - 1]
    }

    async fn crosses_below(&self, data1: &[f32], data2: &[f32], index: usize) -> bool {
        if index < 1 || index >= data1.len() || index >= data2.len() {
            return false;
        }
        data1[index] < data2[index] && data1[index - 1] >= data2[index - 1]
    }

    async fn crosses_above_threshold(&self, data: &[f32], threshold: f32, index: usize) -> bool {
        if index < 1 || index >= data.len() {
            return false;
        }
        data[index] > threshold && data[index - 1] <= threshold
    }

    async fn crosses_below_threshold(&self, data: &[f32], threshold: f32, index: usize) -> bool {
        if index < 1 || index >= data.len() {
            return false;
        }
        data[index] < threshold && data[index - 1] >= threshold
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
        };

        Ok(Self {
            period: period_usize,
            config,
        })
    }
}

#[async_trait]
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

    async fn check_simple(&self, data: &[f32]) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();
        let period = self.period;

        if data.len() < period {
            return Err(ConditionError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        // Первые period-1 элементов не могут быть трендом
        for _ in 0..period - 1 {
            signals.push(false);
            strengths.push(SignalStrength::Weak);
            directions.push(TrendDirection::Sideways);
        }

        for i in period - 1..data.len() {
            let trend_data = &data[i - period + 1..=i];
            let signal = self.is_rising_trend(trend_data);
            signals.push(signal);

            let strength = if signal {
                let slope = self.calculate_slope(trend_data);
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

    async fn check_ohlc(
        &self,
        data: &crate::indicators::types::OHLCData,
    ) -> ConditionResult<ConditionResultData> {
        self.check_simple(&data.close).await
    }

    async fn check_dual(
        &self,
        data1: &[f32],
        _data2: &[f32],
    ) -> ConditionResult<ConditionResultData> {
        // Для dual проверки используем data1 как основную линию
        self.check_simple(data1).await
    }

    async fn check_single(&self, index: usize, data: &[f32]) -> ConditionResult<bool> {
        let period = self.period;

        if index < period - 1 || index >= data.len() {
            return Err(ConditionError::InsufficientData {
                required: index + 1,
                actual: data.len(),
            });
        }

        let trend_data = &data[index - period + 1..=index];
        Ok(self.is_rising_trend(trend_data))
    }

    fn validate_input_data(&self, data: &[f32]) -> Result<(), ConditionError> {
        if data.len() < self.min_data_points() {
            Err(ConditionError::InsufficientData {
                required: self.min_data_points(),
                actual: data.len(),
            })
        } else {
            Ok(())
        }
    }

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync> {
        Box::new(Self::new(self.period as f32).unwrap())
    }
}

impl RisingTrendCondition {
    // Вспомогательные методы
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

#[async_trait]
impl TrendCondition for RisingTrendCondition {
    async fn get_trend_direction(
        &self,
        data: &[f32],
        period: usize,
    ) -> ConditionResult<TrendDirection> {
        if data.len() < period {
            return Err(ConditionError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let trend_data = &data[data.len() - period..];
        if self.is_rising_trend(trend_data) {
            Ok(TrendDirection::Rising)
        } else {
            Ok(TrendDirection::Falling)
        }
    }

    async fn is_rising(&self, data: &[f32], period: usize) -> bool {
        if data.len() < period {
            return false;
        }

        let trend_data = &data[data.len() - period..];
        self.is_rising_trend(trend_data)
    }

    async fn is_falling(&self, data: &[f32], period: usize) -> bool {
        if data.len() < period {
            return false;
        }

        let trend_data = &data[data.len() - period..];
        !self.is_rising_trend(trend_data)
    }

    async fn is_sideways(&self, data: &[f32], period: usize) -> bool {
        if data.len() < period {
            return false;
        }

        let trend_data = &data[data.len() - period..];
        let slope = self.calculate_slope(trend_data);
        slope.abs() < 0.01 // Очень маленький наклон
    }

    async fn get_trend_strength(
        &self,
        data: &[f32],
        period: usize,
    ) -> ConditionResult<SignalStrength> {
        if data.len() < period {
            return Err(ConditionError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let trend_data = &data[data.len() - period..];
        let slope = self.calculate_slope(trend_data);

        let strength = if slope > 0.1 {
            SignalStrength::VeryStrong
        } else if slope > 0.05 {
            SignalStrength::Strong
        } else if slope > 0.02 {
            SignalStrength::Medium
        } else {
            SignalStrength::Weak
        };

        Ok(strength)
    }

    async fn check_rising_bars(
        &self,
        data: &[f32],
        window_size: usize,
    ) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        if data.len() < window_size {
            return Err(ConditionError::InsufficientData {
                required: window_size,
                actual: data.len(),
            });
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        // Первые window_size-1 элементов не могут быть трендом
        for _ in 0..window_size - 1 {
            signals.push(false);
            strengths.push(SignalStrength::Weak);
            directions.push(TrendDirection::Sideways);
        }

        for i in window_size - 1..data.len() {
            let window_data = &data[i - window_size + 1..=i];
            let signal = self.is_rising_trend(window_data);
            signals.push(signal);

            let strength = if signal {
                let slope = self.calculate_slope(window_data);
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

    async fn check_falling_bars(
        &self,
        data: &[f32],
        window_size: usize,
    ) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        if data.len() < window_size {
            return Err(ConditionError::InsufficientData {
                required: window_size,
                actual: data.len(),
            });
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        // Первые window_size-1 элементов не могут быть трендом
        for _ in 0..window_size - 1 {
            signals.push(false);
            strengths.push(SignalStrength::Weak);
            directions.push(TrendDirection::Sideways);
        }

        for i in window_size - 1..data.len() {
            let window_data = &data[i - window_size + 1..=i];
            let signal = !self.is_rising_trend(window_data); // Инвертируем для падения
            signals.push(signal);

            let strength = if signal {
                let slope = self.calculate_slope(window_data);
                if slope < -0.1 {
                    SignalStrength::VeryStrong
                } else if slope < -0.05 {
                    SignalStrength::Strong
                } else if slope < -0.02 {
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

    async fn check_falling_to_rising(&self, data: &[f32]) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        if data.len() < 3 {
            return Err(ConditionError::InsufficientData {
                required: 3,
                actual: data.len(),
            });
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        // Первые 2 элемента не могут быть разворотом
        signals.push(false);
        signals.push(false);
        strengths.push(SignalStrength::Weak);
        strengths.push(SignalStrength::Weak);
        directions.push(TrendDirection::Sideways);
        directions.push(TrendDirection::Sideways);

        for i in 2..data.len() {
            let a = data[i - 2];
            let b = data[i - 1];
            let c = data[i];
            let signal = a > b && b < c; // Падение, затем рост
            signals.push(signal);

            let strength = if signal {
                let diff = (c - b) / b;
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
                TrendDirection::Sideways
            });
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: data.len(),
            confidence_score: 0.7,
            additional_info: HashMap::new(),
        };

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }

    async fn check_rising_to_falling(&self, data: &[f32]) -> ConditionResult<ConditionResultData> {
        let start_time = Instant::now();

        if data.len() < 3 {
            return Err(ConditionError::InsufficientData {
                required: 3,
                actual: data.len(),
            });
        }

        let mut signals = Vec::with_capacity(data.len());
        let mut strengths = Vec::with_capacity(data.len());
        let mut directions = Vec::with_capacity(data.len());

        // Первые 2 элемента не могут быть разворотом
        signals.push(false);
        signals.push(false);
        strengths.push(SignalStrength::Weak);
        strengths.push(SignalStrength::Weak);
        directions.push(TrendDirection::Sideways);
        directions.push(TrendDirection::Sideways);

        for i in 2..data.len() {
            let a = data[i - 2];
            let b = data[i - 1];
            let c = data[i];
            let signal = a < b && b > c; // Рост, затем падение
            signals.push(signal);

            let strength = if signal {
                let diff = (b - c) / b;
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
                TrendDirection::Sideways
            });
        }

        let metadata = ConditionMetadata {
            execution_time: start_time.elapsed(),
            data_points_processed: data.len(),
            confidence_score: 0.7,
            additional_info: HashMap::new(),
        };

        Ok(ConditionResultData {
            signals,
            strengths,
            directions,
            metadata,
        })
    }
}
