use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::backtest::{BacktestConfig, BacktestEngine};
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::{StrategyCandidate, StrategyConverter};
use crate::metrics::backtest::BacktestReport;
use crate::strategy::types::StrategyParameterMap;
use anyhow::{Context, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CacheKey {
    candidate_signature: String,
    parameters_signature: String,
}

impl CacheKey {
    fn from_candidate_and_params(
        candidate: &StrategyCandidate,
        parameters: &StrategyParameterMap,
    ) -> Self {
        let candidate_sig = Self::candidate_signature(candidate);
        let params_sig = Self::parameters_signature(parameters);
        Self {
            candidate_signature: candidate_sig,
            parameters_signature: params_sig,
        }
    }

    fn candidate_signature(candidate: &StrategyCandidate) -> String {
        let mut parts = Vec::with_capacity(4);
        parts.push(format!("indicators:{}", candidate.indicators.len()));
        parts.push(format!("nested:{}", candidate.nested_indicators.len()));
        parts.push(format!("conditions:{}", candidate.conditions.len()));
        parts.push(format!("timeframes:{}", candidate.timeframes.len()));
        parts.sort();
        parts.join("|")
    }

    fn parameters_signature(parameters: &StrategyParameterMap) -> String {
        let mut pairs: Vec<(String, String)> = parameters
            .iter()
            .map(|(k, v)| (k.clone(), format!("{:?}", v)))
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
            .into_iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join("|")
    }
}

pub struct StrategyEvaluationRunner {
    frames: Arc<HashMap<TimeFrame, QuoteFrame>>,
    base_timeframe: TimeFrame,
    available_higher_timeframes: Vec<TimeFrame>,
    cache: Arc<RwLock<HashMap<CacheKey, BacktestReport>>>,
    backtest_config: BacktestConfig,
}

impl StrategyEvaluationRunner {
    pub fn available_timeframes(&self) -> Vec<TimeFrame> {
        let mut result = vec![self.base_timeframe.clone()];
        let mut higher: Vec<TimeFrame> = self.available_higher_timeframes.clone();
        higher.sort_by_key(|tf| tf.duration());
        result.extend(higher);
        result
    }
}

impl StrategyEvaluationRunner {
    pub fn new(frames: HashMap<TimeFrame, QuoteFrame>, base_timeframe: TimeFrame) -> Self {
        Self::with_higher_timeframes(frames, base_timeframe, vec![])
    }

    pub fn with_higher_timeframes(
        frames: HashMap<TimeFrame, QuoteFrame>,
        base_timeframe: TimeFrame,
        available_higher_timeframes: Vec<TimeFrame>,
    ) -> Self {
        Self {
            frames: Arc::new(frames),
            base_timeframe,
            available_higher_timeframes,
            cache: Arc::new(RwLock::new(HashMap::new())),
            backtest_config: BacktestConfig::default(),
        }
    }

    pub fn with_backtest_config(mut self, config: BacktestConfig) -> Self {
        self.backtest_config = config;
        self
    }

    pub fn set_backtest_config(&mut self, config: BacktestConfig) {
        self.backtest_config = config;
    }

    pub async fn evaluate_strategy(
        &self,
        candidate: &StrategyCandidate,
        parameters: StrategyParameterMap,
    ) -> Result<BacktestReport> {
        let cache_key = CacheKey::from_candidate_and_params(candidate, &parameters);

        {
            let cache = self.cache.read().await;
            if let Some(cached_report) = cache.get(&cache_key) {
                return Ok(cached_report.clone());
            }
        }

        let definition =
            StrategyConverter::candidate_to_definition(candidate, self.base_timeframe.clone())
                .context("Не удалось конвертировать StrategyCandidate в StrategyDefinition")?;

        let mut frames_clone = HashMap::with_capacity(self.frames.len());
        for (k, v) in self.frames.iter() {
            frames_clone.insert(k.clone(), v.clone());
        }

        let mut executor =
            BacktestEngine::from_definition(definition, Some(parameters.clone()), frames_clone)
                .context("Не удалось создать BacktestEngine")?
                .with_config(self.backtest_config.clone());

        let report = executor.run().context("Ошибка выполнения backtest")?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, report.clone());
        }

        Ok(report)
    }
}

