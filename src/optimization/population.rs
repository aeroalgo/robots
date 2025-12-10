use crate::discovery::StrategyCandidate;
use crate::optimization::types::{GeneticIndividual, Population};
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
        let mut selected = Vec::with_capacity(count);
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

        let estimated_size = params1.len().max(params2.len());
        let mut child1 = HashMap::with_capacity(estimated_size);
        let mut child2 = HashMap::with_capacity(estimated_size);

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
        parameter_specs: &[crate::strategy::types::StrategyParameterSpec],
    ) {
        use crate::strategy::types::{ParameterKind, StrategyParameterSpec};

        let mut rng = rand::thread_rng();
        let keys: Vec<String> = parameters.keys().cloned().collect();

        for key in keys {
            if rng.gen::<f64>() < self.config.mutation_rate {
                let spec = match parameter_specs.iter().find(|s| s.name == key) {
                    Some(s) if s.mutatable => s,
                    _ => continue,
                };

                let indicator_name_for_param =
                    if let ParameterKind::IndicatorParameter { indicator_name_ref } =
                        &spec.parameter_kind
                    {
                        parameters
                            .get(indicator_name_ref)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "SMA".to_string())
                    } else {
                        String::new()
                    };

                if let Some(param_value) = parameters.get_mut(&key) {
                    match &spec.parameter_kind {
                        ParameterKind::Numeric => {
                            if let (Some(min), Some(max), Some(step)) =
                                (spec.min, spec.max, spec.step)
                            {
                                let range = crate::indicators::types::ParameterRange {
                                    start: min as f32,
                                    end: max as f32,
                                    step: step as f32,
                                    current: param_value.as_f64().unwrap_or(min) as f32,
                                };
                                Self::mutate_parameter_with_range(
                                    param_value,
                                    &range,
                                    mutation_config.param_mutation_min_percent,
                                    mutation_config.param_mutation_max_percent,
                                );
                            }
                        }
                        ParameterKind::Discrete => {
                            if let Some(discrete_values) = &spec.discrete_values {
                                Self::mutate_discrete_parameter(param_value, discrete_values);
                            }
                        }
                        ParameterKind::IndicatorName { category } => {
                            let old_indicator_name = param_value.as_str().map(|s| s.to_string());
                            Self::mutate_indicator_name(param_value, category);
                            let new_indicator_name = param_value.as_str().map(|s| s.to_string());

                            if old_indicator_name != new_indicator_name {
                                Self::update_related_indicator_parameters(
                                    parameters,
                                    &key,
                                    &new_indicator_name.unwrap_or_default(),
                                    parameter_specs,
                                    candidate,
                                );
                            }
                        }
                        ParameterKind::ConditionOperator {
                            compatible_operators,
                        } => {
                            Self::mutate_operator(param_value, compatible_operators);
                        }
                        ParameterKind::IndicatorParameter { .. } => {
                            if let (Some(min), Some(max), Some(step)) =
                                (spec.min, spec.max, spec.step)
                            {
                                let range = crate::indicators::types::ParameterRange {
                                    start: min as f32,
                                    end: max as f32,
                                    step: step as f32,
                                    current: param_value.as_f64().unwrap_or(min) as f32,
                                };
                                Self::mutate_parameter_with_range(
                                    param_value,
                                    &range,
                                    mutation_config.param_mutation_min_percent,
                                    mutation_config.param_mutation_max_percent,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn extract_indicator_name_from_condition(
        candidate: &StrategyCandidate,
        condition: &crate::discovery::ConditionInfo,
    ) -> Option<String> {
        let alias = &condition.primary_indicator_alias;

        if let Some(ind) = candidate.indicators.iter().find(|i| i.alias == *alias) {
            return Some(ind.name.clone());
        }

        if let Some(nested) = candidate
            .nested_indicators
            .iter()
            .find(|n| n.indicator.alias == *alias)
        {
            return Some(nested.indicator.name.clone());
        }

        None
    }

    fn mutate_parameter_with_range(
        value: &mut crate::strategy::types::StrategyParamValue,
        range: &crate::indicators::types::ParameterRange,
        min_percent: f64,
        max_percent: f64,
    ) {
        use crate::strategy::types::StrategyParamValue;
        let mut rng = rand::thread_rng();

        let range_size = (range.end - range.start) as f64;
        let mutation_percent = rng.gen_range(min_percent..=max_percent);
        let mutation_amount = range_size * mutation_percent;

        match value {
            StrategyParamValue::Number(n) => {
                let current = *n;
                let range_start = range.start as f64;
                let range_end = range.end as f64;

                let distance_to_start = current - range_start;
                let distance_to_end = range_end - current;

                let max_mutation_up = distance_to_end.min(mutation_amount);
                let max_mutation_down = distance_to_start.min(mutation_amount);

                let mutation = if max_mutation_up > 0.0 && max_mutation_down > 0.0 {
                    rng.gen_range(-max_mutation_down..=max_mutation_up)
                } else if max_mutation_up > 0.0 {
                    rng.gen_range(0.0..=max_mutation_up)
                } else if max_mutation_down > 0.0 {
                    rng.gen_range(-max_mutation_down..=0.0)
                } else {
                    0.0
                };

                *n = (current + mutation).max(range_start).min(range_end);
            }
            StrategyParamValue::Integer(i) => {
                let current = *i as f64;
                let range_start = range.start as f64;
                let range_end = range.end as f64;

                let distance_to_start = current - range_start;
                let distance_to_end = range_end - current;

                let max_mutation_up = distance_to_end.min(mutation_amount);
                let max_mutation_down = distance_to_start.min(mutation_amount);

                let mutation = if max_mutation_up > 0.0 && max_mutation_down > 0.0 {
                    rng.gen_range(-max_mutation_down..=max_mutation_up)
                } else if max_mutation_up > 0.0 {
                    rng.gen_range(0.0..=max_mutation_up)
                } else if max_mutation_down > 0.0 {
                    rng.gen_range(-max_mutation_down..=0.0)
                } else {
                    0.0
                };

                let new_value = (current + mutation).max(range_start).min(range_end);
                *i = new_value as i64;
            }
            StrategyParamValue::Flag(b) => {
                *b = !*b;
            }
            _ => {}
        }
    }

    fn mutate_discrete_parameter(
        value: &mut crate::strategy::types::StrategyParamValue,
        discrete_values: &[crate::strategy::types::StrategyParamValue],
    ) {
        if discrete_values.is_empty() {
            return;
        }

        let current_idx = discrete_values.iter().position(|v| v == value).unwrap_or(0);

        let mut rng = rand::thread_rng();
        let new_idx = if discrete_values.len() > 1 {
            let mut new_idx = rng.gen_range(0..discrete_values.len());
            while new_idx == current_idx && discrete_values.len() > 1 {
                new_idx = rng.gen_range(0..discrete_values.len());
            }
            new_idx
        } else {
            0
        };

        *value = discrete_values[new_idx].clone();
    }

    fn mutate_indicator_name(
        value: &mut crate::strategy::types::StrategyParamValue,
        category: &str,
    ) {
        use crate::indicators::registry::IndicatorRegistry;
        use crate::indicators::types::IndicatorCategory;

        let registry = IndicatorRegistry::new();

        let category_enum = match category {
            "trend" => IndicatorCategory::Trend,
            "oscillator" => IndicatorCategory::Oscillator,
            "volatility" => IndicatorCategory::Volatility,
            "volume" => IndicatorCategory::Volume,
            _ => IndicatorCategory::Trend,
        };

        let indicators = registry.get_indicators_by_category(&category_enum);
        let indicator_names: Vec<String> = indicators
            .iter()
            .map(|ind| ind.name().to_string())
            .collect();

        if indicator_names.is_empty() {
            return;
        }

        let current_name = value.as_str().unwrap_or("");
        let current_idx = indicator_names
            .iter()
            .position(|name| name == current_name)
            .unwrap_or(0);

        let mut rng = rand::thread_rng();
        let new_idx = if indicator_names.len() > 1 {
            let mut new_idx = rng.gen_range(0..indicator_names.len());
            while new_idx == current_idx && indicator_names.len() > 1 {
                new_idx = rng.gen_range(0..indicator_names.len());
            }
            new_idx
        } else {
            0
        };

        *value = crate::strategy::types::StrategyParamValue::Text(indicator_names[new_idx].clone());
    }

    fn mutate_operator(
        value: &mut crate::strategy::types::StrategyParamValue,
        compatible_operators: &[crate::strategy::types::ConditionOperator],
    ) {
        use crate::strategy::types::{ConditionOperator, StrategyParamValue};

        if compatible_operators.is_empty() {
            return;
        }

        let current_op = if let StrategyParamValue::Text(op_str) = value {
            match op_str.as_str() {
                "above" => Some(ConditionOperator::Above),
                "below" => Some(ConditionOperator::Below),
                "rising_trend" => Some(ConditionOperator::RisingTrend),
                "falling_trend" => Some(ConditionOperator::FallingTrend),
                "greater_percent" => Some(ConditionOperator::GreaterPercent),
                "lower_percent" => Some(ConditionOperator::LowerPercent),
                "between" => Some(ConditionOperator::Between),
                _ => None,
            }
        } else {
            None
        };

        let current_idx = current_op
            .and_then(|op| compatible_operators.iter().position(|o| o == &op))
            .unwrap_or(0);

        let mut rng = rand::thread_rng();
        let new_idx = if compatible_operators.len() > 1 {
            let mut new_idx = rng.gen_range(0..compatible_operators.len());
            while new_idx == current_idx && compatible_operators.len() > 1 {
                new_idx = rng.gen_range(0..compatible_operators.len());
            }
            new_idx
        } else {
            0
        };

        let new_op = &compatible_operators[new_idx];
        *value = StrategyParamValue::Text(new_op.as_str().to_string());
    }

    fn update_related_indicator_parameters(
        parameters: &mut StrategyParameterMap,
        indicator_name_param_key: &str,
        new_indicator_name: &str,
        parameter_specs: &[crate::strategy::types::StrategyParameterSpec],
        candidate: &StrategyCandidate,
    ) {
        use crate::indicators::parameters::ParameterPresets;
        use crate::indicators::types::ParameterType;
        use crate::strategy::types::ParameterKind;

        for spec in parameter_specs {
            if let ParameterKind::IndicatorParameter { indicator_name_ref } = &spec.parameter_kind {
                if indicator_name_ref == indicator_name_param_key {
                    if let Some(param_value) = parameters.get_mut(&spec.name) {
                        if let Some(indicator) = candidate
                            .indicators
                            .iter()
                            .find(|i| i.name == new_indicator_name)
                        {
                            for param in &indicator.parameters {
                                if param.name == "period" && param.optimizable {
                                    if let Some(range) = ParameterPresets::get_optimization_range(
                                        new_indicator_name,
                                        "period",
                                        &ParameterType::Period,
                                    ) {
                                        let mut rng = rand::thread_rng();
                                        let steps =
                                            ((range.end - range.start) / range.step) as usize;
                                        let step_index = rng.gen_range(0..=steps);
                                        let new_value =
                                            range.start + (step_index as f32 * range.step);
                                        *param_value =
                                            crate::strategy::types::StrategyParamValue::Number(
                                                new_value as f64,
                                            );
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn sync_parameters_with_structure(
        &self,
        parameters: &mut StrategyParameterMap,
        candidate: &StrategyCandidate,
        parameter_specs: &[crate::strategy::types::StrategyParameterSpec],
    ) {
        use crate::condition::parameters::ConditionParameterPresets;
        use crate::indicators::parameters::ParameterPresets;
        use crate::indicators::types::ParameterType;
        use crate::optimization::condition_id::ConditionId;
        use crate::strategy::types::ParameterKind;
        use crate::strategy::types::StrategyParamValue;
        use rand::Rng;

        let mut rng = rand::thread_rng();

        for spec in parameter_specs {
            if !spec.optimize || !spec.mutatable {
                continue;
            }

            match &spec.parameter_kind {
                ParameterKind::Numeric => {
                    if parameters.get(&spec.name).is_none() {
                        if let (Some(min), Some(max), Some(step)) = (spec.min, spec.max, spec.step)
                        {
                            let steps = ((max - min) / step) as usize;
                            let step_index = rng.gen_range(0..=steps);
                            let value = min + (step_index as f64 * step);
                            parameters.insert(spec.name.clone(), spec.default_value.clone());
                        } else {
                            parameters.insert(spec.name.clone(), spec.default_value.clone());
                        }
                    }
                }
                ParameterKind::IndicatorParameter { indicator_name_ref } => {
                    if let Some(indicator_name_value) = parameters.get(indicator_name_ref) {
                        if let Some(indicator_name) = indicator_name_value.as_str() {
                            if let Some(indicator) = candidate
                                .indicators
                                .iter()
                                .find(|i| i.name == indicator_name)
                            {
                                for param in &indicator.parameters {
                                    if param.name == "period" && param.optimizable {
                                        if parameters.get(&spec.name).is_none() {
                                            if let Some(range) =
                                                ParameterPresets::get_optimization_range(
                                                    indicator_name,
                                                    "period",
                                                    &ParameterType::Period,
                                                )
                                            {
                                                let steps = ((range.end - range.start) / range.step)
                                                    as usize;
                                                let step_index = rng.gen_range(0..=steps);
                                                let value =
                                                    range.start + (step_index as f32 * range.step);
                                                parameters.insert(
                                                    spec.name.clone(),
                                                    StrategyParamValue::Number(value as f64),
                                                );
                                            } else {
                                                parameters.insert(
                                                    spec.name.clone(),
                                                    spec.default_value.clone(),
                                                );
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                ParameterKind::ConditionOperator { .. } => {
                    for condition in candidate
                        .conditions
                        .iter()
                        .chain(candidate.exit_conditions.iter())
                    {
                        let param_name = if condition.id.starts_with("exit_") {
                            ConditionId::parameter_name(&condition.id, "period")
                        } else {
                            ConditionId::parameter_name(&condition.id, "period")
                        };

                        if spec.name == param_name || spec.name.contains(&condition.id) {
                            for param in &condition.optimization_params {
                                if param.optimizable && parameters.get(&spec.name).is_none() {
                                    let condition_name = condition.operator.factory_name();
                                    if let Some(range) =
                                        ConditionParameterPresets::get_range_for_condition(
                                            condition_name,
                                        )
                                    {
                                        let steps = ((range.max - range.min) / range.step) as usize;
                                        let step_index = rng.gen_range(0..=steps);
                                        let value = range.min + (step_index as f32 * range.step);
                                        parameters.insert(
                                            spec.name.clone(),
                                            StrategyParamValue::Number(value as f64),
                                        );
                                    } else {
                                        parameters
                                            .insert(spec.name.clone(), spec.default_value.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    if parameters.get(&spec.name).is_none() {
                        parameters.insert(spec.name.clone(), spec.default_value.clone());
                    }
                }
            }
        }

        let keys_to_remove: Vec<String> = parameters
            .keys()
            .filter(|key| !parameter_specs.iter().any(|spec| &spec.name == *key))
            .cloned()
            .collect();

        for key in keys_to_remove {
            parameters.remove(&key);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::types::{EvaluatedStrategy, GeneticIndividual, Population};
    use crate::strategy::types::StrategyParamValue;

    fn create_test_individual(fitness: f64, params: StrategyParameterMap) -> GeneticIndividual {
        GeneticIndividual {
            strategy: EvaluatedStrategy {
                candidate: None,
                parameters: params,
                fitness: Some(fitness),
                backtest_report: None,
            },
            generation: 0,
            island_id: None,
        }
    }

    fn create_test_population(individuals: Vec<GeneticIndividual>) -> Population {
        Population {
            individuals,
            generation: 0,
            island_id: None,
        }
    }

    #[test]
    fn test_population_config_default() {
        let config = PopulationConfig::default();
        assert_eq!(config.size, 100);
        assert_eq!(config.elitism_count, 5);
        assert_eq!(config.crossover_rate, 0.7);
        assert_eq!(config.mutation_rate, 0.1);
    }

    #[test]
    fn test_population_manager_new() {
        let config = PopulationConfig::default();
        let manager = PopulationManager::new(config);
        assert_eq!(manager.config.size, 100);
    }

    #[test]
    fn test_select_parents_with_fitness() {
        let config = PopulationConfig::default();
        let manager = PopulationManager::new(config);

        let mut params1 = HashMap::new();
        params1.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        let mut params2 = HashMap::new();
        params2.insert("param2".to_string(), StrategyParamValue::Number(20.0));
        let mut params3 = HashMap::new();
        params3.insert("param3".to_string(), StrategyParamValue::Number(30.0));

        let individuals = vec![
            create_test_individual(1.0, params1),
            create_test_individual(2.0, params2),
            create_test_individual(3.0, params3),
        ];
        let population = create_test_population(individuals);

        let parents = manager.select_parents(&population, 2);
        assert_eq!(parents.len(), 2);
    }

    #[test]
    fn test_select_parents_zero_fitness() {
        let config = PopulationConfig::default();
        let manager = PopulationManager::new(config);

        let mut params1 = HashMap::new();
        params1.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        let mut params2 = HashMap::new();
        params2.insert("param2".to_string(), StrategyParamValue::Number(20.0));

        let individuals = vec![
            create_test_individual(0.0, params1),
            create_test_individual(0.0, params2),
        ];
        let population = create_test_population(individuals);

        let parents = manager.select_parents(&population, 2);
        assert_eq!(parents.len(), 2);
    }

    #[test]
    fn test_select_parents_empty_population() {
        let config = PopulationConfig::default();
        let manager = PopulationManager::new(config);
        let population = create_test_population(vec![]);

        let parents = manager.select_parents(&population, 0);
        assert_eq!(parents.len(), 0);
    }

    #[test]
    fn test_crossover_success() {
        let config = PopulationConfig {
            crossover_rate: 1.0,
            ..Default::default()
        };
        let manager = PopulationManager::new(config);

        let mut params1 = HashMap::new();
        params1.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        params1.insert("param2".to_string(), StrategyParamValue::Number(20.0));

        let mut params2 = HashMap::new();
        params2.insert("param1".to_string(), StrategyParamValue::Number(15.0));
        params2.insert("param3".to_string(), StrategyParamValue::Number(30.0));

        let parent1 = create_test_individual(1.0, params1);
        let parent2 = create_test_individual(2.0, params2);

        let result = manager.crossover(&parent1, &parent2);
        assert!(result.is_some());

        let (child1, child2) = result.unwrap();
        assert!(!child1.is_empty() || !child2.is_empty());
    }

    #[test]
    fn test_crossover_fails_rate() {
        let config = PopulationConfig {
            crossover_rate: 0.0,
            ..Default::default()
        };
        let manager = PopulationManager::new(config);

        let mut params1 = HashMap::new();
        params1.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        let mut params2 = HashMap::new();
        params2.insert("param2".to_string(), StrategyParamValue::Number(20.0));

        let parent1 = create_test_individual(1.0, params1);
        let parent2 = create_test_individual(2.0, params2);

        let result = manager.crossover(&parent1, &parent2);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_elitism() {
        let config = PopulationConfig {
            elitism_count: 2,
            ..Default::default()
        };
        let manager = PopulationManager::new(config);

        let mut params1 = HashMap::new();
        params1.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        let mut params2 = HashMap::new();
        params2.insert("param2".to_string(), StrategyParamValue::Number(20.0));
        let mut params3 = HashMap::new();
        params3.insert("param3".to_string(), StrategyParamValue::Number(30.0));

        let individuals = vec![
            create_test_individual(1.0, params1),
            create_test_individual(2.0, params2),
            create_test_individual(3.0, params3),
        ];
        let mut population = create_test_population(individuals);

        let elites = vec![
            create_test_individual(10.0, HashMap::new()),
            create_test_individual(20.0, HashMap::new()),
        ];

        manager.apply_elitism(&mut population, elites);

        assert_eq!(population.individuals.len(), 3);
        assert_eq!(population.individuals[0].strategy.fitness, Some(10.0));
        assert_eq!(population.individuals[1].strategy.fitness, Some(20.0));
    }

    #[test]
    fn test_apply_elitism_empty_population() {
        let config = PopulationConfig::default();
        let manager = PopulationManager::new(config);
        let mut population = create_test_population(vec![]);
        let elites = vec![create_test_individual(10.0, HashMap::new())];

        manager.apply_elitism(&mut population, elites);
        assert_eq!(population.individuals.len(), 0);
    }
}
