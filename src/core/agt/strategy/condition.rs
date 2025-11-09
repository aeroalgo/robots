use crate::core::agt::{
    candles,
    indicators::common::{IndicatorData, IndicatorsEnum, IndicatorsMeta, OptimizationParam},
    opt::iterating::conditions::ConditionEnum,
};
use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index, vec};

#[derive(Clone, Debug)]
pub struct StrategyCondition {
    data: Vec<f64>,
    indicator: Vec<f64>,
    constant: f64,
    condition: ConditionEnum,
    result: Vec<bool>, // Изменено с Vec<f64> на Vec<bool> для булевых сигналов
    name_indicator: String,
}

impl StrategyCondition {
    pub async fn new(
        data: Vec<f64>,
        indicator: Vec<f64>,
        condition: ConditionEnum,
        constant: f64,
        name_indicator: String,
    ) -> Self {
        // Оптимизация: избегаем клонирования, используем move
        return StrategyCondition {
            data,
            indicator,
            constant,
            condition,
            result: Vec::new(),
            name_indicator,
        };
    }

    /// Генерирует список булевых сигналов для всех данных
    pub async fn generate_signals(&mut self) -> Vec<bool> {
        let data_len = self.data.len();
        let indicator_len = self.indicator.len();

        if data_len == 0 || indicator_len == 0 {
            return Vec::new();
        }

        let min_len = std::cmp::min(data_len, indicator_len);
        let mut signals = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let signal = self.check_single_condition(i).await;
            signals.push(signal);
        }

