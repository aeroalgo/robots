use crate::optimization::migration::MigrationSystem;
use crate::optimization::types::GeneticAlgorithmConfig;

pub struct EvolutionManager {
    config: GeneticAlgorithmConfig,
    migration_system: MigrationSystem,
    stagnation_threshold: usize,
    stagnation_counter: usize,
    best_fitness_history: Vec<f64>,
}

impl EvolutionManager {
    pub fn new(config: GeneticAlgorithmConfig) -> Self {
        Self {
            stagnation_threshold: 10,
            stagnation_counter: 0,
            best_fitness_history: Vec::new(),
            migration_system: MigrationSystem::new(config.clone()),
            config,
        }
    }

    pub fn should_restart(&self) -> bool {
        if self.config.restart_on_stagnation {
            self.stagnation_counter >= self.stagnation_threshold
        } else {
            false
        }
    }

    pub fn update_fitness_history(&mut self, best_fitness: f64) {
        self.best_fitness_history.push(best_fitness);

        if self.best_fitness_history.len() > self.stagnation_threshold {
            self.best_fitness_history.remove(0);
        }

        if self.best_fitness_history.len() >= self.stagnation_threshold {
            let first = self.best_fitness_history[0];
            let last = self.best_fitness_history[self.best_fitness_history.len() - 1];
            let improvement = (last - first).abs() / first.max(0.001);

            if improvement < 0.01 {
                self.stagnation_counter += 1;
            } else {
                self.stagnation_counter = 0;
            }
        }
    }

    pub fn reset_stagnation(&mut self) {
        self.stagnation_counter = 0;
        self.best_fitness_history.clear();
    }

    fn should_migrate(&self, generation: usize) -> bool {
        generation > 0 && generation % self.config.migration_interval == 0
    }
}

