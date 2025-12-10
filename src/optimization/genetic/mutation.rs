use crate::data_model::types::TimeFrame;
use crate::discovery::StopHandlerConfig;
use crate::discovery::StrategyCandidate;
use crate::optimization::candidate_builder::CandidateBuilder;
use crate::optimization::genetic::helpers;
use crate::optimization::types::GeneticAlgorithmConfig;
use crate::strategy::types::{ConditionOperator, PriceField};
use rand::Rng;

pub fn mutate_structure(
    candidate: &mut StrategyCandidate,
    config: &GeneticAlgorithmConfig,
    available_indicators: &[crate::discovery::IndicatorInfo],
    price_fields: &[PriceField],
    operators: &[ConditionOperator],
    stop_handler_configs: &[StopHandlerConfig],
) {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < config.mutation_rate {
        mutate_indicators(
            candidate,
            config,
            available_indicators,
            price_fields,
            operators,
        );
    }

    if rng.gen::<f64>() < config.mutation_rate {
        mutate_entry_conditions(
            candidate,
            config,
            available_indicators,
            price_fields,
            operators,
        );
    }

    if rng.gen::<f64>() < config.mutation_rate {
        mutate_exit_conditions(
            candidate,
            config,
            available_indicators,
            price_fields,
            operators,
        );
    }

    if rng.gen::<f64>() < config.mutation_rate {
        mutate_stop_handlers(candidate, stop_handler_configs);
    }

    if rng.gen::<f64>() < config.mutation_rate {
        mutate_nested_indicators(candidate, available_indicators);
    }

    if rng.gen::<f64>() < config.mutation_rate {
        mutate_take_handlers(candidate, stop_handler_configs);
    }

    if rng.gen::<f64>() < config.mutation_rate * 0.5 {
        mutate_timeframes(candidate);
    }
}

fn mutate_indicators(
    candidate: &mut StrategyCandidate,
    config: &GeneticAlgorithmConfig,
    available_indicators: &[crate::discovery::IndicatorInfo],
    price_fields: &[PriceField],
    operators: &[ConditionOperator],
) {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < 0.3 && !candidate.indicators.is_empty() {
        let idx = rng.gen_range(0..candidate.indicators.len());
        let removed_indicator = &candidate.indicators[idx];
        let removed_type = removed_indicator.indicator_type.clone();
        let removed_alias = removed_indicator.alias.clone();
        candidate.indicators.remove(idx);

        helpers::remove_conditions_with_indicator(candidate, &removed_alias);
        helpers::remove_unused_indicators(candidate);

        let same_type_indicators: Vec<_> = available_indicators
            .iter()
            .filter(|ind| ind.indicator_type == removed_type)
            .collect();

        if !same_type_indicators.is_empty() {
            let new_indicator =
                same_type_indicators[rng.gen_range(0..same_type_indicators.len())].clone();
            candidate.indicators.push(new_indicator.clone());

            if let Some(condition) = helpers::create_condition_for_indicator(
                &new_indicator,
                candidate,
                true,
                config,
                price_fields,
                operators,
            ) {
                if !CandidateBuilder::has_conflicting_comparison_operator(
                    &condition,
                    &candidate.conditions,
                ) {
                    candidate.conditions.push(condition);
                }
            }
        }
    } else if !available_indicators.is_empty() {
        let new_indicator =
            available_indicators[rng.gen_range(0..available_indicators.len())].clone();
        let new_indicator_clone = new_indicator.clone();
        candidate.indicators.push(new_indicator);

        if let Some(condition) = helpers::create_condition_for_indicator(
            &new_indicator_clone,
            candidate,
            true,
            config,
            price_fields,
            operators,
        ) {
            if !CandidateBuilder::has_conflicting_comparison_operator(
                &condition,
                &candidate.conditions,
            ) {
                candidate.conditions.push(condition);
            }
        }
    }
}

