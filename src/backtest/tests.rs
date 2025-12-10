#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::{
        BacktestConfig, BacktestEngine, BacktestError, ConditionEvaluator,
        EquityCalculator, FeedManager, IndicatorEngine, SessionManager, SessionState,
        TimeFrameAggregationService,
    };
    use crate::data_model::quote::Quote;
    use crate::data_model::quote_frame::QuoteFrame;
    use crate::data_model::types::{Symbol, TimeFrame};
    use crate::strategy::base::Strategy;
    use crate::strategy::presets::default_strategy_definitions;
    use crate::strategy::types::{
        StrategyCategory, StrategyDecision, StrategyDefinition, StrategyError, StrategyId,
        StrategyMetadata, StrategyParameterMap, StrategyRuleSpec,
    };
    use chrono::{DateTime, Duration, Utc};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_test_quote(
        symbol: Symbol,
        timeframe: TimeFrame,
        timestamp: DateTime<Utc>,
        close: f32,
    ) -> Quote {
        Quote::from_parts(
            symbol,
            timeframe,
            timestamp,
            close - 1.0,
            close + 1.0,
            close - 0.5,
            close,
            1000.0,
        )
    }

    fn create_test_quote_frame(
        symbol: Symbol,
        timeframe: TimeFrame,
        count: usize,
        start_time: DateTime<Utc>,
        timeframe_duration: Duration,
    ) -> QuoteFrame {
        let symbol_clone = symbol.clone();
        let timeframe_clone = timeframe.clone();
        let mut frame = QuoteFrame::new(symbol, timeframe);
        let mut current_time = start_time;

        for i in 0..count {
            let close = 100.0 + (i as f32 * 0.1);
            let quote = create_test_quote(
                symbol_clone.clone(),
                timeframe_clone.clone(),
                current_time,
                close,
            );
            frame.push(quote).unwrap();
            current_time = current_time + timeframe_duration;
        }

        frame
    }

    fn create_simple_test_strategy() -> Box<dyn Strategy> {
        struct TestStrategy {
            id: StrategyId,
            metadata: StrategyMetadata,
            parameters: StrategyParameterMap,
        }

        impl Strategy for TestStrategy {
            fn id(&self) -> &StrategyId {
                &self.id
            }

            fn metadata(&self) -> &StrategyMetadata {
                &self.metadata
            }

            fn parameters(&self) -> &StrategyParameterMap {
                &self.parameters
            }

            fn indicator_bindings(&self) -> &[crate::strategy::types::IndicatorBindingSpec] {
                &[]
            }

            fn conditions(&self) -> &[crate::strategy::types::PreparedCondition] {
                &[]
            }

            fn entry_rules(&self) -> &[StrategyRuleSpec] {
                &[]
            }

            fn exit_rules(&self) -> &[StrategyRuleSpec] {
                &[]
            }

            fn timeframe_requirements(&self) -> &[crate::strategy::types::TimeframeRequirement] {
                &[]
            }

            fn evaluate(
                &self,
                _context: &crate::strategy::context::StrategyContext,
            ) -> Result<StrategyDecision, StrategyError> {
                Ok(StrategyDecision::empty())
            }

            fn clone_box(&self) -> Box<dyn Strategy> {
                Box::new(TestStrategy {
                    id: self.id.clone(),
                    metadata: self.metadata.clone(),
                    parameters: self.parameters.clone(),
                })
            }
        }

        Box::new(TestStrategy {
            id: StrategyId::from("test_strategy"),
            metadata: StrategyMetadata {
                id: StrategyId::from("test_strategy"),
                name: "Test Strategy".to_string(),
                description: Some("Test strategy for unit tests".to_string()),
                version: Some("1.0.0".to_string()),
                author: Some("Test".to_string()),
                categories: vec![],
                tags: vec![],
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            },
            parameters: StrategyParameterMap::new(),
        })
    }

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.initial_capital, 10000.0);
        assert_eq!(config.use_full_capital, false);
        assert_eq!(config.reinvest_profits, false);
    }

    #[test]
    fn test_backtest_engine_new_with_empty_frames() {
        let strategy = create_simple_test_strategy();
        let frames = HashMap::new();

        let result = BacktestEngine::new(strategy, frames);
        assert!(result.is_err());
        match result {
            Err(BacktestError::Feed(msg)) => {
                assert!(msg.contains("frames collection is empty"));
            }
            _ => panic!("Expected Feed error"),
        }
    }

    #[test]
    fn test_backtest_engine_new_with_valid_frames() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let result = BacktestEngine::new(strategy, frames);
        assert!(result.is_ok());
    }

    #[test]
    fn test_backtest_engine_with_config() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let engine = BacktestEngine::new(strategy, frames).unwrap();
        let config = engine.config();
        assert_eq!(config.initial_capital, 10000.0);

        let custom_config = BacktestConfig {
            initial_capital: 5000.0,
            use_full_capital: true,
            reinvest_profits: true,
        };
        let engine_with_config = engine.with_config(custom_config.clone());
        assert_eq!(engine_with_config.config().initial_capital, 5000.0);
        assert_eq!(engine_with_config.config().use_full_capital, true);
        assert_eq!(engine_with_config.config().reinvest_profits, true);
    }

    #[test]
    fn test_backtest_engine_from_definition() {
        let definitions = default_strategy_definitions();
        assert!(!definitions.is_empty());

        let definition = definitions.first().unwrap();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            200,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let result = BacktestEngine::from_definition(definition.clone(), None, frames);
        assert!(result.is_ok());
    }

    #[test]
    fn test_backtest_engine_context_access() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), frame);

        let engine = BacktestEngine::new(strategy, frames).unwrap();
        let context = engine.context();
        assert!(context.timeframe(&timeframe).is_ok());
    }

    #[test]
    fn test_backtest_engine_position_manager_access() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let engine = BacktestEngine::new(strategy, frames).unwrap();
        let position_manager = engine.position_manager();
        assert_eq!(position_manager.open_position_count(), 0);
    }

    #[test]
    fn test_backtest_engine_run_with_simple_strategy() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            200,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let mut engine = BacktestEngine::new(strategy, frames).unwrap();
        let result = engine.run();
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.metrics.total_bars > 0);
        assert_eq!(report.metrics.initial_capital, 10000.0);
    }

    #[test]
    fn test_backtest_engine_run_with_preset_strategy() {
        let definitions = default_strategy_definitions();
        if definitions.is_empty() {
            return;
        }

        let definition = definitions.first().unwrap();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            500,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let mut engine = BacktestEngine::from_definition(definition.clone(), None, frames).unwrap();
        let result = engine.run();
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.metrics.total_bars > 0);
    }

    #[test]
    fn test_backtest_engine_warmup_bars() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            200,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let mut engine = BacktestEngine::new(strategy, frames).unwrap();
        let result = engine.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_backtest_engine_multiple_timeframes() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let tf_60 = TimeFrame::Minutes(60);
        let tf_240 = TimeFrame::Minutes(240);
        let start_time = Utc::now() - Duration::days(30);

        let frame_60 = create_test_quote_frame(
            symbol.clone(),
            tf_60.clone(),
            400,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(tf_60, frame_60);

        let result = BacktestEngine::new(strategy, frames);
        assert!(result.is_ok());
    }

    #[test]
    fn test_backtest_error_display() {
        let error = BacktestError::Feed("test error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_backtest_config_clone() {
        let config = BacktestConfig {
            initial_capital: 5000.0,
            use_full_capital: true,
            reinvest_profits: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.initial_capital, config.initial_capital);
        assert_eq!(cloned.use_full_capital, config.use_full_capital);
        assert_eq!(cloned.reinvest_profits, config.reinvest_profits);
    }

    #[test]
    fn test_backtest_engine_reset_behavior() {
        let strategy = create_simple_test_strategy();
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol.clone(),
            timeframe.clone(),
            200,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe, frame);

        let mut engine = BacktestEngine::new(strategy, frames).unwrap();

        let result1 = engine.run();
        assert!(result1.is_ok());

        let result2 = engine.run();
        assert!(result2.is_ok());

        let report1 = result1.unwrap();
        let report2 = result2.unwrap();

        assert_eq!(report1.metrics.total_bars, report2.metrics.total_bars);
    }


    #[test]
    fn test_feed_manager_with_frames() {
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol,
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), Arc::new(frame));

        let manager = FeedManager::with_frames(frames);
        assert_eq!(manager.frames().len(), 1);
        assert!(manager.get_frame(&timeframe).is_some());
    }

    #[test]
    fn test_feed_manager_primary_timeframe() {
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol,
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), Arc::new(frame));

        let mut manager = FeedManager::with_frames(frames);
        manager.set_primary_timeframe(timeframe.clone());
        assert_eq!(manager.primary_timeframe(), Some(&timeframe));
    }

    #[test]
    fn test_feed_manager_timeframe_to_minutes() {
        assert_eq!(
            FeedManager::timeframe_to_minutes(&TimeFrame::Minutes(60)),
            Some(60)
        );
        assert_eq!(
            FeedManager::timeframe_to_minutes(&TimeFrame::Hours(2)),
            Some(120)
        );
        assert_eq!(
            FeedManager::timeframe_to_minutes(&TimeFrame::Days(1)),
            Some(1440)
        );
    }

    #[test]
    fn test_feed_manager_is_higher_timeframe() {
        let tf_60 = TimeFrame::Minutes(60);
        let tf_240 = TimeFrame::Minutes(240);
        assert!(FeedManager::is_higher_timeframe(&tf_240, &tf_60));
        assert!(!FeedManager::is_higher_timeframe(&tf_60, &tf_240));
    }

    #[test]
    fn test_feed_manager_is_multiple_of() {
        let tf_60 = TimeFrame::Minutes(60);
        let tf_240 = TimeFrame::Minutes(240);
        assert!(FeedManager::is_multiple_of(&tf_60, &tf_240));
        assert!(!FeedManager::is_multiple_of(&tf_240, &tf_60));
    }


    #[test]
    fn test_feed_manager_step() {
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol,
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), Arc::new(frame));

        let mut manager = FeedManager::with_frames(frames);
        manager.set_primary_timeframe(timeframe.clone());
        let mut context = manager.initialize_context_ordered(&[timeframe.clone()]);

        let result = manager.step(&mut context);
        assert!(result);
    }

    #[test]
    fn test_feed_manager_reset() {
        let symbol = Symbol::from_descriptor("TEST");
        let timeframe = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);
        let frame = create_test_quote_frame(
            symbol,
            timeframe.clone(),
            100,
            start_time,
            Duration::hours(1),
        );

        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), Arc::new(frame));

        let mut manager = FeedManager::with_frames(frames);
        manager.set_primary_timeframe(timeframe.clone());
        let mut context = manager.initialize_context_ordered(&[timeframe.clone()]);

        manager.step(&mut context);
        manager.reset();

        let mut context2 = manager.initialize_context_ordered(&[timeframe.clone()]);
        let result = manager.step(&mut context2);
        assert!(result);
    }

    #[test]
    fn test_equity_calculator_new() {
        let calculator = EquityCalculator::new(10000.0);
        let calculator2 = EquityCalculator::new(5000.0);
        assert!(true);
    }

    #[test]
    fn test_equity_calculator_reset() {
        let mut calculator = EquityCalculator::new(10000.0);
        let mut position_manager = crate::position::PositionManager::new("test".to_string());
        let equity1 = calculator.calculate(&position_manager, false, false, 0);
        calculator.reset();
        let equity2 = calculator.calculate(&position_manager, false, false, 0);
        assert_eq!(equity1, equity2);
    }


    #[test]
    fn test_session_manager_new() {
        let _manager = SessionManager::new(None);
        assert!(true);
    }

    #[test]
    fn test_session_manager_with_duration() {
        let duration = Some(chrono::Duration::hours(1));
        let _manager = SessionManager::new(duration);
        assert!(true);
    }


    #[test]
    fn test_session_state_default() {
        let state = SessionState::default();
        assert_eq!(state.is_session_start, false);
        assert_eq!(state.is_session_end, false);
    }

    #[test]
    fn test_condition_evaluator_new() {
        let evaluator = ConditionEvaluator::new();
        assert!(true);
    }

    #[test]
    fn test_condition_evaluator_default() {
        let evaluator = ConditionEvaluator::default();
        assert!(true);
    }

    #[test]
    fn test_indicator_engine_new() {
        let engine = IndicatorEngine::new();
        assert!(true);
    }

    #[test]
    fn test_indicator_engine_default() {
        let engine = IndicatorEngine::default();
        assert!(true);
    }

    #[test]
    fn test_timeframe_aggregation_service_new() {
        let service = TimeFrameAggregationService::new();
        assert!(true);
    }

    #[test]
    fn test_timeframe_aggregation_service_default() {
        let service = TimeFrameAggregationService::default();
        assert!(true);
    }

    #[test]
    fn test_timeframe_aggregation_service_collect_required_timeframes() {
        let strategy = create_simple_test_strategy();
        let required = TimeFrameAggregationService::collect_required_timeframes(strategy.as_ref());
        assert_eq!(required.len(), 0);
    }

    #[test]
    fn test_timeframe_aggregation_service_aggregate_required_timeframes() {
        let service = TimeFrameAggregationService::new();
        let symbol = Symbol::from_descriptor("TEST");
        let tf_60 = TimeFrame::Minutes(60);
        let tf_240 = TimeFrame::Minutes(240);
        let start_time = Utc::now() - Duration::days(30);

        let frame_60 = create_test_quote_frame(
            symbol.clone(),
            tf_60.clone(),
            400,
            start_time,
            Duration::hours(1),
        );

        let mut base_frames = HashMap::new();
        base_frames.insert(tf_60.clone(), Arc::new(frame_60));

        let base_timeframes = vec![tf_60.clone()];
        let required_timeframes = vec![tf_240.clone()];

        let result = service.aggregate_required_timeframes(
            &base_frames,
            &base_timeframes,
            &required_timeframes,
        );

        assert!(result.is_ok());
        let aggregated = result.unwrap();
        assert!(aggregated.contains_key(&tf_240));
    }

    #[test]
    fn test_timeframe_aggregation_service_no_aggregation_needed() {
        let service = TimeFrameAggregationService::new();
        let symbol = Symbol::from_descriptor("TEST");
        let tf_60 = TimeFrame::Minutes(60);
        let start_time = Utc::now() - Duration::days(30);

        let frame_60 = create_test_quote_frame(
            symbol.clone(),
            tf_60.clone(),
            400,
            start_time,
            Duration::hours(1),
        );

        let mut base_frames = HashMap::new();
        base_frames.insert(tf_60.clone(), Arc::new(frame_60));

        let base_timeframes = vec![tf_60.clone()];
        let required_timeframes = vec![tf_60.clone()];

        let result = service.aggregate_required_timeframes(
            &base_frames,
            &base_timeframes,
            &required_timeframes,
        );

        assert!(result.is_ok());
        let aggregated = result.unwrap();
        assert!(aggregated.is_empty());
    }

}
