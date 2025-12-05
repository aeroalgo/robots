use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::{StrategyCandidate, StrategyConverter};
use crate::metrics::backtest::BacktestReport;
use crate::strategy::executor::{BacktestConfig, BacktestExecutor};
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
            BacktestExecutor::from_definition(definition, Some(parameters.clone()), frames_clone)
                .context("Не удалось создать BacktestExecutor")?
                .with_config(self.backtest_config.clone());

        let report = executor
            .run_backtest()
            .context("Ошибка выполнения backtest")?;

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
