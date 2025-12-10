use crate::discovery::StrategyCandidate;
use crate::optimization::candidate_builder::CandidateBuilder;
use crate::optimization::genetic::helpers;
use crate::optimization::types::GeneticAlgorithmConfig;
use rand::seq::SliceRandom;
use rand::Rng;

pub fn crossover_structure_hybrid(
    parent1: &StrategyCandidate,
    parent2: &StrategyCandidate,
    fitness1: Option<f64>,
    fitness2: Option<f64>,
    config: &GeneticAlgorithmConfig,
) -> (StrategyCandidate, StrategyCandidate) {
    let mut rng = rand::thread_rng();

    let max_entry = config
        .candidate_builder_config
        .as_ref()
        .map(|c| c.constraints.max_entry_conditions)
        .unwrap_or(4);

    let max_exit = config
        .candidate_builder_config
        .as_ref()
        .map(|c| c.constraints.max_exit_conditions)
        .unwrap_or(2);

    let min_entry = config
        .candidate_builder_config
        .as_ref()
        .map(|c| c.constraints.min_entry_conditions)
        .unwrap_or(1);

    let max_indicators = config
        .candidate_builder_config
        .as_ref()
        .map(|c| c.constraints.max_indicators)
        .unwrap_or(4);

    let mut child1 = parent1.clone();
    let mut child2 = parent2.clone();

    if rng.gen::<f64>() < config.crossover_rate {
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
            let (child1_entry, child2_entry) = crossover_conditions_hybrid(
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

            sync_indicators_with_conditions(&mut child1, parent1, parent2);
            sync_indicators_with_conditions(&mut child2, parent1, parent2);
        }

        if rng.gen::<f64>() < 0.5 {
            let (child1_exit, child2_exit) = crossover_conditions_hybrid(
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

            sync_indicators_with_conditions(&mut child1, parent1, parent2);
            sync_indicators_with_conditions(&mut child2, parent1, parent2);
        }

        helpers::remove_unused_indicators(&mut child1);
        helpers::remove_unused_indicators(&mut child2);

        enforce_indicator_limits(&mut child1, max_indicators);
        enforce_indicator_limits(&mut child2, max_indicators);

        if rng.gen::<f64>() < 0.5 {
            std::mem::swap(&mut child1.stop_handlers, &mut child2.stop_handlers);
        }

        if rng.gen::<f64>() < 0.5 {
            std::mem::swap(&mut child1.take_handlers, &mut child2.take_handlers);
        }

        if rng.gen::<f64>() < 0.5 {
            std::mem::swap(&mut child1.timeframes, &mut child2.timeframes);
        }

        ensure_minimum_conditions(&mut child1, parent1, min_entry);
        ensure_minimum_conditions(&mut child2, parent2, min_entry);

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

pub fn crossover_conditions_hybrid(
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
        assign_conditions_weighted(
            &unique_conditions,
            &mut child1_conditions,
            &mut child2_conditions,
            max_conditions,
            weight1,
            weight2,
        );
    } else {
        assign_conditions_uniform(
            &unique_conditions,
            &mut child1_conditions,
            &mut child2_conditions,
            max_conditions,
        );
    }

    let mut child1_ids: std::collections::HashSet<String> =
        child1_conditions.iter().map(|c| c.id.clone()).collect();
    let mut child2_ids: std::collections::HashSet<String> =
        child2_conditions.iter().map(|c| c.id.clone()).collect();

    while child1_conditions.len() < min_conditions && !unique_conditions.is_empty() {
        let idx = rng.gen_range(0..unique_conditions.len());
        let (cond, _, _) = &unique_conditions[idx];
        if !child1_ids.contains(&cond.id)
            && !CandidateBuilder::has_conflicting_comparison_operator(cond, &child1_conditions)
        {
            child1_conditions.push(cond.clone());
            child1_ids.insert(cond.id.clone());
        }
    }

    while child2_conditions.len() < min_conditions && !unique_conditions.is_empty() {
        let idx = rng.gen_range(0..unique_conditions.len());
        let (cond, _, _) = &unique_conditions[idx];
        if !child2_ids.contains(&cond.id)
            && !CandidateBuilder::has_conflicting_comparison_operator(cond, &child2_conditions)
        {
            child2_conditions.push(cond.clone());
            child2_ids.insert(cond.id.clone());
        }
    }

    child1_conditions.truncate(max_conditions);
    child2_conditions.truncate(max_conditions);

    (child1_conditions, child2_conditions)
}

fn assign_conditions_weighted(
    unique_conditions: &[(crate::discovery::ConditionInfo, &StrategyCandidate, f64)],
    child1_conditions: &mut Vec<crate::discovery::ConditionInfo>,
    child2_conditions: &mut Vec<crate::discovery::ConditionInfo>,
    max_conditions: usize,
    weight1: f64,
    weight2: f64,
) {
    let mut rng = rand::thread_rng();

    for (cond, _parent, weight) in unique_conditions {
        if child1_conditions.len() < max_conditions
            && rng.gen::<f64>() < *weight
            && !CandidateBuilder::has_conflicting_comparison_operator(cond, child1_conditions)
        {
            child1_conditions.push(cond.clone());
        }
    }

    for (cond, _parent, weight) in unique_conditions {
        let inverse_weight = 1.0 - weight;
        if child2_conditions.len() < max_conditions
            && rng.gen::<f64>() < inverse_weight
            && !CandidateBuilder::has_conflicting_comparison_operator(cond, child2_conditions)
        {
            child2_conditions.push(cond.clone());
        }
    }
}

fn assign_conditions_uniform(
    unique_conditions: &[(crate::discovery::ConditionInfo, &StrategyCandidate, f64)],
    child1_conditions: &mut Vec<crate::discovery::ConditionInfo>,
    child2_conditions: &mut Vec<crate::discovery::ConditionInfo>,
    max_conditions: usize,
) {
    let mut rng = rand::thread_rng();

    for (_i, (cond, _parent, _weight)) in unique_conditions.iter().enumerate() {
        if child1_conditions.len() >= max_conditions && child2_conditions.len() >= max_conditions {
            break;
        }

        if rng.gen::<f64>() < 0.5 {
            if child1_conditions.len() < max_conditions
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, child1_conditions)
            {
                child1_conditions.push(cond.clone());
            } else if child2_conditions.len() < max_conditions
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, child2_conditions)
            {
                child2_conditions.push(cond.clone());
            }
        } else {
            if child2_conditions.len() < max_conditions
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, child2_conditions)
            {
                child2_conditions.push(cond.clone());
            } else if child1_conditions.len() < max_conditions
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, child1_conditions)
            {
                child1_conditions.push(cond.clone());
            }
        }
    }
}

