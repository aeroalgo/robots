use std::collections::HashSet;
use crate::discovery::StrategyCandidate;
use crate::optimization::types::{GeneticAlgorithmConfig, GeneticIndividual, Population};
use crate::optimization::initial_population::InitialPopulationGenerator;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use std::collections::HashMap;

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
        let replace_count = (population.individuals.len() as f64 * self.config.fresh_blood_rate) as usize;
        let actual_replace = replace_count.min(new_individuals.len());

        population.individuals.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_a.partial_cmp(&fitness_b).unwrap_or(std::cmp::Ordering::Equal)
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

        let replace_count = (population.individuals.len() as f64 * self.config.fresh_blood_rate) as usize;
        if to_replace.len() < replace_count {
            population.individuals.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_a.partial_cmp(&fitness_b).unwrap_or(std::cmp::Ordering::Equal)
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
                    let mut new_ind = new_individuals[i].clone();
                    new_ind.generation = population.generation;
                    new_ind.island_id = population.island_id;
                    population.individuals[*idx] = new_ind;
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


    pub async fn inject_fresh_blood_v2(
        &self,
        population: &mut Population,
        generator: &crate::optimization::InitialPopulationGeneratorV2,
    ) -> Result<(), anyhow::Error> {
        let duplicates = self.detect_duplicates(population);
        let mut to_replace = duplicates;

        let replace_count = (population.individuals.len() as f64 * self.config.fresh_blood_rate) as usize;
        if to_replace.len() < replace_count {
            population.individuals.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_a.partial_cmp(&fitness_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            for i in to_replace.len()..replace_count.min(population.individuals.len()) {
                to_replace.push(i);
            }
        }

        if !to_replace.is_empty() {
            let new_population = generator.generate(None).await?;
            let mut new_individuals: Vec<crate::optimization::types::GeneticIndividual> = new_population
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
