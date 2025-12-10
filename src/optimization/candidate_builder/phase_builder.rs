use crate::condition::ConditionParameterPresets;
use crate::data_model::types::TimeFrame;
use crate::discovery::types::{
    ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};
use crate::strategy::types::ConditionOperator;
use rand::seq::SliceRandom;
use rand::Rng;

use super::super::builders::ConditionBuilder;
use super::super::candidate_builder_config::{ElementConstraints, ElementProbabilities};
use super::helpers;
use super::CandidateElements;

pub fn build_phase_1(
    candidate: &mut CandidateElements,
    available_indicators: &[IndicatorInfo],
    available_stop_handlers: &[StopHandlerConfig],
    available_timeframes: &[TimeFrame],
    constraints: &ElementConstraints,
    probabilities: &ElementProbabilities,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) {
    if !available_timeframes.is_empty() {
        candidate.timeframes.push(available_timeframes[0].clone());
    }

    let exclude_aliases: Vec<String> = candidate
        .indicators
        .iter()
        .map(|i| i.alias.clone())
        .collect();
    if let Some(indicator) = helpers::select_single_indicator(
        available_indicators,
        probabilities,
        &exclude_aliases,
        true,
        config,
        rng,
    ) {
        candidate.indicators.push(indicator);
    }

    try_add_nested_indicator(candidate, available_indicators, config, rng);

    if !candidate.indicators.is_empty()
        && candidate.entry_conditions.len() < constraints.max_entry_conditions
    {
        if let Some(condition) = build_condition(
            &candidate.indicators,
            &candidate.nested_indicators,
            &probabilities.conditions,
            true,
            config,
            rng,
        ) {
            if !helpers::is_duplicate_condition(&condition, &candidate.entry_conditions)
                && !ConditionBuilder::has_conflicting_comparison_operator(
                    &condition,
                    &candidate.entry_conditions,
                )
            {
                candidate.entry_conditions.push(condition);
            }
        }
    }

    if candidate.stop_handlers.is_empty() && constraints.min_stop_handlers > 0 {
        if available_stop_handlers.is_empty() {
            eprintln!("      ⚠️  ВНИМАНИЕ: Нет доступных stop handlers для добавления!");
        } else if let Some(stop) = helpers::select_single_stop_handler_required(
            available_stop_handlers,
            available_indicators,
            config,
            rng,
        ) {
            candidate.stop_handlers.push(stop);
        } else {
            eprintln!(
                "      ⚠️  ВНИМАНИЕ: Не удалось выбрать stop handler из {} доступных",
                available_stop_handlers.len()
            );
        }
    }

    if candidate.take_handlers.is_empty()
        && candidate.take_handlers.len() < constraints.max_take_handlers
        && helpers::should_add(probabilities.take_handlers.add_take_profit, rng)
    {
        let take_configs: Vec<&StopHandlerConfig> = available_stop_handlers
            .iter()
            .filter(|c| c.stop_type == "take_profit")
            .collect();

        if let Some(config_item) = take_configs.choose(rng) {
            candidate.take_handlers.push(StopHandlerInfo {
                id: format!("take_{}", rng.gen::<u32>()),
                name: config_item.handler_name.clone(),
                handler_name: config_item.handler_name.clone(),
                stop_type: config_item.stop_type.clone(),
                optimization_params: helpers::make_handler_params(
                    config_item,
                    available_stop_handlers,
                ),
                priority: config_item.priority,
            });
        }
    }
}

