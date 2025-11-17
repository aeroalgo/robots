use crate::data_model::types::TimeFrame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Конфигурация для автоматического поиска стратегий
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDiscoveryConfig {
    /// Максимальное количество параметров оптимизации
    /// Если у всех индикаторов по 2 параметра оптимизации, то будет 5 индикаторов
    /// Если по 1 параметру, то 10. В это число входит оптимизация стоплоса/тейкпрофита
    pub max_optimization_params: usize,

    /// Количество таймфреймов для использования
    /// Если текущий TF 60 минут, то пробуем комбинации с 120, 180, 240 и т.д.
    pub timeframe_count: usize,

    /// Базовый таймфрейм для генерации комбинаций
    pub base_timeframe: TimeFrame,

    /// Глобальные настройки периодов оптимизации
    /// Ключ - имя параметра (например, "period", "coeff_atr", "pct")
    /// Значение - диапазон оптимизации
    pub global_param_ranges: HashMap<String, GlobalParamRange>,

    /// Разрешить построение индикаторов по индикаторам
    /// Если true, то часть индикаторов может строиться не по цене, а по уже построенным индикаторам
    pub allow_indicator_on_indicator: bool,

    /// Максимальная глубина вложенности индикаторов (если allow_indicator_on_indicator = true)
    pub max_indicator_depth: usize,

    /// Пороги для условий индикатор-константа осцилляторов
    /// Например, [30.0, 50.0, 70.0] для RSI (перепроданность/нейтральность/перекупленность)
    /// Если пустой массив, условия индикатор-константа не генерируются
    pub oscillator_thresholds: Vec<f64>,
}

/// Глобальный диапазон параметра оптимизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalParamRange {
    pub start: f64,
    pub end: f64,
    pub step: f64,
}

impl GlobalParamRange {
    pub fn new(start: f64, end: f64, step: f64) -> Self {
        Self { start, end, step }
    }

    pub fn validate(&self) -> bool {
        self.start < self.end && self.step > 0.0
    }

    pub fn count_values(&self) -> usize {
        ((self.end - self.start) / self.step + 1.0) as usize
    }

    pub fn generate_values(&self) -> Vec<f64> {
        let mut values = Vec::new();
        let mut current = self.start;
        while current <= self.end {
            values.push(current);
            current += self.step;
        }
        values
    }
}

impl Default for StrategyDiscoveryConfig {
    fn default() -> Self {
        let mut global_param_ranges = HashMap::new();
        global_param_ranges.insert(
            "period".to_string(),
            GlobalParamRange::new(10.0, 250.0, 10.0),
        );
        global_param_ranges.insert(
            "coeff_atr".to_string(),
            GlobalParamRange::new(1.0, 10.0, 0.5),
        );
        global_param_ranges.insert("pct".to_string(), GlobalParamRange::new(1.0, 10.0, 0.2));

        Self {
            max_optimization_params: 10,
            timeframe_count: 3,
            base_timeframe: TimeFrame::Minutes(60),
            global_param_ranges,
            allow_indicator_on_indicator: false,
            max_indicator_depth: 1,
            oscillator_thresholds: vec![], // По умолчанию не генерируем условия с константой
        }
    }
}

