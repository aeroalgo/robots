use crate::core::agt::indicators::any::SimpleIndicatorsEnum;
use crate::core::agt::opt::iterating::indicators::SimpleCombinationIndicators;
use std::collections::HashMap;

/// Пример использования SimpleCombinationIndicators
pub struct IndicatorsExample;

impl IndicatorsExample {
    /// Демонстрирует базовое использование execute
    pub fn demonstrate_basic_execute() {
        println!("=== Демонстрация базового execute ===");

        // Генерируем комбинации из 2 индикаторов
        let combinations = SimpleCombinationIndicators::execute(2);
        println!("Комбинации из 2 индикаторов (первые 5):");
        for (i, combo) in combinations.iter().take(5).enumerate() {
            println!("  {}. {:?}", i + 1, combo);
        }
        println!("Всего комбинаций: {}", combinations.len());

        // Генерируем комбинации из 3 индикаторов
        let combinations_3 = SimpleCombinationIndicators::execute(3);
        println!("\nКомбинации из 3 индикаторов (первые 3):");
        for (i, combo) in combinations_3.iter().take(3).enumerate() {
            println!("  {}. {:?}", i + 1, combo);
        }
        println!("Всего комбинаций: {}", combinations_3.len());
    }

    /// Демонстрирует использование execute_filtered
    pub fn demonstrate_filtered_execute() {
        println!("\n=== Демонстрация execute_filtered ===");

        // Создаем список разрешенных индикаторов
        let allowed_indicators = vec![
            SimpleIndicatorsEnum::RSI,
            SimpleIndicatorsEnum::SMA,
            SimpleIndicatorsEnum::EMA,
            SimpleIndicatorsEnum::SUPERTRAND,
        ];

        let combinations =
            SimpleCombinationIndicators::execute_filtered(2, Some(allowed_indicators));

        println!("Фильтрованные комбинации из 2 индикаторов:");
        for (i, combo) in combinations.iter().enumerate() {
            println!("  {}. {:?}", i + 1, combo);
        }
        println!("Всего фильтрованных комбинаций: {}", combinations.len());
    }

    /// Демонстрирует использование execute_unique
    pub fn demonstrate_unique_execute() {
        println!("\n=== Демонстрация execute_unique ===");

        let combinations = SimpleCombinationIndicators::execute_unique(3);

        println!("Уникальные комбинации из 3 индикаторов (первые 5):");
        for (i, combo) in combinations.iter().take(5).enumerate() {
            println!("  {}. {:?}", i + 1, combo);
        }
        println!("Всего уникальных комбинаций: {}", combinations.len());

        // Проверяем, что нет повторений
        let mut has_duplicates = false;
        for combo in &combinations {
            let mut sorted = combo.clone();
            sorted.sort();
            sorted.dedup();
            if sorted.len() != combo.len() {
                has_duplicates = true;
                break;
            }
        }
        println!("Есть дубликаты: {}", has_duplicates);
    }

    /// Демонстрирует использование execute_balanced
    pub fn demonstrate_balanced_execute() {
        println!("\n=== Демонстрация execute_balanced ===");

        let combinations = SimpleCombinationIndicators::execute_balanced(
            4, // общее количество индикаторов
            2, // количество трендовых индикаторов
            1, // количество осцилляторов
        );

        println!(
            "Сбалансированные комбинации (4 индикатора: 2 трендовых + 1 осциллятор + 1 другой):"
        );
        for (i, combo) in combinations.iter().take(5).enumerate() {
            println!("  {}. {:?}", i + 1, combo);
        }
        println!("Всего сбалансированных комбинаций: {}", combinations.len());
    }

    /// Демонстрирует использование get_combination_stats
    pub fn demonstrate_stats() {
        println!("\n=== Демонстрация get_combination_stats ===");

        for quantity in 1..=4 {
            let stats = SimpleCombinationIndicators::get_combination_stats(quantity);
            println!("Статистика для комбинаций из {} индикаторов:", quantity);
            println!(
                "  Всего индикаторов: {}",
                stats.get("total_indicators").unwrap()
            );
            println!(
                "  Размер комбинации: {}",
                stats.get("combination_size").unwrap()
            );
            println!(
                "  Всего комбинаций: {}",
                stats.get("total_combinations").unwrap()
            );
            println!(
                "  Уникальных комбинаций: {}",
                stats.get("unique_combinations").unwrap()
            );
            println!();
        }
    }

