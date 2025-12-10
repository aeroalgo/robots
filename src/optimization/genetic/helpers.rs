use crate::data_model::types::TimeFrame;
use crate::discovery::StrategyCandidate;
use crate::optimization::builders::ConditionBuilder;
use crate::optimization::candidate_builder::CandidateBuilder;
use crate::optimization::candidate_builder_config::ConditionProbabilities;
use crate::optimization::types::GeneticAlgorithmConfig;
use crate::strategy::types::{ConditionOperator, PriceField};

pub fn remove_unused_indicators(candidate: &mut StrategyCandidate) {
    let used_aliases = get_used_indicator_aliases(candidate);
    let used_timeframes = get_used_timeframes(candidate);

    candidate
        .indicators
        .retain(|ind| used_aliases.contains(&ind.alias));
    candidate
        .nested_indicators
        .retain(|nested| used_aliases.contains(&nested.indicator.alias));
    candidate
        .timeframes
        .retain(|tf| used_timeframes.contains(tf));
}

pub fn get_used_timeframes(candidate: &StrategyCandidate) -> std::collections::HashSet<TimeFrame> {
    let mut used_timeframes = std::collections::HashSet::new();

    for condition in candidate
        .conditions
        .iter()
        .chain(candidate.exit_conditions.iter())
    {
        if let Some(tf) = &condition.primary_timeframe {
            used_timeframes.insert(tf.clone());
        }
        if let Some(tf) = &condition.secondary_timeframe {
            used_timeframes.insert(tf.clone());
        }
    }

    used_timeframes
}

pub fn get_used_indicator_aliases(
    candidate: &StrategyCandidate,
) -> std::collections::HashSet<String> {
    let mut used_aliases = std::collections::HashSet::new();

    for condition in candidate
        .conditions
        .iter()
        .chain(candidate.exit_conditions.iter())
    {
        for alias in condition.all_indicator_aliases() {
            used_aliases.insert(alias);
        }
    }

    for nested in &candidate.nested_indicators {
        used_aliases.insert(nested.input_indicator_alias.clone());
    }

    used_aliases
}

pub fn remove_conditions_with_indicator(candidate: &mut StrategyCandidate, alias: &str) {
    candidate
        .conditions
        .retain(|cond| !cond.all_indicator_aliases().contains(&alias.to_string()));

    candidate
        .exit_conditions
        .retain(|cond| !cond.all_indicator_aliases().contains(&alias.to_string()));
}

pub fn create_condition_for_indicator(
    indicator: &crate::discovery::IndicatorInfo,
    candidate: &StrategyCandidate,
    is_entry: bool,
    config: &GeneticAlgorithmConfig,
    _price_fields: &[PriceField],
    _operators: &[ConditionOperator],
) -> Option<crate::discovery::ConditionInfo> {
    let default_probabilities = ConditionProbabilities::default();
    let probabilities = config
        .candidate_builder_config
        .as_ref()
        .map(|c| &c.probabilities.conditions)
        .unwrap_or(&default_probabilities);

    ConditionBuilder::create_for_candidate_indicator(indicator, candidate, is_entry, probabilities)
}

pub fn flip_operator(operator: &ConditionOperator) -> ConditionOperator {
    operator.opposite()
}

pub fn get_safe_flipped_operator(
    conditions: &[crate::discovery::ConditionInfo],
    idx: usize,
) -> Option<ConditionOperator> {
    let condition = &conditions[idx];

    if condition.condition_type == "trend_condition" {
        return Some(flip_operator(&condition.operator));
    }

    if !CandidateBuilder::is_comparison_operator(&condition.operator) {
        return Some(flip_operator(&condition.operator));
    }

    let new_operator = flip_operator(&condition.operator);

    let mut test_condition = condition.clone();
    test_condition.operator = new_operator.clone();

    let other_conditions: Vec<crate::discovery::ConditionInfo> = conditions
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != idx)
        .map(|(_, c)| c.clone())
        .collect();

    if !CandidateBuilder::has_conflicting_comparison_operator(&test_condition, &other_conditions) {
        Some(new_operator)
    } else {
        None
    }
}

pub fn log_strategy_details(
    candidate: &StrategyCandidate,
    parameters: &crate::strategy::types::StrategyParameterMap,
    label: &str,
) {
    println!("      📋 [{}] Детали стратегии:", label);
    println!("         Индикаторы ({}):", candidate.indicators.len());
    for ind in &candidate.indicators {
        println!("           - {} (alias: {})", ind.name, ind.alias);
        for param in &ind.parameters {
            let param_key = format!("{}_{}", ind.alias, param.name);
            if let Some(value) = parameters.get(&param_key) {
                println!("             {}: {:?}", param.name, value);
            }
        }
    }
    if !candidate.nested_indicators.is_empty() {
        println!(
            "         Вложенные индикаторы ({}):",
            candidate.nested_indicators.len()
        );
        for nested in &candidate.nested_indicators {
            println!(
                "           - {} (alias: {}) <- {} (depth: {})",
                nested.indicator.name,
                nested.indicator.alias,
                nested.input_indicator_alias,
                nested.depth
            );
        }
    }
    println!("         Условия входа ({}):", candidate.conditions.len());
    for cond in &candidate.conditions {
        let tf_info = if let Some(tf) = &cond.primary_timeframe {
            format!(" [TF: {:?}]", tf)
        } else {
            " [TF: base]".to_string()
        };
        println!("           - {} ({}){}", cond.name, cond.id, tf_info);
    }
    if !candidate.exit_conditions.is_empty() {
        println!(
            "         Условия выхода ({}):",
            candidate.exit_conditions.len()
        );
        for cond in &candidate.exit_conditions {
            let tf_info = if let Some(tf) = &cond.primary_timeframe {
                format!(" [TF: {:?}]", tf)
            } else {
                " [TF: base]".to_string()
            };
            println!("           - {} ({}){}", cond.name, cond.id, tf_info);
        }
    }
    if !candidate.timeframes.is_empty() {
        println!("         Таймфреймы: {:?}", candidate.timeframes);
    }
    if !candidate.stop_handlers.is_empty() {
        println!(
            "         Stop handlers ({}):",
            candidate.stop_handlers.len()
        );
        for handler in &candidate.stop_handlers {
            println!("           - {} ({})", handler.name, handler.handler_name);
        }
    }
    if !candidate.take_handlers.is_empty() {
        println!(
            "         Take handlers ({}):",
            candidate.take_handlers.len()
        );
        for handler in &candidate.take_handlers {
            println!("           - {} ({})", handler.name, handler.handler_name);
        }
    }
}

pub fn get_strategy_signature(candidate: &StrategyCandidate) -> String {
    use std::collections::BTreeSet;

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

pub fn update_optimization_params_for_operator(
    condition: &mut crate::discovery::ConditionInfo,
    operator: &ConditionOperator,
) {
    condition.optimization_params =
        crate::discovery::condition::ConditionCombinationGenerator::create_optimization_params_for_operator(operator);
}
