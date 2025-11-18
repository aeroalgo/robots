use anyhow::Result;
use chrono::Utc;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::optimization::{
    FitnessThresholds, FitnessWeights, GeneticAlgorithmConfig, PerStructureOptimizer,
    StrategySaver,
};
use robots::discovery::{StrategyDiscoveryConfig, StrategyDiscoveryEngine};
use robots::optimization::per_structure_optimizer::OptimizedStrategyResult;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Оптимизация по структурам стратегий ===\n");
    println!("Каждая структура оптимизируется отдельно, затем фильтруется по порогам\n");

    let mut connector = ClickHouseConnector::with_config(ClickHouseConfig::default());
    connector.connect().await?;
    connector.ping().await?;

    let symbol = Symbol::from_descriptor("AFLT.MM");
    let base_timeframe = TimeFrame::from_identifier("60");
    let start = Utc::now() - chrono::Duration::days(94);
    let end = Utc::now() + chrono::Duration::hours(3);

    println!("📊 Загрузка данных...");
    let candles = connector
        .get_ohlcv_typed(&symbol, &base_timeframe, start, end, None)
        .await?;

    println!("   Получено {} свечей\n", candles.len());

    let frame = QuoteFrame::try_from_ohlcv(candles, symbol.clone(), base_timeframe.clone())?;
    let mut frames = HashMap::new();
    frames.insert(base_timeframe.clone(), frame);

    println!("⚙️  Настройка оптимизации...");
    let config = GeneticAlgorithmConfig {
        population_size: 20,
        max_generations: 10,
        crossover_rate: 0.7,
        mutation_rate: 0.1,
        elitism_count: 2,
        islands_count: 1,
        migration_interval: 5,
        migration_rate: 0.05,
        fitness_thresholds: FitnessThresholds {
            min_sharpe_ratio: Some(0.5),
            max_drawdown_pct: Some(30.0),
            min_win_rate: Some(0.40),
            min_profit_factor: Some(1.5),
            min_total_profit: Some(500.0),
            min_trades_count: Some(20),
            min_cagr: Some(5.0),
            max_max_drawdown: Some(10000.0),
        },
        fitness_weights: FitnessWeights::default(),
        use_existing_strategies: false,
        decimation_coefficient: 2.0,
        filter_initial_population: false,
        restart_on_finish: false,
        restart_on_stagnation: false,
        fresh_blood_rate: 0.1,
        detect_duplicates: true,
    };

    println!("   Размер популяции для каждой структуры: {}", config.population_size);
    println!("   Поколений оптимизации: {}", config.max_generations);
    println!("\n📋 Пороги фильтрации:");
    println!("   Profit Factor >= {:.1}", config.fitness_thresholds.min_profit_factor.unwrap_or(0.0));
    println!("   Sharpe Ratio >= {:.1}", config.fitness_thresholds.min_sharpe_ratio.unwrap_or(0.0));
    println!("   Win Rate >= {:.1}%", config.fitness_thresholds.min_win_rate.unwrap_or(0.0) * 100.0);
    println!("   Total Profit >= {:.0}", config.fitness_thresholds.min_total_profit.unwrap_or(0.0));
    println!("   Min Trades >= {}", config.fitness_thresholds.min_trades_count.unwrap_or(0));

    println!("\n🧬 Настройка генерации структур стратегий...");
    let discovery_config = StrategyDiscoveryConfig {
        max_optimization_params: 6,
        timeframe_count: 1,
        base_timeframe: base_timeframe.clone(),
        allow_indicator_on_indicator: false,
        max_indicator_depth: 0,
    };
    println!("   Максимум параметров оптимизации: {}", discovery_config.max_optimization_params);
    println!("   Количество таймфреймов: {}", discovery_config.timeframe_count);
    println!("   Индикаторы на индикаторах: {}", discovery_config.allow_indicator_on_indicator);
    println!();

    println!("🔍 Генерация структур стратегий...");
    use crate::discovery::IndicatorInfoCollector;
    use crate::indicators::registry::IndicatorRegistry;
    use crate::strategy::types::{ConditionOperator, PriceField};

    let registry = IndicatorRegistry::new();
    let available_indicators = IndicatorInfoCollector::collect_from_registry(&registry);
    let price_fields = vec![
        PriceField::Close,
        PriceField::Open,
        PriceField::High,
        PriceField::Low,
    ];
    let operators = vec![
        ConditionOperator::GreaterThan,
        ConditionOperator::LessThan,
        ConditionOperator::CrossesAbove,
        ConditionOperator::CrossesBelow,
    ];
    let stop_handler_configs = vec![];

    let mut engine = StrategyDiscoveryEngine::new(discovery_config.clone());
    let mut strategy_iterator = engine.generate_strategies_random(
        &available_indicators,
        &price_fields,
        &operators,
        &stop_handler_configs,
    );

    let max_structures = 5;
    let mut structures = Vec::new();
    for _ in 0..max_structures {
        if let Some(candidate) = strategy_iterator.next() {
            structures.push(candidate);
        } else {
            break;
        }
    }

    println!("   Сгенерировано {} структур стратегий\n", structures.len());

    let optimizer = PerStructureOptimizer::new(
        config.clone(),
        frames.clone(),
        base_timeframe.clone(),
        discovery_config.clone(),
    );

    let saver = StrategySaver::new();
    let mut all_passed_strategies = Vec::new();

    println!("═══════════════════════════════════════════════════════");
    println!("🚀 Начало оптимизации структур");
    println!("═══════════════════════════════════════════════════════\n");

    for (idx, structure) in structures.iter().enumerate() {
        println!("═══════════════════════════════════════════════════════");
        println!("Структура {}/{}", idx + 1, structures.len());
        println!("═══════════════════════════════════════════════════════");

        match optimizer.optimize_structure(structure.clone()).await {
            Ok(results) => {
                println!("\n🔍 Фильтрация результатов по порогам...");
                let total_results = results.len();
                let passed = optimizer.filter_by_thresholds(results, &config.fitness_thresholds);
                
                println!("   Всего результатов: {}", total_results);
                println!("   Прошло пороги: {}", passed.len());

                if !passed.is_empty() {
                    println!("\n✅ Найдено {} стратегий, прошедших пороги:", passed.len());
                    for (i, result) in passed.iter().enumerate() {
                        println!("\n   📊 Стратегия {}:", i + 1);
                        println!("      {}", saver.format_for_storage(result));
                        
                        println!("\n      ⚙️  Параметры:");
                        for (param_name, param_value) in &result.parameters {
                            match param_value {
                                robots::strategy::types::StrategyParamValue::Number(n) => {
                                    println!("         {} = {:.2}", param_name, n);
                                }
                                robots::strategy::types::StrategyParamValue::Integer(i) => {
                                    println!("         {} = {}", param_name, i);
                                }
                                robots::strategy::types::StrategyParamValue::Flag(b) => {
                                    println!("         {} = {}", param_name, b);
                                }
                                _ => {
                                    println!("         {} = {:?}", param_name, param_value);
                                }
                            }
                        }

                        all_passed_strategies.push(result.clone());
                    }
                } else {
                    println!("   ⚠️  Нет стратегий, прошедших пороги для этой структуры");
                }
            }
            Err(e) => {
                println!("   ❌ Ошибка при оптимизации: {}", e);
            }
        }

        println!();
    }

    println!("═══════════════════════════════════════════════════════");
    println!("📊 Итоговая статистика");
    println!("═══════════════════════════════════════════════════════\n");
    println!("Всего оптимизировано структур: {}", structures.len());
    println!("Всего стратегий, прошедших пороги: {}", all_passed_strategies.len());

    if !all_passed_strategies.is_empty() {
        println!("\n💾 Подготовка стратегий для сохранения в БД...");
        for (idx, result) in all_passed_strategies.iter().enumerate() {
            match saver.serialize_for_db(result, base_timeframe.clone()) {
                Ok(json_data) => {
                    println!("   ✅ Стратегия {} подготовлена для сохранения", idx + 1);
                    println!("      {}", saver.format_for_storage(result));
                    println!("      JSON данные готовы ({} байт)", json_data.len());
                    
                    // Здесь можно добавить сохранение в БД
                    // Например: connector.save_strategy(&json_data).await?;
                }
                Err(e) => {
                    println!("   ❌ Ошибка при подготовке стратегии {}: {}", idx + 1, e);
                }
            }
        }
        println!("\n✅ Все стратегии обработаны и готовы к сохранению в БД");
        println!("   Примечание: Для сохранения в БД добавьте вызов метода сохранения");
    } else {
        println!("\n⚠️  Нет стратегий для сохранения");
    }

    Ok(())
}

