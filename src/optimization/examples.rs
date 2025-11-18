use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::{Symbol, TimeFrame};
use crate::optimization::*;
use anyhow::Result;
use std::collections::HashMap;

pub async fn example_genetic_optimization() -> Result<()> {
    println!("=== Пример запуска генетической оптимизации ===\n");

    let symbol = Symbol::from_descriptor("AFLT.MM");
    let base_timeframe = TimeFrame::from_identifier("60");

    println!("📊 Подготовка данных...");
    println!("   Символ: {}", symbol.descriptor());
    println!(
        "   Базовый таймфрейм: {} минут\n",
        base_timeframe.total_minutes().unwrap_or(60)
    );

    let mut frames = HashMap::new();
    let frame = QuoteFrame::new(symbol.clone(), base_timeframe.clone());
    frames.insert(base_timeframe.clone(), frame);

    println!("⚙️  Создание конфигурации генетического алгоритма...");
    let config = GeneticAlgorithmConfig {
        population_size: 50,
        max_generations: 10,
        crossover_rate: 0.7,
        mutation_rate: 0.1,
        elitism_count: 5,
        islands_count: 2,
        migration_interval: 5,
        migration_rate: 0.05,
        fitness_thresholds: FitnessThresholds {
            min_sharpe_ratio: Some(1.0),
            max_drawdown_pct: Some(20.0),
            min_win_rate: Some(0.45),
            min_profit_factor: Some(1.5),
            min_total_profit: Some(1000.0),
            min_trades_count: Some(30),
            min_cagr: Some(10.0),
            max_max_drawdown: Some(5000.0),
        },
        fitness_weights: FitnessWeights {
            sharpe_ratio_weight: 0.3,
            profit_factor_weight: 0.25,
            win_rate_weight: 0.15,
            cagr_weight: 0.2,
            drawdown_penalty: 0.05,
            trades_count_bonus: 0.05,
        },
        use_existing_strategies: false,
        decimation_coefficient: 2.0,
        filter_initial_population: true,
        restart_on_finish: false,
        restart_on_stagnation: true,
        fresh_blood_rate: 0.1,
        detect_duplicates: true,
    };

    println!("   Размер популяции: {}", config.population_size);
    println!("   Максимум поколений: {}", config.max_generations);
    println!("   Количество островов: {}", config.islands_count);
    println!(
        "   Интервал миграции: каждые {} поколений",
        config.migration_interval
    );
    println!("   Процент миграции: {:.1}%", config.migration_rate * 100.0);
    println!("   Элитизм: {} особей", config.elitism_count);
    println!(
        "   Вероятность скрещивания: {:.1}%",
        config.crossover_rate * 100.0
    );
    println!(
        "   Вероятность мутации: {:.1}%\n",
        config.mutation_rate * 100.0
    );

    println!("🧬 Генерация начальной популяции...");
    let generator =
        InitialPopulationGenerator::new(config.clone(), frames.clone(), base_timeframe.clone());

    let initial_population = generator.generate(None).await?;
    println!(
        "   Сгенерировано {} особей\n",
        initial_population.individuals.len()
    );

    println!("🏝️  Создание островов...");
    let mut initial_populations = vec![initial_population.clone()];
    for i in 1..config.islands_count {
        let mut pop = initial_population.clone();
        pop.island_id = Some(i);
        initial_populations.push(pop);
    }

    let mut island_manager = IslandManager::new(config.clone(), initial_populations);
    println!("   Создано {} островов\n", island_manager.islands_count());

    println!("🧬 Создание генетического алгоритма...");
    let genetic_algorithm =
        GeneticAlgorithm::new(config.clone(), frames.clone(), base_timeframe.clone());

    println!("📈 Создание менеджеров эволюции...");
    let mut evolution_manager = EvolutionManager::new(config.clone());
    let migration_system = MigrationSystem::new(config.clone());
    let fresh_blood = FreshBloodSystem::new(config.clone());

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

                evolution_manager.update_fitness_history(fitness);
            }
        }

        if generation > 0 && (generation + 1) % config.migration_interval == 0 {
            println!("\n🔄 Миграция между островами...");
            let islands = island_manager.get_all_islands_mut();
            migration_system.migrate_between_islands(islands)?;
            println!("   Миграция завершена");
        }

        if generation > 0 && generation % 3 == 0 {
            println!("\n🩸 Инъекция свежей крови...");
            let islands = island_manager.get_all_islands_mut();
            for island in islands.iter_mut() {
                fresh_blood.inject_fresh_blood(island, &generator).await?;
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

    println!("🏆 Лучшие стратегии по островам:\n");
    let islands = island_manager.get_all_islands();
    for (island_idx, island) in islands.iter().enumerate() {
        let best = island.individuals.iter().max_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_a
                .partial_cmp(&fitness_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(best_individual) = best {
            println!("Остров {}:", island_idx);
            println!(
                "  Fitness: {:.4}",
                best_individual.strategy.fitness.unwrap_or(0.0)
            );
            if let Some(ref report) = best_individual.strategy.backtest_report {
                println!("  Total Profit: {:.2}", report.metrics.total_profit);
                if let Some(sharpe) = report.metrics.sharpe_ratio {
                    println!("  Sharpe Ratio: {:.2}", sharpe);
                }
                println!(
                    "  Win Rate: {:.1}%",
                    report.metrics.winning_percentage * 100.0
                );
                println!("  Trades: {}", report.trades.len());
            }
            println!();
        }
    }

    Ok(())
}
