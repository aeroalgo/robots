use crate::optimization::types::GeneticAlgorithmConfig;

pub struct EvolutionManager {
    config: GeneticAlgorithmConfig,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> GeneticAlgorithmConfig {
        GeneticAlgorithmConfig {
            restart_on_stagnation: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_evolution_manager_new() {
        let config = create_test_config();
        let manager = EvolutionManager::new(config);
        assert_eq!(manager.stagnation_counter, 0);
        assert!(manager.best_fitness_history.is_empty());
    }

    #[test]
    fn test_should_restart_false() {
        let config = GeneticAlgorithmConfig {
            restart_on_stagnation: false,
            ..Default::default()
        };
        let manager = EvolutionManager::new(config);
        assert!(!manager.should_restart());
    }

    #[test]
    fn test_should_restart_true() {
        let config = create_test_config();
        let mut manager = EvolutionManager::new(config);
        for _ in 0..20 {
            manager.update_fitness_history(1.0);
        }
        assert!(manager.stagnation_counter >= manager.stagnation_threshold);
        assert!(manager.should_restart());
    }

    #[test]
    fn test_update_fitness_history() {
        let config = create_test_config();
        let mut manager = EvolutionManager::new(config);
        manager.update_fitness_history(1.0);
        assert_eq!(manager.best_fitness_history.len(), 1);
        assert_eq!(manager.best_fitness_history[0], 1.0);
    }

    #[test]
    fn test_update_fitness_history_stagnation() {
        let config = create_test_config();
        let mut manager = EvolutionManager::new(config);
        for _ in 0..10 {
            manager.update_fitness_history(1.0);
        }
        assert!(manager.stagnation_counter > 0);
    }

    #[test]
    fn test_update_fitness_history_improvement() {
        let config = create_test_config();
        let mut manager = EvolutionManager::new(config);
        manager.update_fitness_history(1.0);
        manager.update_fitness_history(2.0);
        assert_eq!(manager.stagnation_counter, 0);
    }

    #[test]
    fn test_reset_stagnation() {
        let config = create_test_config();
        let mut manager = EvolutionManager::new(config);
        for _ in 0..10 {
            manager.update_fitness_history(1.0);
        }
        manager.reset_stagnation();
        assert_eq!(manager.stagnation_counter, 0);
        assert!(manager.best_fitness_history.is_empty());
    }
}
