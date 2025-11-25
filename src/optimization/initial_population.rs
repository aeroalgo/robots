use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::StrategyCandidate;
use crate::optimization::candidate_builder::{CandidateBuilder, CandidateElements};
use crate::optimization::candidate_builder_config::CandidateBuilderConfig;
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
        Self {
            config,
            evaluator: StrategyEvaluationRunner::new(frames, base_timeframe),
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

        // Этап 1: Генерация кандидатов стратегий с использованием decimation_coefficient
        let strategies_to_generate = if candidates.is_empty() {
            // Генерируем больше структур стратегий для большего разнообразия
            // Используем decimation_coefficient: генерируем population_size * decimation_coefficient кандидатов
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
            let generated = self.generate_candidates(strategies_to_generate).await?;
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

        // Этап 2: Тестирование всех кандидатов с множеством вариантов параметров
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

                let random_params = self.generate_random_parameters(candidate);
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

        println!(
            "\n   [Этап 2] Протестировано {} стратегий ({} кандидатов × {} вариантов параметров)",
            individuals.len(),
            all_strategy_candidates.len(),
            param_variants_count
        );

        // Этап 3: Отбор лучших особей
        let total_tested = individuals.len();
        println!(
            "\n   [Этап 3] Отбор лучших {} особей из {} протестированных...",
            self.config.population_size, total_tested
        );

        // Round-robin отбор с группировкой по стратегиям для поддержания разнообразия
        let final_individuals =
            Self::select_with_diversity(individuals, self.config.population_size);

        println!(
            "\n   ✅ Генерация завершена: отобрано {} особей из {} протестированных",
            final_individuals.len(),
            total_tested
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
        println!(
            "   [Генерация кандидатов] Начало генерации {} кандидатов стратегий...",
            count
        );

        use crate::discovery::IndicatorInfoCollector;

        let mut candidates = Vec::with_capacity(count);

        use crate::indicators::registry::IndicatorRegistry;
        use crate::risk::registry::StopHandlerRegistry;

        println!("   [Генерация кандидатов] Создание IndicatorRegistry...");
        let registry = IndicatorRegistry::new();
        println!("   [Генерация кандидатов] Сбор информации об индикаторах...");
        let available_indicators_vec = IndicatorInfoCollector::collect_from_registry(&registry);
        println!(
            "   [Генерация кандидатов] Найдено индикаторов: {}",
            available_indicators_vec.len()
        );

        println!("   [Генерация кандидатов] Создание StopHandlerRegistry...");
        let stop_handler_registry = StopHandlerRegistry::new();
        let stop_handler_configs = stop_handler_registry.get_all_configs();
        println!(
            "   [Генерация кандидатов] Найдено стоп-обработчиков: {}",
            stop_handler_configs.len()
        );

        let available_timeframes = self.evaluator.available_timeframes();

        println!("   [Генерация кандидатов] Использование CandidateBuilder с правилами...");
        let mut builder = CandidateBuilder::new(self.candidate_builder_config.clone());

        for i in 0..count {
            let candidate_elements = builder.build_candidate(
                &available_indicators_vec,
                &stop_handler_configs,
                &available_timeframes,
            );

            if let Some(candidate) = Self::convert_candidate_elements_to_strategy_candidate(
                candidate_elements,
                &self.discovery_config,
            ) {
                println!("\n   📋 Кандидат стратегии #{}:", i + 1);
                Self::log_strategy_details(&candidate, None);
                candidates.push(candidate);
                if (i + 1) % 5 == 0 || i == 0 {
                    println!(
                        "   [Генерация кандидатов] Сгенерировано {}/{} кандидатов",
                        i + 1,
                        count
                    );
                }
            }
        }

        println!(
            "   [Генерация кандидатов] Завершено: создано {} кандидатов стратегий",
            candidates.len()
        );
        Ok(candidates)
    }

    fn convert_candidate_elements_to_strategy_candidate(
        elements: CandidateElements,
        discovery_config: &crate::discovery::StrategyDiscoveryConfig,
    ) -> Option<StrategyCandidate> {
        use crate::discovery::types::StopHandlerInfo;

        let all_handlers: Vec<StopHandlerInfo> = elements
            .stop_handlers
            .into_iter()
            .chain(elements.take_handlers.into_iter())
            .collect();

        let (stop_handlers, take_handlers) = StrategyCandidate::split_handlers(&all_handlers);

        Some(StrategyCandidate {
            indicators: elements.indicators,
            nested_indicators: elements.nested_indicators,
            conditions: elements.entry_conditions,
            exit_conditions: elements.exit_conditions,
            stop_handlers,
            take_handlers,
            timeframes: elements.timeframes,
            config: discovery_config.clone(),
        })
    }

    fn generate_random_parameters(&self, candidate: &StrategyCandidate) -> StrategyParameterMap {
        use crate::indicators::implementations::get_optimization_range;
        use crate::risk::stops::get_optimization_range as get_stop_optimization_range;
        use crate::strategy::types::StrategyParamValue;

        let mut rng = rand::thread_rng();
        let total_params: usize = candidate
            .indicators
            .iter()
            .map(|i| i.parameters.len())
            .sum();
        let mut params = HashMap::with_capacity(total_params);

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_type_str = format!("{:?}", param.param_type);
                    let param_value = if param_type_str.contains("Boolean") {
                        StrategyParamValue::Flag(rng.gen())
                    } else {
                        let range = if Self::should_apply_volatility_constraint(
                            indicator,
                            &candidate.conditions,
                            &candidate.exit_conditions,
                            &self.candidate_builder_config,
                        ) {
                            Self::get_volatility_percentage_range(
                                &self.candidate_builder_config,
                                indicator,
                            )
                        } else {
                            get_optimization_range(&indicator.name, &param.name, &param.param_type)
                        };

                        if let Some(range) = range {
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
                    let param_key = format!("{}_{}", indicator.alias, param.name);
                    params.insert(param_key, param_value);
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
                    let param_key = format!("{}_{}", nested.indicator.alias, param.name);
                    params.insert(param_key, param_value);
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
                    } else if param.name.to_lowercase() == "percentage"
                        && condition.condition_type == "indicator_constant"
                        && indicator_name.is_some()
                    {
                        // Для volatility индикаторов: процент от цены из конфигурации
                        if let Some(ind_name) = &indicator_name {
                            if let Some(indicator) = candidate
                                .indicators
                                .iter()
                                .find(|i| i.name == *ind_name && i.indicator_type == "volatility")
                            {
                                // Используем диапазон из конфигурации для volatility
                                if let Some(range) = Self::get_volatility_percentage_range(
                                    &self.candidate_builder_config,
                                    indicator,
                                ) {
                                    let steps = ((range.end - range.start) / range.step) as usize;
                                    let step_index = rng.gen_range(0..=steps);
                                    let value = range.start + (step_index as f32 * range.step);
                                    StrategyParamValue::Number(value as f64)
                                } else {
                                    // Fallback: используем значение из constant_value если есть
                                    StrategyParamValue::Number(
                                        condition.constant_value.unwrap_or(2.0),
                                    )
                                }
                            } else {
                                // Не volatility индикатор, используем стандартный диапазон
                                use crate::indicators::types::ParameterType;
                                if let Some(range) = get_optimization_range(
                                    ind_name,
                                    &param.name,
                                    &ParameterType::Multiplier,
                                ) {
                                    let steps = ((range.end - range.start) / range.step) as usize;
                                    let step_index = rng.gen_range(0..=steps);
                                    let value = range.start + (step_index as f32 * range.step);
                                    StrategyParamValue::Number(value as f64)
                                } else {
                                    continue;
                                }
                            }
                        } else {
                            continue;
                        }
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

    fn extract_all_indicator_aliases_from_condition(condition_id: &str) -> Vec<String> {
        if condition_id.starts_with("ind_ind_") {
            let rest = condition_id.strip_prefix("ind_ind_").unwrap_or("");
            let parts: Vec<&str> = if let Some(tf_pos) = rest.find("_tf") {
                rest[..tf_pos].split('_').collect()
            } else {
                rest.split('_').collect()
            };
            if parts.len() >= 2 {
                return vec![parts[0].to_string(), parts[1].to_string()];
            }
        } else if condition_id.starts_with("entry_") {
            let rest = condition_id.strip_prefix("entry_").unwrap_or("");
            let parts: Vec<&str> = rest.split('_').collect();
            if parts.len() >= 3 {
                let last_part = parts[parts.len() - 1];
                if last_part.parse::<u32>().is_ok() {
                    return vec![parts[0].to_string(), parts[1].to_string()];
                }
            }
            if parts.len() >= 1 {
                return vec![parts[0].to_string()];
            }
        } else if condition_id.starts_with("exit_") {
            let rest = condition_id.strip_prefix("exit_").unwrap_or("");
            let parts: Vec<&str> = rest.split('_').collect();
            if parts.len() >= 3 {
                let last_part = parts[parts.len() - 1];
                if last_part.parse::<u32>().is_ok() {
                    return vec![parts[0].to_string(), parts[1].to_string()];
                }
            }
            if parts.len() >= 1 {
                return vec![parts[0].to_string()];
            }
        } else if condition_id.starts_with("ind_price_") {
            let rest = condition_id.strip_prefix("ind_price_").unwrap_or("");
            let parts: Vec<&str> = if let Some(tf_pos) = rest.find("_tf") {
                rest[..tf_pos].split('_').collect()
            } else {
                rest.split('_').collect()
            };
            if !parts.is_empty() {
                return vec![parts[0].to_string()];
            }
        } else if condition_id.starts_with("ind_const_") {
            let rest = condition_id.strip_prefix("ind_const_").unwrap_or("");
            let parts: Vec<&str> = if let Some(tf_pos) = rest.find("_tf") {
                rest[..tf_pos].split('_').collect()
            } else {
                rest.split('_').collect()
            };
            if !parts.is_empty() {
                return vec![parts[0].to_string()];
            }
        }
        Vec::new()
    }

    fn extract_indicator_alias_from_condition_id(condition_id: &str) -> Option<String> {
        let aliases = Self::extract_all_indicator_aliases_from_condition(condition_id);
        aliases.first().cloned()
    }

    fn should_apply_volatility_constraint(
        indicator: &crate::discovery::IndicatorInfo,
        conditions: &[crate::discovery::ConditionInfo],
        exit_conditions: &[crate::discovery::ConditionInfo],
        config: &CandidateBuilderConfig,
    ) -> bool {
        if indicator.indicator_type != "volatility" {
            return false;
        }

        let rules = &config.rules.indicator_parameter_rules;
        for rule in rules {
            if rule.indicator_type == "volatility" {
                if !rule.indicator_names.is_empty() {
                    if !rule.indicator_names.contains(&indicator.name) {
                        continue;
                    }
                }

                if let Some(constraint) = &rule.price_field_constraint {
                    if constraint.required_price_field == "Close" {
                        for condition in conditions.iter().chain(exit_conditions.iter()) {
                            if let Some(price_field) = &condition.price_field {
                                if price_field == "Close" {
                                    if let Some(alias) =
                                        Self::extract_indicator_alias_from_condition_id(
                                            &condition.id,
                                        )
                                    {
                                        if alias == indicator.alias {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    fn get_volatility_percentage_range(
        config: &CandidateBuilderConfig,
        indicator: &crate::discovery::IndicatorInfo,
    ) -> Option<crate::indicators::implementations::OptimizationRange> {
        let rules = &config.rules.indicator_parameter_rules;
        for rule in rules {
            if rule.indicator_type == "volatility" {
                if !rule.indicator_names.is_empty() {
                    if !rule.indicator_names.contains(&indicator.name) {
                        continue;
                    }
                }

                if let Some(constraint) = &rule.price_field_constraint {
                    if let super::candidate_builder_config::ParameterConstraint::PercentageFromPrice {
                        min_percent,
                        max_percent,
                        step,
                    } = &constraint.parameter_constraint
                    {
                        return Some(crate::indicators::implementations::OptimizationRange {
                            start: *min_percent as f32,
                            end: *max_percent as f32,
                            step: *step as f32,
                        });
                    }
                }
            }
        }
        None
    }

    fn log_strategy_details(
        candidate: &StrategyCandidate,
        parameters: Option<&StrategyParameterMap>,
    ) {
        println!("   ═══════════════════════════════════════════════════════");
        println!("   📊 ДЕТАЛИ СТРАТЕГИИ");
        println!("   ═══════════════════════════════════════════════════════");

        println!("\n   🕐 ТАЙМФРЕЙМЫ:");
        if candidate.timeframes.is_empty() {
            println!("      (нет таймфреймов)");
        } else {
            for (idx, tf) in candidate.timeframes.iter().enumerate() {
                println!("      {}. {}", idx + 1, tf.identifier());
            }
        }

        println!("\n   📈 ИНДИКАТОРЫ:");
        if candidate.indicators.is_empty() && candidate.nested_indicators.is_empty() {
            println!("      (нет индикаторов)");
        } else {
            for (idx, indicator) in candidate.indicators.iter().enumerate() {
                println!(
                    "      {}. {} ({})",
                    idx + 1,
                    indicator.name,
                    indicator.alias
                );
                if !indicator.parameters.is_empty() {
                    println!("         Параметры:");
                    for param in &indicator.parameters {
                        if let Some(params) = parameters {
                            let param_key = format!("{}_{}", indicator.alias, param.name);
                            if let Some(value) = params.get(&param_key) {
                                println!("            - {}: {:?}", param.name, value);
                            } else {
                                println!("            - {}: (не оптимизируется)", param.name);
                            }
                        } else {
                            println!(
                                "            - {}: (тип: {:?}, оптимизируемый: {})",
                                param.name, param.param_type, param.optimizable
                            );
                        }
                    }
                }
            }

            if !candidate.nested_indicators.is_empty() {
                println!("\n      Вложенные индикаторы:");
                for (idx, nested) in candidate.nested_indicators.iter().enumerate() {
                    println!(
                        "         {}. {} ({}) [вход: {}]",
                        idx + 1,
                        nested.indicator.name,
                        nested.indicator.alias,
                        nested.input_indicator_alias
                    );
                    if !nested.indicator.parameters.is_empty() {
                        println!("            Параметры:");
                        for param in &nested.indicator.parameters {
                            if let Some(params) = parameters {
                                let param_key =
                                    format!("{}_{}", nested.indicator.alias, param.name);
                                if let Some(value) = params.get(&param_key) {
                                    println!("               - {}: {:?}", param.name, value);
                                } else {
                                    println!(
                                        "               - {}: (не оптимизируется)",
                                        param.name
                                    );
                                }
                            } else {
                                println!(
                                    "               - {}: (тип: {:?}, оптимизируемый: {})",
                                    param.name, param.param_type, param.optimizable
                                );
                            }
                        }
                    }
                }
            }
        }

        println!("\n   🚪 УСЛОВИЯ ВХОДА (Entry Rules):");
        if candidate.conditions.is_empty() {
            println!("      (нет условий входа)");
        } else {
            for (idx, condition) in candidate.conditions.iter().enumerate() {
                println!("      {}. {} ({})", idx + 1, condition.name, condition.id);
                println!("         Тип: {}", condition.condition_type);
                println!("         Оператор: {:?}", condition.operator);
                if let Some(tf) = &condition.primary_timeframe {
                    println!("         Таймфрейм: {}", tf.identifier());
                }
                if let Some(price_field) = &condition.price_field {
                    println!("         Поле цены: {}", price_field);
                }
                if let Some(const_val) = condition.constant_value {
                    println!("         Константа: {}", const_val);
                }
                if !condition.optimization_params.is_empty() {
                    println!("         Параметры оптимизации:");
                    for param in &condition.optimization_params {
                        if let Some(params) = parameters {
                            let param_key = format!("condition_{}_{}", condition.id, param.name);
                            if let Some(value) = params.get(&param_key) {
                                println!("            - {}: {:?}", param.name, value);
                            } else {
                                println!("            - {}: (не оптимизируется)", param.name);
                            }
                        } else {
                            println!(
                                "            - {}: (оптимизируемый: {})",
                                param.name, param.optimizable
                            );
                        }
                    }
                }
            }
        }

        println!("\n   🚪 УСЛОВИЯ ВЫХОДА (Exit Rules):");
        if candidate.exit_conditions.is_empty() {
            println!("      (нет условий выхода)");
        } else {
            for (idx, condition) in candidate.exit_conditions.iter().enumerate() {
                println!("      {}. {} ({})", idx + 1, condition.name, condition.id);
                println!("         Тип: {}", condition.condition_type);
                println!("         Оператор: {:?}", condition.operator);
                if let Some(tf) = &condition.primary_timeframe {
                    println!("         Таймфрейм: {}", tf.identifier());
                }
                if let Some(price_field) = &condition.price_field {
                    println!("         Поле цены: {}", price_field);
                }
                if let Some(const_val) = condition.constant_value {
                    println!("         Константа: {}", const_val);
                }
                if !condition.optimization_params.is_empty() {
                    println!("         Параметры оптимизации:");
                    for param in &condition.optimization_params {
                        if let Some(params) = parameters {
                            let param_key = format!("condition_{}_{}", condition.id, param.name);
                            if let Some(value) = params.get(&param_key) {
                                println!("            - {}: {:?}", param.name, value);
                            } else {
                                println!("            - {}: (не оптимизируется)", param.name);
                            }
                        } else {
                            println!(
                                "            - {}: (оптимизируемый: {})",
                                param.name, param.optimizable
                            );
                        }
                    }
                }
            }
        }

        println!("\n   🛑 STOP HANDLERS:");
        if candidate.stop_handlers.is_empty() {
            println!("      (нет стоп-обработчиков)");
        } else {
            for (idx, stop) in candidate.stop_handlers.iter().enumerate() {
                println!("      {}. {} ({})", idx + 1, stop.name, stop.handler_name);
                println!("         Тип: {}", stop.stop_type);
                println!("         Приоритет: {}", stop.priority);
                if !stop.optimization_params.is_empty() {
                    println!("         Параметры оптимизации:");
                    for param in &stop.optimization_params {
                        if let Some(params) = parameters {
                            let param_key = format!("stop_{}_{}", stop.name, param.name);
                            if let Some(value) = params.get(&param_key) {
                                println!("            - {}: {:?}", param.name, value);
                            } else {
                                println!("            - {}: (не оптимизируется)", param.name);
                            }
                        } else {
                            println!(
                                "            - {}: (оптимизируемый: {})",
                                param.name, param.optimizable
                            );
                        }
                    }
                }
            }
        }

        println!("\n   🎯 TAKE HANDLERS:");
        if candidate.take_handlers.is_empty() {
            println!("      (нет тейк-обработчиков)");
        } else {
            for (idx, take) in candidate.take_handlers.iter().enumerate() {
                println!("      {}. {} ({})", idx + 1, take.name, take.handler_name);
                println!("         Тип: {}", take.stop_type);
                println!("         Приоритет: {}", take.priority);
                if !take.optimization_params.is_empty() {
                    println!("         Параметры оптимизации:");
                    for param in &take.optimization_params {
                        if let Some(params) = parameters {
                            let param_key = format!("take_{}_{}", take.name, param.name);
                            if let Some(value) = params.get(&param_key) {
                                println!("            - {}: {:?}", param.name, value);
                            } else {
                                println!("            - {}: (не оптимизируется)", param.name);
                            }
                        } else {
                            println!(
                                "            - {}: (оптимизируемый: {})",
                                param.name, param.optimizable
                            );
                        }
                    }
                }
            }
        }

        println!("   ═══════════════════════════════════════════════════════\n");
    }

    /// Отбор особей с поддержанием разнообразия стратегий (round-robin)
    /// Группирует особи по стратегиям, сортирует каждую группу по fitness,
    /// затем по очереди выбирает по одной особи от каждой стратегии
    fn select_with_diversity(
        individuals: Vec<GeneticIndividual>,
        target_size: usize,
    ) -> Vec<GeneticIndividual> {
        use std::collections::HashMap;

        // Группируем особи по стратегиям
        let mut strategy_groups: HashMap<String, Vec<GeneticIndividual>> = HashMap::new();

        for individual in individuals {
            // Создаем уникальный идентификатор стратегии на основе её структуры
            let strategy_id = if let Some(ref candidate) = individual.strategy.candidate {
                Self::get_strategy_signature(candidate)
            } else {
                // Если нет кандидата, используем хеш параметров как идентификатор
                format!("no_candidate_{:?}", individual.strategy.parameters)
            };

            strategy_groups
                .entry(strategy_id)
                .or_insert_with(Vec::new)
                .push(individual);
        }

        // Сортируем каждую группу по fitness (от лучшего к худшему)
        for group in strategy_groups.values_mut() {
            group.sort_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_b
                    .partial_cmp(&fitness_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Round-robin отбор: по очереди берем по одной особи от каждой стратегии
        let mut selected = Vec::with_capacity(target_size);
        let mut strategy_indices: HashMap<String, usize> = HashMap::new();

        // Инициализируем индексы для каждой стратегии
        for strategy_id in strategy_groups.keys() {
            strategy_indices.insert(strategy_id.clone(), 0);
        }

        while selected.len() < target_size {
            let mut found_any = false;

            // Проходим по всем стратегиям в каждом раунде
            for (strategy_id, group) in &strategy_groups {
                if selected.len() >= target_size {
                    break;
                }

                let index = strategy_indices.get(strategy_id).copied().unwrap_or(0);

                // Если в этой стратегии еще есть особи
                if index < group.len() {
                    selected.push(group[index].clone());
                    strategy_indices.insert(strategy_id.clone(), index + 1);
                    found_any = true;
                }
            }

            // Если не нашли ни одной особи в этом раунде, значит все стратегии исчерпаны
            if !found_any {
                break;
            }
        }

        println!(
            "   [Отбор с разнообразием] Выбрано {} особей из {} уникальных стратегий (round-robin)",
            selected.len(),
            strategy_groups.len()
        );

        selected
    }

    /// Создает уникальный идентификатор стратегии на основе её структуры
    fn get_strategy_signature(candidate: &StrategyCandidate) -> String {
        use std::collections::BTreeSet;

        // Сортируем индикаторы по alias для стабильности
        let indicator_aliases: BTreeSet<String> = candidate
            .indicators
            .iter()
            .map(|ind| ind.alias.clone())
            .collect();

        let nested_aliases: BTreeSet<String> = candidate
            .nested_indicators
            .iter()
            .map(|nested| {
                format!(
                    "{}->{}",
                    nested.input_indicator_alias, nested.indicator.alias
                )
            })
            .collect();

        let condition_ids: BTreeSet<String> = candidate
            .conditions
            .iter()
            .map(|cond| format!("{}:{}:{:?}", cond.condition_type, cond.id, cond.operator))
            .collect();

        let exit_condition_ids: BTreeSet<String> = candidate
            .exit_conditions
            .iter()
            .map(|cond| format!("{}:{}:{:?}", cond.condition_type, cond.id, cond.operator))
            .collect();

        let stop_handler_names: BTreeSet<String> = candidate
            .stop_handlers
            .iter()
            .map(|h| h.handler_name.clone())
            .collect();

        let take_handler_names: BTreeSet<String> = candidate
            .take_handlers
            .iter()
            .map(|h| h.handler_name.clone())
            .collect();

        let timeframe_strings: BTreeSet<String> = candidate
            .timeframes
            .iter()
            .map(|tf| format!("{:?}", tf))
            .collect();

        format!(
            "indicators:{:?}|nested:{:?}|conditions:{:?}|exit:{:?}|stops:{:?}|takes:{:?}|timeframes:{:?}",
            indicator_aliases,
            nested_aliases,
            condition_ids,
            exit_condition_ids,
            stop_handler_names,
            take_handler_names,
            timeframe_strings
        )
    }
}
