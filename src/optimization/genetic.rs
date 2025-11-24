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
use rand::seq::SliceRandom;
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

        // Для entry conditions используем только Close
        // High и Low используются только в стоп-обработчиках для определения пробития стопа
        let price_fields = vec![PriceField::Close];

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
            sds.apply_diffusion(&mut temp_population, &self.evaluator)
                .await?;

            combined_population = temp_population.individuals;
            println!("      [SDS] Диффузионный поиск завершен");
        }

        // Round-robin отбор с группировкой по стратегиям для поддержания разнообразия
        population.individuals = Self::select_with_diversity(combined_population, mu);

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
                if rng.gen::<f64>() < 0.5 {
                    let (new_cond1, new_indicators1, new_nested1, new_tf1) = 
                        Self::extract_conditions_with_indicators(&cond2, parent2);
                    child1.conditions.extend(new_cond1);
                    Self::add_indicators_if_missing(&mut child1, &new_indicators1, &new_nested1, &new_tf1);

                    let (new_cond2, new_indicators2, new_nested2, new_tf2) = 
                        Self::extract_conditions_with_indicators(&cond1, parent1);
                    child2.conditions.extend(new_cond2);
                    Self::add_indicators_if_missing(&mut child2, &new_indicators2, &new_nested2, &new_tf2);
                }

                if rng.gen::<f64>() < 0.5 {
                    let (new_exit1, new_indicators1, new_nested1, new_tf1) = 
                        Self::extract_conditions_with_indicators(&exit2, parent2);
                    child1.exit_conditions.extend(new_exit1);
                    Self::add_indicators_if_missing(&mut child1, &new_indicators1, &new_nested1, &new_tf1);

                    let (new_exit2, new_indicators2, new_nested2, new_tf2) = 
                        Self::extract_conditions_with_indicators(&exit1, parent1);
                    child2.exit_conditions.extend(new_exit2);
                    Self::add_indicators_if_missing(&mut child2, &new_indicators2, &new_nested2, &new_tf2);
                }
            } else {
                if rng.gen::<f64>() < 0.5 {
                    let (new_cond1, new_indicators1, new_nested1, new_tf1) = 
                        Self::extract_conditions_with_indicators(&cond2, parent2);
                    child1.conditions.extend(new_cond1);
                    Self::add_indicators_if_missing(&mut child1, &new_indicators1, &new_nested1, &new_tf1);

                    let (new_cond2, new_indicators2, new_nested2, new_tf2) = 
                        Self::extract_conditions_with_indicators(&cond1, parent1);
                    child2.conditions.extend(new_cond2);
                    Self::add_indicators_if_missing(&mut child2, &new_indicators2, &new_nested2, &new_tf2);
                }

                if rng.gen::<f64>() < 0.5 {
                    let (new_exit1, new_indicators1, new_nested1, new_tf1) = 
                        Self::extract_conditions_with_indicators(&exit2, parent2);
                    child1.exit_conditions.extend(new_exit1);
                    Self::add_indicators_if_missing(&mut child1, &new_indicators1, &new_nested1, &new_tf1);

                    let (new_exit2, new_indicators2, new_nested2, new_tf2) = 
                        Self::extract_conditions_with_indicators(&exit1, parent1);
                    child2.exit_conditions.extend(new_exit2);
                    Self::add_indicators_if_missing(&mut child2, &new_indicators2, &new_nested2, &new_tf2);
                }
            }

            Self::remove_unused_indicators(&mut child1);
            Self::remove_unused_indicators(&mut child2);

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.stop_handlers, &mut child2.stop_handlers);
            }

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.take_handlers, &mut child2.take_handlers);
            }

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.timeframes, &mut child2.timeframes);
            }
        }

        (child1, child2)
    }

    fn get_all_indicator_aliases(
        candidate: &StrategyCandidate,
    ) -> std::collections::HashSet<String> {
        let mut aliases = std::collections::HashSet::new();
        for ind in &candidate.indicators {
            aliases.insert(ind.alias.clone());
        }
        for nested in &candidate.nested_indicators {
            aliases.insert(nested.indicator.alias.clone());
        }
        aliases
    }

    fn extract_conditions_with_indicators(
        conditions: &[crate::discovery::ConditionInfo],
        parent: &StrategyCandidate,
    ) -> (
        Vec<crate::discovery::ConditionInfo>,
        Vec<crate::discovery::IndicatorInfo>,
        Vec<crate::discovery::NestedIndicator>,
        Vec<TimeFrame>,
    ) {
        let mut new_conditions = Vec::new();
        let mut new_indicators = Vec::new();
        let mut new_nested = Vec::new();
        let mut new_timeframes = Vec::new();
        let mut added_aliases = std::collections::HashSet::new();
        let mut added_timeframes = std::collections::HashSet::new();

        for condition in conditions {
            let mut condition_aliases = Vec::new();
            
            if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id) {
                condition_aliases.push(alias);
            }
            
            if let Some(aliases) = Self::extract_indicator_aliases_from_condition_id(&condition.id) {
                condition_aliases.extend(aliases);
            }

            let mut condition_valid = true;
            for alias in &condition_aliases {
                if !Self::has_indicator_alias(parent, alias) {
                    condition_valid = false;
                    break;
                }
            }

            if condition_valid {
                new_conditions.push(condition.clone());
                
                if let Some(tf) = &condition.primary_timeframe {
                    if !added_timeframes.contains(tf) {
                        new_timeframes.push(tf.clone());
                        added_timeframes.insert(tf.clone());
                    }
                }
                
                if let Some(tf) = &condition.secondary_timeframe {
                    if !added_timeframes.contains(tf) {
                        new_timeframes.push(tf.clone());
                        added_timeframes.insert(tf.clone());
                    }
                }
                
                for alias in condition_aliases {
                    if !added_aliases.contains(&alias) {
                        if let Some(ind) = parent.indicators.iter().find(|i| i.alias == alias) {
                            new_indicators.push(ind.clone());
                            added_aliases.insert(alias.clone());
                        } else if let Some(nested) = parent.nested_indicators.iter().find(|n| n.indicator.alias == alias) {
                            new_nested.push(nested.clone());
                            added_aliases.insert(alias.clone());
                        }
                    }
                }
            }
        }

        (new_conditions, new_indicators, new_nested, new_timeframes)
    }

    fn add_indicators_if_missing(
        candidate: &mut StrategyCandidate,
        indicators: &[crate::discovery::IndicatorInfo],
        nested: &[crate::discovery::NestedIndicator],
        timeframes: &[TimeFrame],
    ) {
        let existing_aliases = Self::get_all_indicator_aliases(candidate);

        for ind in indicators {
            if !existing_aliases.contains(&ind.alias) {
                candidate.indicators.push(ind.clone());
            }
        }

        for nested_ind in nested {
            if !existing_aliases.contains(&nested_ind.indicator.alias) {
                candidate.nested_indicators.push(nested_ind.clone());
            }
        }

        for tf in timeframes {
            if !candidate.timeframes.contains(tf) {
                candidate.timeframes.push(tf.clone());
            }
        }
    }

    fn remove_unused_indicators(candidate: &mut StrategyCandidate) {
        let used_aliases = Self::get_used_indicator_aliases(candidate);
        let used_timeframes = Self::get_used_timeframes(candidate);

        candidate.indicators.retain(|ind| used_aliases.contains(&ind.alias));
        candidate.nested_indicators.retain(|nested| used_aliases.contains(&nested.indicator.alias));
        candidate.timeframes.retain(|tf| used_timeframes.contains(tf));
    }

    fn get_used_timeframes(candidate: &StrategyCandidate) -> std::collections::HashSet<TimeFrame> {
        let mut used_timeframes = std::collections::HashSet::new();

        for condition in candidate.conditions.iter().chain(candidate.exit_conditions.iter()) {
            if let Some(tf) = &condition.primary_timeframe {
                used_timeframes.insert(tf.clone());
            }
            if let Some(tf) = &condition.secondary_timeframe {
                used_timeframes.insert(tf.clone());
            }
        }

        used_timeframes
    }

    fn get_used_indicator_aliases(candidate: &StrategyCandidate) -> std::collections::HashSet<String> {
        let mut used_aliases = std::collections::HashSet::new();

        for condition in candidate.conditions.iter().chain(candidate.exit_conditions.iter()) {
            if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id) {
                used_aliases.insert(alias);
            }
            
            if let Some(aliases) = Self::extract_indicator_aliases_from_condition_id(&condition.id) {
                for alias in aliases {
                    used_aliases.insert(alias);
                }
            }
        }

        for nested in &candidate.nested_indicators {
            used_aliases.insert(nested.input_indicator_alias.clone());
        }

        used_aliases
    }

    fn remove_conditions_with_indicator(candidate: &mut StrategyCandidate, alias: &str) {
        candidate.conditions.retain(|cond| {
            let cond_uses_alias = Self::extract_indicator_alias_from_condition_id(&cond.id)
                .map(|a| a == alias)
                .unwrap_or(false)
                || Self::extract_indicator_aliases_from_condition_id(&cond.id)
                    .map(|aliases| aliases.contains(&alias.to_string()))
                    .unwrap_or(false);
            !cond_uses_alias
        });

        candidate.exit_conditions.retain(|cond| {
            let cond_uses_alias = Self::extract_indicator_alias_from_condition_id(&cond.id)
                .map(|a| a == alias)
                .unwrap_or(false)
                || Self::extract_indicator_aliases_from_condition_id(&cond.id)
                    .map(|aliases| aliases.contains(&alias.to_string()))
                    .unwrap_or(false);
            !cond_uses_alias
        });
    }

    fn has_indicator_alias(candidate: &StrategyCandidate, alias: &str) -> bool {
        candidate.indicators.iter().any(|ind| ind.alias == alias)
            || candidate
                .nested_indicators
                .iter()
                .any(|nested| nested.indicator.alias == alias)
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
                if let Some(indicator_alias) =
                    Self::extract_indicator_alias_from_condition_id(&condition.id)
                {
                    available_aliases.contains(&indicator_alias)
                } else {
                    false
                }
            }
            "indicator_indicator" => {
                if let Some(aliases) =
                    Self::extract_indicator_aliases_from_condition_id(&condition.id)
                {
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
        let mut tf_indicators: std::collections::HashMap<
            TimeFrame,
            std::collections::HashSet<String>,
        > = std::collections::HashMap::with_capacity(total_conditions / 2 + 1);

        for condition in candidate
            .conditions
            .iter()
            .chain(candidate.exit_conditions.iter())
        {
            if let Some(tf) = condition.primary_timeframe.as_ref() {
                if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id)
                {
                    tf_indicators
                        .entry(tf.clone())
                        .or_insert_with(std::collections::HashSet::new)
                        .insert(alias);
                }
                if let Some(aliases) =
                    Self::extract_indicator_aliases_from_condition_id(&condition.id)
                {
                    for alias in aliases {
                        tf_indicators
                            .entry(tf.clone())
                            .or_insert_with(std::collections::HashSet::new)
                            .insert(alias);
                    }
                }
            }
            if let Some(tf) = condition.secondary_timeframe.as_ref() {
                if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id)
                {
                    tf_indicators
                        .entry(tf.clone())
                        .or_insert_with(std::collections::HashSet::new)
                        .insert(alias);
                }
                if let Some(aliases) =
                    Self::extract_indicator_aliases_from_condition_id(&condition.id)
                {
                    for alias in aliases {
                        tf_indicators
                            .entry(tf.clone())
                            .or_insert_with(std::collections::HashSet::new)
                            .insert(alias);
                    }
                }
            }
        }

        let mut result: Vec<(TimeFrame, Vec<crate::discovery::IndicatorInfo>)> =
            Vec::with_capacity(tf_indicators.len());
        for (tf, aliases) in tf_indicators {
            let mut indicators = Vec::with_capacity(aliases.len());
            for alias in aliases {
                if let Some(ind) = candidate.indicators.iter().find(|i| i.alias == alias) {
                    indicators.push(ind.clone());
                }
                if let Some(nested) = candidate
                    .nested_indicators
                    .iter()
                    .find(|n| n.indicator.alias == alias)
                {
                    indicators.push(nested.indicator.clone());
                }
            }
            if !indicators.is_empty() {
                result.push((tf, indicators));
            }
        }

        (candidate.timeframes.clone(), result)
    }

    fn get_timeframe_range(
        timeframes: &[TimeFrame],
    ) -> Option<(chrono::Duration, chrono::Duration)> {
        let durations: Vec<chrono::Duration> =
            timeframes.iter().filter_map(|tf| tf.duration()).collect();

        if durations.is_empty() {
            return None;
        }

        let min_duration = durations.iter().min().copied()?;
        let max_duration = durations.iter().max().copied()?;

        Some((min_duration, max_duration))
    }

    fn is_timeframe_in_range(
        tf: &TimeFrame,
        range: &Option<(chrono::Duration, chrono::Duration)>,
    ) -> bool {
        if let Some((min_dur, max_dur)) = range {
            if let Some(tf_dur) = tf.duration() {
                return tf_dur >= *min_dur && tf_dur <= *max_dur;
            }
        }
        false
    }

    fn create_condition_for_indicator(
        indicator: &crate::discovery::IndicatorInfo,
        candidate: &StrategyCandidate,
        is_entry: bool,
        config: &GeneticAlgorithmConfig,
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
    ) -> Option<crate::discovery::ConditionInfo> {
        use crate::optimization::candidate_builder_config::ConditionProbabilities;
        use crate::indicators::implementations::get_oscillator_threshold_range;
        let mut rng = rand::thread_rng();

        let default_probabilities = ConditionProbabilities::default();
        let probabilities = config
            .candidate_builder_config
            .as_ref()
            .map(|c| &c.probabilities.conditions)
            .unwrap_or(&default_probabilities);

        let all_indicators: Vec<&crate::discovery::IndicatorInfo> = candidate
            .indicators
            .iter()
            .chain(candidate.nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        let is_built_on_oscillator = candidate
            .nested_indicators
            .iter()
            .find(|n| n.indicator.alias == indicator.alias)
            .and_then(|nested| {
                all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                    .map(|input| input.indicator_type == "oscillator")
            })
            .unwrap_or(false);

        let is_oscillator_used_in_nested = candidate
            .nested_indicators
            .iter()
            .any(|nested| nested.input_indicator_alias == indicator.alias);

        let condition_type = if indicator.indicator_type == "oscillator" && !is_oscillator_used_in_nested {
            "indicator_constant"
        } else if indicator.indicator_type == "volatility" {
            "indicator_constant"
        } else if is_built_on_oscillator {
            "indicator_indicator"
        } else if rng.gen::<f64>() < probabilities.use_trend_condition {
            "trend_condition"
        } else if indicator.indicator_type == "trend" || indicator.indicator_type == "channel" {
            if rng.gen::<f64>() < probabilities.use_indicator_indicator_condition {
                "indicator_indicator"
            } else {
                "indicator_price"
            }
        } else if rng.gen::<f64>() < probabilities.use_indicator_indicator_condition {
            "indicator_indicator"
        } else {
            "indicator_price"
        };

        let operator = if condition_type == "trend_condition" {
            if rng.gen::<f64>() < 0.5 {
                ConditionOperator::GreaterThan
            } else {
                ConditionOperator::LessThan
            }
        } else if rng.gen::<f64>() < probabilities.use_crosses_operator {
            if rng.gen::<f64>() < 0.5 {
                ConditionOperator::CrossesAbove
            } else {
                ConditionOperator::CrossesBelow
            }
        } else if rng.gen::<f64>() < 0.5 {
            ConditionOperator::GreaterThan
        } else {
            ConditionOperator::LessThan
        };

        let (condition_id, condition_name, constant_value, price_field, optimization_params) = 
            if condition_type == "indicator_constant" {
                let const_val = if indicator.indicator_type == "volatility" {
                    rng.gen_range(0.2..=10.0)
                } else if indicator.name == "RSI" {
                    if operator == ConditionOperator::GreaterThan {
                        rng.gen_range(70.0..=90.0)
                    } else {
                        rng.gen_range(10.0..=30.0)
                    }
                } else if indicator.name == "Stochastic" {
                    if operator == ConditionOperator::GreaterThan {
                        rng.gen_range(80.0..=95.0)
                    } else {
                        rng.gen_range(5.0..=20.0)
                    }
                } else {
                    rng.gen_range(0.0..=100.0)
                };

                let id = format!(
                    "{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    indicator.alias,
                    rng.gen::<u32>()
                );
                let name = if indicator.indicator_type == "volatility" {
                    format!("{} {:?} Close * {:.2}%", indicator.name, operator, const_val)
                } else {
                    format!("{} {:?} {:.1}", indicator.name, operator, const_val)
                };

                let opt_params = if indicator.indicator_type == "volatility" {
                    vec![crate::discovery::ConditionParamInfo {
                        name: "percentage".to_string(),
                        optimizable: true,
                        global_param_name: None,
                    }]
                } else {
                    vec![crate::discovery::ConditionParamInfo {
                        name: "threshold".to_string(),
                        optimizable: true,
                        global_param_name: None,
                    }]
                };

                (id, name, Some(const_val), None, opt_params)
            } else if condition_type == "trend_condition" {
                let period = rng.gen_range(10.0..=50.0);
                let trend_name = match operator {
                    ConditionOperator::GreaterThan => "RisingTrend",
                    ConditionOperator::LessThan => "FallingTrend",
                    _ => "RisingTrend",
                };
                let id = format!(
                    "{}_{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    indicator.alias,
                    trend_name.to_lowercase(),
                    rng.gen::<u32>()
                );
                let name = format!("{} {} (period: {:.0})", indicator.name, trend_name, period);
                let opt_params = vec![crate::discovery::ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }];
                (id, name, None, None, opt_params)
            } else if condition_type == "indicator_indicator" {
                let available_secondary: Vec<&crate::discovery::IndicatorInfo> = all_indicators
                    .iter()
                    .filter(|ind| ind.alias != indicator.alias)
                    .filter(|ind| {
                        Self::can_compare_indicators_for_mutation(
                            indicator,
                            *ind,
                            &candidate.nested_indicators,
                            &candidate.indicators,
                        )
                    })
                    .copied()
                    .collect();

                if let Some(secondary) = available_secondary.choose(&mut rng) {
                    let id = format!(
                        "{}_{}_{}_{}",
                        if is_entry { "entry" } else { "exit" },
                        indicator.alias,
                        secondary.alias,
                        rng.gen::<u32>()
                    );
                    let name = format!("{} {:?} {}", indicator.name, operator, secondary.name);
                    let (opt_params, percent_val) = if rng.gen::<f64>() < probabilities.use_percent_condition {
                        let percent = rng.gen_range(0.1..=5.0);
                        (
                            vec![crate::discovery::ConditionParamInfo {
                                name: "percentage".to_string(),
                                optimizable: true,
                                global_param_name: None,
                            }],
                            Some(percent),
                        )
                    } else {
                        (Vec::new(), None)
                    };
                    let final_name = if let Some(percent) = percent_val {
                        format!("{} на {:.2}%", name, percent)
                    } else {
                        name
                    };
                    (id, final_name, percent_val, None, opt_params)
                } else {
                    return None;
                }
            } else {
                let price_field_str = if price_fields.len() == 1 {
                    format!("{:?}", price_fields[0])
                } else {
                    format!("{:?}", price_fields.choose(&mut rng).unwrap_or(&PriceField::Close))
                };

                let (opt_params, percent_val) = if rng.gen::<f64>() < probabilities.use_percent_condition {
                    let percent = rng.gen_range(0.1..=5.0);
                    (
                        vec![crate::discovery::ConditionParamInfo {
                            name: "percentage".to_string(),
                            optimizable: true,
                            global_param_name: None,
                        }],
                        Some(percent),
                    )
                } else {
                    (Vec::new(), None)
                };

                let id = format!(
                    "{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    indicator.alias,
                    rng.gen::<u32>()
                );
                let name = if let Some(percent) = percent_val {
                    format!("{} {:?} {} на {:.2}%", indicator.name, operator, "target", percent)
                } else {
                    format!("{} {:?} {}", indicator.name, operator, "target")
                };
                (id, name, percent_val, Some(price_field_str), opt_params)
            };

        Some(crate::discovery::types::ConditionInfo {
            id: condition_id,
            name: condition_name,
            operator,
            condition_type: condition_type.to_string(),
            optimization_params,
            constant_value,
            primary_timeframe: None,
            secondary_timeframe: None,
            price_field,
        })
    }

    fn can_compare_indicators_for_mutation(
        primary: &crate::discovery::IndicatorInfo,
        secondary: &crate::discovery::IndicatorInfo,
        nested_indicators: &[crate::discovery::NestedIndicator],
        all_indicators: &[crate::discovery::IndicatorInfo],
    ) -> bool {
        if primary.indicator_type == "oscillator" && secondary.indicator_type == "oscillator" {
            return false;
        }

        let is_built_on_oscillator = |indicator: &crate::discovery::IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    input_indicator.indicator_type == "oscillator"
                } else {
                    false
                }
            } else {
                false
            }
        };

        let primary_built_on_oscillator = is_built_on_oscillator(primary);
        let secondary_built_on_oscillator = is_built_on_oscillator(secondary);

        if primary.indicator_type == "oscillator" && !primary_built_on_oscillator {
            return secondary_built_on_oscillator;
        }

        if secondary.indicator_type == "oscillator" && !secondary_built_on_oscillator {
            return primary_built_on_oscillator;
        }

        true
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
                let removed_alias = candidate.indicators[idx].alias.clone();
                candidate.indicators.remove(idx);
                
                Self::remove_conditions_with_indicator(candidate, &removed_alias);
                Self::remove_unused_indicators(candidate);
            } else if !available_indicators.is_empty() {
                let new_indicator =
                    available_indicators[rng.gen_range(0..available_indicators.len())].clone();
                candidate.indicators.push(new_indicator.clone());
                
                if let Some(condition) = Self::create_condition_for_indicator(
                    &new_indicator,
                    candidate,
                    true,
                    config,
                    price_fields,
                    operators,
                ) {
                    candidate.conditions.push(condition);
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
                let idx = rng.gen_range(0..candidate.conditions.len());
                candidate.conditions.remove(idx);
                Self::remove_unused_indicators(candidate);
            } else {
                if !available_indicators.is_empty() && !candidate.indicators.is_empty() {
                    let indicator = &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];
                    if let Some(condition) = Self::create_condition_for_indicator(
                        indicator,
                        candidate,
                        true,
                        config,
                        price_fields,
                        operators,
                    ) {
                        candidate.conditions.push(condition);
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            let has_exit_conditions = !candidate.exit_conditions.is_empty();
            let has_stop_handlers = !candidate.stop_handlers.is_empty();
            let has_take_handlers = !candidate.take_handlers.is_empty();
            let can_remove_exit = has_exit_conditions
                && (candidate.exit_conditions.len() > 1 || has_stop_handlers || has_take_handlers);

            if rng.gen::<f64>() < 0.3 && can_remove_exit {
                let idx = rng.gen_range(0..candidate.exit_conditions.len());
                let removed_condition = &candidate.exit_conditions[idx];
                if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&removed_condition.id) {
                    candidate.exit_conditions.remove(idx);
                    Self::remove_conditions_with_indicator(candidate, &alias);
                    Self::remove_unused_indicators(candidate);
                } else {
                    candidate.exit_conditions.remove(idx);
                }
            } else {
                if !available_indicators.is_empty() && !candidate.indicators.is_empty() {
                    let indicator = &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];
                    if let Some(condition) = Self::create_condition_for_indicator(
                        indicator,
                        candidate,
                        false,
                        config,
                        price_fields,
                        operators,
                    ) {
                        candidate.exit_conditions.push(condition);
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            let has_exit_conditions = !candidate.exit_conditions.is_empty();
            let has_stop_handlers = !candidate.stop_handlers.is_empty();
            let has_take_handlers = !candidate.take_handlers.is_empty();
            let can_remove_stop = has_stop_handlers
                && (candidate.stop_handlers.len() > 1 || has_exit_conditions || has_take_handlers);

            if rng.gen::<f64>() < 0.3 && can_remove_stop {
                let idx = rng.gen_range(0..candidate.stop_handlers.len());
                candidate.stop_handlers.remove(idx);
            } else if !stop_handler_configs.is_empty() {
                // Просто выбираем случайный стоп из доступных
                let stop_configs: Vec<&StopHandlerConfig> = stop_handler_configs
                    .iter()
                    .filter(|c| c.stop_type == "stop_loss")
                    .collect();
                if !stop_configs.is_empty() {
                    let stop_config = stop_configs[rng.gen_range(0..stop_configs.len())];
                    let stop = crate::discovery::types::StopHandlerInfo {
                        id: format!("stop_{}", rng.gen::<u32>()),
                        name: stop_config.handler_name.clone(),
                        handler_name: stop_config.handler_name.clone(),
                        stop_type: stop_config.stop_type.clone(),
                        optimization_params: Vec::new(),
                        priority: stop_config.priority,
                    };
                    candidate.stop_handlers.push(stop);
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            let has_exit_conditions = !candidate.exit_conditions.is_empty();
            let has_stop_handlers = !candidate.stop_handlers.is_empty();
            let has_take_handlers = !candidate.take_handlers.is_empty();
            let can_remove_take = has_take_handlers
                && (candidate.take_handlers.len() > 1 || has_exit_conditions || has_stop_handlers);

            if rng.gen::<f64>() < 0.3 && can_remove_take {
                let idx = rng.gen_range(0..candidate.take_handlers.len());
                candidate.take_handlers.remove(idx);
            } else if !stop_handler_configs.is_empty() {
                let has_exit_conditions = !candidate.exit_conditions.is_empty();
                let has_stop_handlers = !candidate.stop_handlers.is_empty();
                let can_add_take = has_exit_conditions || has_stop_handlers;

                if can_add_take {
                    // Просто выбираем случайный тейк из доступных
                    let take_configs: Vec<&StopHandlerConfig> = stop_handler_configs
                        .iter()
                        .filter(|c| c.stop_type == "take_profit")
                        .collect();
                    if !take_configs.is_empty() {
                        let take_config = take_configs[rng.gen_range(0..take_configs.len())];
                        let take = crate::discovery::types::StopHandlerInfo {
                            id: format!("take_{}", rng.gen::<u32>()),
                            name: take_config.handler_name.clone(),
                            handler_name: take_config.handler_name.clone(),
                            stop_type: take_config.stop_type.clone(),
                            optimization_params: Vec::new(),
                            priority: take_config.priority,
                        };
                        candidate.take_handlers.push(take);
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

    /// Отбор особей с поддержанием разнообразия стратегий (round-robin)
    /// Группирует особи по стратегиям, сортирует каждую группу по fitness,
    /// затем по очереди выбирает по одной особи от каждой стратегии
    fn select_with_diversity(
        individuals: Vec<GeneticIndividual>,
        target_size: usize,
    ) -> Vec<GeneticIndividual> {
        use std::collections::HashMap;
        
        // Группируем особи по стратегиям
        let mut strategy_groups: HashMap<String, Vec<GeneticIndividual>> = HashMap::new();
        
        for individual in individuals {
            // Создаем уникальный идентификатор стратегии на основе её структуры
            let strategy_id = if let Some(ref candidate) = individual.strategy.candidate {
                Self::get_strategy_signature(candidate)
            } else {
                // Если нет кандидата, используем хеш параметров как идентификатор
                format!("no_candidate_{:?}", individual.strategy.parameters)
            };
            
            strategy_groups
                .entry(strategy_id)
                .or_insert_with(Vec::new)
                .push(individual);
        }
        
        // Сортируем каждую группу по fitness (от лучшего к худшему)
        for group in strategy_groups.values_mut() {
            group.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_b
                    .partial_cmp(&fitness_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        
        // Round-robin отбор: по очереди берем по одной особи от каждой стратегии
        let mut selected = Vec::with_capacity(target_size);
        let mut strategy_indices: HashMap<String, usize> = HashMap::new();
        
        // Инициализируем индексы для каждой стратегии
        for strategy_id in strategy_groups.keys() {
            strategy_indices.insert(strategy_id.clone(), 0);
        }
        
        while selected.len() < target_size {
            let mut found_any = false;
            
            // Проходим по всем стратегиям в каждом раунде
            for (strategy_id, group) in &strategy_groups {
                if selected.len() >= target_size {
                    break;
                }
                
                let index = strategy_indices.get(strategy_id).copied().unwrap_or(0);
                
                // Если в этой стратегии еще есть особи
                if index < group.len() {
                    selected.push(group[index].clone());
                    strategy_indices.insert(strategy_id.clone(), index + 1);
                    found_any = true;
                }
            }
            
            // Если не нашли ни одной особи в этом раунде, значит все стратегии исчерпаны
            if !found_any {
                break;
            }
        }
        
        println!(
            "      [Отбор с разнообразием] Выбрано {} особей из {} уникальных стратегий (round-robin)",
            selected.len(),
            strategy_groups.len()
        );
        
        selected
    }

    /// Создает уникальный идентификатор стратегии на основе её структуры
    fn get_strategy_signature(candidate: &StrategyCandidate) -> String {
        use std::collections::BTreeSet;
        
        // Сортируем индикаторы по alias для стабильности
        let mut indicator_aliases: BTreeSet<String> = candidate
            .indicators
            .iter()
            .map(|ind| ind.alias.clone())
            .collect();
        
        let mut nested_aliases: BTreeSet<String> = candidate
            .nested_indicators
            .iter()
            .map(|nested| format!("{}->{}", nested.input_indicator_alias, nested.indicator.alias))
            .collect();
        
        let mut condition_ids: BTreeSet<String> = candidate
            .conditions
            .iter()
            .map(|cond| format!("{}:{}:{:?}", cond.condition_type, cond.id, cond.operator))
            .collect();
        
        let mut exit_condition_ids: BTreeSet<String> = candidate
            .exit_conditions
            .iter()
            .map(|cond| format!("{}:{}:{:?}", cond.condition_type, cond.id, cond.operator))
            .collect();
        
        let mut stop_handler_names: BTreeSet<String> = candidate
            .stop_handlers
            .iter()
            .map(|h| h.handler_name.clone())
            .collect();
        
        let mut take_handler_names: BTreeSet<String> = candidate
            .take_handlers
            .iter()
            .map(|h| h.handler_name.clone())
            .collect();
        
        let mut timeframe_strings: BTreeSet<String> = candidate
            .timeframes
            .iter()
            .map(|tf| format!("{:?}", tf))
            .collect();
        
        format!(
            "indicators:{:?}|nested:{:?}|conditions:{:?}|exit:{:?}|stops:{:?}|takes:{:?}|timeframes:{:?}",
            indicator_aliases,
            nested_aliases,
            condition_ids,
            exit_condition_ids,
            stop_handler_names,
            take_handler_names,
            timeframe_strings
        )
    }
}
