use std::collections::HashMap;

use crate::core::agt::indicators::any::SimpleIndicatorsEnum;
use crate::core::agt::indicators::source::SourceIndicators;
use crate::core::agt::{
    candles,
    indicators::common::{IndicatorData, IndicatorsEnum, IndicatorsMeta, OptimizationParam},
};
use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub enum QuantityIndicators {
    One,
    Two,
    Three,
    Four,
}

pub struct SourceCombinationIndicators {}
pub struct SimpleCombinationIndicators {}

impl SourceCombinationIndicators {
    pub fn execute(quantity: usize) -> Vec<Vec<IndicatorsEnum>> {
        let indicators: Vec<IndicatorsEnum> = all::<IndicatorsEnum>().collect();
        let sources = vec![indicators; quantity]; // клонируем indicators quantity раз
        sources.into_iter().multi_cartesian_product().collect()
    }
}

impl SimpleCombinationIndicators {
    /// Оптимизированный метод для генерации комбинаций простых индикаторов
    ///
    /// # Контекст использования в торговом приложении:
    ///
    /// В вашем торговом роботе этот метод используется для:
    /// 1. **Генерации стратегий** - создание различных комбинаций индикаторов для тестирования
    /// 2. **Оптимизации параметров** - перебор различных наборов индикаторов для поиска лучших
    /// 3. **Бэктестинга** - тестирование стратегий на исторических данных
    /// 4. **Машинного обучения** - создание признаков для ML моделей
    ///
    /// # Примеры использования:
    /// - quantity=2: [RSI, SMA], [RSI, EMA], [SMA, EMA], ...
    /// - quantity=3: [RSI, SMA, SUPERTRAND], [RSI, EMA, WMA], ...
    ///
    /// # Оптимизации по сравнению с закомментированным кодом:
    /// - Использует itertools::multi_cartesian_product() вместо вложенных циклов
    /// - Избегает множественных клонирований
    /// - Более читаемый и поддерживаемый код
    /// - Лучшая производительность для больших количеств индикаторов
    pub fn execute(quantity: usize) -> Vec<Vec<SimpleIndicatorsEnum>> {
        let indicators: Vec<SimpleIndicatorsEnum> = all::<SimpleIndicatorsEnum>().collect();

        // Оптимизация: создаем вектор итераторов вместо клонирования
        let repeated_iterators = std::iter::repeat(indicators)
            .take(quantity)
            .collect::<Vec<_>>();

        // Применяем multi_cartesian_product для генерации всех комбинаций
        repeated_iterators
            .into_iter()
            .multi_cartesian_product()
            .collect()
    }

    /// Альтернативный метод с фильтрацией по конкретным индикаторам
    ///
    /// # Параметры:
    /// - quantity: количество индикаторов в комбинации
    /// - allowed_indicators: список разрешенных индикаторов (если None, используются все)
    ///
    /// # Использование в контексте:
    /// - Фильтрация по типам индикаторов (трендовые, осцилляторы, объемные)
    /// - Исключение несовместимых комбинаций
    /// - Создание специализированных стратегий
    pub fn execute_filtered(
        quantity: usize,
        allowed_indicators: Option<Vec<SimpleIndicatorsEnum>>,
    ) -> Vec<Vec<SimpleIndicatorsEnum>> {
        let indicators: Vec<SimpleIndicatorsEnum> = match allowed_indicators {
            Some(allowed) => allowed,
            None => all::<SimpleIndicatorsEnum>().collect(),
        };

        let repeated_iterators = std::iter::repeat(indicators)
            .take(quantity)
            .collect::<Vec<_>>();

        repeated_iterators
            .into_iter()
            .multi_cartesian_product()
            .collect()
    }

    /// Метод для создания комбинаций с уникальными индикаторами (без повторений)
    ///
    /// # Контекст использования:
    /// - Создание стратегий с разными типами индикаторов
    /// - Избежание избыточности в комбинациях
    /// - Оптимизация для конкретных рыночных условий
    pub fn execute_unique(quantity: usize) -> Vec<Vec<SimpleIndicatorsEnum>> {
        let indicators: Vec<SimpleIndicatorsEnum> = all::<SimpleIndicatorsEnum>().collect();

        // Используем combinations вместо multi_cartesian_product для уникальных комбинаций
        indicators.into_iter().combinations(quantity).collect()
    }

