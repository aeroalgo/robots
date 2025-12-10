use crate::optimization::types::{GeneticAlgorithmConfig, Population};

pub struct IslandManager {
    config: GeneticAlgorithmConfig,
    islands: Vec<Population>,
}

impl IslandManager {
    pub fn new(config: GeneticAlgorithmConfig, initial_populations: Vec<Population>) -> Self {
        let islands_count = config.islands_count.min(initial_populations.len());
        let mut islands = Vec::new();

        for i in 0..islands_count {
            if i < initial_populations.len() {
                let mut population = initial_populations[i].clone();
                population.island_id = Some(i);
                islands.push(population);
            } else {
                let mut population = initial_populations[0].clone();
                population.island_id = Some(i);
                islands.push(population);
            }
        }

        Self { config, islands }
    }

    pub fn get_all_islands(&self) -> &[Population] {
        &self.islands
    }

    pub fn get_all_islands_mut(&mut self) -> &mut [Population] {
        &mut self.islands
    }

    pub fn islands_count(&self) -> usize {
        self.islands.len()
    }

    pub fn should_migrate(&self, generation: usize) -> bool {
        generation > 0 && generation % self.config.migration_interval == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::types::{EvaluatedStrategy, GeneticIndividual, Population};

    fn create_test_population(id: usize) -> Population {
        Population {
            individuals: vec![],
            generation: 0,
            island_id: Some(id),
        }
    }

    fn create_test_config() -> GeneticAlgorithmConfig {
        GeneticAlgorithmConfig {
            islands_count: 3,
            migration_interval: 5,
            ..Default::default()
        }
    }

    #[test]
    fn test_island_manager_new() {
        let config = create_test_config();
        let populations = vec![
            create_test_population(0),
            create_test_population(1),
            create_test_population(2),
        ];
        let manager = IslandManager::new(config, populations);
        assert_eq!(manager.islands_count(), 3);
    }

    #[test]
    fn test_island_manager_limits_islands() {
        let config = create_test_config();
        let populations = vec![
            create_test_population(0),
            create_test_population(1),
            create_test_population(2),
            create_test_population(3),
        ];
        let manager = IslandManager::new(config, populations);
        assert_eq!(manager.islands_count(), 3);
    }

    #[test]
    fn test_get_all_islands() {
        let config = create_test_config();
        let populations = vec![create_test_population(0), create_test_population(1)];
        let manager = IslandManager::new(config, populations);
        let islands = manager.get_all_islands();
        assert_eq!(islands.len(), 2);
    }

    #[test]
    fn test_should_migrate() {
        let config = create_test_config();
        let populations = vec![create_test_population(0)];
        let manager = IslandManager::new(config, populations);
        assert!(!manager.should_migrate(0));
        assert!(!manager.should_migrate(4));
        assert!(manager.should_migrate(5));
        assert!(!manager.should_migrate(6));
        assert!(manager.should_migrate(10));
    }
}
