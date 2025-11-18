use std::collections::HashMap;
use crate::discovery::StrategyCandidate;
use crate::strategy::types::StrategyParameterMap;
use crate::optimization::fitness::{FitnessThresholds, FitnessWeights};

#[derive(Clone, Debug)]
pub struct EvaluatedStrategy {
    pub candidate: Option<StrategyCandidate>,
    pub parameters: StrategyParameterMap,
    pub fitness: Option<f64>,
    pub backtest_report: Option<crate::metrics::backtest::BacktestReport>,
}

#[derive(Clone, Debug)]
pub struct GeneticIndividual {
    pub strategy: EvaluatedStrategy,
    pub generation: usize,
    pub island_id: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Population {
    pub individuals: Vec<GeneticIndividual>,
    pub generation: usize,
    pub island_id: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct GeneticAlgorithmConfig {
    pub population_size: usize,
    pub max_generations: usize,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
    pub elitism_count: usize,
    pub islands_count: usize,
    pub migration_interval: usize,
    pub migration_rate: f64,
    pub fitness_thresholds: FitnessThresholds,
    pub fitness_weights: FitnessWeights,
    pub use_existing_strategies: bool,
    pub decimation_coefficient: f64,
    pub filter_initial_population: bool,
    pub restart_on_finish: bool,
    pub restart_on_stagnation: bool,
    pub fresh_blood_rate: f64,
    pub detect_duplicates: bool,
}

impl Default for GeneticAlgorithmConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 50,
            crossover_rate: 0.7,
            mutation_rate: 0.1,
            elitism_count: 5,
            islands_count: 1,
            migration_interval: 10,
            migration_rate: 0.05,
            fitness_thresholds: FitnessThresholds::default(),
            fitness_weights: FitnessWeights::default(),
            use_existing_strategies: false,
            decimation_coefficient: 2.0,
            filter_initial_population: true,
            restart_on_finish: false,
            restart_on_stagnation: false,
            fresh_blood_rate: 0.1,
            detect_duplicates: true,
        }
    }
}

