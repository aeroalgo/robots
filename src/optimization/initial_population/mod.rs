mod candidate_generator;
mod helpers;
mod parameter_generator;
mod selector;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StrategyCandidate;
use crate::optimization::candidate_builder_config::CandidateBuilderConfig;
use crate::optimization::evaluator::StrategyEvaluationRunner;
use crate::optimization::fitness::FitnessFunction;
use crate::optimization::types::{
    EvaluatedStrategy, GeneticAlgorithmConfig, GeneticIndividual, Population,
};
use std::collections::HashMap;

pub struct InitialPopulationGenerator {
    config: GeneticAlgorithmConfig,
    evaluator: StrategyEvaluationRunner,
    discovery_config: crate::discovery::StrategyDiscoveryConfig,
    candidate_builder_config: CandidateBuilderConfig,
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
        let candidate_builder_config = config
            .candidate_builder_config
            .clone()
            .unwrap_or_else(|| CandidateBuilderConfig::default());

        let base_minutes = discovery_config
            .base_timeframe
            .total_minutes()
            .unwrap_or(60) as u32;
        let max_minutes = discovery_config.max_timeframe_minutes;
        let higher_timeframes: Vec<TimeFrame> = (base_minutes * 2..=max_minutes)
            .step_by(base_minutes as usize)
            .map(TimeFrame::minutes)
            .collect();

