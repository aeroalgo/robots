use crate::data_model::quote::Quote;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::{Symbol, TimeFrame};

use super::base::Strategy;
use super::builder::StrategyBuilder;
use super::context::{StrategyContext, TimeframeData};
use super::presets::default_strategy_definitions;

fn build_test_quote_frame(prices: &[f32], timeframe: TimeFrame) -> QuoteFrame {
    let symbol = Symbol::from_descriptor("TEST.TEST");
    let mut frame = QuoteFrame::new(symbol.clone(), timeframe.clone());
    for (idx, price) in prices.iter().enumerate() {
        let quote = Quote::from_parts(
            symbol.clone(),
            timeframe.clone(),
            chrono::Utc::now() + chrono::Duration::minutes(idx as i64),
            *price,
            *price,
            *price,
            *price,
            1.0,
        );
        frame.push(quote).unwrap();
    }
    frame
}

fn context_with_series(
    timeframe: TimeFrame,
    fast: Vec<f32>,
    slow: Vec<f32>,
    index: usize,
) -> StrategyContext {
    let frame = build_test_quote_frame(&slow, timeframe.clone());
    let mut tf_data = TimeframeData::with_quote_frame(&frame, index);
    tf_data.insert_indicator("fast_sma", fast);
    tf_data.insert_indicator("slow_sma", slow.clone());

    let mut context = StrategyContext::new();
    context.insert_timeframe(timeframe, tf_data);
    context
}

#[tokio::test]
async fn sma_crossover_entry_generates_signal() {
    let definition = default_strategy_definitions()
        .into_iter()
        .find(|def| def.metadata.id == "SMA_CROSSOVER_LONG")
        .expect("definition not found");

    let timeframe = definition
        .timeframe_requirements
        .first()
        .map(|req| req.timeframe.clone())
        .unwrap_or_else(|| TimeFrame::minutes(60));

    let strategy = StrategyBuilder::new(definition)
        .build()
        .expect("strategy build failed");

    let fast = vec![1.0, 1.2, 1.6, 2.0];
    let slow = vec![1.0, 1.1, 1.2, 1.3];
    let mut context = context_with_series(timeframe.clone(), fast, slow, 1);
    context
        .metadata
        .insert("test_case".to_string(), "entry".to_string());

    let decision = Strategy::evaluate(&strategy, &context)
        .await
        .expect("strategy evaluation failed");

    assert_eq!(decision.entries.len(), 1, "entry signal expected");
    assert_eq!(
        decision.exits.len(),
        0,
        "no exit expected on entry scenario"
    );
    let signal = &decision.entries[0];
    assert_eq!(signal.direction, super::types::PositionDirection::Long);
    assert_eq!(signal.signal_type, super::types::StrategySignalType::Entry);
    assert_eq!(signal.timeframe, timeframe);
    assert_eq!(signal.rule_id, "enter_long");
}

#[tokio::test]
async fn sma_crossover_exit_generates_signal() {
    let definition = default_strategy_definitions()
        .into_iter()
        .find(|def| def.metadata.id == "SMA_CROSSOVER_LONG")
        .expect("definition not found");

    let timeframe = definition
        .timeframe_requirements
        .first()
        .map(|req| req.timeframe.clone())
        .unwrap_or_else(|| TimeFrame::minutes(60));

    let strategy = StrategyBuilder::new(definition)
        .build()
        .expect("strategy build failed");
    let fast = vec![2.0, 1.8, 1.4, 1.0];
    let slow = vec![1.5, 1.6, 1.55, 1.5];
    let context = context_with_series(timeframe.clone(), fast, slow, 2);

    let decision = Strategy::evaluate(&strategy, &context)
        .await
        .expect("strategy evaluation failed");

    assert_eq!(
        decision.entries.len(),
        0,
        "no entry expected on exit scenario"
    );
    assert_eq!(decision.exits.len(), 1, "exit signal expected");
    let signal = &decision.exits[0];
    assert_eq!(signal.direction, super::types::PositionDirection::Long);
    assert_eq!(signal.signal_type, super::types::StrategySignalType::Exit);
    assert_eq!(signal.timeframe, timeframe);
    assert_eq!(signal.rule_id, "exit_long");
}
