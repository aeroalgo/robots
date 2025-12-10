use crate::optimization::types::{GeneticAlgorithmConfig, GeneticIndividual, Population};

pub struct MigrationSystem {
    config: GeneticAlgorithmConfig,
}

impl MigrationSystem {
    pub fn new(config: GeneticAlgorithmConfig) -> Self {
        Self { config }
    }

    pub fn migrate_between_islands(&self, islands: &mut [Population]) -> Result<(), anyhow::Error> {
        if islands.len() < 2 {
            return Ok(());
        }

        let migration_count =
            (self.config.population_size as f64 * self.config.migration_rate) as usize;
        if migration_count == 0 {
            return Ok(());
        }

        let mut migrants: Vec<Vec<GeneticIndividual>> = vec![Vec::new(); islands.len()];

        for (island_idx, island) in islands.iter().enumerate() {
            let mut sorted: Vec<&GeneticIndividual> = island.individuals.iter().collect();
            sorted.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_b
                    .partial_cmp(&fitness_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
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
                    fitness_a
                        .partial_cmp(&fitness_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::types::{EvaluatedStrategy, GeneticIndividual, Population};
    use crate::strategy::types::StrategyParamValue;
    use std::collections::HashMap;

    fn create_test_individual(fitness: f64, island_id: Option<usize>) -> GeneticIndividual {
        GeneticIndividual {
            strategy: EvaluatedStrategy {
                candidate: None,
                parameters: HashMap::new(),
                fitness: Some(fitness),
                backtest_report: None,
            },
            generation: 0,
            island_id,
        }
    }

    fn create_test_population(
        individuals: Vec<GeneticIndividual>,
        island_id: Option<usize>,
    ) -> Population {
        Population {
            individuals,
            generation: 0,
            island_id,
        }
    }

    fn create_test_config() -> GeneticAlgorithmConfig {
        GeneticAlgorithmConfig {
            population_size: 10,
            migration_rate: 0.1,
            ..Default::default()
        }
    }

    #[test]
    fn test_migration_system_new() {
        let config = create_test_config();
        let system = MigrationSystem::new(config);
        assert!(true);
    }

    #[test]
    fn test_migrate_between_islands_empty() {
        let config = create_test_config();
        let system = MigrationSystem::new(config);
        let mut islands = vec![];
        let result = system.migrate_between_islands(&mut islands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_between_islands_single() {
        let config = create_test_config();
        let system = MigrationSystem::new(config);
        let mut islands = vec![create_test_population(vec![], None)];
        let result = system.migrate_between_islands(&mut islands);
        assert!(result.is_ok());
    }

    #[test]
    fn test_migrate_between_islands_multiple() {
        let config = create_test_config();
        let system = MigrationSystem::new(config);
        let mut islands = vec![
            create_test_population(
                vec![
                    create_test_individual(1.0, Some(0)),
                    create_test_individual(2.0, Some(0)),
                    create_test_individual(3.0, Some(0)),
                ],
                Some(0),
            ),
            create_test_population(
                vec![
                    create_test_individual(4.0, Some(1)),
                    create_test_individual(5.0, Some(1)),
                    create_test_individual(6.0, Some(1)),
                ],
                Some(1),
            ),
        ];
        let result = system.migrate_between_islands(&mut islands);
        assert!(result.is_ok());
    }
}
