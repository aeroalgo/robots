use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StopHandlerConfig;
use crate::discovery::{StrategyCandidate, StrategyDiscoveryEngine};
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::population::PopulationManager;
use crate::optimization::sds::StochasticDiffusionSearch;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::{ConditionOperator, PriceField, StrategyParameterMap};
use rand::Rng;
use std::collections::HashMap;

pub struct GeneticAlgorithmV3 {
    config: GeneticAlgorithmConfig,
    population_manager: PopulationManager,
    evaluator: StrategyEvaluationRunner,
    discovery_engine: StrategyDiscoveryEngine,
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
        discovery_config: crate::discovery::StrategyDiscoveryConfig,
    ) -> Self {
        let population_config = crate::optimization::population::PopulationConfig {
            size: config.population_size,
            elitism_count: config.elitism_count,
            crossover_rate: config.crossover_rate,
            mutation_rate: config.mutation_rate,
        };

        let discovery_engine = StrategyDiscoveryEngine::new(discovery_config.clone());

        use crate::discovery::IndicatorInfoCollector;
        use crate::indicators::registry::IndicatorRegistry;

        let registry = IndicatorRegistry::new();
        let available_indicators = IndicatorInfoCollector::collect_from_registry(&registry);

        let price_fields = vec![
            PriceField::Close,
            PriceField::Open,
            PriceField::High,
            PriceField::Low,
        ];

        let operators = vec![
            ConditionOperator::GreaterThan,
            ConditionOperator::LessThan,
            ConditionOperator::CrossesAbove,
            ConditionOperator::CrossesBelow,
        ];

        let stop_handler_configs = vec![];

        Self {
            config,
            population_manager: PopulationManager::new(population_config),
            evaluator: StrategyEvaluationRunner::new(frames, base_timeframe),
            discovery_engine,
            available_indicators,
            price_fields,
            operators,
            stop_handler_configs,
        }
    }

    pub async fn evolve_generation(
        &mut self,
        population: &mut Population,
    ) -> Result<(), anyhow::Error> {
        let elites = self.select_elites(population);
        let lambda = self.config.lambda_size;
        let mu = population.individuals.len();
        let mut offspring = Vec::with_capacity(lambda);
        let mut evaluated_count = 0;

        while offspring.len() < lambda {
            let parents = self.population_manager.select_parents(population, 2);
            if parents.len() < 2 {
                break;
            }

            let parent1_candidate = parents[0].strategy.candidate.clone();
            let parent2_candidate = parents[1].strategy.candidate.clone();

            if let (Some(cand1), Some(cand2)) = (parent1_candidate, parent2_candidate) {
                let (mut child1_candidate, mut child2_candidate) =
                    self.crossover_structure(&cand1, &cand2);

                let (child1_params, child2_params) = if let Some(params) =
                    self.population_manager.crossover(parents[0], parents[1])
                {
                    params
                } else {
                    (
                        parents[0].strategy.parameters.clone(),
                        parents[1].strategy.parameters.clone(),
                    )
                };

                let mut child1_params = child1_params;
                let mut child2_params = child2_params;

                Self::mutate_structure(
                    &mut child1_candidate,
                    &self.config,
                    &self.available_indicators,
                    &self.price_fields,
                    &self.operators,
                    &self.stop_handler_configs,
                );
                Self::mutate_structure(
                    &mut child2_candidate,
                    &self.config,
                    &self.available_indicators,
                    &self.price_fields,
                    &self.operators,
                    &self.stop_handler_configs,
                );

                self.population_manager
                    .mutate(&mut child1_params, &child1_candidate, &self.config);
                self.population_manager
                    .mutate(&mut child2_params, &child2_candidate, &self.config);

                evaluated_count += 1;
                let progress = (evaluated_count as f64 / lambda as f64) * 100.0;
                println!(
                    "      [{}/{}] ({:.1}%) Оценка новой особи...",
                    evaluated_count, lambda, progress
                );

                let child1 = self
                    .create_individual(
                        child1_candidate,
                        child1_params,
                        population.generation + 1,
                        population.island_id,
                    )
                    .await?;
                offspring.push(child1);

                if offspring.len() < lambda {
                    evaluated_count += 1;
                    let progress = (evaluated_count as f64 / lambda as f64) * 100.0;
                    println!(
                        "      [{}/{}] ({:.1}%) Оценка новой особи...",
                        evaluated_count, lambda, progress
                    );

                    let child2 = self
                        .create_individual(
                            child2_candidate,
                            child2_params,
                            population.generation + 1,
                            population.island_id,
                        )
                        .await?;
                    offspring.push(child2);
                }
            }
        }

        let mut combined_population = population.individuals.clone();
        combined_population.extend(offspring);
        
        if self.config.enable_sds {
            let mut temp_population = Population {
                individuals: combined_population.clone(),
                generation: population.generation,
                island_id: population.island_id,
            };
            
            let sds = StochasticDiffusionSearch::new(self.config.clone());
            println!("      [SDS] Применение стохастического диффузионного поиска...");
            sds.apply_diffusion(&mut temp_population, &self.evaluator).await?;
            
            combined_population = temp_population.individuals;
            println!("      [SDS] Диффузионный поиск завершен");
        }
        
        combined_population.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        population.individuals = combined_population.into_iter().take(mu).collect();
        
        self.population_manager.apply_elitism(population, elites);
        population.generation += 1;

        Ok(())
    }

    fn crossover_structure(
        &self,
        parent1: &StrategyCandidate,
        parent2: &StrategyCandidate,
    ) -> (StrategyCandidate, StrategyCandidate) {
        let mut rng = rand::thread_rng();

        let mut child1 = parent1.clone();
        let mut child2 = parent2.clone();

        if rng.gen::<f64>() < self.config.crossover_rate {
            let cond1 = child1.conditions.clone();
            let exit1 = child1.exit_conditions.clone();
            let cond2 = child2.conditions.clone();
            let exit2 = child2.exit_conditions.clone();
            
            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.indicators, &mut child2.indicators);
                std::mem::swap(&mut child1.nested_indicators, &mut child2.nested_indicators);
                
                let child1_aliases = Self::get_all_indicator_aliases(&child1);
                let child2_aliases = Self::get_all_indicator_aliases(&child2);
                
                child1.conditions = Self::merge_conditions_with_indicators(
                    &cond1,
                    &cond2,
                    &child1_aliases,
                );
                child2.conditions = Self::merge_conditions_with_indicators(
                    &cond2,
                    &cond1,
                    &child2_aliases,
                );
                
                child1.exit_conditions = Self::merge_conditions_with_indicators(
                    &exit1,
                    &exit2,
                    &child1_aliases,
                );
                child2.exit_conditions = Self::merge_conditions_with_indicators(
                    &exit2,
                    &exit1,
                    &child2_aliases,
                );
            } else {
                let child1_aliases = Self::get_all_indicator_aliases(&child1);
                let child2_aliases = Self::get_all_indicator_aliases(&child2);
                
                if rng.gen::<f64>() < 0.5 {
                    child1.conditions = Self::filter_conditions_by_indicators(
                        &cond1,
                        &child1_aliases,
                    );
                    child1.conditions.extend(
                        Self::filter_conditions_by_indicators(&cond2, &child1_aliases)
                            .into_iter(),
                    );
                    
                    child2.conditions = Self::filter_conditions_by_indicators(
                        &cond2,
                        &child2_aliases,
                    );
                    child2.conditions.extend(
                        Self::filter_conditions_by_indicators(&cond1, &child2_aliases)
                            .into_iter(),
                    );
                }
                
                if rng.gen::<f64>() < 0.5 {
                    child1.exit_conditions = Self::filter_conditions_by_indicators(
                        &exit1,
                        &child1_aliases,
                    );
                    child1.exit_conditions.extend(
                        Self::filter_conditions_by_indicators(&exit2, &child1_aliases)
                            .into_iter(),
                    );
                    
                    child2.exit_conditions = Self::filter_conditions_by_indicators(
                        &exit2,
                        &child2_aliases,
                    );
                    child2.exit_conditions.extend(
                        Self::filter_conditions_by_indicators(&exit1, &child2_aliases)
                            .into_iter(),
                    );
                }
            }
            
            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.stop_handlers, &mut child2.stop_handlers);
            }

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.take_handlers, &mut child2.take_handlers);
            }
            
            if rng.gen::<f64>() < 0.5 {
                let (_tf1, ind1_for_tf) = Self::extract_timeframes_with_indicators(&child1);
                let (_tf2, ind2_for_tf) = Self::extract_timeframes_with_indicators(&child2);
                
                std::mem::swap(&mut child1.timeframes, &mut child2.timeframes);
                
                let child1_tf_range = Self::get_timeframe_range(&child1.timeframes);
                let child2_tf_range = Self::get_timeframe_range(&child2.timeframes);
                
                for (tf, indicators) in ind2_for_tf {
                    if Self::is_timeframe_in_range(&tf, &child1_tf_range) {
                        for ind in indicators {
                            if !Self::has_indicator_alias(&child1, &ind.alias) {
                                child1.indicators.push(ind);
                            }
                        }
                    }
                }
                
                for (tf, indicators) in ind1_for_tf {
                    if Self::is_timeframe_in_range(&tf, &child2_tf_range) {
                        for ind in indicators {
                            if !Self::has_indicator_alias(&child2, &ind.alias) {
                                child2.indicators.push(ind);
                            }
                        }
                    }
                }
            }
        }

        (child1, child2)
    }
    
    
    fn get_all_indicator_aliases(candidate: &StrategyCandidate) -> std::collections::HashSet<String> {
        let mut aliases = std::collections::HashSet::new();
        for ind in &candidate.indicators {
            aliases.insert(ind.alias.clone());
        }
        for nested in &candidate.nested_indicators {
            aliases.insert(nested.indicator.alias.clone());
        }
        aliases
    }
    
    fn has_indicator_alias(candidate: &StrategyCandidate, alias: &str) -> bool {
        candidate.indicators.iter().any(|ind| ind.alias == alias)
            || candidate.nested_indicators.iter().any(|nested| nested.indicator.alias == alias)
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
    
    fn extract_indicator_aliases_from_condition_id(condition_id: &str) -> Option<Vec<String>> {
        if condition_id.starts_with("ind_ind_") {
            let rest = condition_id.strip_prefix("ind_ind_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if parts.len() >= 2 {
                    return Some(vec![parts[0].to_string(), parts[1].to_string()]);
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if parts.len() >= 2 {
                    return Some(vec![parts[0].to_string(), parts[1].to_string()]);
                }
            }
        }
        None
    }
    
    fn condition_uses_indicators(
        condition: &crate::discovery::ConditionInfo,
        available_aliases: &std::collections::HashSet<String>,
    ) -> bool {
        match condition.condition_type.as_str() {
            "indicator_price" | "indicator_constant" => {
                if let Some(indicator_alias) = Self::extract_indicator_alias_from_condition_id(&condition.id) {
                    available_aliases.contains(&indicator_alias)
                } else {
                    false
                }
            }
            "indicator_indicator" => {
                if let Some(aliases) = Self::extract_indicator_aliases_from_condition_id(&condition.id) {
                    aliases.len() >= 2
                        && available_aliases.contains(&aliases[0])
                        && available_aliases.contains(&aliases[1])
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    fn filter_conditions_by_indicators(
        conditions: &[crate::discovery::ConditionInfo],
        available_aliases: &std::collections::HashSet<String>,
    ) -> Vec<crate::discovery::ConditionInfo> {
        conditions
            .iter()
            .filter(|cond| Self::condition_uses_indicators(cond, available_aliases))
            .cloned()
            .collect()
    }
    
    fn merge_conditions_with_indicators(
        existing: &[crate::discovery::ConditionInfo],
        new: &[crate::discovery::ConditionInfo],
        available_aliases: &std::collections::HashSet<String>,
    ) -> Vec<crate::discovery::ConditionInfo> {
        let mut result: Vec<crate::discovery::ConditionInfo> = existing
            .iter()
            .filter(|cond| Self::condition_uses_indicators(cond, available_aliases))
            .cloned()
            .collect();
        
        let filtered_new: Vec<crate::discovery::ConditionInfo> = new
            .iter()
            .filter(|cond| Self::condition_uses_indicators(cond, available_aliases))
            .cloned()
            .collect();
        
        result.extend(filtered_new);
        result
    }
    
    fn extract_timeframes_with_indicators(
        candidate: &StrategyCandidate,
    ) -> (
        Vec<TimeFrame>,
        Vec<(TimeFrame, Vec<crate::discovery::IndicatorInfo>)>,
    ) {
        let total_conditions = candidate.conditions.len() + candidate.exit_conditions.len();
        let mut tf_indicators: std::collections::HashMap<TimeFrame, std::collections::HashSet<String>> = std::collections::HashMap::with_capacity(total_conditions / 2 + 1);
        
        for condition in candidate.conditions.iter().chain(candidate.exit_conditions.iter()) {
            if let Some(tf) = condition.primary_timeframe.as_ref() {
                if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id) {
                    tf_indicators.entry(tf.clone()).or_insert_with(std::collections::HashSet::new).insert(alias);
                }
                if let Some(aliases) = Self::extract_indicator_aliases_from_condition_id(&condition.id) {
                    for alias in aliases {
                        tf_indicators.entry(tf.clone()).or_insert_with(std::collections::HashSet::new).insert(alias);
                    }
                }
            }
            if let Some(tf) = condition.secondary_timeframe.as_ref() {
                if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id) {
                    tf_indicators.entry(tf.clone()).or_insert_with(std::collections::HashSet::new).insert(alias);
                }
                if let Some(aliases) = Self::extract_indicator_aliases_from_condition_id(&condition.id) {
                    for alias in aliases {
                        tf_indicators.entry(tf.clone()).or_insert_with(std::collections::HashSet::new).insert(alias);
                    }
                }
            }
        }
        
        let mut result: Vec<(TimeFrame, Vec<crate::discovery::IndicatorInfo>)> = Vec::with_capacity(tf_indicators.len());
        for (tf, aliases) in tf_indicators {
            let mut indicators = Vec::with_capacity(aliases.len());
            for alias in aliases {
                if let Some(ind) = candidate.indicators.iter().find(|i| i.alias == alias) {
                    indicators.push(ind.clone());
                }
                if let Some(nested) = candidate.nested_indicators.iter().find(|n| n.indicator.alias == alias) {
                    indicators.push(nested.indicator.clone());
                }
            }
            if !indicators.is_empty() {
                result.push((tf, indicators));
            }
        }
        
        (candidate.timeframes.clone(), result)
    }
    
    fn get_timeframe_range(timeframes: &[TimeFrame]) -> Option<(chrono::Duration, chrono::Duration)> {
        let durations: Vec<chrono::Duration> = timeframes
            .iter()
            .filter_map(|tf| tf.duration())
            .collect();
        
        if durations.is_empty() {
            return None;
        }
        
        let min_duration = durations.iter().min().copied()?;
        let max_duration = durations.iter().max().copied()?;
        
        Some((min_duration, max_duration))
    }
    
    fn is_timeframe_in_range(tf: &TimeFrame, range: &Option<(chrono::Duration, chrono::Duration)>) -> bool {
        if let Some((min_dur, max_dur)) = range {
            if let Some(tf_dur) = tf.duration() {
                return tf_dur >= *min_dur && tf_dur <= *max_dur;
            }
        }
        false
    }

    fn mutate_structure(
        candidate: &mut StrategyCandidate,
        config: &GeneticAlgorithmConfig,
        available_indicators: &[crate::discovery::IndicatorInfo],
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
        stop_handler_configs: &[StopHandlerConfig],
    ) {
        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            if rng.gen::<f64>() < 0.3 && !candidate.indicators.is_empty() {
                let idx = rng.gen_range(0..candidate.indicators.len());
                candidate.indicators.remove(idx);
            } else if !available_indicators.is_empty() {
                let new_indicator =
                    available_indicators[rng.gen_range(0..available_indicators.len())].clone();
                candidate.indicators.push(new_indicator);
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
                let idx = rng.gen_range(0..candidate.conditions.len());
                candidate.conditions.remove(idx);
            } else {
                let mut engine = StrategyDiscoveryEngine::new(candidate.config.clone());
                let mut iter = engine.generate_strategies_random(
                    available_indicators,
                    price_fields,
                    operators,
                    stop_handler_configs,
                    None,
                );
                if let Some(new_candidate) = iter.next() {
                    if !new_candidate.conditions.is_empty() {
                        let new_condition = new_candidate.conditions
                            [rng.gen_range(0..new_candidate.conditions.len())]
                        .clone();
                        candidate.conditions.push(new_condition);
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            let has_exit_conditions = !candidate.exit_conditions.is_empty();
            let has_stop_handlers = !candidate.stop_handlers.is_empty();
            let has_take_handlers = !candidate.take_handlers.is_empty();
            let has_any_exit = has_exit_conditions || has_stop_handlers || has_take_handlers;
            let can_remove_exit = has_exit_conditions && (candidate.exit_conditions.len() > 1 || has_stop_handlers || has_take_handlers);
            
            if rng.gen::<f64>() < 0.3 && can_remove_exit {
                let idx = rng.gen_range(0..candidate.exit_conditions.len());
                candidate.exit_conditions.remove(idx);
            } else {
                let mut engine = StrategyDiscoveryEngine::new(candidate.config.clone());
                let mut iter = engine.generate_strategies_random(
                    available_indicators,
                    price_fields,
                    operators,
                    stop_handler_configs,
                    None,
                );
                if let Some(new_candidate) = iter.next() {
                    if !new_candidate.exit_conditions.is_empty() {
                        let new_exit = new_candidate.exit_conditions
                            [rng.gen_range(0..new_candidate.exit_conditions.len())]
                        .clone();
                        candidate.exit_conditions.push(new_exit);
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            let has_exit_conditions = !candidate.exit_conditions.is_empty();
            let has_stop_handlers = !candidate.stop_handlers.is_empty();
            let has_take_handlers = !candidate.take_handlers.is_empty();
            let can_remove_stop = has_stop_handlers && (candidate.stop_handlers.len() > 1 || has_exit_conditions || has_take_handlers);
            
            if rng.gen::<f64>() < 0.3 && can_remove_stop {
                let idx = rng.gen_range(0..candidate.stop_handlers.len());
                candidate.stop_handlers.remove(idx);
            } else if !stop_handler_configs.is_empty() {
                let mut engine = StrategyDiscoveryEngine::new(candidate.config.clone());
                let mut iter = engine.generate_strategies_random(
                    available_indicators,
                    price_fields,
                    operators,
                    stop_handler_configs,
                    None,
                );
                if let Some(new_candidate) = iter.next() {
                    if !new_candidate.stop_handlers.is_empty() {
                        let new_stop = new_candidate.stop_handlers
                            [rng.gen_range(0..new_candidate.stop_handlers.len())]
                        .clone();
                        candidate.stop_handlers.push(new_stop);
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            let has_exit_conditions = !candidate.exit_conditions.is_empty();
            let has_stop_handlers = !candidate.stop_handlers.is_empty();
            let has_take_handlers = !candidate.take_handlers.is_empty();
            let can_remove_take = has_take_handlers && (candidate.take_handlers.len() > 1 || has_exit_conditions || has_stop_handlers);
            
            if rng.gen::<f64>() < 0.3 && can_remove_take {
                let idx = rng.gen_range(0..candidate.take_handlers.len());
                candidate.take_handlers.remove(idx);
            } else if !stop_handler_configs.is_empty() {
                let has_exit_conditions = !candidate.exit_conditions.is_empty();
                let has_stop_handlers = !candidate.stop_handlers.is_empty();
                let can_add_take = has_exit_conditions || has_stop_handlers;
                
                if can_add_take {
                    let mut engine = StrategyDiscoveryEngine::new(candidate.config.clone());
                    let mut iter = engine.generate_strategies_random(
                        available_indicators,
                        price_fields,
                        operators,
                        stop_handler_configs,
                        None,
                    );
                    if let Some(new_candidate) = iter.next() {
                        if !new_candidate.take_handlers.is_empty() {
                            let new_take = new_candidate.take_handlers
                                [rng.gen_range(0..new_candidate.take_handlers.len())]
                            .clone();
                            candidate.take_handlers.push(new_take);
                        }
                    }
                }
            }
        }

        if rng.gen::<f64>() < config.mutation_rate * 0.5 {
            let base_tf = &candidate.config.base_timeframe;
            let base_duration = base_tf.duration();
            
            let all_timeframes = vec![
                TimeFrame::from_identifier("1"),
                TimeFrame::from_identifier("5"),
                TimeFrame::from_identifier("15"),
                TimeFrame::from_identifier("30"),
                TimeFrame::from_identifier("60"),
                TimeFrame::from_identifier("240"),
                TimeFrame::from_identifier("D"),
            ];
            
            let available_timeframes: Vec<TimeFrame> = if let Some(base_dur) = base_duration {
                all_timeframes
                    .into_iter()
                    .filter(|tf| {
                        if let Some(tf_dur) = tf.duration() {
                            tf_dur >= base_dur
                        } else {
                            false
                        }
                    })
                    .collect()
            } else {
                all_timeframes
            };

            if !candidate.timeframes.is_empty() && rng.gen::<f64>() < 0.5 {
                let idx = rng.gen_range(0..candidate.timeframes.len());
                candidate.timeframes.remove(idx);
            } else if !available_timeframes.is_empty() {
                let new_tf =
                    available_timeframes[rng.gen_range(0..available_timeframes.len())].clone();
                if !candidate.timeframes.contains(&new_tf) {
                    candidate.timeframes.push(new_tf);
                }
            }
        }
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
