use crate::condition::ConditionParameterPresets;
use crate::data_model::types::TimeFrame;
use crate::discovery::types::{
    ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerConfig, StopHandlerInfo,
};
use crate::strategy::types::ConditionOperator;
use rand::seq::SliceRandom;
use rand::Rng;

use super::super::build_rules_provider::{
    can_accept_nested_input, get_allowed_conditions, has_absolute_threshold,
    has_percent_of_price_threshold, is_oscillator_like, is_phase_1_allowed,
};
use super::super::builders::ConditionBuilder;
use super::super::candidate_builder_config::{CandidateBuilderConfig, ElementProbabilities};

pub fn make_handler_params(
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
                    mutatable: true,
                    global_param_name: cfg.global_param_name.clone(),
                });
            }
        }
    }

    params
}

pub fn select_single_indicator(
    available: &[IndicatorInfo],
    probabilities: &ElementProbabilities,
    exclude_aliases: &[String],
    is_phase_1: bool,
    config: &CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<IndicatorInfo> {
    let exclude_set: std::collections::HashSet<&str> =
        exclude_aliases.iter().map(|s| s.as_str()).collect();

    let excluded_indicators: Vec<String> = config.rules.excluded_indicators.clone();
    let excluded_indicators_set: std::collections::HashSet<&str> =
        excluded_indicators.iter().map(|s| s.as_str()).collect();

    let available_filtered: Vec<&IndicatorInfo> = available
        .iter()
        .filter(|indicator| !exclude_set.contains(indicator.alias.as_str()))
        .filter(|indicator| {
            if excluded_indicators_set.contains(indicator.name.as_str()) {
                return false;
            }
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
            should_add(weight, rng)
        })
        .collect();

    available_filtered.choose(rng).map(|ind| (*ind).clone())
}

pub fn select_single_stop_handler_required(
    available: &[StopHandlerConfig],
    available_indicators: &[IndicatorInfo],
    config: &CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<StopHandlerInfo> {
    let excluded_stop_handlers: std::collections::HashSet<&str> = config
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

    stop_loss_configs.choose(rng).map(|config_item| {
        let mut handler_name = config_item.handler_name.clone();

        if config_item.handler_name == "ATRTrailIndicatorStop"
            || config_item.handler_name == "PercentTrailIndicatorStop"
        {
            if let Some(indicator_name) = select_random_trend_indicator(available_indicators, rng) {
                handler_name = format!("{}:{}", config_item.handler_name, indicator_name);
            }
        }

        StopHandlerInfo {
            id: format!("stop_{}", rng.gen::<u32>()),
            name: handler_name.clone(),
            handler_name: handler_name,
            stop_type: config_item.stop_type.clone(),
            optimization_params: make_handler_params(config_item, available),
            priority: config_item.priority,
        }
    })
}

pub fn select_random_trend_indicator(
    available_indicators: &[IndicatorInfo],
    rng: &mut rand::rngs::ThreadRng,
) -> Option<String> {
    let trend_indicators: Vec<&IndicatorInfo> = available_indicators
        .iter()
        .filter(|ind| ind.indicator_type == "trend")
        .collect();

    if trend_indicators.is_empty() {
        let default_trend_indicators = vec!["SMA", "EMA", "WMA", "AMA", "ZLEMA"];
        default_trend_indicators.choose(rng).map(|s| s.to_string())
    } else {
        trend_indicators.choose(rng).map(|ind| ind.name.clone())
    }
}

pub fn build_condition_with_timeframe(
    indicator: &IndicatorInfo,
    is_entry: bool,
    timeframe: Option<TimeFrame>,
    config: &CandidateBuilderConfig,
    rng: &mut rand::rngs::ThreadRng,
) -> Option<ConditionInfo> {
    let operator = if rng.gen_bool(0.5) {
        ConditionOperator::Above
    } else {
        ConditionOperator::Below
    };

    let condition_id = format!(
        "{}_{}_{}",
        if is_entry { "entry" } else { "exit" },
        indicator.alias,
        rng.gen::<u32>()
    );

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

pub fn should_add(probability: f64, rng: &mut rand::rngs::ThreadRng) -> bool {
    rng.gen_bool(probability.clamp(0.0, 1.0))
}

pub fn weighted_condition_type_choice(
    probabilities: &super::super::candidate_builder_config::ConditionProbabilities,
    rng: &mut rand::rngs::ThreadRng,
) -> &'static str {
    let w_price = probabilities.use_indicator_price_condition;
    let w_indicator = probabilities.use_indicator_indicator_condition;
    let w_trend = probabilities.use_trend_condition;
    let total = w_price + w_indicator + w_trend;

    if total <= 0.0 {
        return "indicator_price";
    }

    let random = rng.gen::<f64>() * total;

    if random < w_price {
        "indicator_price"
    } else if random < w_price + w_indicator {
        "indicator_indicator"
    } else {
        "trend_condition"
    }
}

pub fn weighted_choice_for_oscillator_based(
    probabilities: &super::super::candidate_builder_config::ConditionProbabilities,
    rng: &mut rand::rngs::ThreadRng,
) -> &'static str {
    let w_indicator = probabilities.use_indicator_indicator_condition;
    let w_trend = probabilities.use_trend_condition;
    let total = w_indicator + w_trend;

    if total <= 0.0 {
        return "indicator_indicator";
    }

    let random = rng.gen::<f64>() * total;

    if random < w_indicator {
        "indicator_indicator"
    } else {
        "trend_condition"
    }
}

pub fn is_oscillator_used_in_nested(
    indicator: &IndicatorInfo,
    nested_indicators: &[NestedIndicator],
) -> bool {
    nested_indicators
        .iter()
        .any(|nested| nested.input_indicator_alias == indicator.alias)
}

pub fn is_duplicate_condition(
    new_condition: &ConditionInfo,
    existing_conditions: &[ConditionInfo],
) -> bool {
    for existing in existing_conditions {
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

        if existing.condition_type == "indicator_price" {
            if existing.price_field != new_condition.price_field {
                continue;
            }
        }

        if existing.condition_type == "indicator_constant" {
            if existing.constant_value != new_condition.constant_value {
                continue;
            }
        }

        if existing.primary_timeframe != new_condition.primary_timeframe {
            continue;
        }

        if existing.secondary_timeframe != new_condition.secondary_timeframe {
            continue;
        }

        return true;
    }
    false
}

pub fn can_compare_indicators(
    primary: &IndicatorInfo,
    secondary: &IndicatorInfo,
    nested_indicators: &[NestedIndicator],
    all_indicators: &[IndicatorInfo],
) -> bool {
    if is_oscillator_like(&primary.name) && is_oscillator_like(&secondary.name) {
        return false;
    }

    let is_built_on_oscillator = |indicator: &IndicatorInfo| -> bool {
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

    let is_built_on_non_oscillator = |indicator: &IndicatorInfo| -> bool {
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

    let get_source_oscillator_alias = |indicator: &IndicatorInfo| -> Option<String> {
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

    true
}
