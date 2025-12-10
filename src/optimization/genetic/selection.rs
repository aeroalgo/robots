use crate::optimization::genetic::helpers;
use crate::optimization::types::{GeneticIndividual, Population};
use std::collections::HashMap;

pub fn select_elites(population: &Population, elitism_count: usize) -> Vec<GeneticIndividual> {
    let mut sorted: Vec<&GeneticIndividual> = population.individuals.iter().collect();
    sorted.sort_by(|a, b| {
        let fitness_a = a.strategy.fitness.unwrap_or(0.0);
        let fitness_b = b.strategy.fitness.unwrap_or(0.0);
        fitness_b
            .partial_cmp(&fitness_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    sorted.into_iter().take(elitism_count).cloned().collect()
}

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

    let mut selected = Vec::with_capacity(target_size);
    let mut strategy_indices: HashMap<String, usize> =
        strategy_groups.keys().map(|k| (k.clone(), 0)).collect();
    let strategy_ids: Vec<String> = strategy_groups.keys().cloned().collect();

    while selected.len() < target_size {
        let mut found_any = false;

        for strategy_id in &strategy_ids {
            if selected.len() >= target_size {
                break;
            }

            if let Some(group) = strategy_groups.get(strategy_id) {
                let index = strategy_indices.get(strategy_id).copied().unwrap_or(0);

                if index < group.len() {
                    selected.push(group[index].clone());
                    strategy_indices.insert(strategy_id.clone(), index + 1);
                    found_any = true;
                }
            }
        }

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