        Self {
            config,
            evaluator: StrategyEvaluationRunner::with_higher_timeframes(
                frames,
                base_timeframe,
                higher_timeframes,
            ),
            discovery_config,
            candidate_builder_config,
        }
    }

    pub async fn generate(
        &self,
        existing_candidates: Option<Vec<StrategyCandidate>>,
    ) -> Result<Population, anyhow::Error> {
        let initial_capacity = existing_candidates.as_ref().map(|v| v.len()).unwrap_or(0);
        let mut candidates = Vec::with_capacity(initial_capacity);

        if self.config.use_existing_strategies {
            if let Some(existing) = existing_candidates {
                candidates.extend(existing);
            }
        }

        let strategies_to_generate = if candidates.is_empty() {
            (self.config.population_size as f64 * self.config.decimation_coefficient) as usize
        } else {
            1
        };

        let mut all_strategy_candidates =
            Vec::with_capacity(strategies_to_generate.max(initial_capacity));
        if candidates.is_empty() {
            println!(
                "   [Этап 1] Генерация {} кандидатов стратегий (population_size: {} × decimation_coefficient: {:.1})",
                strategies_to_generate,
                self.config.population_size,
                self.config.decimation_coefficient
            );
            let generated = candidate_generator::generate_candidates(
                strategies_to_generate,
                &self.candidate_builder_config,
                &self.evaluator,
                &self.discovery_config,
            )
            .await?;
            all_strategy_candidates.extend(generated);
        } else {
            all_strategy_candidates.extend(candidates);
        }

        if all_strategy_candidates.is_empty() {
            return Ok(Population {
                individuals: Vec::new(),
                generation: 0,
                island_id: None,
            });
        }

        println!(
            "   [Этап 1] Сгенерировано {} кандидатов стратегий",
            all_strategy_candidates.len()
        );

        let param_variants_count = self.config.param_variants_per_candidate;
        println!(
            "\n   [Этап 2] Тестирование всех {} кандидатов (по {} вариантов параметров для каждого)...",
            all_strategy_candidates.len(),
            param_variants_count
        );

        let total_strategies = all_strategy_candidates.len() * param_variants_count;
        let mut individuals = Vec::with_capacity(total_strategies);
        let mut current_strategy = 0;

        for (candidate_idx, candidate) in all_strategy_candidates.iter().enumerate() {
            for param_variant in 0..param_variants_count {
                current_strategy += 1;
                let progress = (current_strategy as f64 / total_strategies as f64) * 100.0;

                println!(
                    "\n   [{}/{}] ({:.1}%) Тестирование кандидата #{} (вариант параметров #{})...",
                    current_strategy,
                    total_strategies,
                    progress,
                    candidate_idx + 1,
                    param_variant + 1
                );

                if current_strategy % 5 == 1 {
                    println!("      📊 Структура кандидата:");
                    println!("         Таймфреймы: {:?}", candidate.timeframes);

                    println!("         Индикаторы:");
                    let base_tf = candidate.timeframes.first();
                    for ind in &candidate.indicators {
                        let params: Vec<String> = ind
                            .parameters
                            .iter()
                            .map(|p| format!("{}:{:?}", p.name, p.param_type))
                            .collect();

                        let mut ind_timeframes: Vec<String> = candidate
                            .conditions
                            .iter()
                            .chain(candidate.exit_conditions.iter())
                            .filter(|c| c.name.starts_with(&ind.name))
                            .map(|c| {
                                c.primary_timeframe
                                    .as_ref()
                                    .or(base_tf)
                                    .map(|tf| tf.identifier())
                                    .unwrap_or_default()
                            })
                            .filter(|s| !s.is_empty())
                            .collect();
                        ind_timeframes.sort();
                        ind_timeframes.dedup();

                        let tf_str = if ind_timeframes.is_empty() {
                            String::new()
                        } else {
                            format!(" TF:[{}]", ind_timeframes.join(","))
                        };

                        if params.is_empty() {
                            println!("            {} (нет параметров){}", ind.name, tf_str);
                        } else {
                            println!("            {} [{}]{}", ind.name, params.join(", "), tf_str);
                        }
                    }

                    if !candidate.nested_indicators.is_empty() {
                        println!("         Nested индикаторы:");
                        for n in &candidate.nested_indicators {
                            let params: Vec<String> = n
                                .indicator
                                .parameters
                                .iter()
                                .map(|p| format!("{}:{:?}", p.name, p.param_type))
                                .collect();

                            let mut ind_timeframes: Vec<String> = candidate
                                .conditions
                                .iter()
                                .chain(candidate.exit_conditions.iter())
                                .filter(|c| c.name.starts_with(&n.indicator.name))
                                .filter_map(|c| c.primary_timeframe.as_ref())
                                .map(|tf| tf.identifier())
                                .collect();
                            ind_timeframes.sort();
                            ind_timeframes.dedup();

                            let tf_str = if ind_timeframes.is_empty() {
                                String::new()
                            } else {
                                format!(" TF:[{}]", ind_timeframes.join(","))
                            };

                            println!(
                                "            {} на {} [{}]{}",
                                n.indicator.name,
                                n.input_indicator_alias,
                                params.join(", "),
                                tf_str
                            );
                        }
                    }

                    println!("         Entry условия:");
                    for c in &candidate.conditions {
                        let params: Vec<String> = c
                            .optimization_params
                            .iter()
                            .map(|p| p.name.clone())
                            .collect();
                        let params_str = if params.is_empty() {
                            String::new()
                        } else {
                            format!(" params=[{}]", params.join(", "))
                        };
                        let tf_str = c
                            .primary_timeframe
                            .as_ref()
                            .map(|tf| format!(" TF:{}", tf.identifier()))
                            .unwrap_or_default();
                        println!(
                            "            {} [{}]{}{}",
                            c.name, c.condition_type, params_str, tf_str
                        );
                    }

                    if !candidate.exit_conditions.is_empty() {
                        println!("         Exit условия:");
                        for c in &candidate.exit_conditions {
                            let tf_str = c
                                .primary_timeframe
                                .as_ref()
                                .map(|tf| format!(" TF:{}", tf.identifier()))
                                .unwrap_or_default();
                            println!("            {} [{}]{}", c.name, c.condition_type, tf_str);
                        }
                    }

                    println!("         Stop handlers:");
                    for s in &candidate.stop_handlers {
                        let params: Vec<String> = s
                            .optimization_params
                            .iter()
                            .map(|p| p.name.clone())
                            .collect();
                        if params.is_empty() {
                            println!("            {}", s.handler_name);
                        } else {
                            println!("            {} [{}]", s.handler_name, params.join(", "));
                        }
                    }
                }

                let random_params = parameter_generator::generate_random_parameters(
                    candidate,
                    &self.candidate_builder_config,
                );

                if current_strategy % 5 == 1 {
                    println!("         📈 Значения параметров:");
                    let mut sorted_params: Vec<_> = random_params.iter().collect();
                    sorted_params.sort_by(|a, b| a.0.cmp(b.0));
                    for (key, value) in sorted_params {
                        let val_str = match value {
                            crate::strategy::types::StrategyParamValue::Number(n) => {
                                format!("{:.2}", n)
                            }
                            crate::strategy::types::StrategyParamValue::Integer(i) => {
                                format!("{}", i)
                            }
                            crate::strategy::types::StrategyParamValue::Text(s) => s.clone(),
                            crate::strategy::types::StrategyParamValue::Flag(b) => format!("{}", b),
                            crate::strategy::types::StrategyParamValue::List(_) => {
                                "[...]".to_string()
                            }
                        };
                        println!("            {} = {}", key, val_str);
                    }
                }

                let report = match self
                    .evaluator
                    .evaluate_strategy(candidate, random_params.clone())
                    .await
                {
                    Ok(report) => report,
                    Err(e) => {
                        eprintln!(
                            "      ❌ Ошибка выполнения backtest для кандидата #{} (вариант #{})",
                            candidate_idx + 1,
                            param_variant + 1
                        );
                        eprintln!("      Детали ошибки: {:?}", e);
                        if let Some(source) = e.source() {
                            eprintln!("      Источник ошибки: {:?}", source);
                        }
                        continue;
                    }
                };

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

        let total_tested = all_strategy_candidates.len() * param_variants_count;
        println!(
            "\n   [Этап 2] Выполнено {} тестов ({} кандидатов × {} вариантов параметров), прошло фильтр: {} стратегий",
            total_tested,
            all_strategy_candidates.len(),
            param_variants_count,
            individuals.len()
        );

        let passed_filter = individuals.len();
        println!(
            "\n   [Этап 3] Отбор лучших {} особей из {} прошедших фильтр...",
            self.config.population_size, passed_filter
        );

        let final_individuals =
            selector::select_with_diversity(individuals, self.config.population_size);

        println!(
            "\n   ✅ Генерация завершена: отобрано {} особей из {} прошедших фильтр",
            final_individuals.len(),
            passed_filter
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
}
