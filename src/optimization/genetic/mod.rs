mod crossover;
mod helpers;
mod mutation;
mod selection;

use crate::backtest::BacktestConfig;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StopHandlerConfig;
use crate::discovery::StrategyCandidate;
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::population::PopulationManager;
use crate::optimization::sds::StochasticDiffusionSearch;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::{ConditionOperator, PriceField, StrategyParameterMap};
use std::collections::HashMap;

pub struct GeneticAlgorithmV3 {
    config: GeneticAlgorithmConfig,
    population_manager: PopulationManager,
    evaluator: StrategyEvaluationRunner,
    available_indicators: Vec<crate::discovery::IndicatorInfo>,
    price_fields: Vec<PriceField>,
    operators: Vec<ConditionOperator>,
    stop_handler_configs: Vec<StopHandlerConfig>,
}

impl GeneticAlgorithmV3 {
    pub fn new(
        config: GeneticAlgorithmConfig,
        frames: HashMap<TimeFrame, QuoteFrame>,
        base_timeframe: TimeFrame,
        _discovery_config: crate::discovery::StrategyDiscoveryConfig,
    ) -> Self {
        let population_config = crate::optimization::population::PopulationConfig {
            size: config.population_size,
            elitism_count: config.elitism_count,
            crossover_rate: config.crossover_rate,
            mutation_rate: config.mutation_rate,
        };

        use crate::discovery::IndicatorInfoCollector;
        use crate::indicators::registry::IndicatorRegistry;

        let registry = IndicatorRegistry::new();
        let available_indicators = IndicatorInfoCollector::collect_from_registry(&registry);

        let price_fields = vec![PriceField::Close];

        let operators = vec![
            ConditionOperator::Above,
            ConditionOperator::Below,
            ConditionOperator::RisingTrend,
            ConditionOperator::FallingTrend,
            ConditionOperator::GreaterPercent,
            ConditionOperator::LowerPercent,
        ];

        let stop_handler_configs = vec![];

        Self {
            config,
            population_manager: PopulationManager::new(population_config),
            evaluator: StrategyEvaluationRunner::new(frames, base_timeframe),
            available_indicators,
            price_fields,
            operators,
            stop_handler_configs,
        }
    }

    pub fn with_backtest_config(mut self, config: BacktestConfig) -> Self {
        self.evaluator.set_backtest_config(config);
        self
    }

    pub async fn evolve_generation(
        &mut self,
        population: &mut Population,
    ) -> Result<(), anyhow::Error> {
        let elites = selection::select_elites(population, self.config.elitism_count);
        let lambda = self.config.lambda_size;
        let mu = population.individuals.len();
        let mut offspring = Vec::with_capacity(lambda);
        let mut evaluated_count = 0;

        while offspring.len() < lambda {
            let parents = self.population_manager.select_parents(population, 2);
            if parents.len() < 2 {
                break;
            }

            let parent1_candidate = parents[0].strategy.candidate.as_ref();
            let parent2_candidate = parents[1].strategy.candidate.as_ref();

            if let (Some(cand1), Some(cand2)) = (parent1_candidate, parent2_candidate) {
                let fitness1 = parents[0].strategy.fitness;
                let fitness2 = parents[1].strategy.fitness;

                let (mut child1_candidate, mut child2_candidate) =
                    crossover::crossover_structure_hybrid(
                        cand1,
                        cand2,
                        fitness1,
                        fitness2,
                        &self.config,
                    );

                let (mut child1_params, mut child2_params) = if let Some(params) =
                    self.population_manager.crossover(parents[0], parents[1])
                {
                    params
                } else {
                    (
                        parents[0].strategy.parameters.clone(),
                        parents[1].strategy.parameters.clone(),
                    )
                };

                mutation::mutate_structure(
                    &mut child1_candidate,
                    &self.config,
                    &self.available_indicators,
                    &self.price_fields,
                    &self.operators,
                    &self.stop_handler_configs,
                );
                mutation::mutate_structure(
                    &mut child2_candidate,
                    &self.config,
                    &self.available_indicators,
                    &self.price_fields,
                    &self.operators,
                    &self.stop_handler_configs,
                );

                use crate::discovery::strategy_converter::ParameterExtractor;
                let parameter_specs1 = ParameterExtractor::extract_all(&child1_candidate);
                let parameter_specs2 = ParameterExtractor::extract_all(&child2_candidate);

                self.population_manager.sync_parameters_with_structure(
                    &mut child1_params,
                    &child1_candidate,
                    &parameter_specs1,
                );
                self.population_manager.sync_parameters_with_structure(
                    &mut child2_params,
                    &child2_candidate,
                    &parameter_specs2,
                );

                self.population_manager.mutate(
                    &mut child1_params,
                    &child1_candidate,
                    &self.config,
                    &parameter_specs1,
                );
                self.population_manager.mutate(
                    &mut child2_params,
                    &child2_candidate,
                    &self.config,
                    &parameter_specs2,
                );

                evaluated_count += 1;
                let progress = (evaluated_count as f64 / lambda as f64) * 100.0;
                println!(
                    "      [{}/{}] ({:.1}%) Оценка новой особи...",
                    evaluated_count, lambda, progress
                );

                helpers::log_strategy_details(&child1_candidate, &child1_params, "Child1");

                match self
                    .create_individual(
                        child1_candidate,
                        child1_params,
                        population.generation + 1,
                        population.island_id,
                    )
                    .await
                {
                    Ok(child1) => {
                        offspring.push(child1);
                    }
                    Err(e) => {
                        eprintln!("      ❌ Ошибка оценки особи: {:?}", e);
                        if let Some(source) = e.source() {
                            eprintln!("      Источник ошибки: {:?}", source);
                        }
                        continue;
                    }
                }

                if offspring.len() < lambda {
                    evaluated_count += 1;
                    let progress = (evaluated_count as f64 / lambda as f64) * 100.0;
                    println!(
                        "      [{}/{}] ({:.1}%) Оценка новой особи...",
                        evaluated_count, lambda, progress
                    );

                    helpers::log_strategy_details(&child2_candidate, &child2_params, "Child2");

                    match self
                        .create_individual(
                            child2_candidate,
                            child2_params,
                            population.generation + 1,
                            population.island_id,
                        )
                        .await
                    {
                        Ok(child2) => {
                            offspring.push(child2);
                        }
                        Err(e) => {
                            eprintln!("      ❌ Ошибка оценки особи: {:?}", e);
                            if let Some(source) = e.source() {
                                eprintln!("      Источник ошибки: {:?}", source);
                            }
                        }
                    }
                }
            }
        }

        let mut combined_population =
            Vec::with_capacity(population.individuals.len() + offspring.len());
        combined_population.extend_from_slice(&population.individuals);
        combined_population.extend(offspring);

        if self.config.enable_sds {
            let mut temp_population = Population {
                individuals: combined_population,
                generation: population.generation,
                island_id: population.island_id,
            };

            let sds = StochasticDiffusionSearch::new(self.config.clone());
            println!("      [SDS] Применение стохастического диффузионного поиска...");
            sds.apply_diffusion(&mut temp_population, &self.evaluator)
                .await?;

            combined_population = temp_population.individuals;
            println!("      [SDS] Диффузионный поиск завершен");
        }

        population.individuals = selection::select_with_diversity(combined_population, mu);

        self.population_manager.apply_elitism(population, elites);
        population.generation += 1;

        Ok(())
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
            candidate: Some(candidate),
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
