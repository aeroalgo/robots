use crate::optimization::types::{GeneticAlgorithmConfig, GeneticIndividual, Population};
use rand::Rng;
use std::collections::HashMap;

pub struct StochasticDiffusionSearch {
    config: GeneticAlgorithmConfig,
}

impl StochasticDiffusionSearch {
    pub fn new(config: GeneticAlgorithmConfig) -> Self {
        Self { config }
    }

    pub async fn apply_diffusion(
        &self,
        population: &mut Population,
        evaluator: &crate::optimization::evaluator::StrategyEvaluationRunner,
    ) -> Result<(), anyhow::Error> {
        if !self.config.enable_sds {
            return Ok(());
        }

        let agents_count =
            (population.individuals.len() as f64 * self.config.sds_agents_ratio) as usize;
        if agents_count == 0 || agents_count > population.individuals.len() {
            return Ok(());
        }

        for iteration in 0..self.config.sds_iterations {
            println!(
                "      [SDS] Итерация диффузии {}/{}...",
                iteration + 1,
                self.config.sds_iterations
            );

            let mut agent_states: Vec<AgentState> =
                Vec::with_capacity(population.individuals.len());
            let mut rng = rand::thread_rng();

            for individual in &population.individuals {
                let hypothesis = individual.strategy.parameters.clone();
                let test_result = self.partial_evaluation(individual);
                let active = test_result >= self.config.sds_test_threshold;

                agent_states.push(AgentState {
                    hypothesis,
                    test_result,
                    active,
                    fitness: individual.strategy.fitness.unwrap_or(0.0),
                });
            }

            let active_count = agent_states.iter().filter(|s| s.active).count();
            if active_count == 0 {
                continue;
            }

            for i in 0..agent_states.len() {
                if !agent_states[i].active {
                    let random_agent_idx = rng.gen_range(0..agent_states.len());
                    if agent_states[random_agent_idx].active {
                        agent_states[i].hypothesis =
                            agent_states[random_agent_idx].hypothesis.clone();
                        agent_states[i].test_result = agent_states[random_agent_idx].test_result;
                    } else {
                        let inactive_agents: Vec<usize> = agent_states
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| !s.active)
                            .map(|(idx, _)| idx)
                            .collect();

                        if !inactive_agents.is_empty() {
                            let random_idx =
                                inactive_agents[rng.gen_range(0..inactive_agents.len())];
                            agent_states[i].hypothesis = population.individuals[random_idx]
                                .strategy
                                .parameters
                                .clone();
                        }
                    }
                }
            }

            let mut clusters: HashMap<String, Vec<usize>> = HashMap::new();
            for (idx, state) in agent_states.iter().enumerate() {
                if state.active {
                    let signature = self.create_hypothesis_signature(&state.hypothesis);
                    clusters.entry(signature).or_insert_with(Vec::new).push(idx);
                }
            }

            let mut best_hypothesis_map: HashMap<String, usize> = HashMap::new();
            for (signature, agent_indices) in clusters.iter() {
                if agent_indices.len() > 1 {
                    let best_idx = *agent_indices
                        .iter()
                        .max_by(|&&a, &&b| {
                            agent_states[a]
                                .fitness
                                .partial_cmp(&agent_states[b].fitness)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .unwrap();
                    best_hypothesis_map.insert(signature.clone(), best_idx);
                }
            }

            let hypothesis_updates: Vec<(usize, crate::strategy::types::StrategyParameterMap)> = {
                let mut updates = Vec::new();
                for (idx, state) in agent_states.iter().enumerate() {
                    if !state.active {
                        continue;
                    }

                    let signature = self.create_hypothesis_signature(&state.hypothesis);
                    if let Some(&best_idx) = best_hypothesis_map.get(&signature) {
                        if idx != best_idx {
                            let best_hypothesis = agent_states[best_idx].hypothesis.clone();
                            updates.push((idx, best_hypothesis));
                        }
                    }
                }
                updates
            };

            for (idx, new_hypothesis) in hypothesis_updates {
                agent_states[idx].hypothesis = new_hypothesis;
            }

            let mut fitness_updates = Vec::new();
            for (i, state) in agent_states.iter().enumerate() {
                if let Some(candidate) = &population.individuals[i].strategy.candidate {
                    let updated_params = state.hypothesis.clone();
                    let current_fitness = state.fitness;

                    let new_fitness = self
                        .evaluate_hypothesis(
                            candidate.clone(),
                            updated_params.clone(),
                            evaluator,
                            population.generation,
                            population.island_id,
                        )
                        .await?;

                    if new_fitness > current_fitness {
                        fitness_updates.push((i, new_fitness, updated_params));
                    }
                }
            }

            for (i, new_fitness, updated_params) in fitness_updates {
                population.individuals[i].strategy.parameters = updated_params;
                population.individuals[i].strategy.fitness = Some(new_fitness);
                agent_states[i].fitness = new_fitness;
            }
        }

        Ok(())
    }

    fn partial_evaluation(&self, individual: &GeneticIndividual) -> f64 {
        let fitness = individual.strategy.fitness.unwrap_or(0.0);
        if fitness <= 0.0 {
            return 0.0;
        }

        let mut test_score = 0.0;
        let mut test_count = 0;

        if let Some(ref report) = individual.strategy.backtest_report {
            if let Some(sharpe) = report.metrics.sharpe_ratio {
                if sharpe > 0.0 {
                    test_score += (sharpe / 3.0).min(1.0);
                    test_count += 1;
                }
            }

            if let Some(pf) = report.metrics.profit_factor {
                if pf > 1.0 {
                    test_score += ((pf - 1.0) / 4.0).min(1.0);
                    test_count += 1;
                }
            }

            if report.metrics.winning_percentage > 0.5 {
                test_score += report.metrics.winning_percentage;
                test_count += 1;
            }

            if report.metrics.total_profit > 0.0 {
                test_score += (report.metrics.total_profit / 10000.0).min(1.0);
                test_count += 1;
            }

            if test_count > 0 {
                test_score /= test_count as f64;
            }
        }

        test_score
    }

    fn create_hypothesis_signature(
        &self,
        hypothesis: &crate::strategy::types::StrategyParameterMap,
    ) -> String {
        let mut keys: Vec<&String> = hypothesis.keys().collect();
        keys.sort();

        let mut parts = Vec::new();
        for key in keys {
            if let Some(value) = hypothesis.get(key) {
                parts.push(format!("{}:{:?}", key, value));
            }
        }

        parts.join("|")
    }

    async fn evaluate_hypothesis(
        &self,
        candidate: crate::discovery::StrategyCandidate,
        parameters: crate::strategy::types::StrategyParameterMap,
        evaluator: &crate::optimization::evaluator::StrategyEvaluationRunner,
        _generation: usize,
        _island_id: Option<usize>,
    ) -> Result<f64, anyhow::Error> {
        let report = evaluator.evaluate_strategy(&candidate, parameters).await?;

        let fitness = crate::optimization::fitness::FitnessFunction::calculate_fitness(
            &report,
            &self.config.fitness_weights,
        );

        Ok(fitness)
    }
}

struct AgentState {
    hypothesis: crate::strategy::types::StrategyParameterMap,
    test_result: f64,
    active: bool,
    fitness: f64,
}
