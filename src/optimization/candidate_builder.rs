use crate::data_model::types::TimeFrame;
use crate::discovery::types::{
    ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};
use crate::strategy::types::ConditionOperator;
use rand::seq::SliceRandom;
use rand::Rng;

use super::candidate_builder_config::{
    CandidateBuilderConfig, ElementConstraints, ElementProbabilities, ElementSelector,
};

pub struct CandidateBuilder {
    config: CandidateBuilderConfig,
    rng: rand::rngs::ThreadRng,
}

impl CandidateBuilder {
    pub fn new(config: CandidateBuilderConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    pub fn build_candidate(
        &mut self,
        available_indicators: &[IndicatorInfo],
        available_stop_handlers: &[StopHandlerConfig],
        available_timeframes: &[TimeFrame],
    ) -> CandidateElements {
        let constraints = self.config.constraints.clone();
        let probabilities = self.config.probabilities.clone();

        // Инициализация пустого кандидата
        let mut candidate = CandidateElements {
            indicators: Vec::new(),
            nested_indicators: Vec::new(),
            entry_conditions: Vec::new(),
            exit_conditions: Vec::new(),
            stop_handlers: Vec::new(),
            take_handlers: Vec::new(),
            timeframes: Vec::new(),
        };

        // ФАЗА 1: Обязательная базовая сборка
        self.build_phase_1(
            &mut candidate,
            available_indicators,
            available_stop_handlers,
            available_timeframes,
            &constraints,
            &probabilities,
        );

        // Применяем правила после первой фазы (например, StopLossPct -> TakeProfitPct)
        self.apply_rules(&mut candidate, available_stop_handlers);

        // ФАЗЫ 2+: Дополнительные фазы с вероятностью
        let mut phase = 2;
        while self.should_add(probabilities.phases.continue_building) {
            if !self.build_additional_phase(
                &mut candidate,
                available_indicators,
                available_stop_handlers,
                available_timeframes,
                &constraints,
                &probabilities,
                phase,
            ) {
                // Если не можем добавить больше элементов (достигли лимитов), прекращаем
                break;
            }
            phase += 1;
        }

        // Финальные проверки и гарантии
        self.ensure_all_indicators_used(
            &candidate.indicators,
            &candidate.nested_indicators,
            &mut candidate.entry_conditions,
            &candidate.exit_conditions,
            &constraints,
        );

        self.ensure_higher_timeframes_used(
            &mut candidate,
            available_timeframes,
            &probabilities.conditions,
            &constraints,
        );

        self.ensure_minimum_requirements(&mut candidate, &constraints, available_stop_handlers);

        candidate
    }

    fn build_phase_1(
        &mut self,
        candidate: &mut CandidateElements,
        available_indicators: &[IndicatorInfo],
        available_stop_handlers: &[StopHandlerConfig],
        available_timeframes: &[TimeFrame],
        constraints: &ElementConstraints,
        probabilities: &ElementProbabilities,
    ) {
        // 1. Добавляем базовый таймфрейм
        if !available_timeframes.is_empty() {
            candidate.timeframes.push(available_timeframes[0].clone());
        }

        // 2. Добавляем случайный индикатор (в первой фазе нельзя добавлять volatility и volume)
        let exclude_aliases: Vec<String> = candidate
            .indicators
            .iter()
            .map(|i| i.alias.clone())
            .collect();
        if let Some(indicator) = self.select_single_indicator(
            available_indicators,
            probabilities,
            &exclude_aliases,
            true, // is_phase_1 = true
        ) {
            candidate.indicators.push(indicator);
        }

        // 3. Добавляем случайное условие входа
        if !candidate.indicators.is_empty()
            && candidate.entry_conditions.len() < constraints.max_entry_conditions
        {
            if let Some(condition) = self.build_condition(
                &candidate.indicators,
                &candidate.nested_indicators,
                &probabilities.conditions,
                true,
            ) {
                if !Self::is_duplicate_condition(&condition, &candidate.entry_conditions) {
                    candidate.entry_conditions.push(condition);
                }
            }
        }

        // 4. Добавляем случайный стоп (только один) - ОБЯЗАТЕЛЬНО в первой фазе
        if candidate.stop_handlers.is_empty() && constraints.min_stop_handlers > 0 {
            if available_stop_handlers.is_empty() {
                eprintln!("      ⚠️  ВНИМАНИЕ: Нет доступных stop handlers для добавления!");
            } else if let Some(stop) = self.select_single_stop_handler_required(
                available_stop_handlers,
                &probabilities.stop_handlers,
            ) {
                candidate.stop_handlers.push(stop);
            } else {
                eprintln!(
                    "      ⚠️  ВНИМАНИЕ: Не удалось выбрать stop handler из {} доступных",
                    available_stop_handlers.len()
                );
            }
        }
    }

    fn build_additional_phase(
        &mut self,
        candidate: &mut CandidateElements,
        available_indicators: &[IndicatorInfo],
        _available_stop_handlers: &[StopHandlerConfig],
        available_timeframes: &[TimeFrame],
        constraints: &ElementConstraints,
        probabilities: &ElementProbabilities,
        _phase: usize,
    ) -> bool {
        // Проверяем лимиты перед добавлением
        if candidate.indicators.len() >= constraints.max_indicators
            && candidate.entry_conditions.len() >= constraints.max_entry_conditions
            && candidate.exit_conditions.len() >= constraints.max_exit_conditions
            && candidate.timeframes.len() >= constraints.max_timeframes
        {
            return false;
        }

        // Решаем: добавляем entry или exit условие?
        let is_entry = if candidate.exit_conditions.len() >= constraints.max_exit_conditions {
            true // Только entry, если exit лимит достигнут
        } else if candidate.entry_conditions.len() >= constraints.max_entry_conditions {
            false // Только exit, если entry лимит достигнут
        } else {
            // Выбираем на основе вероятности
            !self.should_add(probabilities.phases.add_exit_rules)
        };

        // Проверяем лимиты для выбранного типа условия
        if is_entry && candidate.entry_conditions.len() >= constraints.max_entry_conditions {
            return false;
        }
        if !is_entry && candidate.exit_conditions.len() >= constraints.max_exit_conditions {
            return false;
        }

        // Если нет индикаторов, не можем создать условие
        if candidate.indicators.is_empty() && candidate.nested_indicators.is_empty() {
            return false;
        }

        // Опционально: добавляем higher timeframe (если нужно и есть вероятность)
        let mut selected_timeframe: Option<TimeFrame> = None;
        if candidate.timeframes.len() < constraints.max_timeframes
            && available_timeframes.len() > 1
            && self.should_add(probabilities.timeframes.use_higher_timeframe)
        {
            let higher_timeframes: Vec<&TimeFrame> = available_timeframes
                .iter()
                .skip(1)
                .filter(|tf| !candidate.timeframes.contains(tf))
                .collect();

            if let Some(higher_tf) = higher_timeframes.choose(&mut self.rng) {
                candidate.timeframes.push((*higher_tf).clone());
                selected_timeframe = Some((*higher_tf).clone());
            }
        }

        // Опционально: добавляем новый индикатор (если нужно)
        // Используем вероятность из другой логики или просто проверяем лимиты
        if candidate.indicators.len() < constraints.max_indicators && self.should_add(0.5)
        // 50% шанс добавить новый индикатор
        {
            let exclude_aliases: Vec<String> = candidate
                .indicators
                .iter()
                .map(|i| i.alias.clone())
                .collect();
            if let Some(indicator) = self.select_single_indicator(
                available_indicators,
                probabilities,
                &exclude_aliases,
                false, // is_phase_1 = false (это дополнительная фаза)
            ) {
                candidate.indicators.push(indicator);
            }
        }

        // Строим ОДНО условие с учетом всех вероятностей
        let condition = if let Some(timeframe) = selected_timeframe {
            // Если выбран higher timeframe, используем его для условия
            // Выбираем случайный индикатор для условия
            let all_indicators: Vec<&IndicatorInfo> = candidate
                .indicators
                .iter()
                .chain(candidate.nested_indicators.iter().map(|n| &n.indicator))
                .collect();

            if let Some(indicator) = all_indicators.choose(&mut self.rng) {
                self.build_condition_with_timeframe(indicator, is_entry, Some(timeframe))
            } else {
                None
            }
        } else {
            // Обычное условие без специального таймфрейма
            self.build_condition(
                &candidate.indicators,
                &candidate.nested_indicators,
                &probabilities.conditions,
                is_entry,
            )
        };

        // Добавляем условие (только одно!)
        if let Some(condition) = condition {
            if is_entry {
                if !Self::is_duplicate_condition(&condition, &candidate.entry_conditions) {
                    candidate.entry_conditions.push(condition);
                    return true;
                }
            } else {
                if !Self::is_duplicate_condition(&condition, &candidate.exit_conditions) {
                    candidate.exit_conditions.push(condition);
                    return true;
                }
            }
        }

        false
    }

    fn apply_rules(
        &mut self,
        candidate: &mut CandidateElements,
        available_stop_handlers: &[StopHandlerConfig],
    ) {
        let rules = self.config.rules.clone();

        for dependency in &rules.dependencies {
            if self.matches_selector(&dependency.trigger, candidate) {
                if !self.matches_selector(&dependency.required, candidate) {
                    if dependency.strict {
                        self.add_required_element(
                            &dependency.required,
                            candidate,
                            available_stop_handlers,
                        );
                    }
                }
            }
        }

        for exclusion in &rules.exclusions {
            if self.matches_selector(&exclusion.element, candidate) {
                self.remove_excluded_element(&exclusion.excluded, candidate);
            }
        }

        for conditional in &rules.conditions {
            if self.evaluate_condition(&conditional.condition, candidate) {
                self.apply_action(&conditional.action, candidate, available_stop_handlers);
            }
        }
    }

    fn matches_selector(&self, selector: &ElementSelector, candidate: &CandidateElements) -> bool {
        match selector {
            ElementSelector::StopHandler { name } => candidate
                .stop_handlers
                .iter()
                .any(|h| &h.handler_name == name),
            ElementSelector::TakeHandler { name } => candidate
                .take_handlers
                .iter()
                .any(|h| &h.handler_name == name),
            ElementSelector::Indicator { name } => {
                candidate.indicators.iter().any(|i| &i.name == name)
                    || candidate
                        .nested_indicators
                        .iter()
                        .any(|n| &n.indicator.name == name)
            }
            ElementSelector::Condition { condition_type } => candidate
                .entry_conditions
                .iter()
                .chain(candidate.exit_conditions.iter())
                .any(|c| &c.condition_type == condition_type),
            ElementSelector::AnyStopHandler => !candidate.stop_handlers.is_empty(),
            ElementSelector::AnyTakeHandler => !candidate.take_handlers.is_empty(),
            ElementSelector::AnyIndicator => {
                !candidate.indicators.is_empty() || !candidate.nested_indicators.is_empty()
            }
            ElementSelector::AnyCondition => {
                !candidate.entry_conditions.is_empty() || !candidate.exit_conditions.is_empty()
            }
            _ => false,
        }
    }

    fn evaluate_condition(
        &self,
        condition: &super::candidate_builder_config::RuleCondition,
        candidate: &CandidateElements,
    ) -> bool {
        match condition {
            super::candidate_builder_config::RuleCondition::And { conditions } => conditions
                .iter()
                .all(|c| self.matches_selector(c, candidate)),
            super::candidate_builder_config::RuleCondition::Or { conditions } => conditions
                .iter()
                .any(|c| self.matches_selector(c, candidate)),
            super::candidate_builder_config::RuleCondition::Not { condition } => {
                !self.matches_selector(condition, candidate)
            }
            super::candidate_builder_config::RuleCondition::Element { element } => {
                self.matches_selector(element, candidate)
            }
        }
    }

    fn apply_action(
        &mut self,
        action: &super::candidate_builder_config::RuleAction,
        candidate: &mut CandidateElements,
        available_stop_handlers: &[StopHandlerConfig],
    ) {
        match action {
            super::candidate_builder_config::RuleAction::Require { element, strict } => {
                if !self.matches_selector(element, candidate) {
                    if *strict {
                        self.add_required_element(element, candidate, available_stop_handlers);
                    }
                }
            }
            super::candidate_builder_config::RuleAction::Exclude { element } => {
                self.remove_excluded_element(element, candidate);
            }
            super::candidate_builder_config::RuleAction::SetProbability { .. } => {}
        }
    }

    fn add_required_element(
        &mut self,
        selector: &ElementSelector,
        candidate: &mut CandidateElements,
        available_stop_handlers: &[StopHandlerConfig],
    ) {
        match selector {
            ElementSelector::TakeHandler { name } => {
                // Проверяем лимит (max_take_handlers обычно = 1)
                if candidate.take_handlers.len() >= self.config.constraints.max_take_handlers {
                    return;
                }

                if candidate
                    .take_handlers
                    .iter()
                    .any(|h| &h.handler_name == name)
                {
                    return;
                }

                if let Some(config) = available_stop_handlers
                    .iter()
                    .find(|c| c.handler_name == *name && c.stop_type == "take_profit")
                {
                    candidate.take_handlers.push(StopHandlerInfo {
                        id: format!("take_{}", self.rng.gen::<u32>()),
                        name: config.handler_name.clone(),
                        handler_name: config.handler_name.clone(),
                        stop_type: config.stop_type.clone(),
                        optimization_params: Vec::new(),
                        priority: config.priority,
                    });
                    return;
                }

                candidate.take_handlers.push(StopHandlerInfo {
                    id: format!("take_{}", self.rng.gen::<u32>()),
                    name: name.clone(),
                    handler_name: name.clone(),
                    stop_type: "take_profit".to_string(),
                    optimization_params: Vec::new(),
                    priority: 100,
                });
            }
            ElementSelector::StopHandler { name } => {
                if candidate
                    .stop_handlers
                    .iter()
                    .any(|h| &h.handler_name == name)
                {
                    return;
                }

                if let Some(config) = available_stop_handlers
                    .iter()
                    .find(|c| c.handler_name == *name && c.stop_type == "stop_loss")
                {
                    candidate.stop_handlers.push(StopHandlerInfo {
                        id: format!("stop_{}", self.rng.gen::<u32>()),
                        name: config.handler_name.clone(),
                        handler_name: config.handler_name.clone(),
                        stop_type: config.stop_type.clone(),
                        optimization_params: Vec::new(),
                        priority: config.priority,
                    });
                    return;
                }

                candidate.stop_handlers.push(StopHandlerInfo {
                    id: format!("stop_{}", self.rng.gen::<u32>()),
                    name: name.clone(),
                    handler_name: name.clone(),
                    stop_type: "stop_loss".to_string(),
                    optimization_params: Vec::new(),
                    priority: 100,
                });
            }
            _ => {}
        }
    }

    fn remove_excluded_element(
        &mut self,
        selector: &ElementSelector,
        candidate: &mut CandidateElements,
    ) {
        match selector {
            ElementSelector::TakeHandler { name } => {
                candidate.take_handlers.retain(|h| &h.handler_name != name);
            }
            ElementSelector::StopHandler { name } => {
                candidate.stop_handlers.retain(|h| &h.handler_name != name);
            }
            ElementSelector::Indicator { name } => {
                candidate.indicators.retain(|i| &i.name != name);
                candidate
                    .nested_indicators
                    .retain(|n| &n.indicator.name != name);
            }
            _ => {}
        }
    }

    fn select_single_indicator(
        &mut self,
        available: &[IndicatorInfo],
        probabilities: &ElementProbabilities,
        exclude_aliases: &[String],
        is_phase_1: bool,
    ) -> Option<IndicatorInfo> {
        let exclude_set: std::collections::HashSet<&str> =
            exclude_aliases.iter().map(|s| s.as_str()).collect();

        // Получаем список исключенных индикаторов до создания замыкания
        let excluded_indicators: Vec<String> = self.config.rules.excluded_indicators.clone();
        let excluded_indicators_set: std::collections::HashSet<&str> =
            excluded_indicators.iter().map(|s| s.as_str()).collect();

        let available_filtered: Vec<&IndicatorInfo> = available
            .iter()
            .filter(|indicator| !exclude_set.contains(indicator.alias.as_str()))
            .filter(|indicator| {
                // Исключаем индикаторы из списка excluded_indicators
                if excluded_indicators_set.contains(indicator.name.as_str()) {
                    return false;
                }
                // В первой фазе нельзя добавлять volatility и volume индикаторы
                if is_phase_1 {
                    if indicator.indicator_type == "volatility"
                        || indicator.indicator_type == "volume"
                    {
                        return false;
                    }
                }
                let weight = match indicator.indicator_type.as_str() {
                    "trend" => probabilities.indicators.add_trend_indicator,
                    "oscillator" => probabilities.indicators.add_oscillator_indicator,
                    "volume" => probabilities.indicators.add_volume_indicator,
                    "volatility" => probabilities.indicators.add_volatility_indicator,
                    "channel" => probabilities.indicators.add_channel_indicator,
                    _ => probabilities.indicators.add_base_indicator,
                };
                self.should_add(weight)
            })
            .collect();

        available_filtered
            .choose(&mut self.rng)
            .map(|ind| (*ind).clone())
    }

    fn select_single_stop_handler_required(
        &mut self,
        available: &[StopHandlerConfig],
        _probabilities: &super::candidate_builder_config::StopHandlerProbabilities,
    ) -> Option<StopHandlerInfo> {
        let stop_loss_configs: Vec<&StopHandlerConfig> = available
            .iter()
            .filter(|c| c.stop_type == "stop_loss")
            .collect();

        if stop_loss_configs.is_empty() {
            return None;
        }

        stop_loss_configs
            .choose(&mut self.rng)
            .map(|config| StopHandlerInfo {
                id: format!("stop_{}", self.rng.gen::<u32>()),
                name: config.handler_name.clone(),
                handler_name: config.handler_name.clone(),
                stop_type: config.stop_type.clone(),
                optimization_params: Vec::new(),
                priority: config.priority,
            })
    }

    fn build_condition_with_timeframe(
        &mut self,
        indicator: &IndicatorInfo,
        is_entry: bool,
        timeframe: Option<TimeFrame>,
    ) -> Option<ConditionInfo> {
        let operator = if self.rng.gen_bool(0.5) {
            ConditionOperator::GreaterThan
        } else {
            ConditionOperator::LessThan
        };

        let condition_id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            indicator.alias,
            self.rng.gen::<u32>()
        );

        let price_field = if self.config.condition_config.price_fields.len() == 1 {
            self.config.condition_config.price_fields[0].clone()
        } else {
            self.config
                .condition_config
                .price_fields
                .choose(&mut self.rng)
                .cloned()
                .unwrap_or_else(|| "Close".to_string())
        };

        Some(ConditionInfo {
            id: condition_id,
            name: format!("{} {:?} {}", indicator.name, operator, "target"),
            operator,
            condition_type: "indicator_price".to_string(),
            optimization_params: Vec::new(),
            constant_value: None,
            primary_timeframe: timeframe,
            secondary_timeframe: None,
            price_field: Some(price_field),
        })
    }