pub fn build_additional_phase(
    candidate: &mut CandidateElements,
    available_indicators: &[IndicatorInfo],
    _available_stop_handlers: &[StopHandlerConfig],
    available_timeframes: &[TimeFrame],
    constraints: &ElementConstraints,
    probabilities: &ElementProbabilities,
    _phase: usize,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
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

    let mut added_higher_tf_indicator = false;
    if candidate.timeframes.len() < constraints.max_timeframes
        && candidate.indicators.len() < constraints.max_indicators
        && candidate.entry_conditions.len() < constraints.max_entry_conditions
        && available_timeframes.len() > 1
        && helpers::should_add(probabilities.timeframes.use_higher_timeframe, rng)
    {
        let higher_timeframes: Vec<&TimeFrame> = available_timeframes
            .iter()
            .skip(1)
            .filter(|tf| !candidate.timeframes.contains(tf))
            .collect();

        if let Some(higher_tf) = higher_timeframes.choose(rng) {
            let exclude_aliases: Vec<String> = candidate
                .indicators
                .iter()
                .map(|i| i.alias.clone())
                .collect();

            if let Some(mut indicator) = helpers::select_single_indicator(
                available_indicators,
                probabilities,
                &exclude_aliases,
                false,
                config,
                rng,
            ) {
                let higher_tf_minutes = higher_tf.total_minutes().unwrap_or(0);
                indicator.alias = format!("{}_{}", indicator.alias, higher_tf_minutes);

                let condition = build_condition_simple_with_timeframe(
                    &indicator,
                    true,
                    Some((*higher_tf).clone()),
                    config,
                    rng,
                );

                if let Some(cond) = condition {
                    if !helpers::is_duplicate_condition(&cond, &candidate.entry_conditions)
                        && !ConditionBuilder::has_conflicting_comparison_operator(
                            &cond,
                            &candidate.entry_conditions,
                        )
                    {
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
        && helpers::should_add(0.5, rng)
    {
        let exclude_aliases: Vec<String> = candidate
            .indicators
            .iter()
            .map(|i| i.alias.clone())
            .collect();
        if let Some(indicator) = helpers::select_single_indicator(
            available_indicators,
            probabilities,
            &exclude_aliases,
            false,
            config,
            rng,
        ) {
            candidate.indicators.push(indicator);
        }
    }

    try_add_nested_indicator(candidate, available_indicators, config, rng);

    if candidate.entry_conditions.len() < constraints.max_entry_conditions
        && helpers::should_add(probabilities.conditions.add_entry_condition, rng)
    {
        let condition = build_condition(
            &candidate.indicators,
            &candidate.nested_indicators,
            &probabilities.conditions,
            true,
            config,
            rng,
        );

        if let Some(cond) = condition {
            if !helpers::is_duplicate_condition(&cond, &candidate.entry_conditions)
                && !ConditionBuilder::has_conflicting_comparison_operator(
                    &cond,
                    &candidate.entry_conditions,
                )
            {
                candidate.entry_conditions.push(cond);
            }
        }
    }

    if candidate.exit_conditions.len() < constraints.max_exit_conditions
        && helpers::should_add(probabilities.phases.add_exit_rules, rng)
    {
        let condition = build_condition(
            &candidate.indicators,
            &candidate.nested_indicators,
            &probabilities.conditions,
            false,
            config,
            rng,
        );

        if let Some(cond) = condition {
            if !helpers::is_duplicate_condition(&cond, &candidate.exit_conditions)
                && !ConditionBuilder::has_conflicting_comparison_operator(
                    &cond,
                    &candidate.exit_conditions,
                )
            {
                candidate.exit_conditions.push(cond);
            }
        }
    }

    false
}

pub fn try_add_nested_indicator(
    candidate: &mut CandidateElements,
    available_indicators: &[IndicatorInfo],
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) {
    let add_nested_prob = config.probabilities.nested_indicators.add_nested_indicator;
    let max_depth = config.probabilities.nested_indicators.max_nesting_depth;

    if !helpers::should_add(add_nested_prob, rng) {
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

    let Some(input_indicator) = base_indicators.choose(rng) else {
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

    use super::super::build_rules_provider::can_accept_nested_input;
    let nestable_indicators: Vec<&IndicatorInfo> = available_indicators
        .iter()
        .filter(|ind| can_accept_nested_input(&ind.name))
        .filter(|ind| !config.rules.excluded_indicators.contains(&ind.name))
        .collect();

    let Some(wrapper_template) = nestable_indicators.choose(rng) else {
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

pub fn build_condition_simple_with_timeframe(
    indicator: &IndicatorInfo,
    is_entry: bool,
    timeframe: Option<TimeFrame>,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<ConditionInfo> {
    use super::super::build_rules_provider::{
        has_absolute_threshold, has_percent_of_price_threshold,
    };
    use crate::condition::ConditionParameterPresets;

    let operator = if timeframe.is_some() {
        match rng.gen_range(0..4) {
            0 => ConditionOperator::Above,
            1 => ConditionOperator::Below,
            2 => ConditionOperator::RisingTrend,
            _ => ConditionOperator::FallingTrend,
        }
    } else {
        if rng.gen_bool(0.5) {
            ConditionOperator::Above
        } else {
            ConditionOperator::Below
        }
    };

    if matches!(
        operator,
        ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
    ) {
        let trend_range = ConditionParameterPresets::trend_period();
        let period = rng.gen_range(trend_range.min..=trend_range.max);
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
            rng.gen::<u32>()
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
                mutatable: true,
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
        rng.gen::<u32>()
    );

    let (condition_type, condition_name, constant_value, price_field, optimization_params) =
        if has_absolute_threshold(&indicator.name) {
            let const_val = if indicator.name == "RSI" {
                if operator == ConditionOperator::Above {
                    rng.gen_range(70.0..=90.0)
                } else {
                    rng.gen_range(10.0..=30.0)
                }
            } else if indicator.name == "Stochastic" {
                if operator == ConditionOperator::Above {
                    rng.gen_range(80.0..=95.0)
                } else {
                    rng.gen_range(5.0..=20.0)
                }
            } else {
                rng.gen_range(0.0..=100.0)
            };
            (
                "indicator_constant".to_string(),
                format!("{} {:?} {:.1}", indicator.name, operator, const_val),
                Some(const_val),
                None,
                Vec::new(),
            )
        } else if has_percent_of_price_threshold(&indicator.name) {
            let rules = &config.rules.indicator_parameter_rules;
            let mut percentage_range = (0.2, 10.0, 0.1);

            for rule in rules {
                if rule.indicator_type == "volatility" {
                    if !rule.indicator_names.is_empty() {
                        if !rule.indicator_names.contains(&indicator.name) {
                            continue;
                        }
                    }
                    if let Some(constraint) = &rule.price_field_constraint {
                        if let super::super::candidate_builder_config::ParameterConstraint::PercentageFromPrice {
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

            let steps = ((percentage_range.1 - percentage_range.0) / percentage_range.2) as usize;
            let step_index = rng.gen_range(0..=steps);
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
                    mutatable: true,
                    global_param_name: None,
                }],
            )
        } else {
            let price_field = if config.condition_config.price_fields.len() == 1 {
                config.condition_config.price_fields[0].clone()
            } else {
                config
                    .condition_config
                    .price_fields
                    .choose(rng)
                    .cloned()
                    .unwrap_or_else(|| "Close".to_string())
            };

            let probabilities = &config.probabilities.conditions;
            let (optimization_params, constant_value) =
                if helpers::should_add(probabilities.use_percent_condition, rng) {
                    let percent_value = rng.gen_range(0.1..=5.0);
                    (
                        vec![crate::discovery::ConditionParamInfo {
                            name: "percentage".to_string(),
                            optimizable: true,
                            mutatable: true,
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

pub fn build_condition(
    indicators: &[IndicatorInfo],
    nested_indicators: &[NestedIndicator],
    probabilities: &super::super::candidate_builder_config::ConditionProbabilities,
    is_entry: bool,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<ConditionInfo> {
    use super::super::build_rules_provider::{
        get_allowed_conditions, has_absolute_threshold, has_percent_of_price_threshold,
    };

    let all_indicators: Vec<&IndicatorInfo> = indicators
        .iter()
        .chain(nested_indicators.iter().map(|n| &n.indicator))
        .collect();

    if all_indicators.is_empty() {
        return None;
    }

    let Some(primary_indicator) = all_indicators.choose(rng) else {
        return None;
    };

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

    let flat_base_indicators: Vec<IndicatorInfo> =
        indicators.iter().map(|ind| ind.clone()).collect();

    let available_secondary_for_indicator_indicator: Vec<&IndicatorInfo> = all_indicators
        .iter()
        .filter(|ind| ind.alias != primary_indicator.alias)
        .filter(|ind| {
            helpers::can_compare_indicators(
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
        && !helpers::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
    {
        "indicator_constant"
    } else if has_percent_of_price_threshold(&primary_indicator.name) {
        "indicator_constant"
    } else if is_built_on_oscillator {
        helpers::weighted_choice_for_oscillator_based(probabilities, rng)
    } else {
        let chosen_type = helpers::weighted_condition_type_choice(probabilities, rng);
        if chosen_type == "indicator_indicator" && !has_available_secondary {
            if rng.gen_bool(
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

    let allowed_conditions = get_allowed_conditions(&primary_indicator.name);

    let operator = if condition_type == "trend_condition" {
        let trend_ops: Vec<_> = allowed_conditions
            .iter()
            .filter(|op| {
                matches!(
                    op,
                    ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
                )
            })
            .collect();
        if let Some(op) = trend_ops.choose(rng) {
            (*op).clone()
        } else if rng.gen_bool(0.5) {
            ConditionOperator::RisingTrend
        } else {
            ConditionOperator::FallingTrend
        }
    } else {
        let non_trend_ops: Vec<_> = allowed_conditions
            .iter()
            .filter(|op| {
                !matches!(
                    op,
                    ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
                )
            })
            .collect();
        if let Some(op) = non_trend_ops.choose(rng) {
            (*op).clone()
        } else if rng.gen_bool(0.5) {
            ConditionOperator::Above
        } else {
            ConditionOperator::Below
        }
    };

    let (condition_id, condition_name) = if condition_type == "indicator_constant" {
        let constant_value = if has_percent_of_price_threshold(&primary_indicator.name) {
            let rules = &config.rules.indicator_parameter_rules;
            let mut percentage_range = (0.2, 10.0, 0.1);

            for rule in rules {
                if rule.indicator_type == "volatility" {
                    if !rule.indicator_names.is_empty() {
                        if !rule.indicator_names.contains(&primary_indicator.name) {
                            continue;
                        }
                    }
                    if let Some(constraint) = &rule.price_field_constraint {
                        if let super::super::candidate_builder_config::ParameterConstraint::PercentageFromPrice {
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

            let steps = ((percentage_range.1 - percentage_range.0) / percentage_range.2) as usize;
            let step_index = rng.gen_range(0..=steps);
            percentage_range.0 + (step_index as f64 * percentage_range.2)
        } else if primary_indicator.name == "RSI" {
            if operator == ConditionOperator::Above {
                rng.gen_range(70.0..=90.0)
            } else {
                rng.gen_range(10.0..=30.0)
            }
        } else if primary_indicator.name == "Stochastic" {
            if operator == ConditionOperator::Above {
                rng.gen_range(80.0..=95.0)
            } else {
                rng.gen_range(5.0..=20.0)
            }
        } else {
            rng.gen_range(0.0..=100.0)
        };

        let id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            rng.gen::<u32>()
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
        use crate::condition::ConditionParameterPresets;
        let trend_range = ConditionParameterPresets::trend_period();
        let period = rng.gen_range(trend_range.min..=trend_range.max);
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
            rng.gen::<u32>()
        );
        let name = format!(
            "{} {} (period: {:.0})",
            primary_indicator.name, trend_name, period
        );
        (id, name)
    } else if condition_type == "indicator_indicator" {
        let flat_base_indicators: Vec<IndicatorInfo> =
            indicators.iter().map(|ind| ind.clone()).collect();

        let available_secondary: Vec<&IndicatorInfo> = all_indicators
            .iter()
            .filter(|ind| ind.alias != primary_indicator.alias)
            .filter(|ind| {
                helpers::can_compare_indicators(
                    primary_indicator,
                    *ind,
                    nested_indicators,
                    &flat_base_indicators,
                )
            })
            .copied()
            .collect();

        if let Some(secondary) = available_secondary.choose(rng) {
            let id = format!(
                "{}_{}::{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                secondary.alias,
                rng.gen::<u32>()
            );
            let name = format!(
                "{} {:?} {}",
                primary_indicator.name, operator, secondary.name
            );
            (id, name)
        } else {
            let id = format!(
                "{}_{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                rng.gen::<u32>()
            );
            let name = format!("{} {:?} {}", primary_indicator.name, operator, "target");
            (id, name)
        }
    } else {
        if has_absolute_threshold(&primary_indicator.name)
            && !helpers::is_oscillator_used_in_nested(primary_indicator, nested_indicators)
        {
            let const_val = if primary_indicator.name == "RSI" {
                if operator == ConditionOperator::Above {
                    rng.gen_range(70.0..=90.0)
                } else {
                    rng.gen_range(10.0..=30.0)
                }
            } else if primary_indicator.name == "Stochastic" {
                if operator == ConditionOperator::Above {
                    rng.gen_range(80.0..=95.0)
                } else {
                    rng.gen_range(5.0..=20.0)
                }
            } else {
                rng.gen_range(0.0..=100.0)
            };
            let id = format!(
                "{}_{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                rng.gen::<u32>()
            );
            let name = format!("{} {:?} {:.1}", primary_indicator.name, operator, const_val);
            (id, name)
        } else {
            let id = format!(
                "{}_{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                rng.gen::<u32>()
            );
            let name = format!("{} {:?} {}", primary_indicator.name, operator, "target");
            (id, name)
        }
    };

    let price_field = if condition_type == "indicator_price" {
        if has_percent_of_price_threshold(&primary_indicator.name) {
            let rules = &config.rules.indicator_parameter_rules;
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
                        if let Some(constraint) = &rule.price_field_constraint {
                            required_price_field = Some(constraint.required_price_field.clone());
                            break;
                        }
                    }
                }
            }

            Some(required_price_field.unwrap_or_else(|| "Close".to_string()))
        } else if config.condition_config.price_fields.len() == 1 {
            Some(config.condition_config.price_fields[0].clone())
        } else {
            config
                .condition_config
                .price_fields
                .choose(rng)
                .cloned()
                .or(Some("Close".to_string()))
        }
    } else {
        None
    };

    let final_condition_type = if condition_type == "indicator_indicator" {
        if condition_id.contains("::") {
            condition_type
        } else {
            "indicator_price"
        }
    } else if condition_type == "indicator_price" {
        if (has_absolute_threshold(&primary_indicator.name)
            && !helpers::is_oscillator_used_in_nested(primary_indicator, nested_indicators))
            || has_percent_of_price_threshold(&primary_indicator.name)
        {
            "indicator_constant"
        } else {
            condition_type
        }
    } else {
        condition_type
    };

    let (constant_value, trend_period) = if final_condition_type == "indicator_constant" {
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
            let parts: Vec<&str> = condition_name.split_whitespace().collect();
            if parts.len() >= 3 {
                parts.last().and_then(|s| s.parse::<f64>().ok())
            } else {
                None
            }
        };
        (parsed_value, None)
    } else if final_condition_type == "trend_condition" {
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

    let (optimization_params, constant_value_for_percent) = if matches!(
        operator,
        ConditionOperator::LowerPercent | ConditionOperator::GreaterPercent
    ) {
        let percent_value = rng.gen_range(0.1..=5.0);
        (
            vec![crate::discovery::ConditionParamInfo {
                name: "percent".to_string(),
                optimizable: true,
                mutatable: true,
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
                mutatable: true,
                global_param_name: None,
            }],
            constant_value,
        )
    } else if final_condition_type == "trend_condition" && trend_period.is_some() {
        (
            vec![crate::discovery::ConditionParamInfo {
                name: "period".to_string(),
                optimizable: true,
                mutatable: true,
                global_param_name: None,
            }],
            trend_period,
        )
    } else if (final_condition_type == "indicator_price"
        || final_condition_type == "indicator_indicator")
        && helpers::should_add(probabilities.use_percent_condition, rng)
    {
        let percent_value = rng.gen_range(0.1..=5.0);
        (
            vec![crate::discovery::ConditionParamInfo {
                name: "percentage".to_string(),
                optimizable: true,
                mutatable: true,
                global_param_name: None,
            }],
            Some(percent_value),
        )
    } else {
        (Vec::new(), constant_value)
    };

    let final_condition_name = if !optimization_params.is_empty()
        && final_condition_type != "indicator_constant"
        && final_condition_type != "trend_condition"
    {
        if let Some(percent) = constant_value_for_percent {
            if final_condition_type == "indicator_indicator" {
                format!("{} на {:.2}%", condition_name, percent)
            } else {
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
            if config.condition_config.price_fields.len() == 1 {
                Some(config.condition_config.price_fields[0].clone())
            } else {
                config
                    .condition_config
                    .price_fields
                    .choose(rng)
                    .cloned()
                    .or(Some("Close".to_string()))
            }
        } else {
            price_field
        },
    })
}
