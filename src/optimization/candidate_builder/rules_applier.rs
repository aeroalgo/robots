use crate::discovery::types::{StopHandlerConfig, StopHandlerInfo};
use rand::Rng;

use super::super::candidate_builder_config::{ElementSelector, RuleAction, RuleCondition};
use super::helpers;
use super::CandidateElements;

pub fn apply_rules(
    candidate: &mut CandidateElements,
    available_stop_handlers: &[StopHandlerConfig],
    rules: &super::super::candidate_builder_config::BuildRules,
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) {
    for dependency in &rules.dependencies {
        if matches_selector(&dependency.trigger, candidate) {
            if !matches_selector(&dependency.required, candidate) {
                if dependency.strict {
                    add_required_element(
                        &dependency.required,
                        candidate,
                        available_stop_handlers,
                        config,
                        rng,
                    );
                }
            }
        }
    }

    for exclusion in &rules.exclusions {
        if matches_selector(&exclusion.element, candidate) {
            remove_excluded_element(&exclusion.excluded, candidate);
        }
    }

    for conditional in &rules.conditions {
        if evaluate_condition(&conditional.condition, candidate) {
            apply_action(&conditional.action, candidate, available_stop_handlers, config, rng);
        }
    }
}

pub fn matches_selector(selector: &ElementSelector, candidate: &CandidateElements) -> bool {
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

pub fn evaluate_condition(
    condition: &RuleCondition,
    candidate: &CandidateElements,
) -> bool {
    match condition {
        RuleCondition::And { conditions } => conditions
            .iter()
            .all(|c| matches_selector(c, candidate)),
        RuleCondition::Or { conditions } => conditions
            .iter()
            .any(|c| matches_selector(c, candidate)),
        RuleCondition::Not { condition } => {
            !matches_selector(condition, candidate)
        }
        RuleCondition::Element { element } => {
            matches_selector(element, candidate)
        }
    }
}

pub fn apply_action(
    action: &RuleAction,
    candidate: &mut CandidateElements,
    available_stop_handlers: &[StopHandlerConfig],
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) {
    match action {
        RuleAction::Require { element, strict } => {
            if !matches_selector(element, candidate) {
                if *strict {
                    add_required_element(element, candidate, available_stop_handlers, config, rng);
                }
            }
        }
        RuleAction::Exclude { element } => {
            remove_excluded_element(element, candidate);
        }
        RuleAction::SetProbability { .. } => {}
    }
}

pub fn add_required_element(
    selector: &ElementSelector,
    candidate: &mut CandidateElements,
    available_stop_handlers: &[StopHandlerConfig],
    config: &super::super::candidate_builder_config::CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) {
    match selector {
        ElementSelector::TakeHandler { name } => {
            if candidate.take_handlers.len() >= config.constraints.max_take_handlers {
                return;
            }

            if candidate
                .take_handlers
                .iter()
                .any(|h| &h.handler_name == name)
            {
                return;
            }

            if let Some(config_item) = available_stop_handlers
                .iter()
                .find(|c| c.handler_name == *name && c.stop_type == "take_profit")
            {
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
                return;
            }

            candidate.take_handlers.push(StopHandlerInfo {
                id: format!("take_{}", rng.gen::<u32>()),
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

            if let Some(config_item) = available_stop_handlers
                .iter()
                .find(|c| c.handler_name == *name && c.stop_type == "stop_loss")
            {
                candidate.stop_handlers.push(StopHandlerInfo {
                    id: format!("stop_{}", rng.gen::<u32>()),
                    name: config_item.handler_name.clone(),
                    handler_name: config_item.handler_name.clone(),
                    stop_type: config_item.stop_type.clone(),
                    optimization_params: helpers::make_handler_params(
                        config_item,
                        available_stop_handlers,
                    ),
                    priority: config_item.priority,
                });
                return;
            }

            candidate.stop_handlers.push(StopHandlerInfo {
                id: format!("stop_{}", rng.gen::<u32>()),
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

pub fn remove_excluded_element(
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
