use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StrategyCandidate;
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::population::PopulationManager;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::StrategyParameterMap;
use std::collections::HashMap;

pub struct GeneticAlgorithmV2 {
    config: GeneticAlgorithmConfig,
    population_manager: PopulationManager,
    evaluator: StrategyEvaluationRunner,
}

impl GeneticAlgorithmV2 {
    pub fn new(
        config: GeneticAlgorithmConfig,
        frames: HashMap<TimeFrame, QuoteFrame>,
        base_timeframe: TimeFrame,
    ) -> Self {
        let population_config = crate::optimization::population::PopulationConfig {
            size: config.population_size,
            elitism_count: config.elitism_count,
            crossover_rate: config.crossover_rate,
            mutation_rate: config.mutation_rate,
        };

        Self {
            config,
            population_manager: PopulationManager::new(population_config),
            evaluator: StrategyEvaluationRunner::new(frames, base_timeframe),
        }
    }

    pub async fn evolve_generation(
        &self,
        population: &mut Population,
    ) -> Result<(), anyhow::Error> {
        let elites = self.select_elites(population);
        let mut new_individuals = Vec::new();

        while new_individuals.len() < population.individuals.len() - elites.len() {
            let parents = self.population_manager.select_parents(population, 2);
            if parents.len() < 2 {
                break;
            }

            if let Some((child1_params, child2_params)) =
                self.population_manager.crossover(parents[0], parents[1])
            {
                let mut child1_params = child1_params;
                let mut child2_params = child2_params;

                if let Some(ref candidate) = parents[0].strategy.candidate {
                    self.population_manager
                        .mutate(&mut child1_params, candidate);
                }
                if let Some(ref candidate) = parents[1].strategy.candidate {
                    self.population_manager
                        .mutate(&mut child2_params, candidate);
                }

                if let Some(ref candidate) = parents[0].strategy.candidate {
                    let child1 = self
                        .create_individual(
                            candidate.clone(),
                            child1_params,
                            population.generation + 1,
                            population.island_id,
                        )
                        .await?;
                    new_individuals.push(child1);
                }

                if new_individuals.len() < population.individuals.len() - elites.len() {
                    if let Some(ref candidate) = parents[1].strategy.candidate {
                        let child2 = self
                            .create_individual(
                                candidate.clone(),
                                child2_params,
                                population.generation + 1,
                                population.island_id,
                            )
                            .await?;
                        new_individuals.push(child2);
                    }
                }
            }
        }

        self.population_manager
            .replace_weakest(population, new_individuals);
        self.population_manager.apply_elitism(population, elites);
        population.generation += 1;

        Ok(())
    }

    fn select_elites(&self, population: &Population) -> Vec<GeneticIndividual> {
        let mut sorted: Vec<&GeneticIndividual> = population.individuals.iter().collect();
        sorted.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        sorted
            .into_iter()
            .take(self.config.elitism_count)
            .cloned()
            .collect()
    }

    async fn create_individual(
        &self,
        candidate: StrategyCandidate,
        parameters: StrategyParameterMap,
        generation: usize,
        island_id: Option<usize>,
    ) -> Result<GeneticIndividual, anyhow::Error> {
        let report = self
            .evaluator
            .evaluate_strategy(&candidate, parameters.clone())
            .await?;

        let fitness = Some(
            crate::optimization::fitness::FitnessFunction::calculate_fitness(
                &report,
                &self.config.fitness_weights,
            ),
        );

        let evaluated = EvaluatedStrategy {
            candidate: Some(candidate.clone()),
            parameters,
            fitness,
            backtest_report: Some(report),
        };

        Ok(GeneticIndividual {
            strategy: evaluated,
            generation,
            island_id,
        })
    }
}
