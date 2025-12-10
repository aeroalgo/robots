use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerConfig, StopHandlerInfo};
use rand::seq::SliceRandom;
use rand::Rng;

use super::super::candidate_builder_config::ElementConstraints;
use super::helpers;
use super::CandidateElements;
use super::super::builders::ConditionBuilder;
use super::phase_builder;

pub fn ensure_all_indicators_used(
    indicators: &[IndicatorInfo],
    nested_indicators: &[NestedIndicator],
    entry_conditions: &mut Vec<ConditionInfo>,
    exit_conditions: &[ConditionInfo],
    constraints: &ElementConstraints,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
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
            let condition = build_condition_simple(indicator, true, config, rng);
            if let Some(cond) = condition {
                if !helpers::is_duplicate_condition(&cond, entry_conditions)
                    && !ConditionBuilder::has_conflicting_comparison_operator(&cond, entry_conditions)
                {
                    entry_conditions.push(cond);
                    used_indicators.insert(indicator.alias.clone());
                }
            }
        }
    }
}

fn build_condition_simple(
    indicator: &IndicatorInfo,
    is_entry: bool,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<ConditionInfo> {
    phase_builder::build_condition_simple_with_timeframe(indicator, is_entry, None, config, rng)
}

pub fn ensure_minimum_requirements(
    candidate: &mut CandidateElements,
    constraints: &ElementConstraints,
    available_stop_handlers: &[StopHandlerConfig],
    available_indicators: &[IndicatorInfo],
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) {
    while candidate.stop_handlers.len() < constraints.min_stop_handlers {
        if available_stop_handlers.is_empty() {
            eprintln!("      ⚠️  ВНИМАНИЕ: Нет доступных stop handlers для выполнения min_stop_handlers={}", constraints.min_stop_handlers);
            break;
        }

        if let Some(stop) = helpers::select_single_stop_handler_required(
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
            break;
        }
    }

    while candidate.take_handlers.len() < constraints.min_take_handlers {
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
                optimization_params: helpers::make_handler_params(config_item, available_stop_handlers),
                priority: config_item.priority,
            });
        } else {
            break;
        }
    }

    let indicators = candidate.indicators.clone();
    let nested_indicators = candidate.nested_indicators.clone();
    let probabilities_conditions = config.probabilities.conditions.clone();

    while candidate.entry_conditions.len() < constraints.min_entry_conditions {
        if candidate.entry_conditions.len() >= constraints.max_entry_conditions {
            break;
        }
        if !indicators.is_empty() {
            if let Some(condition) = phase_builder::build_condition(
                &indicators,
                &nested_indicators,
                &probabilities_conditions,
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
            if let Some(condition) = phase_builder::build_condition(
                &indicators,
                &nested_indicators,
                &probabilities_conditions,
                false,
                config,
                rng,
            ) {
                if !helpers::is_duplicate_condition(&condition, &candidate.exit_conditions)
                    && !ConditionBuilder::has_conflicting_comparison_operator(
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
