use crate::optimization::initial_population::InitialPopulationGenerator;
use crate::optimization::types::{GeneticAlgorithmConfig, GeneticIndividual, Population};
use std::collections::HashSet;

pub struct FreshBloodSystem {
    config: GeneticAlgorithmConfig,
}

impl FreshBloodSystem {
    pub fn new(config: GeneticAlgorithmConfig) -> Self {
        Self { config }
    }

    pub fn detect_duplicates(&self, population: &Population) -> Vec<usize> {
        if !self.config.detect_duplicates {
            return Vec::new();
        }

        let mut duplicates = Vec::new();
        let mut seen = HashSet::new();

        for (idx, individual) in population.individuals.iter().enumerate() {
            let signature = self.create_signature(&individual.strategy);
            if seen.contains(&signature) {
                duplicates.push(idx);
            } else {
                seen.insert(signature);
            }
        }

        duplicates
    }

    pub fn replace_weakest(
        &self,
        population: &mut Population,
        new_individuals: Vec<GeneticIndividual>,
    ) {
        let replace_count =
            (population.individuals.len() as f64 * self.config.fresh_blood_rate) as usize;
        let actual_replace = replace_count.min(new_individuals.len());

        population.individuals.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_a
                .partial_cmp(&fitness_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (i, new_ind) in new_individuals.into_iter().take(actual_replace).enumerate() {
            if i < population.individuals.len() {
                population.individuals[i] = new_ind;
            }
        }
    }

    pub async fn inject_fresh_blood(
        &self,
        population: &mut Population,
        generator: &InitialPopulationGenerator,
    ) -> Result<(), anyhow::Error> {
        let duplicates = self.detect_duplicates(population);
        let mut to_replace = duplicates;

        let replace_count =
            (population.individuals.len() as f64 * self.config.fresh_blood_rate) as usize;
        if to_replace.len() < replace_count {
            population.individuals.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_a
                    .partial_cmp(&fitness_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for i in to_replace.len()..replace_count.min(population.individuals.len()) {
                to_replace.push(i);
            }
        }

        if !to_replace.is_empty() {
            let new_population = generator.generate(None).await?;
            let mut new_individuals: Vec<GeneticIndividual> = new_population
                .individuals
                .into_iter()
                .take(to_replace.len())
                .collect();

            for (i, idx) in to_replace.iter().enumerate() {
                if i < new_individuals.len() && *idx < population.individuals.len() {
                    if let Some(island_id) = population.island_id {
                        new_individuals[i].island_id = Some(island_id);
                    }
                    new_individuals[i].generation = population.generation;
                    population.individuals[*idx] = new_individuals[i].clone();
                }
            }
        }

        Ok(())
    }

    fn create_signature(&self, strategy: &crate::optimization::types::EvaluatedStrategy) -> String {
        let mut parts = Vec::new();

        if let Some(ref candidate) = strategy.candidate {
            parts.push(format!("indicators:{}", candidate.indicators.len()));
            parts.push(format!("nested:{}", candidate.nested_indicators.len()));
            parts.push(format!("conditions:{}", candidate.conditions.len()));
        }

        for (key, value) in &strategy.parameters {
            parts.push(format!("{}:{:?}", key, value));
        }

        parts.sort();
        parts.join("|")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::types::{EvaluatedStrategy, GeneticIndividual, Population};
    use crate::strategy::types::StrategyParamValue;
    use std::collections::HashMap;

    fn create_test_individual(fitness: f64) -> GeneticIndividual {
        let mut params = HashMap::new();
        params.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        GeneticIndividual {
            strategy: EvaluatedStrategy {
                candidate: None,
                parameters: params,
                fitness: Some(fitness),
                backtest_report: None,
            },
            generation: 0,
            island_id: None,
        }
    }

    fn create_test_population(individuals: Vec<GeneticIndividual>) -> Population {
        Population {
            individuals,
            generation: 0,
            island_id: None,
        }
    }

    fn create_test_config() -> GeneticAlgorithmConfig {
        GeneticAlgorithmConfig {
            detect_duplicates: true,
            fresh_blood_rate: 0.1,
            ..Default::default()
        }
    }

    #[test]
    fn test_fresh_blood_system_new() {
        let config = create_test_config();
        let system = FreshBloodSystem::new(config);
        assert!(true);
    }

    #[test]
    fn test_detect_duplicates_disabled() {
        let config = GeneticAlgorithmConfig {
            detect_duplicates: false,
            ..Default::default()
        };
        let system = FreshBloodSystem::new(config);
        let population = create_test_population(vec![
            create_test_individual(1.0),
            create_test_individual(2.0),
        ]);
        let duplicates = system.detect_duplicates(&population);
        assert_eq!(duplicates.len(), 0);
    }

    #[test]
    fn test_detect_duplicates_no_duplicates() {
        let config = create_test_config();
        let system = FreshBloodSystem::new(config);
        let mut ind1 = create_test_individual(1.0);
        let mut ind2 = create_test_individual(2.0);
        ind1.strategy.parameters.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        ind2.strategy.parameters.insert("param2".to_string(), StrategyParamValue::Number(20.0));
        let population = create_test_population(vec![ind1, ind2]);
        let duplicates = system.detect_duplicates(&population);
        assert_eq!(duplicates.len(), 0);
    }

    #[test]
    fn test_replace_weakest() {
        let config = create_test_config();
        let system = FreshBloodSystem::new(config);
        let mut population = create_test_population(vec![
            create_test_individual(1.0),
            create_test_individual(2.0),
            create_test_individual(3.0),
        ]);
        let new_individuals = vec![create_test_individual(10.0)];
        system.replace_weakest(&mut population, new_individuals);
        assert_eq!(population.individuals.len(), 3);
    }

    #[test]
    fn test_replace_weakest_empty_new() {
        let config = create_test_config();
        let system = FreshBloodSystem::new(config);
        let mut population = create_test_population(vec![
            create_test_individual(1.0),
            create_test_individual(2.0),
        ]);
        let new_individuals = vec![];
        system.replace_weakest(&mut population, new_individuals);
        assert_eq!(population.individuals.len(), 2);
    }
}
