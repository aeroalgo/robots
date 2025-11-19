use crate::discovery::StrategyCandidate;
use crate::optimization::fitness::{FitnessFunction, FitnessThresholds, FitnessWeights};
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

    pub fn mutate(
        &self,
        parameters: &mut StrategyParameterMap,
        candidate: &StrategyCandidate,
        mutation_config: &crate::optimization::types::GeneticAlgorithmConfig,
    ) {
        use crate::indicators::implementations::get_optimization_range;
        use crate::risk::stops::get_optimization_range as get_stop_optimization_range;
        use crate::indicators::types::ParameterType;
        
        let mut rng = rand::thread_rng();
        let keys: Vec<String> = parameters.keys().cloned().collect();

        for key in keys {
            if rng.gen::<f64>() < self.config.mutation_rate {
                if let Some(param_value) = parameters.get_mut(&key) {
                    let range = Self::get_parameter_range(&key, candidate);
                    if let Some(opt_range) = range {
                        Self::mutate_parameter_with_range(
                            param_value,
                            &opt_range,
                            mutation_config.param_mutation_min_percent,
                            mutation_config.param_mutation_max_percent,
                        );
                    } else {
                        Self::mutate_parameter_fallback(param_value);
                    }
                }
            }
        }
    }

    fn get_parameter_range(
        key: &str,
        candidate: &StrategyCandidate,
    ) -> Option<crate::indicators::implementations::OptimizationRange> {
        use crate::indicators::implementations::{get_optimization_range, OptimizationRange};
        use crate::risk::stops::get_optimization_range as get_stop_optimization_range;
        use crate::indicators::types::ParameterType;

        if key.starts_with("stop_") {
            let parts: Vec<&str> = key.strip_prefix("stop_")?.split('_').collect();
            if parts.len() >= 2 {
                let handler_name = parts[0];
                let param_name = parts[1..].join("_");
                return get_stop_optimization_range(handler_name, &param_name);
            }
        } else if key.starts_with("nested_") {
            let parts: Vec<&str> = key.strip_prefix("nested_")?.split('_').collect();
            if parts.len() >= 2 {
                let indicator_name = parts[0];
                let param_name = parts[1..].join("_");
                if let Some(nested) = candidate.nested_indicators.iter().find(|n| n.indicator.name == indicator_name) {
                    for param in &nested.indicator.parameters {
                        if param.name == param_name {
                            return get_optimization_range(&nested.indicator.name, &param_name, &param.param_type);
                        }
                    }
                }
            }
        } else if key.starts_with("condition_") {
            let parts: Vec<&str> = key.split('_').collect();
            if parts.len() >= 3 {
                let condition_id = parts[1];
                let param_name = parts[2..].join("_");
                if let Some(condition) = candidate.conditions.iter().find(|c| c.id == condition_id) {
                    if let Some(indicator_name) = Self::extract_indicator_name_from_condition(candidate, condition) {
                        let param_type = match param_name.to_lowercase().as_str() {
                            "threshold" => ParameterType::Threshold,
                            "percentage" | "percent" => ParameterType::Multiplier,
                            _ => ParameterType::Threshold,
                        };
                        return get_optimization_range(&indicator_name, &param_name, &param_type);
                    }
                }
                if let Some(condition) = candidate.exit_conditions.iter().find(|c| c.id == condition_id) {
                    if let Some(indicator_name) = Self::extract_indicator_name_from_condition(candidate, condition) {
                        let param_type = match param_name.to_lowercase().as_str() {
                            "threshold" => ParameterType::Threshold,
                            "percentage" | "percent" => ParameterType::Multiplier,
                            _ => ParameterType::Threshold,
                        };
                        return get_optimization_range(&indicator_name, &param_name, &param_type);
                    }
                }
            }
        } else {
            let parts: Vec<&str> = key.split('_').collect();
            if parts.len() >= 2 {
                let indicator_name = parts[0];
                let param_name = parts[1..].join("_");
                if let Some(indicator) = candidate.indicators.iter().find(|i| i.name == indicator_name) {
                    for param in &indicator.parameters {
                        if param.name == param_name {
                            return get_optimization_range(&indicator.name, &param_name, &param.param_type);
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_indicator_name_from_condition(
        candidate: &StrategyCandidate,
        condition: &crate::discovery::ConditionInfo,
    ) -> Option<String> {
        let alias = Self::extract_indicator_alias_from_condition_id(&condition.id)?;
        
        if let Some(ind) = candidate.indicators.iter().find(|i| i.alias == alias) {
            return Some(ind.name.clone());
        }
        
        if let Some(nested) = candidate.nested_indicators.iter().find(|n| n.indicator.alias == alias) {
            return Some(nested.indicator.name.clone());
        }
        
        None
    }

    fn extract_indicator_alias_from_condition_id(condition_id: &str) -> Option<String> {
        if condition_id.starts_with("ind_price_") {
            let rest = condition_id.strip_prefix("ind_price_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            }
        } else if condition_id.starts_with("ind_const_") {
            let rest = condition_id.strip_prefix("ind_const_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            }
        }
        None
    }

    fn mutate_parameter_with_range(
        value: &mut crate::strategy::types::StrategyParamValue,
        range: &crate::indicators::implementations::OptimizationRange,
        min_percent: f64,
        max_percent: f64,
    ) {
        use crate::strategy::types::StrategyParamValue;
        let mut rng = rand::thread_rng();
        
        let range_size = (range.end - range.start) as f64;
        let mutation_percent = rng.gen_range(min_percent..=max_percent);
        let mutation_amount = range_size * mutation_percent;
        let mutation = rng.gen_range(-mutation_amount..=mutation_amount);

        match value {
            StrategyParamValue::Number(n) => {
                *n = (*n + mutation).max(range.start as f64).min(range.end as f64);
            }
            StrategyParamValue::Integer(i) => {
                *i = ((*i as f64 + mutation) as i64).max(range.start as i64).min(range.end as i64);
            }
            StrategyParamValue::Flag(b) => {
                *b = !*b;
            }
            _ => {}
        }
    }

    fn mutate_parameter_fallback(value: &mut crate::strategy::types::StrategyParamValue) {
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
