use crate::core::agt::opt::iterating::conditions::ConditionEnum;
use std::collections::HashMap;

/// Оптимизированная структура для работы с условиями стратегий
/// Использует слайсы вместо владения данными для экономии памяти
#[derive(Debug)]
pub struct OptimizedStrategyCondition<'a> {
    data: &'a [f64],
    indicator: &'a [f64],
    constant: f64,
    pub condition: ConditionEnum, // Сделано публичным
    result: Vec<bool>,
    name_indicator: String,
}

impl<'a> OptimizedStrategyCondition<'a> {
    pub fn new(
        data: &'a [f64],
        indicator: &'a [f64],
        condition: ConditionEnum,
        constant: f64,
        name_indicator: String,
    ) -> Self {
        Self {
            data,
            indicator,
            constant,
            condition,
            result: Vec::new(),
            name_indicator,
        }
    }

    /// Генерирует сигналы без клонирования данных
    pub fn generate_signals(&mut self) -> &[bool] {
        let min_len = std::cmp::min(self.data.len(), self.indicator.len());
        self.result.clear();
        self.result.reserve(min_len);

        for i in 0..min_len {
            let signal = self.check_single_condition(i);
            self.result.push(signal);
        }

        &self.result
    }

    /// Проверяет условие для одного индекса
    fn check_single_condition(&self, index: usize) -> bool {
        if index >= self.data.len() || index >= self.indicator.len() {
            return false;
        }

        match self.condition {
            ConditionEnum::ABOVE => self.check_above(self.indicator[index]),
            ConditionEnum::BELOW => self.check_below(self.indicator[index]),
            ConditionEnum::CROSSESABOVE => self.check_crosses_above(index),
            ConditionEnum::CROSSESBELOW => self.check_crosses_below(index),
            ConditionEnum::LOWERPERCENTBARS => self.check_lower_percent_bars(index),
            ConditionEnum::GREATERPERCENTBARS => self.check_greater_percent_bars(index),
            ConditionEnum::FAILINGDATABARS => self.check_failing_data_bars(index),
            ConditionEnum::LOWERPERCENTBARSINDICATORS => self.check_lower_percent_bars(index),
            ConditionEnum::GREATERPERCENTBARSINDICATORS => self.check_greater_percent_bars(index),
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
    fn check_above(&self, value: f64) -> bool {
        value > self.constant
    }

    fn check_below(&self, value: f64) -> bool {
        value < self.constant
    }

    fn check_crosses_above(&self, index: usize) -> bool {
        if index < 1 || index >= self.indicator.len() {
            return false;
        }
        let prev = self.indicator[index - 1];
        let curr = self.indicator[index];
        prev <= self.constant && curr > self.constant
    }

    fn check_crosses_below(&self, index: usize) -> bool {
        if index < 1 || index >= self.indicator.len() {
            return false;
        }
        let prev = self.indicator[index - 1];
        let curr = self.indicator[index];
        prev >= self.constant && curr < self.constant
    }

    fn check_lower_percent_bars(&self, index: usize) -> bool {
        if index < 1 {
            return false;
        }
        let base_value = self.data[index - 1];
        let current_value = self.data[index];
        let target = base_value * (1.0 - self.constant / 100.0);
        current_value < target
    }

    fn check_greater_percent_bars(&self, index: usize) -> bool {
        if index < 1 {
            return false;
        }
        let base_value = self.data[index - 1];
        let current_value = self.data[index];
        let target = base_value * (1.0 + self.constant / 100.0);
        current_value > target
    }

    fn check_failing_data_bars(&self, index: usize) -> bool {
        if index < 2 {
            return false;
        }
        let window_size = 3;
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
        let window_size = 3;
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
        let window_size = 3;
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
        let window_size = 3;
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

    /// Получает последние N сигналов без клонирования
    pub fn get_last_signals(&self, count: usize) -> &[bool] {
        let result_len = self.result.len();
        if count >= result_len {
            &self.result
        } else {
            &self.result[result_len - count..]
        }
    }

    /// Проверяет, есть ли активный сигнал
    pub fn has_active_signal(&self) -> bool {
        self.result.last().copied().unwrap_or(false)
    }

    /// Получает количество активных сигналов
    pub fn count_active_signals(&self) -> usize {
        self.result.iter().filter(|&&signal| signal).count()
    }

    /// Получает имя индикатора
    pub fn get_indicator_name(&self) -> &str {
        &self.name_indicator
    }
}

/// Фабрика для создания оптимизированных условий
pub struct ConditionFactory;

impl ConditionFactory {
    /// Создает оптимизированное условие с проверкой валидности данных
    pub fn create_optimized_condition<'a>(
        data: &'a [f64],
        indicator: &'a [f64],
        condition: ConditionEnum,
        constant: f64,
        name_indicator: String,
    ) -> Option<OptimizedStrategyCondition<'a>> {
        if data.is_empty() || indicator.is_empty() {
            return None;
        }

        Some(OptimizedStrategyCondition::new(
            data,
            indicator,
            condition,
            constant,
            name_indicator,
        ))
    }

    /// Создает несколько условий для одного набора данных
    pub fn create_multiple_conditions<'a>(
        data: &'a [f64],
        indicator: &'a [f64],
        conditions: Vec<ConditionEnum>,
        constant: f64,
        name_indicator: String,
    ) -> Vec<OptimizedStrategyCondition<'a>> {
        conditions
            .into_iter()
            .filter_map(|condition| {
                Self::create_optimized_condition(
                    data,
                    indicator,
                    condition,
                    constant,
                    name_indicator.clone(),
                )
            })
            .collect()
    }
}

/// Утилиты для работы с условиями
pub struct ConditionUtils;

impl ConditionUtils {
    /// Объединяет несколько сигналов с логическим И
    pub fn combine_signals_and(signals: &[&[bool]]) -> Vec<bool> {
        if signals.is_empty() {
            return Vec::new();
        }

        let min_len = signals.iter().map(|s| s.len()).min().unwrap_or(0);
        let mut result = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let combined = signals.iter().all(|signal| i < signal.len() && signal[i]);
            result.push(combined);
        }

        result
    }

    /// Объединяет несколько сигналов с логическим ИЛИ
    pub fn combine_signals_or(signals: &[&[bool]]) -> Vec<bool> {
        if signals.is_empty() {
            return Vec::new();
        }

        let min_len = signals.iter().map(|s| s.len()).min().unwrap_or(0);
        let mut result = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let combined = signals.iter().any(|signal| i < signal.len() && signal[i]);
            result.push(combined);
        }

        result
    }

    /// Находит пересечения сигналов (где сигнал меняется с false на true)
    pub fn find_signal_crossings(signals: &[bool]) -> Vec<usize> {
        let mut crossings = Vec::new();

        for i in 1..signals.len() {
            if !signals[i - 1] && signals[i] {
                crossings.push(i);
            }
        }

        crossings
    }

    /// Вычисляет статистику сигналов
    pub fn calculate_signal_stats(signals: &[bool]) -> HashMap<String, f64> {
        let total = signals.len() as f64;
        let active = signals.iter().filter(|&&s| s).count() as f64;
        let inactive = total - active;

        let mut stats = HashMap::new();
        stats.insert("total".to_string(), total);
        stats.insert("active".to_string(), active);
        stats.insert("inactive".to_string(), inactive);
        stats.insert("active_percentage".to_string(), (active / total) * 100.0);
        stats.insert(
            "inactive_percentage".to_string(),
            (inactive / total) * 100.0,
        );

        stats
    }
}