impl Clone for StrategyEvaluationRunner {
    fn clone(&self) -> Self {
        Self {
            frames: Arc::clone(&self.frames),
            base_timeframe: self.base_timeframe.clone(),
            available_higher_timeframes: self.available_higher_timeframes.clone(),
            cache: Arc::clone(&self.cache),
            backtest_config: self.backtest_config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::quote_frame::QuoteFrame;
    use crate::data_model::types::Symbol;
    use crate::discovery::StrategyCandidate;
    use crate::strategy::types::StrategyParamValue;

    fn create_test_candidate() -> StrategyCandidate {
        StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![],
            config: crate::discovery::config::StrategyDiscoveryConfig::default(),
        }
    }

    fn create_test_parameters() -> StrategyParameterMap {
        let mut params = HashMap::new();
        params.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        params.insert("param2".to_string(), StrategyParamValue::Number(20.0));
        params
    }

    fn create_test_frames() -> HashMap<TimeFrame, QuoteFrame> {
        let mut frames = HashMap::new();
        let tf = TimeFrame::from_identifier("60");
        let frame = QuoteFrame::new(Symbol::new("BTCUSDT".to_string()), tf.clone());
        frames.insert(tf, frame);
        frames
    }

    #[test]
    fn test_cache_key_candidate_signature() {
        let candidate = create_test_candidate();
        let sig = CacheKey::candidate_signature(&candidate);
        assert!(sig.contains("indicators:0"));
        assert!(sig.contains("conditions:0"));
        assert!(sig.contains("timeframes:0"));
    }

    #[test]
    fn test_cache_key_parameters_signature() {
        let params = create_test_parameters();
        let sig = CacheKey::parameters_signature(&params);
        assert!(sig.contains("param1"));
        assert!(sig.contains("param2"));
    }

    #[test]
    fn test_cache_key_from_candidate_and_params() {
        let candidate = create_test_candidate();
        let params = create_test_parameters();
        let key = CacheKey::from_candidate_and_params(&candidate, &params);
        assert!(!key.candidate_signature.is_empty());
        assert!(!key.parameters_signature.is_empty());
    }

    #[test]
    fn test_cache_key_equality() {
        let candidate = create_test_candidate();
        let params = create_test_parameters();
        let key1 = CacheKey::from_candidate_and_params(&candidate, &params);
        let key2 = CacheKey::from_candidate_and_params(&candidate, &params);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_strategy_evaluation_runner_new() {
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let runner = StrategyEvaluationRunner::new(frames, base_tf);
        assert_eq!(runner.available_higher_timeframes.len(), 0);
    }

    #[test]
    fn test_strategy_evaluation_runner_with_higher_timeframes() {
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let higher_tfs = vec![
            TimeFrame::from_identifier("240"),
            TimeFrame::from_identifier("1440"),
        ];
        let runner = StrategyEvaluationRunner::with_higher_timeframes(frames, base_tf, higher_tfs);
        assert_eq!(runner.available_higher_timeframes.len(), 2);
    }

    #[test]
    fn test_available_timeframes() {
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let higher_tfs = vec![
            TimeFrame::from_identifier("1440"),
            TimeFrame::from_identifier("240"),
        ];
        let runner = StrategyEvaluationRunner::with_higher_timeframes(frames, base_tf, higher_tfs);
        let available = runner.available_timeframes();
        assert_eq!(available.len(), 3);
        assert_eq!(available[0].identifier(), "60");
    }

    #[test]
    fn test_with_backtest_config() {
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let runner = StrategyEvaluationRunner::new(frames, base_tf);
        let config = BacktestConfig::default();
        let runner_with_config = runner.with_backtest_config(config.clone());
        assert_eq!(
            runner_with_config.backtest_config.initial_capital,
            config.initial_capital
        );
    }

    #[test]
    fn test_set_backtest_config() {
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let mut runner = StrategyEvaluationRunner::new(frames, base_tf);
        let config = BacktestConfig::default();
        runner.set_backtest_config(config.clone());
        assert_eq!(
            runner.backtest_config.initial_capital,
            config.initial_capital
        );
    }

    #[test]
    fn test_clone() {
        let frames = create_test_frames();
        let base_tf = TimeFrame::from_identifier("60");
        let runner1 = StrategyEvaluationRunner::new(frames, base_tf);
        let runner2 = runner1.clone();
        assert_eq!(runner1.base_timeframe, runner2.base_timeframe);
    }
}