fn mutate_entry_conditions(
    candidate: &mut StrategyCandidate,
    config: &GeneticAlgorithmConfig,
    available_indicators: &[crate::discovery::IndicatorInfo],
    price_fields: &[PriceField],
    operators: &[ConditionOperator],
) {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
        let idx = rng.gen_range(0..candidate.conditions.len());
        candidate.conditions.remove(idx);
        helpers::remove_unused_indicators(candidate);
    } else if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
        let idx = rng.gen_range(0..candidate.conditions.len());
        if let Some(new_operator) = helpers::get_safe_flipped_operator(&candidate.conditions, idx) {
            candidate.conditions[idx].operator = new_operator.clone();
            helpers::update_optimization_params_for_operator(
                &mut candidate.conditions[idx],
                &new_operator,
            );
        }
    } else {
        if !available_indicators.is_empty() && !candidate.indicators.is_empty() {
            let indicator = &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];
            if let Some(condition) = helpers::create_condition_for_indicator(
                indicator,
                candidate,
                true,
                config,
                price_fields,
                operators,
            ) {
                if !CandidateBuilder::has_conflicting_comparison_operator(
                    &condition,
                    &candidate.conditions,
                ) {
                    candidate.conditions.push(condition);
                }
            }
        }
    }
}

fn mutate_exit_conditions(
    candidate: &mut StrategyCandidate,
    config: &GeneticAlgorithmConfig,
    available_indicators: &[crate::discovery::IndicatorInfo],
    price_fields: &[PriceField],
    operators: &[ConditionOperator],
) {
    let mut rng = rand::thread_rng();

    let has_exit_conditions = !candidate.exit_conditions.is_empty();
    let has_stop_handlers = !candidate.stop_handlers.is_empty();
    let has_take_handlers = !candidate.take_handlers.is_empty();
    let can_remove_exit = has_exit_conditions
        && (candidate.exit_conditions.len() > 1 || has_stop_handlers || has_take_handlers);

    if rng.gen::<f64>() < 0.3 && can_remove_exit {
        let idx = rng.gen_range(0..candidate.exit_conditions.len());
        let aliases = candidate.exit_conditions[idx].all_indicator_aliases();
        candidate.exit_conditions.remove(idx);
        for alias in aliases {
            helpers::remove_conditions_with_indicator(candidate, &alias);
        }
        helpers::remove_unused_indicators(candidate);
    } else if rng.gen::<f64>() < 0.3 && !candidate.exit_conditions.is_empty() {
        let idx = rng.gen_range(0..candidate.exit_conditions.len());
        if let Some(new_operator) =
            helpers::get_safe_flipped_operator(&candidate.exit_conditions, idx)
        {
            candidate.exit_conditions[idx].operator = new_operator.clone();
            helpers::update_optimization_params_for_operator(
                &mut candidate.exit_conditions[idx],
                &new_operator,
            );
        }
    } else {
        if !available_indicators.is_empty() && !candidate.indicators.is_empty() {
            let indicator = &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];
            if let Some(condition) = helpers::create_condition_for_indicator(
                indicator,
                candidate,
                false,
                config,
                price_fields,
                operators,
            ) {
                if !CandidateBuilder::has_conflicting_comparison_operator(
                    &condition,
                    &candidate.exit_conditions,
                ) {
                    candidate.exit_conditions.push(condition);
                }
            }
        }
    }
}

fn mutate_stop_handlers(
    candidate: &mut StrategyCandidate,
    stop_handler_configs: &[StopHandlerConfig],
) {
    let mut rng = rand::thread_rng();

    let has_exit_conditions = !candidate.exit_conditions.is_empty();
    let has_stop_handlers = !candidate.stop_handlers.is_empty();
    let has_take_handlers = !candidate.take_handlers.is_empty();
    let can_remove_stop = has_stop_handlers
        && (candidate.stop_handlers.len() > 1 || has_exit_conditions || has_take_handlers);

    if rng.gen::<f64>() < 0.3 && can_remove_stop {
        let idx = rng.gen_range(0..candidate.stop_handlers.len());
        candidate.stop_handlers.remove(idx);
    } else if !stop_handler_configs.is_empty() {
        let stop_configs: Vec<&StopHandlerConfig> = stop_handler_configs
            .iter()
            .filter(|c| c.stop_type == "stop_loss")
            .collect();
        if !stop_configs.is_empty() {
            let stop_config = stop_configs[rng.gen_range(0..stop_configs.len())];
            let stop = crate::discovery::types::StopHandlerInfo {
                id: format!("stop_{}", rng.gen::<u32>()),
                name: stop_config.handler_name.clone(),
                handler_name: stop_config.handler_name.clone(),
                stop_type: stop_config.stop_type.clone(),
                optimization_params: Vec::new(),
                priority: stop_config.priority,
            };
            candidate.stop_handlers.push(stop);
        }
    }
}