pub fn sync_indicators_with_conditions(
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

pub fn enforce_indicator_limits(child: &mut StrategyCandidate, max_indicators: usize) {
    let total_indicators = child.indicators.len() + child.nested_indicators.len();

    if total_indicators > max_indicators {
        let used_aliases = helpers::get_used_indicator_aliases(child);

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

pub fn ensure_minimum_conditions(
    child: &mut StrategyCandidate,
    fallback_parent: &StrategyCandidate,
    min_conditions: usize,
) {
    if child.conditions.len() < min_conditions && !fallback_parent.conditions.is_empty() {
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        let max_attempts = fallback_parent.conditions.len() * 3;
        let mut child_condition_ids: std::collections::HashSet<String> =
            child.conditions.iter().map(|c| c.id.clone()).collect();
        while child.conditions.len() < min_conditions && attempts < max_attempts {
            attempts += 1;
            let idx = rng.gen_range(0..fallback_parent.conditions.len());
            let cond = &fallback_parent.conditions[idx];
            if !child_condition_ids.contains(&cond.id)
                && !CandidateBuilder::has_conflicting_comparison_operator(cond, &child.conditions)
            {
                child.conditions.push(cond.clone());
                child_condition_ids.insert(cond.id.clone());

                let mut child_indicator_aliases: std::collections::HashSet<String> =
                    child.indicators.iter().map(|i| i.alias.clone()).collect();
                child_indicator_aliases.extend(
                    child
                        .nested_indicators
                        .iter()
                        .map(|n| n.indicator.alias.clone()),
                );

                for alias in cond.all_indicator_aliases() {
                    if !child_indicator_aliases.contains(&alias) {
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
