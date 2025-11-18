use crate::data_model::types::TimeFrame;
use serde::{Deserialize, Serialize};

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

impl Default for StrategyDiscoveryConfig {
    fn default() -> Self {
        Self {
            max_optimization_params: 10,
            timeframe_count: 3,
            base_timeframe: TimeFrame::Minutes(60),
            allow_indicator_on_indicator: false,
            max_indicator_depth: 1,
            oscillator_thresholds: vec![], // По умолчанию не генерируем условия с константой
        }
    }
}
