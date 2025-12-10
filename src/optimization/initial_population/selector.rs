use crate::optimization::types::GeneticIndividual;
use std::collections::HashMap;

use super::helpers;

pub fn select_with_diversity(
    individuals: Vec<GeneticIndividual>,
    target_size: usize,
) -> Vec<GeneticIndividual> {
    let mut strategy_groups: HashMap<String, Vec<GeneticIndividual>> = HashMap::new();

    for individual in individuals {
        let strategy_id = if let Some(ref candidate) = individual.strategy.candidate {
            helpers::get_strategy_signature(candidate)
        } else {
            format!("no_candidate_{:?}", individual.strategy.parameters)
        };

        strategy_groups
            .entry(strategy_id)
            .or_insert_with(Vec::new)
            .push(individual);
    }

    for group in strategy_groups.values_mut() {
        group.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let total_individuals_before_selection: usize =
        strategy_groups.values().map(|g| g.len()).sum();
    if total_individuals_before_selection > 0 {
        let best_fitness_before_selection = strategy_groups
            .values()
            .flat_map(|g| g.first().map(|ind| ind.strategy.fitness.unwrap_or(0.0)))
            .fold(0.0f64, |a, b| a.max(b));
        println!(
            "   [Отбор] Перед round-robin: {} особей в {} группах, лучший fitness в группах: {:.4}",
            total_individuals_before_selection,
            strategy_groups.len(),
            best_fitness_before_selection
        );
    }

    let mut selected = Vec::with_capacity(target_size);
    let mut strategy_indices: HashMap<String, usize> = HashMap::new();

    for strategy_id in strategy_groups.keys() {
        strategy_indices.insert(strategy_id.clone(), 0);
    }

    while selected.len() < target_size {
        let mut found_any = false;

        for (strategy_id, group) in &strategy_groups {
            if selected.len() >= target_size {
                break;
            }

            let index = strategy_indices.get(strategy_id).copied().unwrap_or(0);

            if index < group.len() {
                selected.push(group[index].clone());
                strategy_indices.insert(strategy_id.clone(), index + 1);
                found_any = true;
            }
        }

        if !found_any {
            break;
        }
    }

    println!(
        "   [Отбор с разнообразием] Выбрано {} особей из {} уникальных стратегий (round-robin)",
        selected.len(),
        strategy_groups.len()
    );

    if !selected.is_empty() {
        let best_in_selected = selected
            .iter()
            .map(|ind| ind.strategy.fitness.unwrap_or(0.0))
            .fold(0.0f64, |a, b| a.max(b));
        let worst_in_selected = selected
            .iter()
            .map(|ind| ind.strategy.fitness.unwrap_or(0.0))
            .fold(f64::INFINITY, |a, b| a.min(b));
        println!(
            "   [Отбор] Fitness диапазон до сортировки: {:.4} - {:.4}",
            worst_in_selected, best_in_selected
        );
    }

    selected.sort_by(|a, b| {
        let fitness_a = a.strategy.fitness.unwrap_or(0.0);
        let fitness_b = b.strategy.fitness.unwrap_or(0.0);
        fitness_b
            .partial_cmp(&fitness_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if !selected.is_empty() {
        let best_after_sort = selected[0].strategy.fitness.unwrap_or(0.0);
        let worst_after_sort = selected[selected.len() - 1].strategy.fitness.unwrap_or(0.0);
        println!(
            "   [Отбор] После сортировки: лучший fitness = {:.4}, худший = {:.4}",
            best_after_sort, worst_after_sort
        );
    }

    selected
}
