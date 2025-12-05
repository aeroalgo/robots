use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
#[cfg(feature = "profiling")]
use pprof::ProfilerGuard;
use robots::candles::aggregator::TimeFrameAggregator;
use robots::data_access::database::clickhouse::OhlcvData;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::debug::{
    print_conditions_signals, print_conditions_summary, print_equity_curve_summary,
    print_quick_summary, print_strategy_debug, DebugConfig,
};
use robots::discovery::StrategyDiscoveryConfig;
use robots::indicators::registry::IndicatorFactory;
use robots::optimization::*;
use robots::strategy::executor::{BacktestConfig, BacktestExecutor};
use robots::strategy::presets::default_strategy_definitions;
use robots::strategy::types::PriceField;

fn parse_date(s: &str) -> chrono::DateTime<Utc> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .expect(&format!("Invalid date format: {}", s))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

async fn run() -> Result<()> {
    let mut connector = ClickHouseConnector::with_config(ClickHouseConfig::default());
    connector
        .connect()
        .await
        .context("Не удалось подключиться к ClickHouse")?;
    connector
        .ping()
        .await
        .context("ClickHouse не отвечает на ping")?;

    let symbol = Symbol::from_descriptor("AFLT.MM");
    let timeframe = TimeFrame::from_identifier("60");

    let start = parse_date("2020-01-01");
    let end = parse_date("2025-10-01");

    let candles: Vec<_> = connector
        .get_ohlcv_typed(&symbol, &timeframe, start, end, None)
        .await
        .context("Не удалось получить свечи из ClickHouse")?;

    println!(
        "Получено {} свечей для {} {}",
        candles.len(),
        symbol.descriptor(),
        timeframe.identifier()
    );
    if let Some(last) = candles.last() {
        println!(
            "Последняя свеча: close={}, ts={}",
            last.close, last.timestamp
        );
    }
    if candles.is_empty() {
        println!(
            "Нет данных для {} {} за указанный период",
            symbol.descriptor(),
            timeframe.identifier()
        );
        return Ok(());
    }

    let frame = QuoteFrame::try_from_ohlcv(candles.clone(), symbol.clone(), timeframe.clone())
        .context("Не удалось построить QuoteFrame из данных ClickHouse")?;

    let close_values: Vec<f32> = frame.closes().iter().collect();

    let trend_sma =
        IndicatorFactory::create_indicator("SMA", HashMap::from([("period".to_string(), 40.0)]))?;
    let trend_sma_values = trend_sma.calculate_simple(&close_values)?;

    let source_frame = frame.clone();
    let mut frames = HashMap::new();
    frames.insert(timeframe.clone(), frame);

    let strategy_name = "auto_strategy_1764972624";

    let definition = default_strategy_definitions()
        .into_iter()
        .find(|def| def.metadata.id == strategy_name)
        .context(format!("Стратегия {} не найдена", strategy_name))?;
    let config = BacktestConfig {
        initial_capital: 1000.0,
        use_full_capital: true,
        reinvest_profits: false,
    };
    let mut executor = BacktestExecutor::from_definition(definition, None, frames)
        .map_err(anyhow::Error::new)?
        .with_config(config.clone());

    // Проверка диапазонов параметров
    #[cfg(feature = "profiling")]
    let _guard = {
        std::fs::create_dir_all("profiling").ok();
        ProfilerGuard::new(100).expect("Failed to start profiler")
    };
    let start_time = std::time::Instant::now();
    let report = executor.run_backtest().map_err(anyhow::Error::new)?;
    let elapsed = start_time.elapsed();
    #[cfg(feature = "profiling")]
    {
        if let Ok(report) = _guard.report().build() {
            let file_path = "profiling/flamegraph-pprof.svg";
            std::fs::remove_file(file_path).ok();
            match std::fs::File::create(file_path) {
                Ok(file) => {
                    if let Err(e) = report.flamegraph(file) {
                        eprintln!("⚠️  Ошибка при записи flamegraph: {}", e);
                    } else {
                        println!("\n✅ Профиль сохранен в {}", file_path);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Ошибка при создании файла {}: {}", file_path, e);
                    eprintln!("   Проверьте права доступа к папке profiling/");
                }
            }
        }
    }

    println!("\n=== ВРЕМЯ ВЫПОЛНЕНИЯ БЭКТЕСТА ===");
    println!(
        "Время выполнения: {:.2} секунд ({:.2} миллисекунд)",
        elapsed.as_secs_f64(),
        elapsed.as_millis() as f64
    );

    let debug_config = DebugConfig {
        show_metrics: true,
        show_indicators: true,
        indicator_count: 20,
        show_first_trades: 100,
        show_last_trades: 100,
        show_stop_take_details: 10,
        show_conditions: true,
        condition_signals_count: 50,
        only_triggered_conditions: true,
    };

    print_strategy_debug(
        &report,
        executor.context(),
        strategy_name,
        &symbol.descriptor(),
        &timeframe,
        &debug_config,
    );

    print_equity_curve_summary(&report.equity_curve);
    print_quick_summary(&report, strategy_name);

    println!("\n=== ГЕНЕТИЧЕСКАЯ ОПТИМИЗАЦИЯ ===");
    run_genetic_optimization(&symbol, &timeframe, candles, config).await?;

    Ok(())
}

async fn run_genetic_optimization(
    symbol: &Symbol,
    base_timeframe: &TimeFrame,
    candles: Vec<OhlcvData>,
    backtest_config: BacktestConfig,
) -> Result<()> {
    println!("\n🧬 Запуск генетической оптимизации...");
    println!("   Символ: {}", symbol.descriptor());
    println!(
        "   Базовый таймфрейм: {} минут",
        base_timeframe.total_minutes().unwrap_or(60)
    );
    println!("   Количество свечей: {}\n", candles.len());

    let frame = QuoteFrame::try_from_ohlcv(candles, symbol.clone(), base_timeframe.clone())
        .context("Не удалось построить QuoteFrame")?;

    let mut frames = HashMap::new();
    frames.insert(base_timeframe.clone(), frame);

    println!("⚙️  Создание конфигурации генетического алгоритма...");
    let config = GeneticAlgorithmConfig {
        population_size: 80,
        lambda_size: 50,
        max_generations: 80,
        crossover_rate: 0.8,
        mutation_rate: 0.2,
        elitism_count: 3,
        islands_count: 5,
        migration_interval: 5,
        migration_rate: 0.06,
        fitness_thresholds: FitnessThresholds {
            min_sharpe_ratio: None,
            max_drawdown_pct: None,
            min_win_rate: None,
            min_profit_factor: Some(1.0),
            min_trades_count: Some(150),
            min_cagr: None,
            min_recovery_factor: None,
        },
        fitness_weights: FitnessWeights {
            sharpe_ratio_weight: 0.25,
            profit_factor_weight: 0.20,
            win_rate_weight: 0.10,
            cagr_weight: 0.15,
            recovery_factor_weight: 0.20,
            drawdown_penalty: 0.05,
            trades_count_bonus: 0.05,
        },
        use_existing_strategies: false,
        decimation_coefficient: 2.0,
        param_variants_per_candidate: 10,
        filter_initial_population: true,
        restart_on_finish: false,
        restart_on_stagnation: true,
        fresh_blood_rate: 0.1,
        fresh_blood_interval: 3,
        detect_duplicates: true,
        param_mutation_min_percent: 0.1,
        param_mutation_max_percent: 0.2,
        enable_sds: false,
        sds_iterations: 5,
        sds_agents_ratio: 1.0,
        sds_test_threshold: 0.7,
        candidate_builder_config: None,
    };

    println!("   Размер популяции (μ): {}", config.population_size);
    println!("   Количество потомков (λ): {}", config.lambda_size);
    println!("   Максимум поколений: {}", config.max_generations);
    println!("   Количество островов: {}", config.islands_count);
    println!("   Элитизм: {} особей", config.elitism_count);
    println!(
        "   Коэффициент децимации: {:.1}",
        config.decimation_coefficient
    );
    println!(
        "   Вариантов параметров на кандидата: {}",
        config.param_variants_per_candidate
    );
    println!(
        "   Будет сгенерировано кандидатов: {} ({} × {:.1})",
        (config.population_size as f64 * config.decimation_coefficient) as usize,
        config.population_size,
        config.decimation_coefficient
    );
    println!(
        "   Вероятность скрещивания: {:.1}%",
        config.crossover_rate * 100.0
    );
    println!(
        "   Вероятность мутации: {:.1}%",
        config.mutation_rate * 100.0
    );
    if config.enable_sds {
        println!("   Стохастический диффузионный поиск: включен");
        println!("   Итераций SDS: {}", config.sds_iterations);
        println!(
            "   Порог тестирования SDS: {:.2}",
            config.sds_test_threshold
        );
    } else {
        println!("   Стохастический диффузионный поиск: выключен");
    }
    println!();

    println!("🧬 Создание конфигурации discovery...");
    let discovery_config = StrategyDiscoveryConfig {
        max_optimization_params: 8,
        timeframe_count: 2,
        base_timeframe: base_timeframe.clone(),
        max_timeframe_minutes: 240,
    };

    println!("🧬 Генерация начальных популяций для островов...");
    let generator = InitialPopulationGenerator::with_discovery_config(
        config.clone(),
        frames.clone(),
        base_timeframe.clone(),
        discovery_config.clone(),
    );

    let mut initial_populations = Vec::with_capacity(config.islands_count);

    for island_id in 0..config.islands_count {
        println!("\n🏝️  Генерация популяции для острова {}...", island_id);
        let mut population = generator.generate(None).await?;
        population.island_id = Some(island_id);
        println!(
            "   Остров {}: сгенерировано {} особей",
            island_id,
            population.individuals.len()
        );
        initial_populations.push(population);
    }

    let total_individuals: usize = initial_populations
        .iter()
        .map(|p| p.individuals.len())
        .sum();
    println!(
        "\n   ✅ Всего создано {} особей на {} островах (по {} особей на остров)",
        total_individuals, config.islands_count, config.population_size
    );

    let mut island_manager = IslandManager::new(config.clone(), initial_populations);
    println!("   Создано {} островов\n", island_manager.islands_count());

    println!("🧬 Создание генетического алгоритма...");
    let mut genetic_algorithm = GeneticAlgorithmV3::new(
        config.clone(),
        frames.clone(),
        base_timeframe.clone(),
        discovery_config,
    )
    .with_backtest_config(backtest_config);

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

                if let Some(ref candidate) = best_individual.strategy.candidate {
                    print_strategy_info(candidate);
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

        if generation > 0 && generation % config.fresh_blood_interval == 0 {
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
            println!("═══════════════════════════════════════════════════════");
            println!("Остров {} - Лучшая стратегия:", island_idx);
            println!("═══════════════════════════════════════════════════════");
            println!(
                "Fitness: {:.4}",
                best_individual.strategy.fitness.unwrap_or(0.0)
            );

            if let Some(ref report) = best_individual.strategy.backtest_report {
                print_backtest_metrics(report);
            }

            if let Some(ref candidate) = best_individual.strategy.candidate {
                print_strategy_info(candidate);
            }

            println!();
        }
    }

    Ok(())
}

fn print_strategy_info(candidate: &robots::discovery::StrategyCandidate) {
    println!("\n📋 Информация о стратегии:");
    println!("   Индикаторы:");
    for indicator in &candidate.indicators {
        println!("     - {} ({})", indicator.name, indicator.alias);
    }

    if !candidate.nested_indicators.is_empty() {
        println!("   Вложенные индикаторы:");
        for nested in &candidate.nested_indicators {
            println!(
                "     - {} ({})",
                nested.indicator.name, nested.indicator.alias
            );
        }
    }

    if !candidate.conditions.is_empty() {
        println!("   Условия входа:");
        for condition in &candidate.conditions {
            println!("     - {} ({})", condition.name, condition.id);
        }
    }

    if !candidate.exit_conditions.is_empty() {
        println!("   Условия выхода:");
        for condition in &candidate.exit_conditions {
            println!("     - {} ({})", condition.name, condition.id);
        }
    }

    if !candidate.stop_handlers.is_empty() {
        println!("   Stop handlers:");
        for stop in &candidate.stop_handlers {
            println!("     - {} ({})", stop.name, stop.handler_name);
        }
    }

    if !candidate.take_handlers.is_empty() {
        println!("   Take handlers:");
        for take in &candidate.take_handlers {
            println!("     - {} ({})", take.name, take.handler_name);
        }
    }

    if !candidate.timeframes.is_empty() {
        println!("   Таймфреймы:");
        for tf in &candidate.timeframes {
            println!("     - {}", tf.identifier());
        }
    }
}

fn print_backtest_metrics(report: &robots::metrics::backtest::BacktestReport) {
    println!("\n📊 Метрики бэктеста:");
    println!("   === БАЗОВЫЕ МЕТРИКИ ===");
    println!(
        "   Всего сделок: {} | Прибыльных: {} | Убыточных: {}",
        report.metrics.total_trades, report.metrics.number_of_wins, report.metrics.number_of_losses
    );
    println!(
        "   Total Profit: {:.2} | Win Rate: {:.2}% | Average Trade: {:.2}",
        report.metrics.total_profit,
        report.metrics.winning_percentage * 100.0,
        report.metrics.average_trade
    );

    if let Some(aw) = report.metrics.average_win {
        println!("   Average Win: {:.2}", aw);
    }
    if let Some(al) = report.metrics.average_loss {
        println!("   Average Loss: {:.2}", al);
    }
    println!(
        "   Gross Profit: {:.2} | Gross Loss: {:.2}",
        report.metrics.gross_profit, report.metrics.gross_loss
    );

    println!("   === МЕТРИКИ РИСКА И ДОХОДНОСТИ ===");
    if let Some(pf) = report.metrics.profit_factor {
        println!("   Profit Factor: {:.2}", pf);
    }
    if let Some(sr) = report.metrics.sharpe_ratio {
        println!("   Sharpe Ratio: {:.2}", sr);
    }
    if let Some(rdd) = report.metrics.return_dd_ratio {
        println!("   Return/DD Ratio: {:.2}", rdd);
    }
    if let Some(cagr) = report.metrics.cagr {
        println!("   CAGR: {:.2}%", cagr);
    }

    println!("   === МЕТРИКИ ПРОСАДКИ ===");
    if let Some(dd) = report.metrics.drawdown {
        println!("   Max Drawdown: {:.2}", dd);
    }
    if let Some(dd_pct) = report.metrics.drawdown_percent {
        println!("   Max Drawdown %: {:.2}%", dd_pct);
    }
    println!(
        "   Max Consecutive Wins: {} | Max Consecutive Losses: {}",
        report.metrics.max_consec_wins, report.metrics.max_consec_losses
    );
}
