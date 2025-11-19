use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::{StrategyCandidate, StrategyDiscoveryEngine};
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::fitness::FitnessFunction;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use crate::strategy::types::StrategyParameterMap;
use rand::Rng;
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

        let strategies_to_generate = if candidates.is_empty() {
            (self.config.population_size as f64 / self.config.decimation_coefficient).ceil()
                as usize
        } else {
            1
        };

        let mut strategy_candidates = Vec::new();
        if candidates.is_empty() {
            let generated = self.generate_candidates(strategies_to_generate).await?;
            strategy_candidates.extend(generated);
        } else {
            strategy_candidates.extend(candidates);
        }

        if strategy_candidates.is_empty() {
            return Ok(Population {
                individuals: Vec::new(),
                generation: 0,
                island_id: None,
            });
        }

        let target_size =
            (self.config.population_size as f64 * self.config.decimation_coefficient) as usize;
        let mut individuals = Vec::new();

        let total_strategies = strategy_candidates.len() * target_size;
        let mut current_strategy = 0;

        println!("   Всего стратегий для тестирования: {}", total_strategies);

        for (candidate_idx, candidate) in strategy_candidates.iter().enumerate() {
            for param_variant in 0..target_size {
                current_strategy += 1;
                let progress = (current_strategy as f64 / total_strategies as f64) * 100.0;

                println!(
                    "   [{}/{}] ({:.1}%) Тестирование стратегии #{} (вариант параметров #{})...",
                    current_strategy,
                    total_strategies,
                    progress,
                    candidate_idx + 1,
                    param_variant + 1
                );

                let random_params = self.generate_random_parameters(candidate);
                let report = self
                    .evaluator
                    .evaluate_strategy(candidate, random_params.clone())
                    .await?;

                if self.config.filter_initial_population {
                    if !FitnessFunction::passes_thresholds(&report, &self.config.fitness_thresholds)
                    {
                        println!(
                            "      ❌ Стратегия не прошла фильтр пороговых значений (Trades: {}, Profit: {:.2})",
                            report.metrics.total_trades, report.metrics.total_profit
                        );
                        continue;
                    }
                }

                let fitness =
                    FitnessFunction::calculate_fitness(&report, &self.config.fitness_weights);

                println!(
                    "      ✅ Стратегия прошла тест (Fitness: {:.4}, Trades: {}, Profit: {:.2}, Win Rate: {:.1}%)",
                    fitness,
                    report.metrics.total_trades,
                    report.metrics.total_profit,
                    report.metrics.winning_percentage * 100.0
                );

                let evaluated = EvaluatedStrategy {
                    candidate: Some(candidate.clone()),
                    parameters: random_params,
                    fitness: Some(fitness),
                    backtest_report: Some(report),
                };

                individuals.push(GeneticIndividual {
                    strategy: evaluated,
                    generation: 0,
                    island_id: None,
                });
            }
        }

        individuals.sort_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_b
                .partial_cmp(&fitness_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let final_individuals: Vec<GeneticIndividual> = individuals
            .into_iter()
            .take(self.config.population_size)
            .collect();

        println!(
            "\n   ✅ Генерация завершена: создано {} особей из {} протестированных",
            final_individuals.len(),
            current_strategy
        );

        if !final_individuals.is_empty() {
            let best_fitness = final_individuals[0].strategy.fitness.unwrap_or(0.0);
            println!("   🏆 Лучший fitness: {:.4}", best_fitness);
        }

        Ok(Population {
            individuals: final_individuals,
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
        use crate::indicators::implementations::get_optimization_range;
        use crate::risk::stops::get_optimization_range as get_stop_optimization_range;
        use crate::strategy::types::StrategyParamValue;

        let mut rng = rand::thread_rng();
        let mut params = HashMap::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_type_str = format!("{:?}", param.param_type);
                    let param_value = if param_type_str.contains("Boolean") {
                        StrategyParamValue::Flag(rng.gen())
                    } else {
                        if let Some(range) =
                            get_optimization_range(&indicator.name, &param.name, &param.param_type)
                        {
                            let steps = ((range.end - range.start) / range.step) as usize;
                            let step_index = rng.gen_range(0..=steps);
                            let value = range.start + (step_index as f32 * range.step);
                            if param_type_str.contains("Integer") {
                                StrategyParamValue::Integer(value as i64)
                            } else {
                                StrategyParamValue::Number(value as f64)
                            }
                        } else {
                            continue;
                        }
                    };
                    params.insert(format!("{}_{}", indicator.name, param.name), param_value);
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_type_str = format!("{:?}", param.param_type);
                    let param_value = if param_type_str.contains("Boolean") {
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
                            if param_type_str.contains("Integer") {
                                StrategyParamValue::Integer(value as i64)
                            } else {
                                StrategyParamValue::Number(value as f64)
                            }
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
                    if let Some(range) =
                        get_stop_optimization_range(&stop_handler.handler_name, &param.name)
                    {
                        let steps = ((range.end - range.start) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.start + (step_index as f32 * range.step);
                        params.insert(
                            format!("stop_{}_{}", stop_handler.name, param.name),
                            StrategyParamValue::Number(value as f64),
                        );
                    }
                }
            }
        }

        for take_handler in &candidate.take_handlers {
            for param in &take_handler.optimization_params {
                if param.optimizable {
                    if let Some(range) =
                        get_stop_optimization_range(&take_handler.handler_name, &param.name)
                    {
                        let steps = ((range.end - range.start) / range.step) as usize;
                        let step_index = rng.gen_range(0..=steps);
                        let value = range.start + (step_index as f32 * range.step);
                        params.insert(
                            format!("take_{}_{}", take_handler.name, param.name),
                            StrategyParamValue::Number(value as f64),
                        );
                    }
                }
            }
        }

        for condition in candidate
            .conditions
            .iter()
            .chain(candidate.exit_conditions.iter())
        {
            let indicator_name = Self::extract_indicator_name_from_condition(candidate, condition);

            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_value = if param.name.to_lowercase() == "threshold"
                        && condition.constant_value.is_some()
                    {
                        StrategyParamValue::Number(condition.constant_value.unwrap())
                    } else if let Some(ind_name) = &indicator_name {
                        use crate::indicators::types::ParameterType;
                        let param_type = match param.name.to_lowercase().as_str() {
                            "threshold" => ParameterType::Threshold,
                            "percentage" | "percent" => ParameterType::Multiplier,
                            _ => ParameterType::Threshold,
                        };

                        if let Some(range) =
                            get_optimization_range(ind_name, &param.name, &param_type)
                        {
                            let steps = ((range.end - range.start) / range.step) as usize;
                            let step_index = rng.gen_range(0..=steps);
                            let value = range.start + (step_index as f32 * range.step);
                            StrategyParamValue::Number(value as f64)
                        } else {
                            continue;
                        }
                    } else {
                        continue;
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

    fn extract_indicator_name_from_condition(
        candidate: &StrategyCandidate,
        condition: &crate::discovery::ConditionInfo,
    ) -> Option<String> {
        let alias = Self::extract_indicator_alias_from_condition_id(&condition.id)?;

        if let Some(ind) = candidate.indicators.iter().find(|i| i.alias == alias) {
            return Some(ind.name.clone());
        }

        if let Some(nested) = candidate
            .nested_indicators
            .iter()
            .find(|n| n.indicator.alias == alias)
        {
            return Some(nested.indicator.name.clone());
        }

        None
    }

    fn extract_indicator_alias_from_condition_id(condition_id: &str) -> Option<String> {
        if condition_id.starts_with("ind_price_") {
            let rest = condition_id.strip_prefix("ind_price_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            }
        } else if condition_id.starts_with("ind_const_") {
            let rest = condition_id.strip_prefix("ind_const_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            }
        }
        None
    }
}
