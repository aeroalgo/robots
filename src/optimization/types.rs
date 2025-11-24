use crate::discovery::StrategyCandidate;
use crate::optimization::fitness::{FitnessThresholds, FitnessWeights};
use crate::optimization::candidate_builder_config::CandidateBuilderConfig;
use crate::strategy::types::StrategyParameterMap;
use std::collections::HashMap;

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
    pub lambda_size: usize,
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
    pub param_variants_per_candidate: usize,
    pub filter_initial_population: bool,
    pub restart_on_finish: bool,
    pub restart_on_stagnation: bool,
    pub fresh_blood_rate: f64,
    pub fresh_blood_interval: usize,
    pub detect_duplicates: bool,
    pub param_mutation_min_percent: f64,
    pub param_mutation_max_percent: f64,
    pub enable_sds: bool,
    pub sds_iterations: usize,
    pub sds_agents_ratio: f64,
    pub sds_test_threshold: f64,
    pub candidate_builder_config: Option<CandidateBuilderConfig>,
}

impl Default for GeneticAlgorithmConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            lambda_size: 100,
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
            param_variants_per_candidate: 30,
            filter_initial_population: true,
            restart_on_finish: false,
            restart_on_stagnation: false,
            fresh_blood_rate: 0.1,
            fresh_blood_interval: 3,
            detect_duplicates: true,
            param_mutation_min_percent: 0.03,
            param_mutation_max_percent: 0.05,
            enable_sds: false,
            sds_iterations: 5,
            sds_agents_ratio: 1.0,
            sds_test_threshold: 0.7,
            candidate_builder_config: Some(CandidateBuilderConfig::default()),
        }
    }
}
