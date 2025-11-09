use crate::core::agt::{
    opt::iterating::conditions::ConditionEnum,
    strategy::{
        condition::StrategyCondition,
        optimized_condition::{ConditionFactory, ConditionUtils, OptimizedStrategyCondition},
    },
};

/// Пример использования оптимизированного модуля condition
pub struct ConditionExample;

impl ConditionExample {
    /// Демонстрирует базовое использование StrategyCondition
    pub async fn demonstrate_basic_usage() {
        // Создаем тестовые данные
        let data: Vec<f64> = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0];
        let indicator: Vec<f64> = vec![50.0, 51.0, 49.0, 52.0, 48.0, 53.0];

        // Создаем условие
        let mut condition = StrategyCondition::new(
            data.clone(),
            indicator.clone(),
            ConditionEnum::ABOVE,
            50.0,
            "RSI".to_string(),
        )
        .await;

        // Генерируем сигналы
        let signals = condition.generate_signals().await;
        println!("Базовые сигналы: {:?}", signals);
    }

    /// Демонстрирует использование оптимизированного модуля
    pub fn demonstrate_optimized_usage() {
        // Создаем тестовые данные
        let data: Vec<f64> = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0];
        let indicator: Vec<f64> = vec![50.0, 51.0, 49.0, 52.0, 48.0, 53.0];

        // Создаем оптимизированное условие
        if let Some(mut optimized_condition) = ConditionFactory::create_optimized_condition(
            &data,
            &indicator,
            ConditionEnum::ABOVE,
            50.0,
            "RSI".to_string(),
        ) {
            // Генерируем сигналы
            let signals = optimized_condition.generate_signals();
            println!("Оптимизированные сигналы: {:?}", signals);

            // Проверяем статистику
            let stats = ConditionUtils::calculate_signal_stats(signals);
            println!("Статистика сигналов: {:?}", stats);
        }
    }

    /// Демонстрирует работу с несколькими условиями
    pub fn demonstrate_multiple_conditions() {
        let data: Vec<f64> = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0];
        let indicator: Vec<f64> = vec![50.0, 51.0, 49.0, 52.0, 48.0, 53.0];

        let conditions = vec![
            ConditionEnum::ABOVE,
            ConditionEnum::BELOW,
            ConditionEnum::CROSSESABOVE,
        ];

        let optimized_conditions = ConditionFactory::create_multiple_conditions(
            &data,
            &indicator,
            conditions,
            50.0,
            "RSI".to_string(),
        );

        let mut all_signals = Vec::new();

        for mut condition in optimized_conditions {
            // Получаем условие до заимствования
            let condition_type = condition.condition;
            let signals = condition.generate_signals();
            let signals_clone = signals.to_vec(); // Клонируем сигналы
            all_signals.push(signals_clone);

            println!("Сигналы для {:?}: {:?}", condition_type, signals);
        }

        // Объединяем сигналы с логическим И
        if all_signals.len() >= 2 {
            let signal_refs: Vec<&[bool]> = all_signals.iter().map(|s| s.as_slice()).collect();
            let combined_and = ConditionUtils::combine_signals_and(&signal_refs);
            println!("Объединенные сигналы (И): {:?}", combined_and);
        }
    }

    /// Демонстрирует работу с пересечениями
    pub fn demonstrate_crossings() {
        let data: Vec<f64> = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0];
        let indicator: Vec<f64> = vec![50.0, 51.0, 49.0, 52.0, 48.0, 53.0];

        if let Some(mut condition) = ConditionFactory::create_optimized_condition(
            &data,
            &indicator,
            ConditionEnum::CROSSESABOVE,
            50.0,
            "RSI".to_string(),
        ) {
            let signals = condition.generate_signals();
            let crossings = ConditionUtils::find_signal_crossings(signals);
            println!("Пересечения сигналов: {:?}", crossings);
        }
    }

    /// Демонстрирует сравнение производительности
    pub async fn demonstrate_performance_comparison() {
        let data: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let indicator: Vec<f64> = (0..1000).map(|i| (i as f64) * 0.5).collect();

        // Тест базового модуля
        let start = std::time::Instant::now();
        let mut basic_condition = StrategyCondition::new(
            data.clone(),
            indicator.clone(),
            ConditionEnum::ABOVE,
            250.0,
            "Test".to_string(),
        )
        .await;
        let _basic_signals = basic_condition.generate_signals().await;
        let basic_duration = start.elapsed();

        // Тест оптимизированного модуля
        let start = std::time::Instant::now();
        if let Some(mut optimized_condition) = ConditionFactory::create_optimized_condition(
            &data,
            &indicator,
            ConditionEnum::ABOVE,
            250.0,
            "Test".to_string(),
        ) {
            let _optimized_signals = optimized_condition.generate_signals();
            let optimized_duration = start.elapsed();

            println!("Базовый модуль: {:?}", basic_duration);
            println!("Оптимизированный модуль: {:?}", optimized_duration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_condition() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let indicator = vec![0.5, 1.5, 2.5, 3.5, 4.5];

        let mut condition = StrategyCondition::new(
            data,
            indicator,
            ConditionEnum::ABOVE,
            2.0,
            "Test".to_string(),
        )
        .await;

        let signals = condition.generate_signals().await;
        assert_eq!(signals.len(), 5);
        assert_eq!(signals[0], false); // 0.5 > 2.0 = false
        assert_eq!(signals[4], true); // 4.5 > 2.0 = true
    }

    #[test]
    fn test_optimized_condition() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let indicator = vec![0.5, 1.5, 2.5, 3.5, 4.5];

        if let Some(mut condition) = ConditionFactory::create_optimized_condition(
            &data,
            &indicator,
            ConditionEnum::ABOVE,
            2.0,
            "Test".to_string(),
        ) {
            let signals = condition.generate_signals();
            assert_eq!(signals.len(), 5);
            assert_eq!(signals[0], false);
            assert_eq!(signals[4], true);
        }
    }

    #[test]
    fn test_signal_combination() {
        let signals1 = vec![true, false, true, false];
        let signals2 = vec![true, true, false, false];

        let combined_and = ConditionUtils::combine_signals_and(&[&signals1, &signals2]);
        assert_eq!(combined_and, vec![true, false, false, false]);

        let combined_or = ConditionUtils::combine_signals_or(&[&signals1, &signals2]);
        assert_eq!(combined_or, vec![true, true, true, false]);
    }

    #[test]
    fn test_signal_crossings() {
        let signals = vec![false, false, true, false, true, true];
        let crossings = ConditionUtils::find_signal_crossings(&signals);
        assert_eq!(crossings, vec![2, 4]);
    }
}
