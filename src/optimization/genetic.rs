use crate::backtest::BacktestConfig;
use crate::condition::ConditionParameterPresets;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StopHandlerConfig;
use crate::discovery::StrategyCandidate;
use crate::optimization::builders::ConditionBuilder;
use crate::optimization::candidate_builder::CandidateBuilder;
use crate::optimization::candidate_builder_config::ConditionProbabilities;
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::population::PopulationManager;
use crate::optimization::sds::StochasticDiffusionSearch;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::{ConditionOperator, PriceField, StrategyParameterMap};
use rand::seq::SliceRandom;
use rand::Rng;
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

        // Для entry conditions используем только Close
        // High и Low используются только в стоп-обработчиках для определения пробития стопа
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
                let fitness1 = parents[0].strategy.fitness;
                let fitness2 = parents[1].strategy.fitness;

                let (mut child1_candidate, mut child2_candidate) =
                    self.crossover_structure_hybrid(&cand1, &cand2, fitness1, fitness2);

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

                use crate::discovery::strategy_converter::ParameterExtractor;
                let parameter_specs1 = ParameterExtractor::extract_all(&child1_candidate);
                let parameter_specs2 = ParameterExtractor::extract_all(&child2_candidate);
                
                self.population_manager
                    .sync_parameters_with_structure(&mut child1_params, &child1_candidate, &parameter_specs1);
                self.population_manager
                    .sync_parameters_with_structure(&mut child2_params, &child2_candidate, &parameter_specs2);
                
                self.population_manager
                    .mutate(&mut child1_params, &child1_candidate, &self.config, &parameter_specs1);
                self.population_manager
                    .mutate(&mut child2_params, &child2_candidate, &self.config, &parameter_specs2);

                evaluated_count += 1;
                let progress = (evaluated_count as f64 / lambda as f64) * 100.0;
                println!(
                    "      [{}/{}] ({:.1}%) Оценка новой особи...",
                    evaluated_count, lambda, progress
                );

                let child1_candidate_clone = child1_candidate.clone();
                let child1_params_clone = child1_params.clone();
                Self::log_strategy_details(&child1_candidate_clone, &child1_params_clone, "Child1");

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
                        eprintln!("      🔍 Детали стратегии, вызвавшей ошибку:");
                        Self::log_strategy_details(
                            &child1_candidate_clone,
                            &child1_params_clone,
                            "ERROR",
                        );
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

                    let child2_candidate_clone = child2_candidate.clone();
                    let child2_params_clone = child2_params.clone();
                    Self::log_strategy_details(
                        &child2_candidate_clone,
                        &child2_params_clone,
                        "Child2",
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
                            eprintln!("      🔍 Детали стратегии, вызвавшей ошибку:");
                            Self::log_strategy_details(
                                &child2_candidate_clone,
                                &child2_params_clone,
                                "ERROR",
                            );
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

    fn crossover_structure_hybrid(
        &self,
        parent1: &StrategyCandidate,
        parent2: &StrategyCandidate,
        fitness1: Option<f64>,
        fitness2: Option<f64>,
    ) -> (StrategyCandidate, StrategyCandidate) {
        let mut rng = rand::thread_rng();

        let max_entry = self
            .config
            .candidate_builder_config
            .as_ref()
            .map(|c| c.constraints.max_entry_conditions)
            .unwrap_or(4);

        let max_exit = self
            .config
            .candidate_builder_config
            .as_ref()
            .map(|c| c.constraints.max_exit_conditions)
            .unwrap_or(2);

        let min_entry = self
            .config
            .candidate_builder_config
            .as_ref()
            .map(|c| c.constraints.min_entry_conditions)
            .unwrap_or(1);

        let max_indicators = self
            .config
            .candidate_builder_config
            .as_ref()
            .map(|c| c.constraints.max_indicators)
            .unwrap_or(4);

        let mut child1 = parent1.clone();
        let mut child2 = parent2.clone();

        if rng.gen::<f64>() < self.config.crossover_rate {
            let f1 = fitness1.unwrap_or(0.0);
            let f2 = fitness2.unwrap_or(0.0);
            let total_fitness = f1 + f2;

            let relative_diff = if total_fitness > 0.001 {
                (f1 - f2).abs() / total_fitness
            } else {
                0.0
            };

            let use_weighted = relative_diff > 0.15 && fitness1.is_some() && fitness2.is_some();

            let (weight1, weight2) = if use_weighted && total_fitness > 0.0 {
                (f1 / total_fitness, f2 / total_fitness)
            } else {
                (0.5, 0.5)
            };

            println!(
                "      [Crossover] {} | P1: {:.3} ({} cond) | P2: {:.3} ({} cond) | w1={:.2} w2={:.2}",
                if use_weighted { "WEIGHTED" } else { "UNIFORM" },
                f1,
                parent1.conditions.len(),
                f2,
                parent2.conditions.len(),
                weight1,
                weight2
            );

            if rng.gen::<f64>() < 0.5 {
                let (child1_entry, child2_entry) = Self::crossover_conditions_hybrid(
                    &parent1.conditions,
                    &parent2.conditions,
                    parent1,
                    parent2,
                    max_entry,
                    min_entry,
                    weight1,
                    weight2,
                    use_weighted,
                );

                child1.conditions = child1_entry;
                child2.conditions = child2_entry;

                Self::sync_indicators_with_conditions(&mut child1, parent1, parent2);
                Self::sync_indicators_with_conditions(&mut child2, parent1, parent2);
            }

            if rng.gen::<f64>() < 0.5 {
                let (child1_exit, child2_exit) = Self::crossover_conditions_hybrid(
                    &parent1.exit_conditions,
                    &parent2.exit_conditions,
                    parent1,
                    parent2,
                    max_exit,
                    0,
                    weight1,
                    weight2,
                    use_weighted,
                );

                child1.exit_conditions = child1_exit;
                child2.exit_conditions = child2_exit;

                Self::sync_indicators_with_conditions(&mut child1, parent1, parent2);
                Self::sync_indicators_with_conditions(&mut child2, parent1, parent2);
            }

            Self::remove_unused_indicators(&mut child1);
            Self::remove_unused_indicators(&mut child2);

            Self::enforce_indicator_limits(&mut child1, max_indicators);
            Self::enforce_indicator_limits(&mut child2, max_indicators);

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.stop_handlers, &mut child2.stop_handlers);
            }

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.take_handlers, &mut child2.take_handlers);
            }

            if rng.gen::<f64>() < 0.5 {
                std::mem::swap(&mut child1.timeframes, &mut child2.timeframes);
            }

            Self::ensure_minimum_conditions(&mut child1, parent1, min_entry);
            Self::ensure_minimum_conditions(&mut child2, parent2, min_entry);

            println!(
                "      [Crossover Result] C1: {} entry, {} exit, {} ind | C2: {} entry, {} exit, {} ind",
                child1.conditions.len(),
                child1.exit_conditions.len(),
                child1.indicators.len() + child1.nested_indicators.len(),
                child2.conditions.len(),
                child2.exit_conditions.len(),
                child2.indicators.len() + child2.nested_indicators.len()
            );
        }

        (child1, child2)
    }

    fn crossover_conditions_hybrid(
        conditions1: &[crate::discovery::ConditionInfo],
        conditions2: &[crate::discovery::ConditionInfo],
        parent1: &StrategyCandidate,
        parent2: &StrategyCandidate,
        max_conditions: usize,
        min_conditions: usize,
        weight1: f64,
        weight2: f64,
        use_weighted: bool,
    ) -> (
        Vec<crate::discovery::ConditionInfo>,
        Vec<crate::discovery::ConditionInfo>,
    ) {
        let mut rng = rand::thread_rng();

        let mut all_conditions: Vec<(crate::discovery::ConditionInfo, &StrategyCandidate, f64)> =
            Vec::new();

        for cond in conditions1 {
            all_conditions.push((cond.clone(), parent1, weight1));
        }
        for cond in conditions2 {
            all_conditions.push((cond.clone(), parent2, weight2));
        }

        let mut unique_conditions: Vec<(crate::discovery::ConditionInfo, &StrategyCandidate, f64)> =
            Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for (cond, parent, weight) in all_conditions {
            if !seen_ids.contains(&cond.id) {
                seen_ids.insert(cond.id.clone());
                unique_conditions.push((cond, parent, weight));
            }
        }

        unique_conditions.shuffle(&mut rng);

        let mut child1_conditions: Vec<crate::discovery::ConditionInfo> = Vec::new();
        let mut child2_conditions: Vec<crate::discovery::ConditionInfo> = Vec::new();

        if use_weighted {
            for (cond, _parent, weight) in &unique_conditions {
                if child1_conditions.len() < max_conditions
                    && rng.gen::<f64>() < *weight
                    && !CandidateBuilder::has_conflicting_comparison_operator(
                        cond,
                        &child1_conditions,
                    )
                {
                    child1_conditions.push(cond.clone());
                }
            }

            for (cond, _parent, weight) in &unique_conditions {
                let inverse_weight = 1.0 - weight;
                if child2_conditions.len() < max_conditions
                    && rng.gen::<f64>() < inverse_weight
                    && !CandidateBuilder::has_conflicting_comparison_operator(
                        cond,
                        &child2_conditions,
                    )
                {
                    child2_conditions.push(cond.clone());
                }
            }
        } else {
            for (_i, (cond, _parent, _weight)) in unique_conditions.iter().enumerate() {
                if child1_conditions.len() >= max_conditions
                    && child2_conditions.len() >= max_conditions
                {
                    break;
                }

                if rng.gen::<f64>() < 0.5 {
                    if child1_conditions.len() < max_conditions
                        && !CandidateBuilder::has_conflicting_comparison_operator(
                            cond,
                            &child1_conditions,
                        )
                    {
                        child1_conditions.push(cond.clone());
                    } else if child2_conditions.len() < max_conditions
                        && !CandidateBuilder::has_conflicting_comparison_operator(
                            cond,
                            &child2_conditions,
                        )
                    {
                        child2_conditions.push(cond.clone());
                    }
                } else {
                    if child2_conditions.len() < max_conditions
                        && !CandidateBuilder::has_conflicting_comparison_operator(
                            cond,
                            &child2_conditions,
                        )
                    {
                        child2_conditions.push(cond.clone());
                    } else if child1_conditions.len() < max_conditions
                        && !CandidateBuilder::has_conflicting_comparison_operator(
                            cond,
                            &child1_conditions,
                        )
                    {
                        child1_conditions.push(cond.clone());
                    }
                }
            }
        }

        while child1_conditions.len() < min_conditions && !unique_conditions.is_empty() {
            let idx = rng.gen_range(0..unique_conditions.len());
            let (cond, _, _) = &unique_conditions[idx];
            if !child1_conditions.iter().any(|c| c.id == cond.id)
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, &child1_conditions)
            {
                child1_conditions.push(cond.clone());
            }
        }

        while child2_conditions.len() < min_conditions && !unique_conditions.is_empty() {
            let idx = rng.gen_range(0..unique_conditions.len());
            let (cond, _, _) = &unique_conditions[idx];
            if !child2_conditions.iter().any(|c| c.id == cond.id)
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, &child2_conditions)
            {
                child2_conditions.push(cond.clone());
            }
        }

        child1_conditions.truncate(max_conditions);
        child2_conditions.truncate(max_conditions);

        (child1_conditions, child2_conditions)
    }

    fn sync_indicators_with_conditions(
        child: &mut StrategyCandidate,
        parent1: &StrategyCandidate,
        parent2: &StrategyCandidate,
    ) {
        let mut required_aliases = std::collections::HashSet::new();

        for cond in child.conditions.iter().chain(child.exit_conditions.iter()) {
            for alias in cond.all_indicator_aliases() {
                required_aliases.insert(alias);
            }
        }

        for nested in &child.nested_indicators {
            required_aliases.insert(nested.input_indicator_alias.clone());
        }

        for alias in &required_aliases {
            let has_indicator = child.indicators.iter().any(|i| &i.alias == alias)
                || child
                    .nested_indicators
                    .iter()
                    .any(|n| &n.indicator.alias == alias);

            if !has_indicator {
                if let Some(ind) = parent1.indicators.iter().find(|i| &i.alias == alias) {
                    child.indicators.push(ind.clone());
                } else if let Some(ind) = parent2.indicators.iter().find(|i| &i.alias == alias) {
                    child.indicators.push(ind.clone());
                } else if let Some(nested) = parent1
                    .nested_indicators
                    .iter()
                    .find(|n| &n.indicator.alias == alias)
                {
                    child.nested_indicators.push(nested.clone());
                } else if let Some(nested) = parent2
                    .nested_indicators
                    .iter()
                    .find(|n| &n.indicator.alias == alias)
                {
                    child.nested_indicators.push(nested.clone());
                }
            }
        }
    }

    fn enforce_indicator_limits(child: &mut StrategyCandidate, max_indicators: usize) {
        let total_indicators = child.indicators.len() + child.nested_indicators.len();

        if total_indicators > max_indicators {
            let used_aliases = Self::get_used_indicator_aliases(child);

            child
                .indicators
                .retain(|ind| used_aliases.contains(&ind.alias));
            child
                .nested_indicators
                .retain(|nested| used_aliases.contains(&nested.indicator.alias));

            let remaining = child.indicators.len() + child.nested_indicators.len();
            if remaining > max_indicators {
                let excess = remaining - max_indicators;
                for _ in 0..excess {
                    if !child.nested_indicators.is_empty() {
                        child.nested_indicators.pop();
                    } else if child.indicators.len() > 1 {
                        child.indicators.pop();
                    }
                }
            }
        }
    }

    fn ensure_minimum_conditions(
        child: &mut StrategyCandidate,
        fallback_parent: &StrategyCandidate,
        min_conditions: usize,
    ) {
        if child.conditions.len() < min_conditions && !fallback_parent.conditions.is_empty() {
            let mut rng = rand::thread_rng();
            let mut attempts = 0;
            let max_attempts = fallback_parent.conditions.len() * 3;
            while child.conditions.len() < min_conditions && attempts < max_attempts {
                attempts += 1;
                let idx = rng.gen_range(0..fallback_parent.conditions.len());
                let cond = &fallback_parent.conditions[idx];
                if !child.conditions.iter().any(|c| c.id == cond.id)
                    && !CandidateBuilder::has_conflicting_comparison_operator(
                        cond,
                        &child.conditions,
                    )
                {
                    child.conditions.push(cond.clone());

                    for alias in cond.all_indicator_aliases() {
                        if !child.indicators.iter().any(|i| i.alias == alias)
                            && !child
                                .nested_indicators
                                .iter()
                                .any(|n| n.indicator.alias == alias)
                        {
                            if let Some(ind) =
                                fallback_parent.indicators.iter().find(|i| i.alias == alias)
                            {
                                child.indicators.push(ind.clone());
                            } else if let Some(nested) = fallback_parent
                                .nested_indicators
                                .iter()
                                .find(|n| n.indicator.alias == alias)
                            {
                                child.nested_indicators.push(nested.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn remove_unused_indicators(candidate: &mut StrategyCandidate) {
        let used_aliases = Self::get_used_indicator_aliases(candidate);
        let used_timeframes = Self::get_used_timeframes(candidate);

        candidate
            .indicators
            .retain(|ind| used_aliases.contains(&ind.alias));
        candidate
            .nested_indicators
            .retain(|nested| used_aliases.contains(&nested.indicator.alias));
        candidate
            .timeframes
            .retain(|tf| used_timeframes.contains(tf));
    }

    fn get_used_timeframes(candidate: &StrategyCandidate) -> std::collections::HashSet<TimeFrame> {
        let mut used_timeframes = std::collections::HashSet::new();

        for condition in candidate
            .conditions
            .iter()
            .chain(candidate.exit_conditions.iter())
        {
            if let Some(tf) = &condition.primary_timeframe {
                used_timeframes.insert(tf.clone());
            }
            if let Some(tf) = &condition.secondary_timeframe {
                used_timeframes.insert(tf.clone());
            }
        }

        used_timeframes
    }

    fn get_used_indicator_aliases(
        candidate: &StrategyCandidate,
    ) -> std::collections::HashSet<String> {
        let mut used_aliases = std::collections::HashSet::new();

        for condition in candidate
            .conditions
            .iter()
            .chain(candidate.exit_conditions.iter())
        {
            for alias in condition.all_indicator_aliases() {
                used_aliases.insert(alias);
            }
        }

        for nested in &candidate.nested_indicators {
            used_aliases.insert(nested.input_indicator_alias.clone());
        }

        used_aliases
    }

    fn remove_conditions_with_indicator(candidate: &mut StrategyCandidate, alias: &str) {
        candidate
            .conditions
            .retain(|cond| !cond.all_indicator_aliases().contains(&alias.to_string()));

        candidate
            .exit_conditions
            .retain(|cond| !cond.all_indicator_aliases().contains(&alias.to_string()));
    }

    fn create_condition_for_indicator(
        indicator: &crate::discovery::IndicatorInfo,
        candidate: &StrategyCandidate,
        is_entry: bool,
        config: &GeneticAlgorithmConfig,
        _price_fields: &[PriceField],
        _operators: &[ConditionOperator],
    ) -> Option<crate::discovery::ConditionInfo> {
        let default_probabilities = ConditionProbabilities::default();
        let probabilities = config
            .candidate_builder_config
            .as_ref()
            .map(|c| &c.probabilities.conditions)
            .unwrap_or(&default_probabilities);

        ConditionBuilder::create_for_candidate_indicator(
            indicator,
            candidate,
            is_entry,
            probabilities,
        )
    }

    fn flip_operator(operator: &ConditionOperator) -> ConditionOperator {
        operator.opposite()
    }

    fn get_safe_flipped_operator(
        conditions: &[crate::discovery::ConditionInfo],
        idx: usize,
    ) -> Option<ConditionOperator> {
        let condition = &conditions[idx];

        if condition.condition_type == "trend_condition" {
            return Some(Self::flip_operator(&condition.operator));
        }

        if !CandidateBuilder::is_comparison_operator(&condition.operator) {
            return Some(Self::flip_operator(&condition.operator));
        }

        let new_operator = Self::flip_operator(&condition.operator);

        let mut test_condition = condition.clone();
        test_condition.operator = new_operator.clone();

        let other_conditions: Vec<crate::discovery::ConditionInfo> = conditions
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, c)| c.clone())
            .collect();

        if !CandidateBuilder::has_conflicting_comparison_operator(
            &test_condition,
            &other_conditions,
        ) {
            Some(new_operator)
        } else {
            None
        }
    }

    fn can_compare_indicators_for_mutation(
        primary: &crate::discovery::IndicatorInfo,
        secondary: &crate::discovery::IndicatorInfo,
        nested_indicators: &[crate::discovery::NestedIndicator],
        all_indicators: &[crate::discovery::IndicatorInfo],
    ) -> bool {
        ConditionBuilder::can_compare_indicators(
            primary,
            secondary,
            nested_indicators,
            all_indicators,
        )
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
                let removed_indicator = &candidate.indicators[idx];
                let removed_type = removed_indicator.indicator_type.clone();
                let removed_alias = removed_indicator.alias.clone();
                candidate.indicators.remove(idx);

                Self::remove_conditions_with_indicator(candidate, &removed_alias);
                Self::remove_unused_indicators(candidate);

                let same_type_indicators: Vec<_> = available_indicators
                    .iter()
                    .filter(|ind| ind.indicator_type == removed_type)
                    .collect();

                if !same_type_indicators.is_empty() {
                    let new_indicator =
                        same_type_indicators[rng.gen_range(0..same_type_indicators.len())].clone();
                    candidate.indicators.push(new_indicator.clone());

                    if let Some(condition) = Self::create_condition_for_indicator(
                        &new_indicator,
                        candidate,
                        true,
                        config,
                        price_fields,
                        operators,
                    ) {
                        if !CandidateBuilder::has_conflicting_comparison_operator(
                            &condition,
                            &candidate.conditions,
                        ) {
                            candidate.conditions.push(condition);
                        }
                    }
                }
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
                    if !CandidateBuilder::has_conflicting_comparison_operator(
                        &condition,
                        &candidate.conditions,
                    ) {
                        candidate.conditions.push(condition);
                    }
                }
            }
        }

        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < config.mutation_rate {
            if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
                let idx = rng.gen_range(0..candidate.conditions.len());
                candidate.conditions.remove(idx);
                Self::remove_unused_indicators(candidate);
            } else if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
                let idx = rng.gen_range(0..candidate.conditions.len());
                if let Some(new_operator) =
                    Self::get_safe_flipped_operator(&candidate.conditions, idx)
                {
                    candidate.conditions[idx].operator = new_operator.clone();
                    Self::update_optimization_params_for_operator(
                        &mut candidate.conditions[idx],
                        &new_operator,
                    );
                }
            } else {
                if !available_indicators.is_empty() && !candidate.indicators.is_empty() {
                    let indicator =
                        &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];
                    if let Some(condition) = Self::create_condition_for_indicator(
                        indicator,
                        candidate,
                        true,
                        config,
                        price_fields,
                        operators,
                    ) {
                        if !CandidateBuilder::has_conflicting_comparison_operator(
                            &condition,
                            &candidate.conditions,
                        ) {
                            candidate.conditions.push(condition);
                        }
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
                let aliases = candidate.exit_conditions[idx].all_indicator_aliases();
                candidate.exit_conditions.remove(idx);
                for alias in aliases {
                    Self::remove_conditions_with_indicator(candidate, &alias);
                }
                Self::remove_unused_indicators(candidate);
            } else if rng.gen::<f64>() < 0.3 && !candidate.exit_conditions.is_empty() {
                let idx = rng.gen_range(0..candidate.exit_conditions.len());
                if let Some(new_operator) =
                    Self::get_safe_flipped_operator(&candidate.exit_conditions, idx)
                {
                    candidate.exit_conditions[idx].operator = new_operator.clone();
                    Self::update_optimization_params_for_operator(
                        &mut candidate.exit_conditions[idx],
                        &new_operator,
                    );
                }
            } else {
                if !available_indicators.is_empty() && !candidate.indicators.is_empty() {
                    let indicator =
                        &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];
                    if let Some(condition) = Self::create_condition_for_indicator(
                        indicator,
                        candidate,
                        false,
                        config,
                        price_fields,
                        operators,
                    ) {
                        if !CandidateBuilder::has_conflicting_comparison_operator(
                            &condition,
                            &candidate.exit_conditions,
                        ) {
                            candidate.exit_conditions.push(condition);
                        }
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
            if rng.gen::<f64>() < 0.2 && !candidate.nested_indicators.is_empty() {
                let idx = rng.gen_range(0..candidate.nested_indicators.len());
                let removed_nested = &candidate.nested_indicators[idx];
                let removed_type = removed_nested.indicator.indicator_type.clone();
                let removed_alias = removed_nested.indicator.alias.clone();
                candidate.nested_indicators.remove(idx);

                Self::remove_conditions_with_indicator(candidate, &removed_alias);
                Self::remove_unused_indicators(candidate);

                let same_type_indicators: Vec<_> = available_indicators
                    .iter()
                    .filter(|ind| ind.indicator_type == removed_type)
                    .collect();

                if !same_type_indicators.is_empty() && !candidate.indicators.is_empty() {
                    let new_indicator =
                        same_type_indicators[rng.gen_range(0..same_type_indicators.len())].clone();
                    let input_indicator =
                        &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];

                    let new_nested = crate::discovery::NestedIndicator {
                        indicator: new_indicator,
                        input_indicator_alias: input_indicator.alias.clone(),
                        depth: 1,
                    };
                    candidate.nested_indicators.push(new_nested);
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

    fn log_strategy_details(
        candidate: &StrategyCandidate,
        parameters: &StrategyParameterMap,
        label: &str,
    ) {
        println!("      📋 [{}] Детали стратегии:", label);
        println!("         Индикаторы ({}):", candidate.indicators.len());
        for ind in &candidate.indicators {
            println!("           - {} (alias: {})", ind.name, ind.alias);
            for param in &ind.parameters {
                let param_key = format!("{}_{}", ind.alias, param.name);
                if let Some(value) = parameters.get(&param_key) {
                    println!("             {}: {:?}", param.name, value);
                }
            }
        }
        if !candidate.nested_indicators.is_empty() {
            println!(
                "         Вложенные индикаторы ({}):",
                candidate.nested_indicators.len()
            );
            for nested in &candidate.nested_indicators {
                println!(
                    "           - {} (alias: {}) <- {} (depth: {})",
                    nested.indicator.name,
                    nested.indicator.alias,
                    nested.input_indicator_alias,
                    nested.depth
                );
            }
        }
        println!("         Условия входа ({}):", candidate.conditions.len());
        for cond in &candidate.conditions {
            let tf_info = if let Some(tf) = &cond.primary_timeframe {
                format!(" [TF: {:?}]", tf)
            } else {
                " [TF: base]".to_string()
            };
            println!("           - {} ({}){}", cond.name, cond.id, tf_info);
        }
        if !candidate.exit_conditions.is_empty() {
            println!(
                "         Условия выхода ({}):",
                candidate.exit_conditions.len()
            );
            for cond in &candidate.exit_conditions {
                let tf_info = if let Some(tf) = &cond.primary_timeframe {
                    format!(" [TF: {:?}]", tf)
                } else {
                    " [TF: base]".to_string()
                };
                println!("           - {} ({}){}", cond.name, cond.id, tf_info);
            }
        }
        if !candidate.timeframes.is_empty() {
            println!("         Таймфреймы: {:?}", candidate.timeframes);
        }
        if !candidate.stop_handlers.is_empty() {
            println!(
                "         Stop handlers ({}):",
                candidate.stop_handlers.len()
            );
            for handler in &candidate.stop_handlers {
                println!("           - {} ({})", handler.name, handler.handler_name);
            }
        }
        if !candidate.take_handlers.is_empty() {
            println!(
                "         Take handlers ({}):",
                candidate.take_handlers.len()
            );
            for handler in &candidate.take_handlers {
                println!("           - {} ({})", handler.name, handler.handler_name);
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
    /// затем по очереди выбирает по одной особи от каждой стратегии.
    /// После round-robin отбора финальный список сортируется по fitness (от лучшего к худшему),
    /// чтобы гарантировать, что лучшие стратегии попадут в популяцию.
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

        selected.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        selected
    }

    /// Создает уникальный идентификатор стратегии на основе её структуры
    fn get_strategy_signature(candidate: &StrategyCandidate) -> String {
        use std::collections::BTreeSet;

        // Сортируем индикаторы по alias для стабильности
        let indicator_aliases: BTreeSet<String> = candidate
            .indicators
            .iter()
            .map(|ind| ind.alias.clone())
            .collect();

        let nested_aliases: BTreeSet<String> = candidate
            .nested_indicators
            .iter()
            .map(|nested| {
                format!(
                    "{}->{}",
                    nested.input_indicator_alias, nested.indicator.alias
                )
            })
            .collect();

        let condition_ids: BTreeSet<String> = candidate
            .conditions
            .iter()
            .map(|cond| format!("{}:{}:{:?}", cond.condition_type, cond.id, cond.operator))
            .collect();

        let exit_condition_ids: BTreeSet<String> = candidate
            .exit_conditions
            .iter()
            .map(|cond| format!("{}:{}:{:?}", cond.condition_type, cond.id, cond.operator))
            .collect();

        let stop_handler_names: BTreeSet<String> = candidate
            .stop_handlers
            .iter()
            .map(|h| h.handler_name.clone())
            .collect();

        let take_handler_names: BTreeSet<String> = candidate
            .take_handlers
            .iter()
            .map(|h| h.handler_name.clone())
            .collect();

        let timeframe_strings: BTreeSet<String> = candidate
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

    fn update_optimization_params_for_operator(
        condition: &mut crate::discovery::ConditionInfo,
        operator: &ConditionOperator,
    ) {
        condition.optimization_params =
            crate::discovery::condition::ConditionCombinationGenerator::create_optimization_params_for_operator(operator);
    }
}
