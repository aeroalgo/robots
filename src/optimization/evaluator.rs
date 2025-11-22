use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::discovery::{StrategyCandidate, StrategyConverter};
use crate::metrics::backtest::BacktestReport;
use crate::strategy::executor::BacktestExecutor;
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
    timeout: Option<Duration>,
    cache: Arc<RwLock<HashMap<CacheKey, BacktestReport>>>,
}

impl StrategyEvaluationRunner {
    pub fn available_timeframes(&self) -> Vec<TimeFrame> {
        self.frames.keys().cloned().collect()
    }
}

impl StrategyEvaluationRunner {
    pub fn new(frames: HashMap<TimeFrame, QuoteFrame>, base_timeframe: TimeFrame) -> Self {
        Self {
            frames: Arc::new(frames),
            base_timeframe,
            timeout: Some(Duration::from_secs(300)),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
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
                .context("Не удалось создать BacktestExecutor")?;

        let report = executor
            .run_backtest()
            .context("Ошибка выполнения backtest")?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, report.clone());
        }

        Ok(report)
    }

    pub async fn evaluate_batch(
        &self,
        evaluations: Vec<(StrategyCandidate, StrategyParameterMap)>,
    ) -> Vec<Result<BacktestReport>> {
        let mut handles = Vec::with_capacity(evaluations.len());

        for (candidate, parameters) in evaluations {
            let runner = Arc::new(self.clone());
            let candidate = Arc::new(candidate);
            let parameters = Arc::new(parameters);

            let handle = tokio::spawn(async move {
                runner
                    .evaluate_strategy(&candidate, (*parameters).clone())
                    .await
            });

            handles.push(handle);
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            let result = handle
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Ошибка выполнения задачи: {}", e)));
            results.push(result);
        }

        results
    }

    pub fn clear_cache(&self) {
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let mut cache = cache.write().await;
            cache.clear();
        });
    }

    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Clone for StrategyEvaluationRunner {
    fn clone(&self) -> Self {
        Self {
            frames: Arc::clone(&self.frames),
            base_timeframe: self.base_timeframe.clone(),
            timeout: self.timeout,
            cache: Arc::clone(&self.cache),
        }
    }
}
