use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StrategyCandidate;
use crate::optimization::genetic::GeneticAlgorithmV3;
use crate::optimization::initial_population::InitialPopulationGenerator;
use crate::optimization::types::GeneticAlgorithmConfig;
use crate::strategy::types::StrategyParameterMap;
use std::collections::HashMap;

pub struct PerStructureOptimizer {
    config: GeneticAlgorithmConfig,
    frames: HashMap<TimeFrame, QuoteFrame>,
    base_timeframe: TimeFrame,
    discovery_config: crate::discovery::StrategyDiscoveryConfig,
}

pub struct OptimizedStrategyResult {
    pub candidate: StrategyCandidate,
    pub parameters: StrategyParameterMap,
    pub fitness: f64,
    pub backtest_report: crate::metrics::backtest::BacktestReport,
}

impl PerStructureOptimizer {
    pub fn new(
        config: GeneticAlgorithmConfig,
        frames: HashMap<TimeFrame, QuoteFrame>,
        base_timeframe: TimeFrame,
        discovery_config: crate::discovery::StrategyDiscoveryConfig,
    ) -> Self {
        Self {
            config,
            frames,
            base_timeframe,
            discovery_config,
        }
    }

    pub async fn optimize_structure(
        &self,
        candidate: StrategyCandidate,
    ) -> Result<Vec<OptimizedStrategyResult>, anyhow::Error> {
        println!("\n🔬 Оптимизация структуры стратегии:");
        println!("   Индикаторы: {}", candidate.indicators.len());
        println!("   Условия входа: {}", candidate.conditions.len());
        println!("   Условия выхода: {}", candidate.exit_conditions.len());
        println!("   Стоп-обработчики: {}", candidate.stop_handlers.len());

        let generator = InitialPopulationGenerator::with_discovery_config(
            self.config.clone(),
            self.frames.clone(),
            self.base_timeframe.clone(),
            self.discovery_config.clone(),
        );

        let mut population = generator.generate(Some(vec![candidate.clone()])).await?;
        population.island_id = None;

        let mut genetic_algorithm = GeneticAlgorithmV3::new(
            self.config.clone(),
            self.frames.clone(),
            self.base_timeframe.clone(),
            self.discovery_config.clone(),
        );

        println!(
            "   Начальная популяция: {} стратегий",
            population.individuals.len()
        );
        println!(
            "   Запуск эволюции на {} поколений...",
            self.config.max_generations
        );

        for generation in 0..self.config.max_generations {
            genetic_algorithm.evolve_generation(&mut population).await?;

            if (generation + 1) % 5 == 0 || generation == self.config.max_generations - 1 {
                let best = population.individuals.iter().max_by(|a, b| {
                    let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                    let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                    fitness_a
                        .partial_cmp(&fitness_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some(best_individual) = best {
                    let fitness = best_individual.strategy.fitness.unwrap_or(0.0);
                    println!(
                        "   Поколение {}: лучший fitness = {:.4}",
                        generation + 1,
                        fitness
                    );
                }
            }
        }

        println!("   ✅ Оптимизация завершена");

        let mut results = Vec::new();
        for individual in population.individuals {
            if let Some(candidate) = individual.strategy.candidate {
                if let Some(backtest_report) = individual.strategy.backtest_report {
                    results.push(OptimizedStrategyResult {
                        candidate,
                        parameters: individual.strategy.parameters,
                        fitness: individual.strategy.fitness.unwrap_or(0.0),
                        backtest_report,
                    });
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::config::StrategyDiscoveryConfig;

    fn create_test_config() -> GeneticAlgorithmConfig {
        GeneticAlgorithmConfig {
            max_generations: 5,
            ..Default::default()
        }
    }

    fn create_test_candidate() -> StrategyCandidate {
        StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![],
            config: StrategyDiscoveryConfig::default(),
        }
    }

    fn create_test_frames() -> HashMap<TimeFrame, QuoteFrame> {
        let mut frames = HashMap::new();
        let tf = TimeFrame::from_identifier("60");
        let frame = QuoteFrame::new(
            crate::data_model::types::Symbol::new("BTCUSDT".to_string()),
            tf.clone(),
        );
        frames.insert(tf, frame);
        frames
    }

    #[test]
    fn test_per_structure_optimizer_new() {
        let config = create_test_config();
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let discovery_config = StrategyDiscoveryConfig::default();
        let optimizer = PerStructureOptimizer::new(config, frames, base_tf, discovery_config);
        assert!(true);
    }

    #[test]
    fn test_optimized_strategy_result() {
        let candidate = create_test_candidate();
        let params = HashMap::new();
        let result = OptimizedStrategyResult {
            candidate,
            parameters: params,
            fitness: 1.5,
            backtest_report: crate::metrics::backtest::BacktestReport::new(
                vec![],
                crate::metrics::backtest::BacktestMetrics::default(),
                vec![],
            ),
        };
        assert_eq!(result.fitness, 1.5);
    }
}