    /// Демонстрирует сравнение производительности
    pub fn demonstrate_performance_comparison() {
        println!("\n=== Сравнение производительности ===");

        use std::time::Instant;

        // Тест базового execute
        let start = Instant::now();
        let _combinations = SimpleCombinationIndicators::execute(3);
        let basic_duration = start.elapsed();

        // Тест execute_unique
        let start = Instant::now();
        let _unique_combinations = SimpleCombinationIndicators::execute_unique(3);
        let unique_duration = start.elapsed();

        // Тест execute_balanced
        let start = Instant::now();
        let _balanced_combinations = SimpleCombinationIndicators::execute_balanced(4, 2, 1);
        let balanced_duration = start.elapsed();

        println!("Время выполнения базового execute: {:?}", basic_duration);
        println!("Время выполнения execute_unique: {:?}", unique_duration);
        println!("Время выполнения execute_balanced: {:?}", balanced_duration);
    }

    /// Демонстрирует практическое использование в контексте торгового робота
    pub fn demonstrate_trading_context() {
        println!("\n=== Практическое использование в торговом роботе ===");

        // Симуляция создания торговых стратегий
        println!("1. Создание базовых стратегий:");
        let basic_strategies = SimpleCombinationIndicators::execute(2);
        println!("   Создано {} базовых стратегий", basic_strategies.len());

        // Симуляция создания продвинутых стратегий
        println!("2. Создание продвинутых стратегий:");
        let advanced_strategies = SimpleCombinationIndicators::execute_balanced(4, 2, 1);
        println!(
            "   Создано {} продвинутых стратегий",
            advanced_strategies.len()
        );

        // Симуляция фильтрации стратегий
        println!("3. Фильтрация стратегий:");
        let allowed = vec![
            SimpleIndicatorsEnum::RSI,
            SimpleIndicatorsEnum::SMA,
            SimpleIndicatorsEnum::SUPERTRAND,
        ];
        let filtered_strategies = SimpleCombinationIndicators::execute_filtered(2, Some(allowed));
        println!(
            "   Создано {} отфильтрованных стратегий",
            filtered_strategies.len()
        );

        // Симуляция оптимизации
        println!("4. Оптимизация стратегий:");
        let unique_strategies = SimpleCombinationIndicators::execute_unique(3);
        println!(
            "   Создано {} уникальных стратегий для оптимизации",
            unique_strategies.len()
        );

        println!("\nКонтекст использования:");
        println!("- Каждая комбинация представляет торговую стратегию");
        println!("- Стратегии тестируются на исторических данных");
        println!("- Лучшие стратегии выбираются для реальной торговли");
        println!("- Оптимизация происходит автоматически");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_execute() {
        let combinations = SimpleCombinationIndicators::execute(2);
        assert!(!combinations.is_empty());

        // Проверяем, что каждая комбинация имеет правильный размер
        for combo in &combinations {
            assert_eq!(combo.len(), 2);
        }

        // Проверяем, что есть разные комбинации
        let first_combo = &combinations[0];
        let second_combo = &combinations[1];
        assert_ne!(first_combo, second_combo);
    }

    #[test]
    fn test_filtered_execute() {
        let allowed = vec![SimpleIndicatorsEnum::RSI, SimpleIndicatorsEnum::SMA];

        let combinations = SimpleCombinationIndicators::execute_filtered(2, Some(allowed));
        assert!(!combinations.is_empty());

        // Проверяем, что используются только разрешенные индикаторы
        for combo in &combinations {
            for indicator in combo {
                assert!(matches!(
                    indicator,
                    SimpleIndicatorsEnum::RSI | SimpleIndicatorsEnum::SMA
                ));
            }
        }
    }

    #[test]
    fn test_unique_execute() {
        let combinations = SimpleCombinationIndicators::execute_unique(3);
        assert!(!combinations.is_empty());

        // Проверяем, что нет дубликатов в каждой комбинации
        for combo in &combinations {
            let mut sorted = combo.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), combo.len());
        }
    }

    #[test]
    fn test_balanced_execute() {
        let combinations = SimpleCombinationIndicators::execute_balanced(4, 2, 1);
        assert!(!combinations.is_empty());

        // Проверяем, что каждая комбинация имеет правильный размер
        for combo in &combinations {
            assert_eq!(combo.len(), 4);
        }
    }

    #[test]
    fn test_stats() {
        let stats = SimpleCombinationIndicators::get_combination_stats(2);

        assert!(stats.contains_key("total_indicators"));
        assert!(stats.contains_key("combination_size"));
        assert!(stats.contains_key("total_combinations"));
        assert!(stats.contains_key("unique_combinations"));

        assert_eq!(stats.get("combination_size").unwrap(), &2);
    }
}
