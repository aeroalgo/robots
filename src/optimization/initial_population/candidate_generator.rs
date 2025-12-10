use crate::discovery::IndicatorInfoCollector;
use crate::discovery::StrategyCandidate;
use crate::indicators::registry::IndicatorRegistry;
use crate::optimization::candidate_builder::{CandidateBuilder, CandidateElements};
use crate::risk::registry::StopHandlerRegistry;

use super::helpers;
use super::super::candidate_builder_config::CandidateBuilderConfig;
use super::super::evaluator::StrategyEvaluationRunner;

pub async fn generate_candidates(
    count: usize,
    candidate_builder_config: &CandidateBuilderConfig,
    evaluator: &StrategyEvaluationRunner,
    discovery_config: &crate::discovery::StrategyDiscoveryConfig,
) -> Result<Vec<StrategyCandidate>, anyhow::Error> {
    println!(
        "   [Генерация кандидатов] Начало генерации {} кандидатов стратегий...",
        count
    );

    let mut candidates = Vec::with_capacity(count);

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

    let available_timeframes = evaluator.available_timeframes();

    println!("   [Генерация кандидатов] Использование CandidateBuilder с правилами...");
    let mut builder = CandidateBuilder::new(candidate_builder_config.clone());

    for i in 0..count {
        let candidate_elements = builder.build_candidate(
            &available_indicators_vec,
            &stop_handler_configs,
            &available_timeframes,
        );

        if let Some(candidate) = convert_candidate_elements_to_strategy_candidate(
            candidate_elements,
            discovery_config,
        ) {
            println!("\n   📋 Кандидат стратегии #{}:", i + 1);
            helpers::log_strategy_details(&candidate, None);
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

pub fn convert_candidate_elements_to_strategy_candidate(
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