        self.result = signals.clone();
        signals
    }

    /// Проверяет условие для одного индекса
    async fn check_single_condition(&self, index: usize) -> bool {
        if index >= self.data.len() || index >= self.indicator.len() {
            return false;
        }

        let _data_value = self.data[index]; // Префикс _ для неиспользуемой переменной
        let indicator_value = self.indicator[index];
        let constant_value = self.constant;

        match self.condition {
            ConditionEnum::ABOVE => self.check_above(indicator_value, constant_value),
            ConditionEnum::BELOW => self.check_below(indicator_value, constant_value),
            ConditionEnum::CROSSESABOVE => self.check_crosses_above(index),
            ConditionEnum::CROSSESBELOW => self.check_crosses_below(index),
            ConditionEnum::LOWERPERCENTBARS => self.check_lower_percent_bars(index),
            ConditionEnum::GREATERPERCENTBARS => self.check_greater_percent_bars(index),
            ConditionEnum::LOWERPERCENTBARSINDICATORS => self.check_lower_percent_indicators(index),
            ConditionEnum::GREATERPERCENTBARSINDICATORS => {
                self.check_greater_percent_indicators(index)
            }
            ConditionEnum::FAILINGDATABARS => self.check_failing_data_bars(index),
            ConditionEnum::FAILINGINDICATORSBARS => self.check_failing_indicators_bars(index),
            ConditionEnum::FALLINGTORISINGDATA => self.check_falling_to_rising_data(index),
            ConditionEnum::FALLINGTORISINGINDICATORS => {
                self.check_falling_to_rising_indicators(index)
            }
            ConditionEnum::RISINGDATABARS => self.check_rising_data_bars(index),
            ConditionEnum::RISINGINDICATORSBARS => self.check_rising_indicators_bars(index),
            ConditionEnum::RISINGTOFALLINGDATA => self.check_rising_to_falling_data(index),
            ConditionEnum::RISINGTOFALLINGINDICATORS => {
                self.check_rising_to_falling_indicators(index)
            }
        }
    }

    // Реализация всех функций для каждого enum

    fn check_above(&self, value: f64, threshold: f64) -> bool {
        value > threshold
    }

    fn check_below(&self, value: f64, threshold: f64) -> bool {
        value < threshold
    }

    fn check_crosses_above(&self, index: usize) -> bool {
        if index < 1 || index >= self.indicator.len() {
            return false;
        }
        let threshold = self.constant;
        let prev = self.indicator[index - 1];
        let curr = self.indicator[index];
        prev <= threshold && curr > threshold
    }

    fn check_crosses_below(&self, index: usize) -> bool {
        if index < 1 || index >= self.indicator.len() {
            return false;
        }
        let threshold = self.constant;
        let prev = self.indicator[index - 1];
        let curr = self.indicator[index];
        prev >= threshold && curr < threshold
    }
    fn check_lower_percent_indicators(&self, index: usize) -> bool {
        if index < 1 {
            return false;
        }
        let base_value = self.indicator[index - 1];
        let current_value = self.indicator[index];
        let percent_threshold = self.constant;
        let target = base_value * (1.0 - percent_threshold / 100.0);
        current_value < target
    }

    fn check_greater_percent_indicators(&self, index: usize) -> bool {
        if index < 1 {
            return false;
        }
        let base_value = self.indicator[index - 1];
        let current_value = self.indicator[index];
        let percent_threshold = self.constant;
        let target = base_value * (1.0 + percent_threshold / 100.0);
        current_value > target
    }

    fn check_lower_percent_bars(&self, index: usize) -> bool {
        if index < 1 {
            return false;
        }
        let base_value = self.data[index - 1];
        let current_value = self.data[index];
        let percent_threshold = self.constant;
        let target = base_value * (1.0 - percent_threshold / 100.0);
        current_value < target
    }

    fn check_greater_percent_bars(&self, index: usize) -> bool {
        if index < 1 {
            return false;
        }
        let base_value = self.data[index - 1];
        let current_value = self.data[index];
        let percent_threshold = self.constant;
        let target = base_value * (1.0 + percent_threshold / 100.0);
        current_value > target
    }

    fn check_failing_data_bars(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let window_size = 3; // Можно сделать настраиваемым
        if index < window_size - 1 {
            return false;
        }
        let start_idx = index - window_size + 1;
        let end_idx = index + 1;
        self.data[start_idx..end_idx]
            .windows(2)
            .all(|w| w[0] > w[1])
    }

    fn check_failing_indicators_bars(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let window_size = 3; // Можно сделать настраиваемым
        if index < window_size - 1 {
            return false;
        }
        let start_idx = index - window_size + 1;
        let end_idx = index + 1;
        self.indicator[start_idx..end_idx]
            .windows(2)
            .all(|w| w[0] > w[1])
    }

    fn check_falling_to_rising_data(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let a = self.data[index - 2];
        let b = self.data[index - 1];
        let c = self.data[index];
        a > b && b < c
    }

    fn check_falling_to_rising_indicators(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let a = self.indicator[index - 2];
        let b = self.indicator[index - 1];
        let c = self.indicator[index];
        a > b && b < c
    }

    fn check_rising_data_bars(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let window_size = 3; // Можно сделать настраиваемым
        if index < window_size - 1 {
            return false;
        }
        let start_idx = index - window_size + 1;
        let end_idx = index + 1;
        self.data[start_idx..end_idx]
            .windows(2)
            .all(|w| w[0] < w[1])
    }

    fn check_rising_indicators_bars(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let window_size = 3; // Можно сделать настраиваемым
        if index < window_size - 1 {
            return false;
        }
        let start_idx = index - window_size + 1;
        let end_idx = index + 1;
        self.indicator[start_idx..end_idx]
            .windows(2)
            .all(|w| w[0] < w[1])
    }

    fn check_rising_to_falling_data(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let a = self.data[index - 2];
        let b = self.data[index - 1];
        let c = self.data[index];
        a < b && b > c
    }

    fn check_rising_to_falling_indicators(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let a = self.indicator[index - 2];
        let b = self.indicator[index - 1];
        let c = self.indicator[index];
        a < b && b > c
    }

    /// Получает сигнал для конкретного условия (устаревший метод, оставлен для совместимости)
    pub async fn get_signal(&self, condition: ConditionEnum) -> Vec<bool> {
        let mut temp_condition = self.clone();
        temp_condition.condition = condition;
        temp_condition.generate_signals().await
    }

    /// Получает последние N сигналов
    pub fn get_last_signals(&self, count: usize) -> Vec<bool> {
        let result_len = self.result.len();
        if count >= result_len {
            self.result.clone()
        } else {
            self.result[result_len - count..].to_vec()
        }
    }

    /// Получает все сигналы
    pub fn get_all_signals(&self) -> Vec<bool> {
        self.result.clone()
    }

    /// Проверяет, есть ли активный сигнал (последний сигнал true)
    pub fn has_active_signal(&self) -> bool {
        self.result.last().copied().unwrap_or(false)
    }

    /// Получает количество активных сигналов
    pub fn count_active_signals(&self) -> usize {
        self.result.iter().filter(|&&signal| signal).count()
    }
}