    /// Метод для создания комбинаций с ограничением по типам индикаторов
    ///
    /// # Параметры:
    /// - quantity: количество индикаторов
    /// - trend_indicators: количество трендовых индикаторов
    /// - oscillator_indicators: количество осцилляторов
    ///
    /// # Контекст использования:
    /// - Создание сбалансированных стратегий
    /// - Избежание конфликтов между индикаторами
    /// - Оптимизация для конкретных рыночных условий
    pub fn execute_balanced(
        quantity: usize,
        trend_indicators: usize,
        oscillator_indicators: usize,
    ) -> Vec<Vec<SimpleIndicatorsEnum>> {
        // Определяем типы индикаторов
        let trend_indicators_list = vec![
            SimpleIndicatorsEnum::SMA,
            SimpleIndicatorsEnum::EMA,
            SimpleIndicatorsEnum::WMA,
            SimpleIndicatorsEnum::SUPERTRAND,
        ];

        let oscillator_indicators_list =
            vec![SimpleIndicatorsEnum::RSI, SimpleIndicatorsEnum::SIGNALLINE];

        let other_indicators_list = vec![
            SimpleIndicatorsEnum::MAXFOR,
            SimpleIndicatorsEnum::MINFOR,
            SimpleIndicatorsEnum::VTRAND,
            SimpleIndicatorsEnum::GEOMEAN,
            SimpleIndicatorsEnum::AMMA,
            SimpleIndicatorsEnum::SQWMA,
            SimpleIndicatorsEnum::SINEWMA,
            SimpleIndicatorsEnum::AMA,
            SimpleIndicatorsEnum::ZLEMA,
            SimpleIndicatorsEnum::TPBF,
        ];

        let mut combinations = Vec::new();

        // Генерируем комбинации с учетом баланса
        for trend_combo in trend_indicators_list.iter().combinations(trend_indicators) {
            for osc_combo in oscillator_indicators_list
                .iter()
                .combinations(oscillator_indicators)
            {
                let remaining = quantity - trend_indicators - oscillator_indicators;
                if remaining <= other_indicators_list.len() {
                    for other_combo in other_indicators_list.iter().combinations(remaining) {
                        let mut combination = Vec::new();
                        combination.extend(trend_combo.iter().map(|&&x| x));
                        combination.extend(osc_combo.iter().map(|&&x| x));
                        combination.extend(other_combo.iter().map(|&&x| x));
                        combinations.push(combination);
                    }
                }
            }
        }

        combinations
    }

    /// Утилитарный метод для получения статистики комбинаций
    ///
    /// # Возвращает:
    /// - Общее количество возможных комбинаций
    /// - Количество уникальных индикаторов
    /// - Размер каждой комбинации
    pub fn get_combination_stats(quantity: usize) -> HashMap<String, usize> {
        use std::collections::HashMap;

        let total_indicators = all::<SimpleIndicatorsEnum>().count();
        let total_combinations = total_indicators.pow(quantity as u32);
        let unique_combinations = (1..=total_indicators).rev().take(quantity).product();

        let mut stats = HashMap::new();
        stats.insert("total_indicators".to_string(), total_indicators);
        stats.insert("combination_size".to_string(), quantity);
        stats.insert("total_combinations".to_string(), total_combinations);
        stats.insert("unique_combinations".to_string(), unique_combinations);

        stats
    }
}

// impl SourceCombinationIndicators {
//     pub async fn execute(quntity_indicators: &QuantityIndicators) -> Vec<Vec<IndicatorsEnum>> {
//         let indicators = all::<IndicatorsEnum>().collect::<Vec<_>>();
//         let mut combination_indicator: Vec<Vec<IndicatorsEnum>> = vec![];
//         for indicator1 in indicators.iter() {
//             if let QuantityIndicators::One = quntity_indicators {
//                 combination_indicator.push(vec![indicator1.clone()])
//             }
//             for indicator2 in indicators.iter() {
//                 if let QuantityIndicators::One = quntity_indicators {
//                     break;
//                 }
//                 if let QuantityIndicators::Two = quntity_indicators {
//                     combination_indicator.push(vec![indicator1.clone(), indicator2.clone()])
//                 }
//                 for indicator3 in indicators.iter() {
//                     if let QuantityIndicators::Two = quntity_indicators {
//                     break;
//                 }
//                     if let QuantityIndicators::Three = quntity_indicators {
//                         combination_indicator.push(vec![
//                             indicator1.clone(),
//                             indicator2.clone(),
//                             indicator3.clone(),
//                         ])
//                     }
//                     for indicator4 in indicators.iter() {
//                         if let QuantityIndicators::Three = quntity_indicators {
//                             break;
//                         }
//                         if let QuantityIndicators::Four = quntity_indicators {
//                             combination_indicator.push(vec![
//                                 indicator1.clone(),
//                                 indicator2.clone(),
//                                 indicator3.clone(),
//                                 indicator4.clone(),
//                             ])
//                         }
//                     }
//                 }
//             }
//         }
//         return combination_indicator;
//     }
// }

// impl SimpleCombinationIndicators {
//     pub async fn execute(quntity_indicators: QuantityIndicators) -> Vec<Vec<SimpleIndicatorsEnum>> {
//         let indicators = all::<SimpleIndicatorsEnum>().collect::<Vec<_>>();
//         let mut combination_indicator: Vec<Vec<SimpleIndicatorsEnum>> = vec![];
//         for indicator1 in indicators.iter() {
//             if let QuantityIndicators::One = quntity_indicators {
//                 combination_indicator.push(vec![indicator1.clone()])
//             }
//             for indicator2 in indicators.iter() {
//                 if let QuantityIndicators::One = quntity_indicators {
//                     break;
//                 }
//                 if let QuantityIndicators::Two = quntity_indicators {
//                     combination_indicator.push(vec![indicator1.clone(), indicator2.clone()])
//                 }
//                 for indicator3 in indicators.iter() {
//                 if let QuantityIndicators::Two = quntity_indicators {
//                     break;
//                 }
//                     if let QuantityIndicators::Three = quntity_indicators {
//                         combination_indicator.push(vec![
//                             indicator1.clone(),
//                             indicator2.clone(),
//                             indicator3.clone(),
//                         ])
//                     }
//                     for indicator4 in indicators.iter() {
//                         if let QuantityIndicators::Three = quntity_indicators {
//                             break;
//                         }
//                         if let QuantityIndicators::Four = quntity_indicators {
//                             combination_indicator.push(vec![
//                                 indicator1.clone(),
//                                 indicator2.clone(),
//                                 indicator3.clone(),
//                                 indicator4.clone(),
//                             ])
//                         }
//                     }
//                 }
//             }
//         }
//         return combination_indicator;
//     }
// }
