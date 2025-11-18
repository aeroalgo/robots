use std::collections::HashMap;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::{StrategyCandidate, StrategyDiscoveryEngine};
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::StrategyParameterMap;
use rand::Rng;

pub struct InitialPopulationGeneratorV2 {
    config: GeneticAlgorithmConfig,
    evaluator: StrategyEvaluationRunner,
    discovery_config: crate::discovery::StrategyDiscoveryConfig,
}

impl InitialPopulationGeneratorV2 {
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
            let random_params = self.generate_random_parameters(&candidate);
            let report = self
                .evaluator
                .evaluate_strategy(&candidate, random_params.clone())
                .await?;

            let fitness = Some(
                crate::optimization::fitness::FitnessFunction::calculate_fitness(
                    &report,
                    &self.config.fitness_weights,
                ),
            );

            let evaluated = EvaluatedStrategy {
                candidate: Some(candidate.clone()),
                parameters: random_params,
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

    fn generate_random_parameters(&self, candidate: &StrategyCandidate) -> StrategyParameterMap {
        use crate::strategy::types::StrategyParamValue;
        use crate::indicators::implementations::get_optimization_range;

        let mut rng = rand::thread_rng();
        let mut params = HashMap::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_type_str = format!("{:?}", param.param_type);
                    let param_value = if param_type_str.contains("Integer") {
                        let min = 10;
                        let max = 200;
                        let value = rng.gen_range(min..=max);
                        StrategyParamValue::Integer(value)
                    } else if param_type_str.contains("Float") {
                        let min = 0.1;
                        let max = 10.0;
                        let value = rng.gen_range(min..=max);
                        StrategyParamValue::Number(value)
                    } else if param_type_str.contains("Boolean") {
                        StrategyParamValue::Flag(rng.gen())
                    } else {
                        if let Some(range) = get_optimization_range(
                            &indicator.name,
                            &param.name,
                            &param.param_type,
                        ) {
                            let steps = ((range.end - range.start) / range.step) as usize;
                            let step_index = rng.gen_range(0..=steps);
                            let value = range.start + (step_index as f32 * range.step);
                            StrategyParamValue::Number(value as f64)
                        } else {
                            continue;
                        }
                    };
                    params.insert(
                        format!("{}_{}", indicator.name, param.name),
                        param_value,
                    );
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_type_str = format!("{:?}", param.param_type);
                    let param_value = if param_type_str.contains("Integer") {
                        let min = 10;
                        let max = 200;
                        let value = rng.gen_range(min..=max);
                        StrategyParamValue::Integer(value)
                    } else if param_type_str.contains("Float") {
                        let min = 0.1;
                        let max = 10.0;
                        let value = rng.gen_range(min..=max);
                        StrategyParamValue::Number(value)
                    } else if param_type_str.contains("Boolean") {
                        StrategyParamValue::Flag(rng.gen())
                    } else {
                        if let Some(range) = get_optimization_range(
                            &nested.indicator.name,
                            &param.name,
                            &param.param_type,
                        ) {
                            let steps = ((range.end - range.start) / range.step) as usize;
                            let step_index = rng.gen_range(0..=steps);
                            let value = range.start + (step_index as f32 * range.step);
                            StrategyParamValue::Number(value as f64)
                        } else {
                            continue;
                        }
                    };
                    params.insert(
                        format!("nested_{}_{}", nested.indicator.name, param.name),
                        param_value,
                    );
                }
            }
        }

        for stop_handler in &candidate.stop_handlers {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_value = match param.name.to_lowercase().as_str() {
                        "percentage" | "percent" => {
                            let value = rng.gen_range(0.5..=10.0);
                            StrategyParamValue::Number(value)
                        }
                        "value" | "size" => {
                            let value = rng.gen_range(10.0..=1000.0);
                            StrategyParamValue::Number(value)
                        }
                        _ => {
                            let value = rng.gen_range(1.0..=100.0);
                            StrategyParamValue::Number(value)
                        }
                    };
                    params.insert(
                        format!("stop_{}_{}", stop_handler.name, param.name),
                        param_value,
                    );
                }
            }
        }

        for condition in candidate.conditions.iter().chain(candidate.exit_conditions.iter()) {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_value = match param.name.to_lowercase().as_str() {
                        "threshold" => {
                            if let Some(const_val) = condition.constant_value {
                                StrategyParamValue::Number(const_val)
                            } else {
                                let value = rng.gen_range(0.0..=100.0);
                                StrategyParamValue::Number(value)
                            }
                        }
                        "percentage" | "percent" => {
                            let value = rng.gen_range(0.1..=10.0);
                            StrategyParamValue::Number(value)
                        }
                        _ => {
                            let value = rng.gen_range(1.0..=100.0);
                            StrategyParamValue::Number(value)
                        }
                    };
                    params.insert(
                        format!("condition_{}_{}", condition.id, param.name),
                        param_value,
                    );
                }
            }
        }

        params
    }
}

