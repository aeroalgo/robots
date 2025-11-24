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
