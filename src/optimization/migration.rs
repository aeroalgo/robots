use rand::Rng;
use crate::optimization::types::{GeneticAlgorithmConfig, GeneticIndividual, Population};

pub struct MigrationSystem {
    config: GeneticAlgorithmConfig,
}

impl MigrationSystem {
    pub fn new(config: GeneticAlgorithmConfig) -> Self {
        Self { config }
    }

    pub fn migrate_between_islands(
        &self,
        islands: &mut [Population],
    ) -> Result<(), anyhow::Error> {
        if islands.len() < 2 {
            return Ok(());
        }

        let migration_count = (self.config.population_size as f64 * self.config.migration_rate) as usize;
        if migration_count == 0 {
            return Ok(());
        }

        let mut migrants: Vec<Vec<GeneticIndividual>> = vec![Vec::new(); islands.len()];

        for (island_idx, island) in islands.iter().enumerate() {
            let mut sorted: Vec<&GeneticIndividual> = island.individuals.iter().collect();
            sorted.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_b.partial_cmp(&fitness_a).unwrap_or(std::cmp::Ordering::Equal)
            });

            for i in 0..migration_count.min(sorted.len()) {
                migrants[island_idx].push(sorted[i].clone());
            }
        }

        for island_idx in 0..islands.len() {
            let target_island_idx = (island_idx + 1) % islands.len();
            let incoming_migrants = std::mem::take(&mut migrants[target_island_idx]);

            if !incoming_migrants.is_empty() {
                islands[island_idx].individuals.sort_by(|a, b| {
                    let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                    let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                    fitness_a.partial_cmp(&fitness_b).unwrap_or(std::cmp::Ordering::Equal)
                });

                for (i, migrant) in incoming_migrants.into_iter().enumerate() {
                    if i < islands[island_idx].individuals.len() {
                        let mut migrant = migrant;
                        migrant.island_id = Some(island_idx);
                        islands[island_idx].individuals[i] = migrant;
                    }
                }
            }
        }

        Ok(())
    }
}

