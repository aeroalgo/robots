use anyhow::Result;
use chrono::Utc;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::optimization::{
    FitnessFunction, FitnessThresholds, FitnessWeights, GeneticAlgorithmConfig,
    InitialPopulationGeneratorV2, GeneticAlgorithmV2, IslandManager, EvolutionManager,
    MigrationSystem, FreshBloodSystem,
};
use robots::discovery::StrategyDiscoveryConfig;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Запуск генетической оптимизации (V2 - с рандомными параметрами) ===\n");

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

    println!("⚙️  Настройка генетического алгоритма...");
    let config = GeneticAlgorithmConfig {
        population_size: 30,
        max_generations: 5,
        crossover_rate: 0.7,
        mutation_rate: 0.1,
        elitism_count: 3,
        islands_count: 2,
        migration_interval: 5,
        migration_rate: 0.05,
        fitness_thresholds: FitnessThresholds {
            min_sharpe_ratio: Some(0.5),
            max_drawdown_pct: Some(30.0),
            min_win_rate: Some(0.40),
            min_profit_factor: Some(1.2),
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

    println!("   Размер популяции: {}", config.population_size);
    println!("   Максимум поколений: {}", config.max_generations);
    println!("   Количество островов: {}", config.islands_count);
    println!("   Фильтрация начальной популяции: {} (пороги проверяются после оптимизации)\n", config.filter_initial_population);

    println!("🧬 Настройка параметров генерации стратегий...");
    let discovery_config = StrategyDiscoveryConfig {
        max_optimization_params: 8,
        timeframe_count: 2,
        base_timeframe: base_timeframe.clone(),
        allow_indicator_on_indicator: true,
        max_indicator_depth: 1,
    };
    println!("   Максимум параметров оптимизации: {}", discovery_config.max_optimization_params);
    println!("   Количество таймфреймов: {}", discovery_config.timeframe_count);
    println!("   Индикаторы на индикаторах: {}", discovery_config.allow_indicator_on_indicator);
    println!("   Максимальная глубина вложенности: {}\n", discovery_config.max_indicator_depth);

    println!("🧬 Генерация начальных популяций для каждого острова...");
    println!("   ⚠️  ВАЖНО: Параметры генерируются РАНДОМНО (не дефолтные)");
    println!("   ⚠️  Пороги НЕ проверяются на этом этапе\n");

    let mut initial_populations = Vec::new();
    for island_idx in 0..config.islands_count {
        println!("   Генерация популяции для острова {}...", island_idx);
        let generator = InitialPopulationGeneratorV2::with_discovery_config(
            config.clone(),
            frames.clone(),
            base_timeframe.clone(),
            discovery_config.clone(),
        );

        let population = generator.generate(None).await?;
        println!("   ✅ Остров {}: сгенерировано {} стратегий с рандомными параметрами", 
            island_idx, population.individuals.len());
        
        let mut pop = population;
        pop.island_id = Some(island_idx);
        initial_populations.push(pop);
    }

    println!("\n🩸 Применение Fresh Blood после генерации начальных популяций...");
    let fresh_blood = FreshBloodSystem::new(config.clone());
    let generator = InitialPopulationGeneratorV2::with_discovery_config(
        config.clone(),
        frames.clone(),
        base_timeframe.clone(),
        discovery_config.clone(),
    );

    for (island_idx, population) in initial_populations.iter_mut().enumerate() {
        fresh_blood.inject_fresh_blood_v2(population, &generator).await?;
        println!("   ✅ Fresh Blood применен для острова {}", island_idx);
    }
    println!();

    println!("🏝️  Создание островов...");
    let mut island_manager = IslandManager::new(config.clone(), initial_populations);
    println!("   Создано {} островов\n", island_manager.islands_count());

    println!("🧬 Создание генетического алгоритма V2...");
    let genetic_algorithm = GeneticAlgorithmV2::new(
        config.clone(),
        frames.clone(),
        base_timeframe.clone(),
    );

    println!("📈 Создание менеджеров...");
    let mut evolution_manager = EvolutionManager::new(config.clone());
    let migration_system = MigrationSystem::new(config.clone());

    println!("\n🚀 Запуск эволюции...\n");

    for generation in 0..config.max_generations {
        println!("═══════════════════════════════════════════════════════");
        println!("Поколение {}/{}", generation + 1, config.max_generations);
        println!("═══════════════════════════════════════════════════════");

        let islands = island_manager.get_all_islands_mut();
        
        for (island_idx, island) in islands.iter_mut().enumerate() {
            println!(
                "\n🏝️  Остров {} (поколение {})",
                island_idx, island.generation
            );

            genetic_algorithm.evolve_generation(island).await?;

            let best = island.individuals.iter().max_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_a
                    .partial_cmp(&fitness_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if let Some(best_individual) = best {
                let fitness = best_individual.strategy.fitness.unwrap_or(0.0);
                println!("   Лучший fitness: {:.4}", fitness);

                if let Some(ref report) = best_individual.strategy.backtest_report {
                    println!("   Total Profit: {:.2}", report.metrics.total_profit);
                    if let Some(sharpe) = report.metrics.sharpe_ratio {
                        println!("   Sharpe Ratio: {:.2}", sharpe);
                    }
                    if let Some(pf) = report.metrics.profit_factor {
                        println!("   Profit Factor: {:.2}", pf);
                    }
                    println!(
                        "   Win Rate: {:.1}%",
                        report.metrics.winning_percentage * 100.0
                    );
                    println!("   Trades: {}", report.trades.len());
                }

                println!("   Стратегий в популяции: {}", island.individuals.len());
                let with_fitness = island.individuals.iter()
                    .filter(|ind| ind.strategy.fitness.is_some())
                    .count();
                println!("   Стратегий с fitness: {}", with_fitness);

                evolution_manager.update_fitness_history(fitness);
            }
        }

        if generation > 0
            && (generation + 1) % config.migration_interval == 0
            && config.islands_count > 1
        {
            println!("\n🔄 Миграция между островами...");
            let islands = island_manager.get_all_islands_mut();
            migration_system.migrate_between_islands(islands)?;
            println!("   Миграция завершена");
        }

        if generation > 0 && generation % 3 == 0 {
            println!("\n🩸 Инъекция свежей крови...");
            let islands = island_manager.get_all_islands_mut();
            for island in islands.iter_mut() {
                fresh_blood.inject_fresh_blood_v2(island, &generator).await?;
            }
            println!("   Инъекция завершена");
        }

        if evolution_manager.should_restart() {
            println!("\n⚠️  Обнаружен застой! Перезапуск эволюции...");
            evolution_manager.reset_stagnation();
        }

        println!();
    }

    println!("═══════════════════════════════════════════════════════");
    println!("✅ Эволюция завершена!");
    println!("═══════════════════════════════════════════════════════\n");

    println!("🔍 Проверка порогов для всех стратегий...");
    println!("   Пороги проверяются ТОЛЬКО после завершения оптимизации\n");

    let islands = island_manager.get_all_islands();
    let mut all_individuals: Vec<_> = islands
        .iter()
        .flat_map(|island| island.individuals.iter())
        .collect();

    all_individuals.sort_by(|a, b| {
        let fitness_a = a.strategy.fitness.unwrap_or(0.0);
        let fitness_b = b.strategy.fitness.unwrap_or(0.0);
        fitness_b
            .partial_cmp(&fitness_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let passed_thresholds: Vec<_> = all_individuals
        .iter()
        .filter(|ind| {
            if let Some(ref report) = ind.strategy.backtest_report {
                FitnessFunction::passes_thresholds(&report, &config.fitness_thresholds)
            } else {
                false
            }
        })
        .collect();

    println!("═══════════════════════════════════════════════════════");
    println!("📊 Итоговая статистика:");
    println!("═══════════════════════════════════════════════════════\n");
    println!("Всего протестировано стратегий: {}", all_individuals.len());
    println!("Стратегий, прошедших пороги: {}", passed_thresholds.len());
    println!(
        "Процент успешных: {:.1}%",
        (passed_thresholds.len() as f64 / all_individuals.len() as f64) * 100.0
    );

    if let Some(best) = all_individuals.first() {
        println!("\n🏆 Лучшая стратегия (по fitness):");
        println!("   Fitness: {:.4}", best.strategy.fitness.unwrap_or(0.0));
        if let Some(ref report) = best.strategy.backtest_report {
            println!("   Total Profit: {:.2}", report.metrics.total_profit);
            if let Some(sharpe) = report.metrics.sharpe_ratio {
                println!("   Sharpe Ratio: {:.2}", sharpe);
            }
        }
    }

    if let Some(best_passed) = passed_thresholds.first() {
        println!("\n✅ Лучшая стратегия, прошедшая пороги:");
        println!("   Fitness: {:.4}", best_passed.strategy.fitness.unwrap_or(0.0));
        if let Some(ref report) = best_passed.strategy.backtest_report {
            println!("   Total Profit: {:.2}", report.metrics.total_profit);
            if let Some(sharpe) = report.metrics.sharpe_ratio {
                println!("   Sharpe Ratio: {:.2}", sharpe);
            }
            println!("   Win Rate: {:.1}%", report.metrics.winning_percentage * 100.0);
            println!("   Trades: {}", report.trades.len());
        }
    }

    println!("\n🏆 Топ-10 стратегий, прошедших пороги:\n");
    for (rank, individual) in passed_thresholds.iter().take(10).enumerate() {
        println!("📍 Место {}:", rank + 1);
        println!("   Fitness: {:.4}", individual.strategy.fitness.unwrap_or(0.0));
        
        if let Some(ref candidate) = individual.strategy.candidate {
            println!("\n   📊 Структура стратегии:");
            println!("      Индикаторы ({}):", candidate.indicators.len());
            for ind in &candidate.indicators {
                println!("        - {} ({})", ind.name, ind.alias);
            }
            
            if !candidate.nested_indicators.is_empty() {
                println!("      Вложенные индикаторы ({}):", candidate.nested_indicators.len());
                for nested in &candidate.nested_indicators {
                    println!("        - {} ({})", nested.indicator.name, nested.indicator.alias);
                }
            }
            
            println!("      Условия входа ({}):", candidate.conditions.len());
            for cond in &candidate.conditions {
                println!("        - {}", cond.name);
            }
            
            if !candidate.exit_conditions.is_empty() {
                println!("      Условия выхода ({}):", candidate.exit_conditions.len());
                for cond in &candidate.exit_conditions {
                    println!("        - {}", cond.name);
                }
            }
            
            if !candidate.stop_handlers.is_empty() {
                println!("      Стоп-обработчики ({}):", candidate.stop_handlers.len());
                for stop in &candidate.stop_handlers {
                    println!("        - {}", stop.name);
                }
            }
        }
        
        println!("\n   ⚙️  Оптимизированные параметры:");
        for (param_name, param_value) in &individual.strategy.parameters {
            match param_value {
                robots::strategy::types::StrategyParamValue::Number(n) => {
                    println!("      {} = {:.2}", param_name, n);
                }
                robots::strategy::types::StrategyParamValue::Integer(i) => {
                    println!("      {} = {}", param_name, i);
                }
                robots::strategy::types::StrategyParamValue::Flag(b) => {
                    println!("      {} = {}", param_name, b);
                }
                _ => {
                    println!("      {} = {:?}", param_name, param_value);
                }
            }
        }
        
        if let Some(ref report) = individual.strategy.backtest_report {
            println!("\n   💰 Результаты backtest:");
            println!("      Total Profit: {:.2}", report.metrics.total_profit);
            if let Some(sharpe) = report.metrics.sharpe_ratio {
                println!("      Sharpe Ratio: {:.2}", sharpe);
            }
            if let Some(pf) = report.metrics.profit_factor {
                println!("      Profit Factor: {:.2}", pf);
            }
            if let Some(cagr) = report.metrics.cagr {
                println!("      CAGR: {:.2}%", cagr);
            }
            println!(
                "      Win Rate: {:.1}%",
                report.metrics.winning_percentage * 100.0
            );
            if let Some(dd) = report.metrics.drawdown {
                println!("      Drawdown: {:.2}", dd);
            }
            if let Some(dd_pct) = report.metrics.drawdown_percent {
                println!("      Drawdown %: {:.2}%", dd_pct);
            }
            println!("      Trades: {}", report.trades.len());
        }
        
        println!();
    }

    Ok(())
}

