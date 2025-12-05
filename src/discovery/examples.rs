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
    let timeframe_combinations = TimeFrameGenerator::generate_combinations(
        config.base_timeframe.clone(),
        config.timeframe_count,
        config.max_timeframe_minutes,
    );
    for (i, tf_combo) in timeframe_combinations.iter().enumerate() {
        println!("   Комбинация {}: {:?}", i + 1, tf_combo);
    }
    println!("   Всего: {} комбинаций\n", timeframe_combinations.len());

    // Генерация комбинаций индикаторов
    println!("2️⃣ Комбинации индикаторов (с учетом max_optimization_params=10, стопы=2):");
    println!("   Доступно для параметров: {} (10 - 2 стопов)", 10 - 2);
    let indicator_combinations = IndicatorCombinationGenerator::generate_combinations(
        &indicators,
        10,   // max_params
        true, // include_stops
    );
    println!("   Примеры комбинаций:");
    for (i, ind_combo) in indicator_combinations.iter().take(10).enumerate() {
        let param_count: usize = ind_combo
            .iter()
            .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
            .sum();
        let names: Vec<&str> = ind_combo.iter().map(|ind| ind.name.as_str()).collect();
        println!(
            "   Комбинация {}: {:?} (параметров: {})",
            i + 1,
            names,
            param_count
        );
    }
    println!("   Всего: {} комбинаций\n", indicator_combinations.len());

    // Генерация условий
    println!("3️⃣ Условия (индикатор-цена):");
    let conditions = ConditionCombinationGenerator::generate_indicator_price_conditions(
        &indicators,
        &price_fields,
        &operators,
    );
    println!("   Примеры условий:");
    for (i, cond) in conditions.iter().take(10).enumerate() {
        println!("   Условие {}: {}", i + 1, cond.name);
    }
    println!("   Всего: {} условий\n", conditions.len());

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
    println!("   1. Генерирует ВСЕ комбинации таймфреймов");
    println!("   2. Генерирует ВСЕ комбинации индикаторов (с учетом ограничения параметров)");
    println!("   3. Генерирует ВСЕ возможные условия (индикатор-цена и индикатор-индикатор)");
    println!("   4. Для каждой комбинации (таймфреймы × индикаторы):");
    println!("      - Фильтрует релевантные условия");
    println!("      - Генерирует ВСЕ комбинации условий (с учетом ограничения параметров)");
    println!("      - Создает кандидата стратегии");
    println!("   5. Декартово произведение: TF × Indicators × Conditions");
    println!();
    println!("   ⚠️  ВНИМАНИЕ: Количество кандидатов может быть ОЧЕНЬ большим!");
    println!("      Пример: 7 TF × 100 индикаторов × 1000 условий = 700,000+ кандидатов");
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
