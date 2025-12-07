use crate::condition::ConditionParameterPresets;
use crate::data_model::types::TimeFrame;
use crate::discovery::types::{
    ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};
use crate::strategy::types::ConditionOperator;
use rand::seq::SliceRandom;
use rand::Rng;

use super::build_rules_provider::{
    can_accept_nested_input, get_allowed_conditions, has_absolute_threshold,
    has_percent_of_price_threshold, is_oscillator_like, is_phase_1_allowed,
};
use super::builders::ConditionBuilder;
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

    /// Создаёт optimization_params для stop/take handler из конфигурации
    /// Собирает все параметры для хендлера из всех доступных конфигов
    fn make_handler_params(
        config: &StopHandlerConfig,
        all_configs: &[StopHandlerConfig],
    ) -> Vec<crate::discovery::ConditionParamInfo> {
        let handler_name = &config.handler_name;
        let mut params = Vec::new();

        for cfg in all_configs {
            if cfg.handler_name == *handler_name && cfg.stop_type == config.stop_type {
                if !cfg.parameter_name.is_empty() {
                    params.push(crate::discovery::ConditionParamInfo {
                        name: cfg.parameter_name.clone(),
                        optimizable: true,
                        global_param_name: cfg.global_param_name.clone(),
                    });
                }
            }
        }

        params
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

        let mut phase = 2;
        while self.should_add(probabilities.phases.continue_building) {
            let all_limits_reached = self.build_additional_phase(
                &mut candidate,
                available_indicators,
                available_stop_handlers,
                available_timeframes,
                &constraints,
                &probabilities,
                phase,
            );
            if all_limits_reached {
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

        self.ensure_minimum_requirements(&mut candidate, &constraints, available_stop_handlers, available_indicators);

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
            true,
        ) {
            candidate.indicators.push(indicator);
        }

        self.try_add_nested_indicator(candidate, available_indicators);

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
                if !Self::is_duplicate_condition(&condition, &candidate.entry_conditions)
                    && !Self::has_conflicting_comparison_operator(
                        &condition,
                        &candidate.entry_conditions,
                    )
                {
                    candidate.entry_conditions.push(condition);
                }
            }
        }

        // 4. Добавляем случайный стоп (только один) - ОБЯЗАТЕЛЬНО в первой фазе
        if candidate.stop_handlers.is_empty() && constraints.min_stop_handlers > 0 {
            if available_stop_handlers.is_empty() {
                eprintln!("      ⚠️  ВНИМАНИЕ: Нет доступных stop handlers для добавления!");
            } else if let Some(stop) =
                self.select_single_stop_handler_required(available_stop_handlers, available_indicators)
            {
                candidate.stop_handlers.push(stop);
            } else {
                eprintln!(
                    "      ⚠️  ВНИМАНИЕ: Не удалось выбрать stop handler из {} доступных",
                    available_stop_handlers.len()
                );
            }
        }

        // 5. Добавляем take profit с вероятностью
        if candidate.take_handlers.is_empty()
            && candidate.take_handlers.len() < constraints.max_take_handlers
            && self.should_add(probabilities.take_handlers.add_take_profit)
        {
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
                    optimization_params: Self::make_handler_params(config, available_stop_handlers),
                    priority: config.priority,
                });
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
        let all_limits_reached = candidate.indicators.len() >= constraints.max_indicators
            && candidate.entry_conditions.len() >= constraints.max_entry_conditions
            && candidate.exit_conditions.len() >= constraints.max_exit_conditions
            && candidate.timeframes.len() >= constraints.max_timeframes;

        if all_limits_reached {
            return true;
        }

        if candidate.indicators.is_empty() && candidate.nested_indicators.is_empty() {
            return true;
        }

        // Пытаемся добавить higher TF с индикатором и условием
        let mut added_higher_tf_indicator = false;
        if candidate.timeframes.len() < constraints.max_timeframes
            && candidate.indicators.len() < constraints.max_indicators
            && candidate.entry_conditions.len() < constraints.max_entry_conditions
            && available_timeframes.len() > 1
            && self.should_add(probabilities.timeframes.use_higher_timeframe)
        {
            let higher_timeframes: Vec<&TimeFrame> = available_timeframes
                .iter()
                .skip(1)
                .filter(|tf| !candidate.timeframes.contains(tf))
                .collect();

            if let Some(higher_tf) = higher_timeframes.choose(&mut self.rng) {
                // Выбираем индикатор для higher TF
                let exclude_aliases: Vec<String> = candidate
                    .indicators
                    .iter()
                    .map(|i| i.alias.clone())
                    .collect();

                if let Some(mut indicator) = self.select_single_indicator(
                    available_indicators,
                    probabilities,
                    &exclude_aliases,
                    false,
                ) {
                    // Добавляем TF к alias индикатора чтобы различать
                    let higher_tf_minutes = higher_tf.total_minutes().unwrap_or(0);
                    indicator.alias = format!("{}_{}", indicator.alias, higher_tf_minutes);

                    // Создаём условие для этого индикатора на higher TF
                    let condition = self.build_condition_simple_with_timeframe(
                        &indicator,
                        true,
                        Some((*higher_tf).clone()),
                    );

                    if let Some(cond) = condition {
                        if !Self::is_duplicate_condition(&cond, &candidate.entry_conditions)
                            && !Self::has_conflicting_comparison_operator(
                                &cond,
                                &candidate.entry_conditions,
                            )
                        {
                            // Добавляем TF, индикатор и условие вместе
                            candidate.timeframes.push((*higher_tf).clone());
                            candidate.indicators.push(indicator);
                            candidate.entry_conditions.push(cond);
                            added_higher_tf_indicator = true;
                        }
                    }
                }
            }
        }

        if !added_higher_tf_indicator
            && candidate.indicators.len() < constraints.max_indicators
            && self.should_add(0.5)
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
                false,
            ) {
                candidate.indicators.push(indicator);
            }
        }

        self.try_add_nested_indicator(candidate, available_indicators);

        if candidate.entry_conditions.len() < constraints.max_entry_conditions
            && self.should_add(probabilities.conditions.add_entry_condition)
        {
            let condition = self.build_condition(
                &candidate.indicators,
                &candidate.nested_indicators,
                &probabilities.conditions,
                true,
            );

            if let Some(cond) = condition {
                if !Self::is_duplicate_condition(&cond, &candidate.entry_conditions)
                    && !Self::has_conflicting_comparison_operator(
                        &cond,
                        &candidate.entry_conditions,
                    )
                {
                    candidate.entry_conditions.push(cond);
                }
            }
        }

        if candidate.exit_conditions.len() < constraints.max_exit_conditions
            && self.should_add(probabilities.phases.add_exit_rules)
        {
            let condition = self.build_condition(
                &candidate.indicators,
                &candidate.nested_indicators,
                &probabilities.conditions,
                false,
            );

            if let Some(cond) = condition {
                if !Self::is_duplicate_condition(&cond, &candidate.exit_conditions)
                    && !Self::has_conflicting_comparison_operator(&cond, &candidate.exit_conditions)
                {
                    candidate.exit_conditions.push(cond);
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
                        optimization_params: Self::make_handler_params(
                            config,
                            available_stop_handlers,
                        ),
                        priority: config.priority,
                    });
                    return;
                }

                candidate.take_handlers.push(StopHandlerInfo {
                    id: format!("take_{}", self.rng.gen::<u32>()),
                    name: name.clone(),
                    handler_name: name.clone(),
                    stop_type: "take_profit".to_string(),
                    optimization_params: Vec::new(), // fallback без config
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
                        optimization_params: Self::make_handler_params(
                            config,
                            available_stop_handlers,
                        ),
                        priority: config.priority,
                    });
                    return;
                }

                candidate.stop_handlers.push(StopHandlerInfo {
                    id: format!("stop_{}", self.rng.gen::<u32>()),
                    name: name.clone(),
                    handler_name: name.clone(),
                    stop_type: "stop_loss".to_string(),
                    optimization_params: Vec::new(), // fallback без config
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
                if is_phase_1 && !is_phase_1_allowed(&indicator.name) {
                    return false;
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
        available_indicators: &[IndicatorInfo],
    ) -> Option<StopHandlerInfo> {
        let excluded_stop_handlers: std::collections::HashSet<&str> = self
            .config
            .rules
            .excluded_stop_handlers
            .iter()
            .map(|s| s.as_str())
            .collect();

        let stop_loss_configs: Vec<&StopHandlerConfig> = available
            .iter()
            .filter(|c| c.stop_type == "stop_loss")
            .filter(|c| !excluded_stop_handlers.contains(c.handler_name.as_str()))
            .collect();

        if stop_loss_configs.is_empty() {
            return None;
        }

        stop_loss_configs
            .choose(&mut self.rng)
            .map(|config| {
                let mut handler_name = config.handler_name.clone();
                
                // Для новых стопов с индикаторами выбираем случайный трендовый индикатор
                if config.handler_name == "ATRTrailIndicatorStop"
                    || config.handler_name == "PercentTrailIndicatorStop"
                {
                    if let Some(indicator_name) = Self::select_random_trend_indicator(
                        available_indicators,
                        &mut self.rng,
                    ) {
                        // Сохраняем выбранный индикатор в name для последующего извлечения
                        handler_name = format!("{}:{}", config.handler_name, indicator_name);
                    }
                }
                
                StopHandlerInfo {
                    id: format!("stop_{}", self.rng.gen::<u32>()),
                    name: handler_name.clone(),
                    handler_name: handler_name,
                    stop_type: config.stop_type.clone(),
                    optimization_params: Self::make_handler_params(config, available),
                    priority: config.priority,
                }
            })
    }

    /// Выбирает случайный трендовый индикатор из доступных
    fn select_random_trend_indicator(
        available_indicators: &[IndicatorInfo],
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<String> {
        // Фильтруем только трендовые индикаторы
        let trend_indicators: Vec<&IndicatorInfo> = available_indicators
            .iter()
            .filter(|ind| ind.indicator_type == "trend")
            .collect();

        if trend_indicators.is_empty() {
            // Fallback: список популярных трендовых индикаторов
            let default_trend_indicators = vec!["SMA", "EMA", "WMA", "AMA", "ZLEMA"];
            default_trend_indicators
                .choose(rng)
                .map(|s| s.to_string())
        } else {
            trend_indicators
                .choose(rng)
                .map(|ind| ind.name.clone())
        }
    }

    fn build_condition_with_timeframe(
        &mut self,
        indicator: &IndicatorInfo,
        is_entry: bool,
        timeframe: Option<TimeFrame>,
    ) -> Option<ConditionInfo> {
        let operator = if self.rng.gen_bool(0.5) {
            ConditionOperator::Above
        } else {
            ConditionOperator::Below
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
            primary_indicator_alias: indicator.alias.clone(),
            secondary_indicator_alias: None,
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
            if let Some(indicator) = all_indicators
                .iter()
                .find(|i| i.alias == condition.primary_indicator_alias)
            {
                used_indicators.insert(indicator.alias.clone());
            }
        }

        for indicator in &all_indicators {
            if !used_indicators.contains(&indicator.alias) {
                if entry_conditions.len() >= constraints.max_entry_conditions {
                    break;
                }
                let condition = self.build_condition_simple(indicator, true);
                if let Some(cond) = condition {
                    if !Self::is_duplicate_condition(&cond, entry_conditions)
                        && !Self::has_conflicting_comparison_operator(&cond, entry_conditions)
                    {
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
        // Для higher TF (timeframe.is_some()) используем ограниченный набор операторов:
        // Above, Below, RisingTrend, FallingTrend
        // Для базового TF - только Above, Below
        let operator = if timeframe.is_some() {
            // Higher TF: 4 оператора
            match self.rng.gen_range(0..4) {
                0 => ConditionOperator::Above,
                1 => ConditionOperator::Below,
                2 => ConditionOperator::RisingTrend,
                _ => ConditionOperator::FallingTrend,
            }
        } else {
            // Базовый TF: только Above/Below
            if self.rng.gen_bool(0.5) {
                ConditionOperator::Above
            } else {
                ConditionOperator::Below
            }
        };

        // Для trend условий (RisingTrend/FallingTrend) создаём trend_condition
        if matches!(
            operator,
            ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
        ) {
            let trend_range = ConditionParameterPresets::trend_period();
            let period = self.rng.gen_range(trend_range.min..=trend_range.max);
            let trend_name = match operator {
                ConditionOperator::RisingTrend => "risingtrend",
                _ => "fallingtrend",
            };
            let trend_display = match operator {
                ConditionOperator::RisingTrend => "RisingTrend",
                _ => "FallingTrend",
            };
            let prefix = if is_entry { "entry" } else { "exit" };
            let condition_id = format!(
                "{}_{}_{}_{}",
                prefix,
                indicator.alias,
                trend_name,
                self.rng.gen::<u32>()
            );
            let condition_name = format!(
                "{} {} (period: {:.0})",
                indicator.name, trend_display, period
            );

            return Some(ConditionInfo {
                id: condition_id,
                name: condition_name,
                operator,
                condition_type: "trend_condition".to_string(),
                optimization_params: vec![crate::discovery::ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }],
                constant_value: Some(period as f64),
                primary_indicator_alias: indicator.alias.clone(),
                secondary_indicator_alias: None,
                primary_timeframe: timeframe,
                secondary_timeframe: None,
                price_field: None,
            });
        }

        let condition_id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            indicator.alias,
            self.rng.gen::<u32>()
        );

        let (condition_type, condition_name, constant_value, price_field, optimization_params) =
            if has_absolute_threshold(&indicator.name) {
                let const_val = if indicator.name == "RSI" {
                    if operator == ConditionOperator::Above {
                        self.rng.gen_range(70.0..=90.0)
                    } else {
                        self.rng.gen_range(10.0..=30.0)
                    }
                } else if indicator.name == "Stochastic" {
                    if operator == ConditionOperator::Above {
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
            } else if has_percent_of_price_threshold(&indicator.name) {
                let rules = &self.config.rules.indicator_parameter_rules;
                let mut percentage_range = (0.2, 10.0, 0.1);

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
            primary_indicator_alias: indicator.alias.clone(),
            secondary_indicator_alias: None,
            primary_timeframe: timeframe,
            secondary_timeframe: None,
            price_field,
        })
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
                    .map(|input| has_absolute_threshold(&input.name))
            })
            .unwrap_or(false);

        // Проверяем, есть ли подходящий второй индикатор для indicator_indicator
        // Преобразуем только базовые indicators в плоский список для can_compare_indicators
        // (nested индикаторы не нужны, т.к. input индикатор ищется среди базовых)
        let flat_base_indicators: Vec<IndicatorInfo> =
            indicators.iter().map(|ind| ind.clone()).collect();

        let available_secondary_for_indicator_indicator: Vec<&IndicatorInfo> = all_indicators
            .iter()
            .filter(|ind| ind.alias != primary_indicator.alias)
            .filter(|ind| {
                Self::can_compare_indicators(
                    primary_indicator,
                    *ind,
                    nested_indicators,
                    &flat_base_indicators,
                )
            })
            .copied()
            .collect();

        let has_available_secondary = !available_secondary_for_indicator_indicator.is_empty();

        let condition_type = if has_absolute_threshold(&primary_indicator.name)
            && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
        {
            "indicator_constant"
        } else if has_percent_of_price_threshold(&primary_indicator.name) {
            "indicator_constant"
        } else if is_built_on_oscillator {
            self.weighted_choice_for_oscillator_based(probabilities)
        } else {
            // Выбираем тип условия, но если выбран indicator_indicator, проверяем наличие второго индикатора
            let chosen_type = self.weighted_condition_type_choice(probabilities);
            if chosen_type == "indicator_indicator" && !has_available_secondary {
                // Если выбран indicator_indicator, но нет подходящего второго индикатора,
                // выбираем между indicator_price и trend_condition
                if self.rng.gen_bool(
                    probabilities.use_trend_condition
                        / (probabilities.use_indicator_price_condition
                            + probabilities.use_trend_condition),
                ) {
                    "trend_condition"
                } else {
                    "indicator_price"
                }
            } else {
                chosen_type
            }
        };

        // Получаем разрешённые операторы из build_rules индикатора
        let allowed_conditions = get_allowed_conditions(&primary_indicator.name);

        let operator = if condition_type == "trend_condition" {
            // Для trend_condition выбираем только RisingTrend/FallingTrend
            let trend_ops: Vec<_> = allowed_conditions
                .iter()
                .filter(|op| {
                    matches!(
                        op,
                        ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
                    )
                })
                .collect();
            if let Some(op) = trend_ops.choose(&mut self.rng) {
                (*op).clone()
            } else if self.rng.gen_bool(0.5) {
                ConditionOperator::RisingTrend
            } else {
                ConditionOperator::FallingTrend
            }
        } else {
            // Для других типов условий выбираем из allowed_conditions
            // исключая RisingTrend/FallingTrend (они только для trend_condition)
            let non_trend_ops: Vec<_> = allowed_conditions
                .iter()
                .filter(|op| {
                    !matches!(
                        op,
                        ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
                    )
                })
                .collect();
            if let Some(op) = non_trend_ops.choose(&mut self.rng) {
                (*op).clone()
            } else if self.rng.gen_bool(0.5) {
                ConditionOperator::Above
            } else {
                ConditionOperator::Below
            }
        };

        let (condition_id, condition_name) = if condition_type == "indicator_constant" {
            // Для indicator_constant генерируем случайное значение константы
            // Для осцилляторов обычно используются значения от 0 до 100
            // Для volatility индикаторов - процент от цены (0.2% - 10.0%)
            let constant_value = if has_percent_of_price_threshold(&primary_indicator.name) {
                let rules = &self.config.rules.indicator_parameter_rules;
                let mut percentage_range = (0.2, 10.0, 0.1);

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
                if operator == ConditionOperator::Above {
                    self.rng.gen_range(70.0..=90.0)
                } else {
                    self.rng.gen_range(10.0..=30.0)
                }
            } else if primary_indicator.name == "Stochastic" {
                // Stochastic: 20 (перепроданность) или 80 (перекупленность)
                if operator == ConditionOperator::Above {
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
            let name = if has_percent_of_price_threshold(&primary_indicator.name) {
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
            let trend_range = ConditionParameterPresets::trend_period();
            let period = self.rng.gen_range(trend_range.min..=trend_range.max);
            let trend_name = match operator {
                ConditionOperator::RisingTrend => "RisingTrend",
                ConditionOperator::FallingTrend => "FallingTrend",
                _ => "RisingTrend",
            };
            let id = format!(
                "{}_{}::{}_{}",
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
            // Преобразуем только базовые indicators в плоский список для can_compare_indicators
            // (nested индикаторы не нужны, т.к. input индикатор ищется среди базовых)
            let flat_base_indicators: Vec<IndicatorInfo> =
                indicators.iter().map(|ind| ind.clone()).collect();

            let available_secondary: Vec<&IndicatorInfo> = all_indicators
                .iter()
                .filter(|ind| ind.alias != primary_indicator.alias)
                .filter(|ind| {
                    Self::can_compare_indicators(
                        primary_indicator,
                        *ind,
                        nested_indicators,
                        &flat_base_indicators,
                    )
                })
                .copied()
                .collect();

            if let Some(secondary) = available_secondary.choose(&mut self.rng) {
                let id = format!(
                    "{}_{}::{}_{}",
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
                // Если не найден второй индикатор для indicator_indicator,
                // создаем условие как indicator_price (индикатор сравнивается с ценой)
                // Это происходит, когда выбран тип indicator_indicator, но нет подходящего второго индикатора
                let id = format!(
                    "{}_{}_{}",
                    if is_entry { "entry" } else { "exit" },
                    primary_indicator.alias,
                    self.rng.gen::<u32>()
                );
                let name = format!("{} {:?} {}", primary_indicator.name, operator, "target");
                // Возвращаем ID без ::, чтобы в final_condition_type определить, что это indicator_price
                (id, name)
            }
        } else {
            if has_absolute_threshold(&primary_indicator.name)
                && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
            {
                let const_val = if primary_indicator.name == "RSI" {
                    if operator == ConditionOperator::Above {
                        self.rng.gen_range(70.0..=90.0)
                    } else {
                        self.rng.gen_range(10.0..=30.0)
                    }
                } else if primary_indicator.name == "Stochastic" {
                    if operator == ConditionOperator::Above {
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
            if has_percent_of_price_threshold(&primary_indicator.name) {
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
            // Проверяем, содержит ли ID разделитель :: (признак indicator_indicator)
            if condition_id.contains("::") {
                condition_type
            } else {
                // ID не содержит :: - это ошибка в логике, так как мы уже проверили наличие второго индикатора
                // Но на всякий случай меняем тип на indicator_price
                "indicator_price"
            }
        } else if condition_type == "indicator_price" {
            if (has_absolute_threshold(&primary_indicator.name)
                && !Self::is_oscillator_used_in_nested(primary_indicator, nested_indicators))
                || has_percent_of_price_threshold(&primary_indicator.name)
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
            let parsed_value = if has_percent_of_price_threshold(&primary_indicator.name) {
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
        // ВАЖНО: Для операторов LowerPercent и GreaterPercent ВСЕГДА создаем параметр percent,
        // независимо от вероятности use_percent_condition
        let (optimization_params, constant_value_for_percent) = if matches!(
            operator,
            ConditionOperator::LowerPercent | ConditionOperator::GreaterPercent
        ) {
            // Для LowerPercent и GreaterPercent ВСЕГДА создаем параметр percent
            let percent_value = self.rng.gen_range(0.1..=5.0);
            (
                vec![crate::discovery::ConditionParamInfo {
                    name: "percent".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }],
                Some(percent_value),
            )
        } else if final_condition_type == "indicator_constant"
            && has_percent_of_price_threshold(&primary_indicator.name)
            && constant_value.is_some()
        {
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
            // Используем constant_value для хранения значения периода
            (
                vec![crate::discovery::ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    global_param_name: None,
                }],
                trend_period, // Передаём значение периода через constant_value
            )
        } else if (final_condition_type == "indicator_price"
            || final_condition_type == "indicator_indicator")
            && self.should_add(probabilities.use_percent_condition)
        {
            // Добавляем параметр "percentage" для процентных условий (только если use_percent_condition разрешено)
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

        let (primary_alias, secondary_alias) = if final_condition_type == "indicator_indicator" {
            if let Some(separator_pos) = condition_id.find("::") {
                let prefix_len = if condition_id.starts_with("entry_") {
                    6
                } else if condition_id.starts_with("exit_") {
                    5
                } else {
                    0
                };
                let primary = &condition_id[prefix_len..separator_pos];
                let after_separator = &condition_id[separator_pos + 2..];
                if let Some(last_underscore) = after_separator.rfind('_') {
                    let secondary = &after_separator[..last_underscore];
                    (primary.to_string(), Some(secondary.to_string()))
                } else {
                    (primary_indicator.alias.clone(), None)
                }
            } else {
                (primary_indicator.alias.clone(), None)
            }
        } else {
            (primary_indicator.alias.clone(), None)
        };

        Some(ConditionInfo {
            id: condition_id,
            name: final_condition_name,
            operator,
            condition_type: final_condition_type.to_string(),
            optimization_params,
            constant_value: constant_value_for_percent,
            primary_indicator_alias: primary_alias,
            secondary_indicator_alias: secondary_alias,
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

    fn weighted_condition_type_choice(
        &mut self,
        probabilities: &super::candidate_builder_config::ConditionProbabilities,
    ) -> &'static str {
        let w_price = probabilities.use_indicator_price_condition;
        let w_indicator = probabilities.use_indicator_indicator_condition;
        let w_trend = probabilities.use_trend_condition;
        let total = w_price + w_indicator + w_trend;

        if total <= 0.0 {
            return "indicator_price";
        }

        let random = self.rng.gen::<f64>() * total;

        if random < w_price {
            "indicator_price"
        } else if random < w_price + w_indicator {
            "indicator_indicator"
        } else {
            "trend_condition"
        }
    }

    fn weighted_choice_for_oscillator_based(
        &mut self,
        probabilities: &super::candidate_builder_config::ConditionProbabilities,
    ) -> &'static str {
        let w_indicator = probabilities.use_indicator_indicator_condition;
        let w_trend = probabilities.use_trend_condition;
        let total = w_indicator + w_trend;

        if total <= 0.0 {
            return "indicator_indicator";
        }

        let random = self.rng.gen::<f64>() * total;

        if random < w_indicator {
            "indicator_indicator"
        } else {
            "trend_condition"
        }
    }

    fn is_oscillator_used_in_nested(
        indicator: &crate::discovery::IndicatorInfo,
        nested_indicators: &[crate::discovery::NestedIndicator],
    ) -> bool {
        nested_indicators
            .iter()
            .any(|nested| nested.input_indicator_alias == indicator.alias)
    }

    fn try_add_nested_indicator(
        &mut self,
        candidate: &mut CandidateElements,
        available_indicators: &[IndicatorInfo],
    ) {
        let add_nested_prob = self
            .config
            .probabilities
            .nested_indicators
            .add_nested_indicator;
        let max_depth = self
            .config
            .probabilities
            .nested_indicators
            .max_nesting_depth;

        if !self.should_add(add_nested_prob) {
            return;
        }

        if candidate.indicators.is_empty() {
            return;
        }

        let current_max_depth = candidate
            .nested_indicators
            .iter()
            .map(|n| n.depth)
            .max()
            .unwrap_or(0);

        if current_max_depth >= max_depth {
            return;
        }

        let base_indicators: Vec<&IndicatorInfo> = candidate
            .indicators
            .iter()
            .chain(candidate.nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        let Some(input_indicator) = base_indicators.choose(&mut self.rng) else {
            return;
        };

        let input_depth = candidate
            .nested_indicators
            .iter()
            .find(|n| n.indicator.alias == input_indicator.alias)
            .map(|n| n.depth)
            .unwrap_or(0);

        if input_depth >= max_depth {
            return;
        }

        let nestable_indicators: Vec<&IndicatorInfo> = available_indicators
            .iter()
            .filter(|ind| can_accept_nested_input(&ind.name))
            .filter(|ind| !self.config.rules.excluded_indicators.contains(&ind.name))
            .collect();

        let Some(wrapper_template) = nestable_indicators.choose(&mut self.rng) else {
            return;
        };

        let new_alias = format!("{}_on_{}", wrapper_template.alias, input_indicator.alias);

        let already_exists = candidate
            .nested_indicators
            .iter()
            .any(|n| n.indicator.alias == new_alias);

        if already_exists {
            return;
        }

        let nested_indicator = NestedIndicator {
            indicator: IndicatorInfo {
                name: wrapper_template.name.clone(),
                alias: new_alias,
                parameters: wrapper_template.parameters.clone(),
                can_use_indicator_input: true,
                input_type: "indicator".to_string(),
                indicator_type: wrapper_template.indicator_type.clone(),
            },
            input_indicator_alias: input_indicator.alias.clone(),
            depth: input_depth + 1,
        };

        candidate.nested_indicators.push(nested_indicator);
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

            if existing.primary_indicator_alias != new_condition.primary_indicator_alias {
                continue;
            }

            if existing.condition_type == "indicator_indicator" {
                if existing.secondary_indicator_alias != new_condition.secondary_indicator_alias {
                    continue;
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
        if is_oscillator_like(&primary.name) && is_oscillator_like(&secondary.name) {
            return false;
        }

        let is_built_on_oscillator = |indicator: &crate::discovery::IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    is_oscillator_like(&input_indicator.name)
                } else {
                    false
                }
            } else {
                false
            }
        };

        let is_built_on_non_oscillator = |indicator: &crate::discovery::IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    !is_oscillator_like(&input_indicator.name)
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
                        if is_oscillator_like(&input_indicator.name) {
                            return Some(input_indicator.alias.clone());
                        }
                    }
                }
                None
            };

        if is_oscillator_like(&primary.name) && primary_built_on_non_oscillator {
            return false;
        }
        if is_oscillator_like(&secondary.name) && secondary_built_on_non_oscillator {
            return false;
        }

        if primary_built_on_oscillator {
            if let Some(source_oscillator_alias) = get_source_oscillator_alias(primary) {
                return secondary.alias == source_oscillator_alias;
            }
        }
        if secondary_built_on_oscillator {
            if let Some(source_oscillator_alias) = get_source_oscillator_alias(secondary) {
                return primary.alias == source_oscillator_alias;
            }
        }

        if is_oscillator_like(&primary.name) {
            return secondary_built_on_oscillator;
        }

        if is_oscillator_like(&secondary.name) {
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
        available_indicators: &[IndicatorInfo],
    ) {
        while candidate.stop_handlers.len() < constraints.min_stop_handlers {
            if available_stop_handlers.is_empty() {
                eprintln!("      ⚠️  ВНИМАНИЕ: Нет доступных stop handlers для выполнения min_stop_handlers={}", constraints.min_stop_handlers);
                break;
            }

            if let Some(stop) = self.select_single_stop_handler_required(available_stop_handlers, available_indicators) {
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
                    optimization_params: Self::make_handler_params(config, available_stop_handlers),
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
                    if !Self::is_duplicate_condition(&condition, &candidate.entry_conditions)
                        && !Self::has_conflicting_comparison_operator(
                            &condition,
                            &candidate.entry_conditions,
                        )
                    {
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
                    if !Self::is_duplicate_condition(&condition, &candidate.exit_conditions)
                        && !Self::has_conflicting_comparison_operator(
                            &condition,
                            &candidate.exit_conditions,
                        )
                    {
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

    pub fn is_comparison_operator(operator: &ConditionOperator) -> bool {
        ConditionBuilder::is_comparison_operator(operator)
    }

    pub fn extract_operands(condition: &ConditionInfo) -> Option<ConditionOperands> {
        ConditionBuilder::extract_operands(condition)
    }

    pub fn has_conflicting_comparison_operator(
        new_condition: &ConditionInfo,
        existing_conditions: &[ConditionInfo],
    ) -> bool {
        ConditionBuilder::has_conflicting_comparison_operator(new_condition, existing_conditions)
    }
}

pub use super::builders::condition_builder::ConditionOperands;

pub struct CandidateElements {
    pub indicators: Vec<IndicatorInfo>,
    pub nested_indicators: Vec<NestedIndicator>,
    pub entry_conditions: Vec<ConditionInfo>,
    pub exit_conditions: Vec<ConditionInfo>,
    pub stop_handlers: Vec<StopHandlerInfo>,
    pub take_handlers: Vec<StopHandlerInfo>,
    pub timeframes: Vec<TimeFrame>,
}