    fn ensure_all_indicators_used(
        &mut self,
        indicators: &[IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        entry_conditions: &mut Vec<ConditionInfo>,
        exit_conditions: &[ConditionInfo],
        constraints: &ElementConstraints,
    ) {
        let all_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .chain(nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        let mut used_indicators = std::collections::HashSet::new();

        for condition in entry_conditions.iter().chain(exit_conditions.iter()) {
            if let Some(alias) = Self::extract_indicator_alias_from_condition_id(&condition.id) {
                if let Some(indicator) = all_indicators.iter().find(|i| i.alias == alias) {
                    used_indicators.insert(indicator.alias.clone());
                }
            }
        }

        for indicator in &all_indicators {
            if !used_indicators.contains(&indicator.alias) {
                if entry_conditions.len() >= constraints.max_entry_conditions {
                    break;
                }
                let condition = self.build_condition_simple(indicator, true);
                if let Some(cond) = condition {
                    if !Self::is_duplicate_condition(&cond, entry_conditions) {
                        entry_conditions.push(cond);
                        used_indicators.insert(indicator.alias.clone());
                    }
                }
            }
        }
    }

    fn build_condition_simple(
        &mut self,
        indicator: &IndicatorInfo,
        is_entry: bool,
    ) -> Option<ConditionInfo> {
        self.build_condition_simple_with_timeframe(indicator, is_entry, None)
    }

    fn build_condition_simple_with_timeframe(
        &mut self,
        indicator: &IndicatorInfo,
        is_entry: bool,
        timeframe: Option<TimeFrame>,
    ) -> Option<ConditionInfo> {
        let operator = if self.rng.gen_bool(0.5) {
            ConditionOperator::GreaterThan
        } else {
            ConditionOperator::LessThan
        };

        let condition_id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            indicator.alias,
            self.rng.gen::<u32>()
        );

        // Осцилляторы и volatility индикаторы ВСЕГДА используют indicator_constant
        let (condition_type, condition_name, constant_value, price_field, optimization_params) =
            if indicator.indicator_type == "oscillator" {
                let const_val = if indicator.name == "RSI" {
                    if operator == ConditionOperator::GreaterThan {
                        self.rng.gen_range(70.0..=90.0)
                    } else {
                        self.rng.gen_range(10.0..=30.0)
                    }
                } else if indicator.name == "Stochastic" {
                    if operator == ConditionOperator::GreaterThan {
                        self.rng.gen_range(80.0..=95.0)
                    } else {
                        self.rng.gen_range(5.0..=20.0)
                    }
                } else {
                    self.rng.gen_range(0.0..=100.0)
                };
                (
                    "indicator_constant".to_string(),
                    format!("{} {:?} {:.1}", indicator.name, operator, const_val),
                    Some(const_val),
                    None,
                    Vec::new(),
                )
            } else if indicator.indicator_type == "volatility" {
                // Volatility индикаторы: процент от цены (0.2% - 10.0%)
                let rules = &self.config.rules.indicator_parameter_rules;
                let mut percentage_range = (0.2, 10.0, 0.1); // По умолчанию

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
                            } = &constraint.parameter_constraint {
                                percentage_range = (*min_percent, *max_percent, *step);
                                break;
                            }
                        }
                    }
                }

                let steps =
                    ((percentage_range.1 - percentage_range.0) / percentage_range.2) as usize;
                let step_index = self.rng.gen_range(0..=steps);
                let const_val = percentage_range.0 + (step_index as f64 * percentage_range.2);

                (
                    "indicator_constant".to_string(),
                    format!(
                        "{} {:?} Close * {:.2}%",
                        indicator.name, operator, const_val
                    ),
                    Some(const_val),
                    None,
                    vec![crate::discovery::ConditionParamInfo {
                        name: "percentage".to_string(),
                        optimizable: true,
                        global_param_name: None,
                    }],
                )
            } else {
                let price_field = if self.config.condition_config.price_fields.len() == 1 {
                    self.config.condition_config.price_fields[0].clone()
                } else {
                    self.config
                        .condition_config
                        .price_fields
                        .choose(&mut self.rng)
                        .cloned()
                        .unwrap_or_else(|| "Close".to_string())
                };

                let probabilities = &self.config.probabilities.conditions;
                let (optimization_params, constant_value) =
                    if self.should_add(probabilities.use_percent_condition) {
                        let percent_value = self.rng.gen_range(0.1..=5.0);
                        (
                            vec![crate::discovery::ConditionParamInfo {
                                name: "percentage".to_string(),
                                optimizable: true,
                                global_param_name: None,
                            }],
                            Some(percent_value),
                        )
                    } else {
                        (Vec::new(), None)
                    };

                (
                    "indicator_price".to_string(),
                    if let Some(percent) = constant_value {
                        format!(
                            "{} {:?} {} на {:.2}%",
                            indicator.name, operator, "target", percent
                        )
                    } else {
                        format!("{} {:?} {}", indicator.name, operator, "target")
                    },
                    constant_value,
                    Some(price_field),
                    optimization_params,
                )
            };

        Some(ConditionInfo {
            id: condition_id,
            name: condition_name,
            operator,
            condition_type,
            optimization_params,
            constant_value,
            primary_timeframe: timeframe,
            secondary_timeframe: None,
            price_field,
        })
    }

    fn ensure_higher_timeframes_used(
        &mut self,
        candidate: &mut CandidateElements,
        available_timeframes: &[TimeFrame],
        _probabilities: &super::candidate_builder_config::ConditionProbabilities,
        constraints: &ElementConstraints,
    ) {
        if available_timeframes.is_empty() || candidate.timeframes.is_empty() {
            return;
        }

        let base_timeframe = &available_timeframes[0];
        let higher_timeframes: Vec<&TimeFrame> = candidate
            .timeframes
            .iter()
            .filter(|tf| *tf != base_timeframe)
            .collect();

        if higher_timeframes.is_empty() {
            return;
        }

        let all_indicators: Vec<&IndicatorInfo> = candidate
            .indicators
            .iter()
            .chain(candidate.nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        if all_indicators.is_empty() {
            return;
        }

        for higher_tf in higher_timeframes {
            if candidate.entry_conditions.len() >= constraints.max_entry_conditions {
                break;
            }

            let is_used = candidate
                .entry_conditions
                .iter()
                .chain(candidate.exit_conditions.iter())
                .any(|cond| {
                    cond.primary_timeframe.as_ref() == Some(higher_tf)
                        || cond.secondary_timeframe.as_ref() == Some(higher_tf)
                });

            if !is_used {
                // Выбираем случайный индикатор из доступных
                if let Some(indicator) = all_indicators.choose(&mut self.rng) {
                    // Создаем условие, где индикатор будет построен по higher timeframe
                    // (через primary_timeframe)
                    let condition = self.build_condition_simple_with_timeframe(
                        indicator,
                        true,
                        Some(higher_tf.clone()), // Индикатор строится по этому higher timeframe
                    );
                    if let Some(cond) = condition {
                        if !Self::is_duplicate_condition(&cond, &candidate.entry_conditions) {
                            candidate.entry_conditions.push(cond);
                        }
                    }
                }
            }
        }
    }

    fn extract_indicator_alias_from_condition_id(condition_id: &str) -> Option<String> {
        if condition_id.starts_with("entry_") {
            let rest = condition_id.strip_prefix("entry_")?;
            let parts: Vec<&str> = rest.split('_').collect();
            if !parts.is_empty() {
                return Some(parts[0].to_string());
            }
        } else if condition_id.starts_with("exit_") {
            let rest = condition_id.strip_prefix("exit_")?;
            let parts: Vec<&str> = rest.split('_').collect();
            if !parts.is_empty() {
                return Some(parts[0].to_string());
            }
        } else if condition_id.starts_with("ind_price_") {
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

    fn build_condition(
        &mut self,
        indicators: &[IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        probabilities: &super::candidate_builder_config::ConditionProbabilities,
        is_entry: bool,
    ) -> Option<ConditionInfo> {
        let all_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .chain(nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        if all_indicators.is_empty() {
            return None;
        }

        let Some(primary_indicator) = all_indicators.choose(&mut self.rng) else {
            return None;
        };

        // Правила выбора типа условия:
        // 1. Осцилляторы ВСЕГДА работают только с константами (indicator_constant),
        //    КРОМЕ случаев когда по осцилляторам построены другие индикаторы
        // 2. Трендовые и канальные индикаторы НИКОГДА не могут участвовать в условиях с константами
        //    Они могут создавать только indicator_price или indicator_indicator условия
        // 3. Volatility индикаторы (WATR, ATR) могут создавать только indicator_price условия
        //    с процентом от цены Close (0.2% - 10%)
        // Проверяем, построен ли индикатор по осциллятору
        let all_indicators_for_check: Vec<&IndicatorInfo> = indicators
            .iter()
            .chain(nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        let is_built_on_oscillator = nested_indicators
            .iter()
            .find(|n| n.indicator.alias == primary_indicator.alias)
            .and_then(|nested| {
                all_indicators_for_check
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                    .map(|input| input.indicator_type == "oscillator")
            })
            .unwrap_or(false);

        // 4. Трендовые условия (RisingTrend/FallingTrend) могут создаваться для любых индикаторов
        let condition_type = if primary_indicator.indicator_type == "oscillator"
            && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
        {
            "indicator_constant"
        } else if primary_indicator.indicator_type == "volatility" {
            // Volatility индикаторы создают indicator_constant условия с процентом от цены
            // (аналогично осцилляторам, но с процентом вместо абсолютного значения)
            "indicator_constant"
        } else if is_built_on_oscillator {
            // Если индикатор построен по осциллятору (например, SMA(RSI)),
            // он может создавать ТОЛЬКО indicator_indicator условия с исходным осциллятором
            "indicator_indicator"
        } else if self.should_add(probabilities.use_trend_condition) {
            // Трендовые условия могут создаваться для любых индикаторов
            "trend_condition"
        } else if primary_indicator.indicator_type == "trend"
            || primary_indicator.indicator_type == "channel"
        {
            // Трендовые и канальные индикаторы не могут создавать indicator_constant условия
            if self.should_add(probabilities.use_indicator_indicator_condition) {
                "indicator_indicator"
            } else {
                "indicator_price"
            }
        } else if self.should_add(probabilities.use_indicator_indicator_condition) {
            "indicator_indicator"
        } else {
            "indicator_price"
        };

        // Для trend_condition используем только GreaterThan или LessThan
        // CrossesAbove/CrossesBelow требуют два вектора, а трендовые условия работают с одним
        let operator = if condition_type == "trend_condition" {
            // Для трендовых условий используем только GreaterThan или LessThan
            if self.rng.gen_bool(0.5) {
                ConditionOperator::GreaterThan
            } else {
                ConditionOperator::LessThan
            }
        } else if self.should_add(probabilities.use_crosses_operator) {
            if self.rng.gen_bool(0.5) {
                ConditionOperator::CrossesAbove
            } else {
                ConditionOperator::CrossesBelow
            }
        } else if self.rng.gen_bool(0.5) {
            ConditionOperator::GreaterThan
        } else {
            ConditionOperator::LessThan
        };

        let (condition_id, condition_name) = if condition_type == "indicator_constant" {
            // Для indicator_constant генерируем случайное значение константы
            // Для осцилляторов обычно используются значения от 0 до 100
            // Для volatility индикаторов - процент от цены (0.2% - 10.0%)
            let constant_value = if primary_indicator.indicator_type == "volatility" {
                // Volatility индикаторы: процент от цены (0.2% - 10.0%)
                // Получаем диапазон из конфигурации
                let rules = &self.config.rules.indicator_parameter_rules;
                let mut percentage_range = (0.2, 10.0, 0.1); // По умолчанию

                for rule in rules {
                    if rule.indicator_type == "volatility" {
                        if !rule.indicator_names.is_empty() {
                            if !rule.indicator_names.contains(&primary_indicator.name) {
                                continue;
                            }
                        }
                        if let Some(constraint) = &rule.price_field_constraint {
                            if let super::candidate_builder_config::ParameterConstraint::PercentageFromPrice {
                                min_percent,
                                max_percent,
                                step,
                            } = &constraint.parameter_constraint {
                                percentage_range = (*min_percent, *max_percent, *step);
                                break;
                            }
                        }
                    }
                }

                // Генерируем случайный процент в диапазоне
                let steps =
                    ((percentage_range.1 - percentage_range.0) / percentage_range.2) as usize;
                let step_index = self.rng.gen_range(0..=steps);
                percentage_range.0 + (step_index as f64 * percentage_range.2)
            } else if primary_indicator.name == "RSI" {
                // RSI: 30 (перепроданность) или 70 (перекупленность)
                if operator == ConditionOperator::GreaterThan {
                    self.rng.gen_range(70.0..=90.0)
                } else {
                    self.rng.gen_range(10.0..=30.0)
                }
            } else if primary_indicator.name == "Stochastic" {
                // Stochastic: 20 (перепроданность) или 80 (перекупленность)
                if operator == ConditionOperator::GreaterThan {
                    self.rng.gen_range(80.0..=95.0)
                } else {
                    self.rng.gen_range(5.0..=20.0)
                }
            } else {
                // Для других осцилляторов: 0-100
                self.rng.gen_range(0.0..=100.0)
            };

            let id = format!(
                "{}_{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                self.rng.gen::<u32>()
            );
            let name = if primary_indicator.indicator_type == "volatility" {
                // Для volatility: показываем процент от цены
                format!(
                    "{} {:?} Close * {:.2}%",
                    primary_indicator.name, operator, constant_value
                )
            } else {
                format!(
                    "{} {:?} {:.1}",
                    primary_indicator.name, operator, constant_value
                )
            };
            (id, name)
        } else if condition_type == "trend_condition" {
            // Для trend_condition создаем трендовое условие
            // GreaterThan -> RisingTrend, LessThan -> FallingTrend
            let period = self.rng.gen_range(10.0..=50.0);
            let trend_name = match operator {
                ConditionOperator::GreaterThan => "RisingTrend",
                ConditionOperator::LessThan => "FallingTrend",
                _ => "RisingTrend", // По умолчанию
            };
            let id = format!(
                "{}_{}_{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                trend_name.to_lowercase(),
                self.rng.gen::<u32>()
            );
            let name = format!(
                "{} {} (period: {:.0})",
                primary_indicator.name, trend_name, period
            );
            (id, name)
        } else if condition_type == "indicator_indicator" {
            // Для indicator_indicator нужен второй индикатор
            // Правила: осцилляторы не могут сравниваться с осцилляторами
            // Осцилляторы могут сравниваться только с индикаторами, построенными по осцилляторам
            let available_secondary: Vec<&IndicatorInfo> = all_indicators
                .iter()
                .filter(|ind| ind.alias != primary_indicator.alias)
                .filter(|ind| {
                    Self::can_compare_indicators(
                        primary_indicator,
                        *ind,
                        nested_indicators,
                        indicators,
                    )
                })
                .copied()
                .collect();

            if let Some(secondary) = available_secondary.choose(&mut self.rng) {
                let id = format!(
                    "{}_{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    primary_indicator.alias,
                    secondary.alias,
                    self.rng.gen::<u32>()
                );
                let name = format!(
                    "{} {:?} {}",
                    primary_indicator.name, operator, secondary.name
                );
                (id, name)
            } else {
                // Если нет подходящего второго индикатора и это осциллятор (не во вложенных),
                // создаем indicator_constant, иначе indicator_price
                if primary_indicator.indicator_type == "oscillator"
                    && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
                {
                    let const_val = if primary_indicator.name == "RSI" {
                        if operator == ConditionOperator::GreaterThan {
                            self.rng.gen_range(70.0..=90.0)
                        } else {
                            self.rng.gen_range(10.0..=30.0)
                        }
                    } else if primary_indicator.name == "Stochastic" {
                        if operator == ConditionOperator::GreaterThan {
                            self.rng.gen_range(80.0..=95.0)
                        } else {
                            self.rng.gen_range(5.0..=20.0)
                        }
                    } else {
                        self.rng.gen_range(0.0..=100.0)
                    };
                    let id = format!(
                        "{}_{}_{}",
                        if is_entry { "entry" } else { "exit" },
                        primary_indicator.alias,
                        self.rng.gen::<u32>()
                    );
                    let name =
                        format!("{} {:?} {:.1}", primary_indicator.name, operator, const_val);
                    (id, name)
                } else {
                    let id = format!(
                        "{}_{}_{}",
                        if is_entry { "entry" } else { "exit" },
                        primary_indicator.alias,
                        self.rng.gen::<u32>()
                    );
                    let name = format!("{} {:?} {}", primary_indicator.name, operator, "target");
                    (id, name)
                }
            }
        } else {
            // Если condition_type не indicator_constant и не indicator_indicator,
            // но это осциллятор (не во вложенных), создаем indicator_constant
            if primary_indicator.indicator_type == "oscillator"
                && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
            {
                let const_val = if primary_indicator.name == "RSI" {
                    if operator == ConditionOperator::GreaterThan {
                        self.rng.gen_range(70.0..=90.0)
                    } else {
                        self.rng.gen_range(10.0..=30.0)
                    }
                } else if primary_indicator.name == "Stochastic" {
                    if operator == ConditionOperator::GreaterThan {
                        self.rng.gen_range(80.0..=95.0)
                    } else {
                        self.rng.gen_range(5.0..=20.0)
                    }
                } else {
                    self.rng.gen_range(0.0..=100.0)
                };
                let id = format!(
                    "{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    primary_indicator.alias,
                    self.rng.gen::<u32>()
                );
                let name = format!("{} {:?} {:.1}", primary_indicator.name, operator, const_val);
                (id, name)
            } else {
                let id = format!(
                    "{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    primary_indicator.alias,
                    self.rng.gen::<u32>()
                );
                let name = format!("{} {:?} {}", primary_indicator.name, operator, "target");
                (id, name)
            }
        };

        let price_field = if condition_type == "indicator_price" {
            if primary_indicator.indicator_type == "volatility" {
                // Для volatility индикаторов проверяем правила из конфигурации
                // Если есть правило для конкретного индикатора (WATR, ATR), используем требуемое price_field
                let rules = &self.config.rules.indicator_parameter_rules;
                let mut required_price_field = None;

                for rule in rules {
                    if rule.indicator_type == "volatility" {
                        if !rule.indicator_names.is_empty() {
                            if rule.indicator_names.contains(&primary_indicator.name) {
                                if let Some(constraint) = &rule.price_field_constraint {
                                    required_price_field =
                                        Some(constraint.required_price_field.clone());
                                    break;
                                }
                            }
                        } else {
                            // Если правило применяется ко всем volatility индикаторам
                            if let Some(constraint) = &rule.price_field_constraint {
                                required_price_field =
                                    Some(constraint.required_price_field.clone());
                                break;
                            }
                        }
                    }
                }

                // Используем требуемое price_field из правил или "Close" по умолчанию
                Some(required_price_field.unwrap_or_else(|| "Close".to_string()))
            } else if self.config.condition_config.price_fields.len() == 1 {
                Some(self.config.condition_config.price_fields[0].clone())
            } else {
                self.config
                    .condition_config
                    .price_fields
                    .choose(&mut self.rng)
                    .cloned()
                    .or(Some("Close".to_string()))
            }
        } else {
            None
        };

        // Если не удалось создать indicator_indicator, проверяем тип индикатора
        let final_condition_type = if condition_type == "indicator_indicator" {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() < 4 {
                // Если не удалось создать indicator_indicator
                // Для осцилляторов и volatility используем indicator_constant, иначе indicator_price
                if (primary_indicator.indicator_type == "oscillator"
                    && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators))
                    || primary_indicator.indicator_type == "volatility"
                {
                    "indicator_constant"
                } else {
                    "indicator_price"
                }
            } else {
                condition_type
            }
        } else if condition_type == "indicator_price" {
            // Если condition_type был indicator_price, но это осциллятор или volatility
            // используем indicator_constant
            if (primary_indicator.indicator_type == "oscillator"
                && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators))
                || primary_indicator.indicator_type == "volatility"
            {
                "indicator_constant"
            } else {
                condition_type
            }
        } else {
            condition_type
        };

        // Извлекаем значение константы из имени условия для indicator_constant и period для trend_condition
        let (constant_value, trend_period) = if final_condition_type == "indicator_constant" {
            // Парсим значение из имени условия
            // Для осцилляторов: "RSI GreaterThan 70.0"
            // Для volatility: "WATR GreaterThan Close * 2.50%"
            let parsed_value = if primary_indicator.indicator_type == "volatility" {
                // Для volatility парсим процент из формата "Close * X.XX%"
                let parts: Vec<&str> = condition_name.split_whitespace().collect();
                if let Some(percent_str) = parts.last() {
                    percent_str
                        .strip_suffix('%')
                        .and_then(|s| s.parse::<f64>().ok())
                } else {
                    None
                }
            } else {
                // Для осцилляторов: просто последнее число
                let parts: Vec<&str> = condition_name.split_whitespace().collect();
                if parts.len() >= 3 {
                    parts.last().and_then(|s| s.parse::<f64>().ok())
                } else {
                    None
                }
            };
            (parsed_value, None)
        } else if final_condition_type == "trend_condition" {
            // Для trend_condition парсим period из формата "SMA RisingTrend (period: 20)"
            let parts: Vec<&str> = condition_name.split_whitespace().collect();
            let period = parts
                .iter()
                .find(|s| s.starts_with("(period:"))
                .and_then(|s| {
                    s.strip_prefix("(period:")
                        .and_then(|s| s.strip_suffix(")"))
                        .and_then(|s| s.trim().parse::<f64>().ok())
                })
                .unwrap_or(20.0);
            (None, Some(period))
        } else {
            (None, None)
        };

        // Создаем optimization_params для volatility условий, процентных условий и трендовых условий
        let (optimization_params, constant_value_for_percent) = if final_condition_type
            == "indicator_constant"
            && primary_indicator.indicator_type == "volatility"
            && constant_value.is_some()
        {
            // Добавляем параметр "percentage" для оптимизации volatility
            (
                vec![crate::discovery::ConditionParamInfo {
                    name: "percentage".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }],
                constant_value,
            )
        } else if final_condition_type == "trend_condition" && trend_period.is_some() {
            // Добавляем параметр "period" для трендовых условий
            // Для trend_condition не устанавливаем constant_value, так как это период, а не процент
            (
                vec![crate::discovery::ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }],
                None, // Не устанавливаем constant_value для trend_condition
            )
        } else if (final_condition_type == "indicator_price"
            || final_condition_type == "indicator_indicator")
            && self.should_add(probabilities.use_percent_condition)
        {
            // Добавляем параметр "percentage" для процентных условий
            let percent_value = self.rng.gen_range(0.1..=5.0);
            (
                vec![crate::discovery::ConditionParamInfo {
                    name: "percentage".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }],
                Some(percent_value),
            )
        } else {
            (Vec::new(), constant_value)
        };

        // Обновляем имя условия, если добавлен процент
        // Для trend_condition не добавляем процент, так как это период, а не процент
        let final_condition_name = if !optimization_params.is_empty()
            && final_condition_type != "indicator_constant"
            && final_condition_type != "trend_condition"
        {
            if let Some(percent) = constant_value_for_percent {
                if final_condition_type == "indicator_indicator" {
                    // Для indicator_indicator: "SMA GreaterThan EMA на 2.5%"
                    format!("{} на {:.2}%", condition_name, percent)
                } else {
                    // Для indicator_price: "SMA GreaterThan Close на 2.5%"
                    format!("{} на {:.2}%", condition_name, percent)
                }
            } else {
                condition_name
            }
        } else {
            condition_name
        };

        Some(ConditionInfo {
            id: condition_id,
            name: final_condition_name,
            operator,
            condition_type: final_condition_type.to_string(),
            optimization_params,
            constant_value: constant_value_for_percent,
            primary_timeframe: None,
            secondary_timeframe: None,
            price_field: if final_condition_type == "indicator_price" && price_field.is_none() {
                if self.config.condition_config.price_fields.len() == 1 {
                    Some(self.config.condition_config.price_fields[0].clone())
                } else {
                    self.config
                        .condition_config
                        .price_fields
                        .choose(&mut self.rng)
                        .cloned()
                        .or(Some("Close".to_string()))
                }
            } else {
                price_field
            },
        })
    }

    fn should_add(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability.clamp(0.0, 1.0))
    }

    fn is_oscillator_used_in_nested(
        indicator: &crate::discovery::IndicatorInfo,
        nested_indicators: &[crate::discovery::NestedIndicator],
    ) -> bool {
        nested_indicators
            .iter()
            .any(|nested| nested.input_indicator_alias == indicator.alias)
    }

    /// Проверяет, является ли условие дубликатом уже существующего условия
    fn is_duplicate_condition(
        new_condition: &crate::discovery::ConditionInfo,
        existing_conditions: &[crate::discovery::ConditionInfo],
    ) -> bool {
        for existing in existing_conditions {
            // Проверяем основные характеристики условия
            if existing.condition_type != new_condition.condition_type {
                continue;
            }

            if existing.operator != new_condition.operator {
                continue;
            }

            // Извлекаем alias индикаторов из ID условий
            let existing_primary_alias =
                Self::extract_indicator_alias_from_condition_id(&existing.id);
            let new_primary_alias =
                Self::extract_indicator_alias_from_condition_id(&new_condition.id);

            if existing_primary_alias != new_primary_alias {
                continue;
            }

            // Для indicator_indicator проверяем также secondary индикатор
            if existing.condition_type == "indicator_indicator" {
                let existing_parts: Vec<&str> = existing.id.split('_').collect();
                let new_parts: Vec<&str> = new_condition.id.split('_').collect();

                if existing_parts.len() >= 3 && new_parts.len() >= 3 {
                    let existing_secondary = existing_parts.get(2);
                    let new_secondary = new_parts.get(2);
                    if existing_secondary != new_secondary {
                        continue;
                    }
                }
            }

            // Проверяем поле цены (для indicator_price)
            if existing.condition_type == "indicator_price" {
                if existing.price_field != new_condition.price_field {
                    continue;
                }
            }

            // Проверяем значение константы (для indicator_constant)
            if existing.condition_type == "indicator_constant" {
                if existing.constant_value != new_condition.constant_value {
                    continue;
                }
            }

            // Проверяем таймфреймы
            if existing.primary_timeframe != new_condition.primary_timeframe {
                continue;
            }

            if existing.secondary_timeframe != new_condition.secondary_timeframe {
                continue;
            }

            // Если все совпадает, это дубликат
            return true;
        }
        false
    }

    /// Проверяет, можно ли сравнивать два индикатора в условии indicator_indicator
    /// Правила:
    /// 1. Осцилляторы не могут сравниваться с осцилляторами
    /// 2. Осцилляторы могут сравниваться только с индикаторами, построенными по осцилляторам
    /// 3. Трендовые/канальные индикаторы не могут сравниваться с осцилляторами,
    ///    КРОМЕ случаев когда они построены по осцилляторам
    /// 4. Если по неосциллятору построен осциллятор, его нельзя сравнивать с другими индикаторами (только с константой)
    fn can_compare_indicators(
        primary: &crate::discovery::IndicatorInfo,
        secondary: &crate::discovery::IndicatorInfo,
        nested_indicators: &[crate::discovery::NestedIndicator],
        all_indicators: &[crate::discovery::IndicatorInfo],
    ) -> bool {
        // Правило 1: Осцилляторы не могут сравниваться с осцилляторами
        if primary.indicator_type == "oscillator" && secondary.indicator_type == "oscillator" {
            return false;
        }

        // Вспомогательная функция: проверяет, построен ли индикатор по осциллятору
        let is_built_on_oscillator = |indicator: &crate::discovery::IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                // Находим входной индикатор
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    input_indicator.indicator_type == "oscillator"
                } else {
                    false
                }
            } else {
                false
            }
        };

        // Вспомогательная функция: проверяет, построен ли индикатор по неосциллятору
        let is_built_on_non_oscillator = |indicator: &crate::discovery::IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                // Находим входной индикатор
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    input_indicator.indicator_type != "oscillator"
                } else {
                    false
                }
            } else {
                false
            }
        };

        let primary_built_on_oscillator = is_built_on_oscillator(primary);
        let secondary_built_on_oscillator = is_built_on_oscillator(secondary);
        let primary_built_on_non_oscillator = is_built_on_non_oscillator(primary);
        let secondary_built_on_non_oscillator = is_built_on_non_oscillator(secondary);

        // Вспомогательная функция: находит исходный осциллятор для индикатора, построенного по осциллятору
        let get_source_oscillator_alias =
            |indicator: &crate::discovery::IndicatorInfo| -> Option<String> {
                if let Some(nested) = nested_indicators
                    .iter()
                    .find(|n| n.indicator.alias == indicator.alias)
                {
                    if let Some(input_indicator) = all_indicators
                        .iter()
                        .find(|ind| ind.alias == nested.input_indicator_alias)
                    {
                        if input_indicator.indicator_type == "oscillator" {
                            return Some(input_indicator.alias.clone());
                        }
                    }
                }
                None
            };

        // Правило 4: Если по неосциллятору построен осциллятор, его нельзя сравнивать с другими индикаторами
        if primary.indicator_type == "oscillator" && primary_built_on_non_oscillator {
            return false;
        }
        if secondary.indicator_type == "oscillator" && secondary_built_on_non_oscillator {
            return false;
        }

        // Правило 5: Если индикатор построен по осциллятору, он может сравниваться ТОЛЬКО с исходным осциллятором
        if primary_built_on_oscillator {
            if let Some(source_oscillator_alias) = get_source_oscillator_alias(primary) {
                // Может сравниваться только с исходным осциллятором
                return secondary.alias == source_oscillator_alias;
            }
        }
        if secondary_built_on_oscillator {
            if let Some(source_oscillator_alias) = get_source_oscillator_alias(secondary) {
                // Может сравниваться только с исходным осциллятором
                return primary.alias == source_oscillator_alias;
            }
        }

        // Правило 2: Если primary - осциллятор, secondary должен быть построен по осциллятору
        if primary.indicator_type == "oscillator" {
            return secondary_built_on_oscillator;
        }

        // Правило 3: Если secondary - осциллятор, primary должен быть построен по осциллятору
        if secondary.indicator_type == "oscillator" {
            return primary_built_on_oscillator;
        }

        // Если оба не осцилляторы, можно сравнивать
        true
    }

    fn ensure_minimum_requirements(
        &mut self,
        candidate: &mut CandidateElements,
        constraints: &ElementConstraints,
        available_stop_handlers: &[StopHandlerConfig],
    ) {
        let stop_handlers_prob = self.config.probabilities.stop_handlers.clone();

        while candidate.stop_handlers.len() < constraints.min_stop_handlers {
            if available_stop_handlers.is_empty() {
                eprintln!("      ⚠️  ВНИМАНИЕ: Нет доступных stop handlers для выполнения min_stop_handlers={}", constraints.min_stop_handlers);
                break;
            }

            if let Some(stop) = self
                .select_single_stop_handler_required(available_stop_handlers, &stop_handlers_prob)
            {
                candidate.stop_handlers.push(stop);
            } else {
                eprintln!(
                    "      ⚠️  ВНИМАНИЕ: Не удалось выбрать stop handler из {} доступных",
                    available_stop_handlers.len()
                );
                break;
            }
        }

        while candidate.take_handlers.len() < constraints.min_take_handlers {
            let take_configs: Vec<&StopHandlerConfig> = available_stop_handlers
                .iter()
                .filter(|c| c.stop_type == "take_profit")
                .collect();

            if let Some(config) = take_configs.choose(&mut self.rng) {
                candidate.take_handlers.push(StopHandlerInfo {
                    id: format!("take_{}", self.rng.gen::<u32>()),
                    name: config.handler_name.clone(),
                    handler_name: config.handler_name.clone(),
                    stop_type: config.stop_type.clone(),
                    optimization_params: Vec::new(),
                    priority: config.priority,
                });
            } else {
                break;
            }
        }

        let indicators = candidate.indicators.clone();
        let nested_indicators = candidate.nested_indicators.clone();
        let probabilities_conditions = self.config.probabilities.conditions.clone();

        while candidate.entry_conditions.len() < constraints.min_entry_conditions {
            if candidate.entry_conditions.len() >= constraints.max_entry_conditions {
                break;
            }
            if !indicators.is_empty() {
                if let Some(condition) = self.build_condition(
                    &indicators,
                    &nested_indicators,
                    &probabilities_conditions,
                    true,
                ) {
                    if !Self::is_duplicate_condition(&condition, &candidate.entry_conditions) {
                        candidate.entry_conditions.push(condition);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        while candidate.exit_conditions.len() < constraints.min_exit_conditions {
            if candidate.exit_conditions.len() >= constraints.max_exit_conditions {
                break;
            }
            if !indicators.is_empty() {
                if let Some(condition) = self.build_condition(
                    &indicators,
                    &nested_indicators,
                    &probabilities_conditions,
                    false,
                ) {
                    if !Self::is_duplicate_condition(&condition, &candidate.exit_conditions) {
                        candidate.exit_conditions.push(condition);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

pub struct CandidateElements {
    pub indicators: Vec<IndicatorInfo>,
    pub nested_indicators: Vec<NestedIndicator>,
    pub entry_conditions: Vec<ConditionInfo>,
    pub exit_conditions: Vec<ConditionInfo>,
    pub stop_handlers: Vec<StopHandlerInfo>,
    pub take_handlers: Vec<StopHandlerInfo>,
    pub timeframes: Vec<TimeFrame>,
}