fn mutate_nested_indicators(
    candidate: &mut StrategyCandidate,
    available_indicators: &[crate::discovery::IndicatorInfo],
) {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < 0.2 && !candidate.nested_indicators.is_empty() {
        let idx = rng.gen_range(0..candidate.nested_indicators.len());
        let removed_nested = &candidate.nested_indicators[idx];
        let removed_type = removed_nested.indicator.indicator_type.clone();
        let removed_alias = removed_nested.indicator.alias.clone();
        candidate.nested_indicators.remove(idx);

        helpers::remove_conditions_with_indicator(candidate, &removed_alias);
        helpers::remove_unused_indicators(candidate);

        let same_type_indicators: Vec<_> = available_indicators
            .iter()
            .filter(|ind| ind.indicator_type == removed_type)
            .collect();

        if !same_type_indicators.is_empty() && !candidate.indicators.is_empty() {
            let new_indicator =
                same_type_indicators[rng.gen_range(0..same_type_indicators.len())].clone();
            let input_indicator =
                &candidate.indicators[rng.gen_range(0..candidate.indicators.len())];

            let new_nested = crate::discovery::NestedIndicator {
                indicator: new_indicator,
                input_indicator_alias: input_indicator.alias.clone(),
                depth: 1,
            };
            candidate.nested_indicators.push(new_nested);
        }
    }
}

fn mutate_take_handlers(
    candidate: &mut StrategyCandidate,
    stop_handler_configs: &[StopHandlerConfig],
) {
    let mut rng = rand::thread_rng();

    let has_exit_conditions = !candidate.exit_conditions.is_empty();
    let has_stop_handlers = !candidate.stop_handlers.is_empty();
    let has_take_handlers = !candidate.take_handlers.is_empty();
    let can_remove_take = has_take_handlers
        && (candidate.take_handlers.len() > 1 || has_exit_conditions || has_stop_handlers);

    if rng.gen::<f64>() < 0.3 && can_remove_take {
        let idx = rng.gen_range(0..candidate.take_handlers.len());
        candidate.take_handlers.remove(idx);
    } else if !stop_handler_configs.is_empty() {
        let can_add_take = has_exit_conditions || has_stop_handlers;

        if can_add_take {
            let take_configs: Vec<&StopHandlerConfig> = stop_handler_configs
                .iter()
                .filter(|c| c.stop_type == "take_profit")
                .collect();
            if !take_configs.is_empty() {
                let take_config = take_configs[rng.gen_range(0..take_configs.len())];
                let take = crate::discovery::types::StopHandlerInfo {
                    id: format!("take_{}", rng.gen::<u32>()),
                    name: take_config.handler_name.clone(),
                    handler_name: take_config.handler_name.clone(),
                    stop_type: take_config.stop_type.clone(),
                    optimization_params: Vec::new(),
                    priority: take_config.priority,
                };
                candidate.take_handlers.push(take);
            }
        }
    }
}

fn mutate_timeframes(candidate: &mut StrategyCandidate) {
    let mut rng = rand::thread_rng();
    let base_tf = &candidate.config.base_timeframe;
    let base_duration = base_tf.duration();

    let all_timeframes = vec![
        TimeFrame::from_identifier("1"),
        TimeFrame::from_identifier("5"),
        TimeFrame::from_identifier("15"),
        TimeFrame::from_identifier("30"),
        TimeFrame::from_identifier("60"),
        TimeFrame::from_identifier("240"),
        TimeFrame::from_identifier("D"),
    ];

    let available_timeframes: Vec<TimeFrame> = if let Some(base_dur) = base_duration {
        all_timeframes
            .into_iter()
            .filter(|tf| {
                if let Some(tf_dur) = tf.duration() {
                    tf_dur >= base_dur
                } else {
                    false
                }
            })
            .collect()
    } else {
        all_timeframes
    };

    if !candidate.timeframes.is_empty() && rng.gen::<f64>() < 0.5 {
        let idx = rng.gen_range(0..candidate.timeframes.len());
        candidate.timeframes.remove(idx);
    } else if !available_timeframes.is_empty() {
        let new_tf = available_timeframes[rng.gen_range(0..available_timeframes.len())].clone();
        if !candidate.timeframes.contains(&new_tf) {
            candidate.timeframes.push(new_tf);
        }
    }
}
