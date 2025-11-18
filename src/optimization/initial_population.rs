use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::{StrategyCandidate, StrategyDiscoveryEngine};
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::fitness::{FitnessFunction, FitnessThresholds, FitnessWeights};
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::StrategyParameterMap;
use std::collections::HashMap;

pub struct InitialPopulationGenerator {
    config: GeneticAlgorithmConfig,
    evaluator: StrategyEvaluationRunner,
    discovery_config: crate::discovery::StrategyDiscoveryConfig,
}

impl InitialPopulationGenerator {
    pub fn new(
        config: GeneticAlgorithmConfig,
        frames: HashMap<TimeFrame, QuoteFrame>,
        base_timeframe: TimeFrame,
    ) -> Self {
        let mut discovery_config = crate::discovery::StrategyDiscoveryConfig::default();
        discovery_config.base_timeframe = base_timeframe.clone();
        Self::with_discovery_config(config, frames, base_timeframe, discovery_config)
    }

    pub fn with_discovery_config(
        config: GeneticAlgorithmConfig,
        frames: HashMap<TimeFrame, QuoteFrame>,
        base_timeframe: TimeFrame,
        discovery_config: crate::discovery::StrategyDiscoveryConfig,
    ) -> Self {
        Self {
            config,
            evaluator: StrategyEvaluationRunner::new(frames, base_timeframe),
            discovery_config,
        }
    }

    pub async fn generate(
        &self,
        existing_candidates: Option<Vec<StrategyCandidate>>,
    ) -> Result<Population, anyhow::Error> {
        let mut candidates = Vec::new();

        if self.config.use_existing_strategies {
            if let Some(existing) = existing_candidates {
                candidates.extend(existing);
            }
        }

        let target_size =
            (self.config.population_size as f64 * self.config.decimation_coefficient) as usize;
        let needed = target_size.saturating_sub(candidates.len());

        if needed > 0 {
            let generated = self.generate_candidates(needed).await?;
            candidates.extend(generated);
        }

        let mut individuals = Vec::new();
        for candidate in candidates {
            let default_params = self.extract_default_parameters(&candidate);
            let report = self
                .evaluator
                .evaluate_strategy(&candidate, default_params.clone())
                .await?;

            if self.config.filter_initial_population {
                if !FitnessFunction::passes_thresholds(&report, &self.config.fitness_thresholds) {
                    continue;
                }
            }

            let fitness = FitnessFunction::evaluate_strategy(
                &report,
                &self.config.fitness_thresholds,
                &self.config.fitness_weights,
            );

            let evaluated = EvaluatedStrategy {
                candidate: Some(candidate.clone()),
                parameters: default_params,
                fitness,
                backtest_report: Some(report),
            };

            individuals.push(GeneticIndividual {
                strategy: evaluated,
                generation: 0,
                island_id: None,
            });

            if individuals.len() >= self.config.population_size {
                break;
            }
        }

        individuals.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(Population {
            individuals,
            generation: 0,
            island_id: None,
        })
    }

    async fn generate_candidates(
        &self,
        count: usize,
    ) -> Result<Vec<StrategyCandidate>, anyhow::Error> {
        use crate::discovery::IndicatorInfoCollector;
        use crate::strategy::types::PriceField;

        let mut candidates = Vec::new();
        let mut engine = StrategyDiscoveryEngine::new(self.discovery_config.clone());

        use crate::indicators::registry::IndicatorRegistry;
        use crate::strategy::types::ConditionOperator;

        let registry = IndicatorRegistry::new();
        let available_indicators_vec = IndicatorInfoCollector::collect_from_registry(&registry);
        let price_fields = vec![
            PriceField::Close,
            PriceField::Open,
            PriceField::High,
            PriceField::Low,
        ];

        let operators = vec![
            ConditionOperator::GreaterThan,
            ConditionOperator::LessThan,
            ConditionOperator::CrossesAbove,
            ConditionOperator::CrossesBelow,
        ];
        let stop_handler_configs = vec![];

        let mut iterator = engine.generate_strategies_random(
            &available_indicators_vec,
            &price_fields,
            &operators,
            &stop_handler_configs,
        );

        for _ in 0..count {
            if let Some(candidate) = iterator.next() {
                candidates.push(candidate);
            } else {
                break;
            }
        }

        Ok(candidates)
    }

    fn extract_default_parameters(&self, candidate: &StrategyCandidate) -> StrategyParameterMap {
        use crate::strategy::types::StrategyParamValue;

        let mut params = HashMap::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_type = &param.param_type;
                    let default_value = match param_type {
                        _ if format!("{:?}", param_type).contains("Integer") => {
                            StrategyParamValue::Integer(20)
                        }
                        _ if format!("{:?}", param_type).contains("Float") => {
                            StrategyParamValue::Number(0.5)
                        }
                        _ if format!("{:?}", param_type).contains("Boolean") => {
                            StrategyParamValue::Flag(false)
                        }
                        _ => continue,
                    };
                    params.insert(format!("{}_{}", indicator.name, param.name), default_value);
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_type = &param.param_type;
                    let default_value = match param_type {
                        _ if format!("{:?}", param_type).contains("Integer") => {
                            StrategyParamValue::Integer(20)
                        }
                        _ if format!("{:?}", param_type).contains("Float") => {
                            StrategyParamValue::Number(0.5)
                        }
                        _ if format!("{:?}", param_type).contains("Boolean") => {
                            StrategyParamValue::Flag(false)
                        }
                        _ => continue,
                    };
                    params.insert(
                        format!("nested_{}_{}", nested.indicator.name, param.name),
                        default_value,
                    );
                }
            }
        }

        params
    }
}
