use crate::discovery::StrategyCandidate;
use crate::strategy::types::StrategyParameterMap;
use std::collections::BTreeSet;

pub fn get_strategy_signature(candidate: &StrategyCandidate) -> String {
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

pub fn log_strategy_details(
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
                            let param_key = format!("{}_{}", nested.indicator.alias, param.name);
                            if let Some(value) = params.get(&param_key) {
                                println!("               - {}: {:?}", param.name, value);
                            } else {
                                println!("               - {}: (не оптимизируется)", param.name);
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

    println!("\n   🎯 УСЛОВИЯ ВХОДА:");
    if candidate.conditions.is_empty() {
        println!("      (нет условий входа)");
    } else {
        for (idx, condition) in candidate.conditions.iter().enumerate() {
            println!("      {}. {} ({})", idx + 1, condition.name, condition.id);
            if !condition.optimization_params.is_empty() {
                println!("         Параметры оптимизации:");
                for param in &condition.optimization_params {
                    if let Some(params) = parameters {
                        let param_key = crate::optimization::condition_id::ConditionId::parameter_name(
                            &condition.id,
                            &param.name,
                        );
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

    if !candidate.exit_conditions.is_empty() {
        println!("\n   🚪 УСЛОВИЯ ВЫХОДА:");
        for (idx, condition) in candidate.exit_conditions.iter().enumerate() {
            println!("      {}. {} ({})", idx + 1, condition.name, condition.id);
            if !condition.optimization_params.is_empty() {
                println!("         Параметры оптимизации:");
                for param in &condition.optimization_params {
                    if let Some(params) = parameters {
                        let param_key = crate::optimization::condition_id::ConditionId::parameter_name(
                            &condition.id,
                            &param.name,
                        );
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

    if !candidate.stop_handlers.is_empty() {
        println!("\n   🛑 STOP HANDLERS:");
        for (idx, handler) in candidate.stop_handlers.iter().enumerate() {
            println!("      {}. {} ({})", idx + 1, handler.name, handler.handler_name);
            if !handler.optimization_params.is_empty() {
                println!("         Параметры оптимизации:");
                for param in &handler.optimization_params {
                    if let Some(params) = parameters {
                        let param_key = crate::optimization::condition_id::ConditionId::stop_handler_parameter_name(
                            &handler.id,
                            &param.name,
                        );
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

    if !candidate.take_handlers.is_empty() {
        println!("\n   💰 TAKE HANDLERS:");
        for (idx, handler) in candidate.take_handlers.iter().enumerate() {
            println!("      {}. {} ({})", idx + 1, handler.name, handler.handler_name);
            if !handler.optimization_params.is_empty() {
                println!("         Параметры оптимизации:");
                for param in &handler.optimization_params {
                    if let Some(params) = parameters {
                        let param_key = crate::optimization::condition_id::ConditionId::take_handler_parameter_name(
                            &handler.id,
                            &param.name,
                        );
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
