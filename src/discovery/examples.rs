use crate::data_model::types::TimeFrame;
use crate::discovery::*;
use crate::strategy::types::{ConditionOperator, PriceField};

/// Пример работы генератора стратегий
///
/// Показывает, как генератор строит все возможные комбинации стратегий
pub fn example_strategy_generation() {
    println!("=== Пример генерации стратегий ===\n");

    let config = StrategyDiscoveryConfig {
        max_optimization_params: 10,
        timeframe_count: 3,
        base_timeframe: TimeFrame::Minutes(60),
        max_timeframe_minutes: 1440,
    };

    // Доступные индикаторы (упрощенный пример)
    let indicators = vec![
        IndicatorInfo {
            name: "SMA".to_string(),
            alias: "sma".to_string(),
            parameters: vec![IndicatorParamInfo {
                name: "period".to_string(),
                param_type: crate::indicators::types::ParameterType::Period,
                optimizable: true,
                global_param_name: Some("period".to_string()),
            }],
            can_use_indicator_input: false,
            input_type: "price".to_string(),
        },
        IndicatorInfo {
            name: "EMA".to_string(),
            alias: "ema".to_string(),
            parameters: vec![IndicatorParamInfo {
                name: "period".to_string(),
                param_type: crate::indicators::types::ParameterType::Period,
                optimizable: true,
                global_param_name: Some("period".to_string()),
            }],
            can_use_indicator_input: false,
            input_type: "price".to_string(),
        },
        IndicatorInfo {
            name: "RSI".to_string(),
            alias: "rsi".to_string(),
            parameters: vec![IndicatorParamInfo {
                name: "period".to_string(),
                param_type: crate::indicators::types::ParameterType::Period,
                optimizable: true,
                global_param_name: Some("period".to_string()),
            }],
            can_use_indicator_input: false,
            input_type: "price".to_string(),
        },
    ];

    // Поля цены
    let price_fields = vec![PriceField::Close, PriceField::High, PriceField::Low];

    // Операторы
    let operators = vec![ConditionOperator::Above, ConditionOperator::Below];

    println!("📊 Входные данные:");
    println!("   Индикаторы: SMA, EMA, RSI (по 1 параметру каждый)");
    println!("   Таймфреймы: 60, 120, 180 минут (count=3)");
    println!("   Поля цены: Close, High, Low");
    println!("   Операторы: >, <");
    println!("   Макс. параметров оптимизации: 10\n");

    // Генерация комбинаций таймфреймов
    println!("1️⃣ Комбинации таймфреймов:");
    println!("   Примечание: Методы generate_combinations() удалены.");
    println!("   Основная генерация происходит через candidate_builder.rs с рандомным выбором.\n");

    // Генерация комбинаций индикаторов
    println!("2️⃣ Комбинации индикаторов:");
    println!("   Примечание: Методы generate_combinations() удалены.");
    println!("   Основная генерация происходит через candidate_builder.rs с рандомным выбором.\n");

    // Генерация условий
    println!("3️⃣ Условия:");
    println!("   Примечание: Методы generate_*_conditions() удалены.");
    println!("   Основная генерация происходит через candidate_builder.rs с рандомным выбором.\n");

    // Полная генерация стратегий (используем CandidateBuilder)
    println!("4️⃣ Полная генерация стратегий:");
    println!("   (Используется CandidateBuilder - см. examples в optimization/)");
    let candidates: Vec<StrategyCandidate> = Vec::new(); // Старый метод generate_strategies удален, используйте CandidateBuilder

    println!("   Примеры кандидатов стратегий:");
    for (i, candidate) in candidates.iter().take(5).enumerate() {
        let ind_names: Vec<&str> = candidate
            .indicators
            .iter()
            .map(|ind| ind.name.as_str())
            .collect();
        let cond_names: Vec<&str> = candidate
            .conditions
            .iter()
            .map(|cond| cond.name.as_str())
            .collect();
        let tf_strs: Vec<String> = candidate
            .timeframes
            .iter()
            .map(|tf| format!("{:?}", tf))
            .collect();

        println!("   Кандидат {}:", i + 1);
        println!("      Индикаторы: {:?}", ind_names);
        println!("      Условия: {:?}", cond_names);
        println!("      Таймфреймы: {:?}", tf_strs);
        println!(
            "      Параметров оптимизации: {}",
            candidate.total_optimization_params()
        );
        println!();
    }

    println!("   Всего кандидатов: {}\n", candidates.len());

    println!("📝 Как работает генератор:");
    println!("   Основная генерация кандидатов происходит через candidate_builder.rs:");
    println!(
        "   1. Использует рандомный выбор элементов с вероятностями из CandidateBuilderConfig"
    );
    println!("   2. Вероятности настраиваются через ElementProbabilities");
    println!("   3. Линейная сложность O(n) вместо экспоненциальной");
    println!("   4. Генерирует кандидатов по фазам с вероятностью продолжения");
    println!();
    println!("   Старые методы generate_combinations() удалены, так как они генерировали");
    println!("   все комбинации экспоненциально и использовались только в примерах.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Запуск примера для визуализации
        example_strategy_generation();
    }
}
