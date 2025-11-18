use crate::discovery::StrategyCandidate;
use crate::optimization::fitness::{FitnessFunction, FitnessThresholds, FitnessWeights};
use crate::optimization::genetic::GeneticAlgorithm;
use crate::optimization::types::{EvaluatedStrategy, GeneticIndividual, Population};
use crate::strategy::types::StrategyParameterMap;
use rand::Rng;
use std::collections::HashMap;

pub struct PopulationManager {
    config: PopulationConfig,
}

#[derive(Clone, Debug)]
pub struct PopulationConfig {
    pub size: usize,
    pub elitism_count: usize,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
}

impl Default for PopulationConfig {
    fn default() -> Self {
        Self {
            size: 100,
            elitism_count: 5,
            crossover_rate: 0.7,
            mutation_rate: 0.1,
        }
    }
}

impl PopulationManager {
    pub fn new(config: PopulationConfig) -> Self {
        Self { config }
    }

    pub fn select_parents<'a>(
        &self,
        population: &'a Population,
        count: usize,
    ) -> Vec<&'a GeneticIndividual> {
        let mut rng = rand::thread_rng();
        let mut selected = Vec::new();
        let total_fitness: f64 = population
            .individuals
            .iter()
            .filter_map(|ind| ind.strategy.fitness)
            .sum();

        if total_fitness == 0.0 {
            for _ in 0..count {
                let idx = rng.gen_range(0..population.individuals.len());
                selected.push(&population.individuals[idx]);
            }
            return selected;
        }

        for _ in 0..count {
            let random = rng.gen_range(0.0..total_fitness);
            let mut cumulative = 0.0;

            for individual in &population.individuals {
                if let Some(fitness) = individual.strategy.fitness {
                    cumulative += fitness;
                    if cumulative >= random {
                        selected.push(individual);
                        break;
                    }
                }
            }
        }

        selected
    }

    pub fn crossover(
        &self,
        parent1: &GeneticIndividual,
        parent2: &GeneticIndividual,
    ) -> Option<(StrategyParameterMap, StrategyParameterMap)> {
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() > self.config.crossover_rate {
            return None;
        }

        let params1 = &parent1.strategy.parameters;
        let params2 = &parent2.strategy.parameters;

        let mut child1 = HashMap::new();
        let mut child2 = HashMap::new();

        let all_keys: Vec<String> = params1
            .keys()
            .chain(params2.keys())
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for key in all_keys {
            let val1 = params1.get(&key);
            let val2 = params2.get(&key);

            if rng.gen::<f64>() < 0.5 {
                if let Some(v1) = val1 {
                    child1.insert(key.clone(), v1.clone());
                }
                if let Some(v2) = val2 {
                    child2.insert(key.clone(), v2.clone());
                }
            } else {
                if let Some(v2) = val2 {
                    child1.insert(key.clone(), v2.clone());
                }
                if let Some(v1) = val1 {
                    child2.insert(key.clone(), v1.clone());
                }
            }
        }

        Some((child1, child2))
    }

    pub fn mutate(&self, parameters: &mut StrategyParameterMap, candidate: &StrategyCandidate) {
        let mut rng = rand::thread_rng();
        let keys: Vec<String> = parameters.keys().cloned().collect();

        for key in keys {
            if rng.gen::<f64>() < self.config.mutation_rate {
                if let Some(param_value) = parameters.get_mut(&key) {
                    Self::mutate_parameter(param_value);
                }
            }
        }
    }

    fn mutate_parameter(value: &mut crate::strategy::types::StrategyParamValue) {
        use crate::strategy::types::StrategyParamValue;
        let mut rng = rand::thread_rng();

        match value {
            StrategyParamValue::Number(n) => {
                let mutation = rng.gen_range(-0.1..0.1) * n.abs();
                *n += mutation;
            }
            StrategyParamValue::Integer(i) => {
                let mutation = rng.gen_range(-2..=2);
                *i += mutation;
            }
            StrategyParamValue::Flag(b) => {
                *b = !*b;
            }
            _ => {}
        }
    }

    pub fn apply_elitism(&self, population: &mut Population, elites: Vec<GeneticIndividual>) {
        population.individuals.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, elite) in elites.into_iter().enumerate() {
            if i < population.individuals.len() {
                population.individuals[i] = elite;
            }
        }
    }

    pub fn replace_weakest(
        &self,
        population: &mut Population,
        new_individuals: Vec<GeneticIndividual>,
    ) {
        population.individuals.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_a
                .partial_cmp(&fitness_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, new_ind) in new_individuals.into_iter().enumerate() {
            if i < population.individuals.len() {
                population.individuals[i] = new_ind;
            }
        }
    }
}
