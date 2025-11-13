use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::{Symbol, TimeFrame};
use crate::indicators::IndicatorFactory;

use super::base::Strategy;
use super::builder::StrategyBuilder;
use super::context::{StrategyContext, TimeframeData};
use super::positions::{ClosedTrade, PositionError, PositionManager};
use super::types::{
    IndicatorSourceSpec, PositionDirection, StrategyDefinition, StrategyError,
    StrategyParameterMap, StrategySignal, StrategySignalType,
};

#[derive(Clone, Debug)]
pub struct StrategyTrade {
    pub position_id: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub direction: PositionDirection,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub entry_time: Option<DateTime<Utc>>,
    pub exit_time: Option<DateTime<Utc>>,
    pub pnl: f64,
}

#[derive(Clone, Debug)]
pub struct BacktestMetrics {
    pub total_pnl: f64,
    pub total_trades: usize,
    pub win_rate: f64,
    pub average_trade: f64,
}

impl BacktestMetrics {
    fn from_data(trades: &[StrategyTrade], realized_pnl: f64) -> Self {
        let total_trades = trades.len();
        let wins = trades.iter().filter(|trade| trade.pnl > 0.0).count();
        let win_rate = if total_trades == 0 {
            0.0
        } else {
            wins as f64 / total_trades as f64
        };
        let average_trade = if total_trades == 0 {
            0.0
        } else {
            realized_pnl / total_trades as f64
        };
        Self {
            total_pnl: realized_pnl,
            total_trades,
            win_rate,
            average_trade,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BacktestReport {
    pub trades: Vec<StrategyTrade>,
    pub metrics: BacktestMetrics,
    pub equity_curve: Vec<f64>,
}

#[derive(Debug, Error)]
pub enum StrategyExecutionError {
    #[error("strategy evaluation error: {0}")]
    Strategy(#[from] StrategyError),
    #[error("position manager error: {0}")]
    Position(#[from] PositionError),
    #[error("feed error: {0}")]
    Feed(String),
}

pub struct BacktestExecutor {
    strategy: Box<dyn Strategy>,
    position_manager: PositionManager,
    feed: HistoricalFeed,
    context: StrategyContext,
    trades: Vec<StrategyTrade>,
    equity_curve: Vec<f64>,
}

impl BacktestExecutor {
    pub fn new(
        strategy: Box<dyn Strategy>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, StrategyExecutionError> {
        if frames.is_empty() {
            return Err(StrategyExecutionError::Feed(
                "frames collection is empty".to_string(),
            ));
        }
        let feed = HistoricalFeed::new(frames);
        let context = feed.initialize_context();
        let position_manager = PositionManager::new(strategy.id().clone());
        Ok(Self {
            strategy,
            position_manager,
            feed,
            context,
            trades: Vec::new(),
            equity_curve: Vec::new(),
        })
    }

    pub fn from_definition(
        definition: StrategyDefinition,
        parameter_overrides: Option<StrategyParameterMap>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, StrategyExecutionError> {
        let mut builder = StrategyBuilder::new(definition);
        if let Some(overrides) = parameter_overrides {
            builder = builder.with_parameters(overrides);
        }
        let strategy = builder.build().map_err(StrategyExecutionError::Strategy)?;
        Self::new(Box::new(strategy), frames)
    }

    pub fn context(&self) -> &StrategyContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut StrategyContext {
        &mut self.context
    }

    pub fn position_manager(&self) -> &PositionManager {
        &self.position_manager
    }

    pub fn position_manager_mut(&mut self) -> &mut PositionManager {
        &mut self.position_manager
    }

    pub async fn run_backtest(&mut self) -> Result<BacktestReport, StrategyExecutionError> {
        self.position_manager.reset();
        self.context.set_active_positions(HashMap::new());
        self.feed.reset();
        self.trades.clear();
        self.equity_curve.clear();
        for timeframe in self.feed.frames.keys() {
            if let Ok(data) = self.context.timeframe_mut(timeframe) {
                data.set_index(0);
            } else if let Some(frame) = self.feed.frames.get(timeframe) {
                let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
                self.context.insert_timeframe(timeframe.clone(), data);
            }
        }
        self.populate_indicators().await?;
        self.equity_curve
            .push(self.position_manager.portfolio_state().total_equity);
        while self.feed.step(&mut self.context) {
            let decision = self
                .strategy
                .evaluate(&self.context)
                .await
                .map_err(StrategyExecutionError::Strategy)?;
            let report = self
                .position_manager
                .process_decision(&mut self.context, &decision)
                .await
                .map_err(StrategyExecutionError::Position)?;
            self.collect_report(&report);
            self.equity_curve
                .push(self.position_manager.portfolio_state().total_equity);
        }
        let metrics = BacktestMetrics::from_data(
            &self.trades,
            self.position_manager.portfolio_state().realized_pnl,
        );
        Ok(BacktestReport {
            trades: self.trades.clone(),
            metrics,
            equity_curve: self.equity_curve.clone(),
        })
    }

    async fn populate_indicators(&mut self) -> Result<(), StrategyExecutionError> {
        for binding in self.strategy.indicator_bindings() {
            let frame = self.feed.frames.get(&binding.timeframe).ok_or_else(|| {
                StrategyExecutionError::Feed(format!(
                    "timeframe {:?} not available in feed",
                    binding.timeframe
                ))
            })?;
            let ohlc = frame.to_indicator_ohlc();
            let values = match &binding.source {
                IndicatorSourceSpec::Registry { name, parameters } => {
                    let indicator = IndicatorFactory::create_indicator(name, parameters.clone())
                        .map_err(|err| {
                            StrategyExecutionError::Feed(format!(
                                "indicator {} creation failed: {}",
                                name, err
                            ))
                        })?;
                    indicator.calculate_ohlc(&ohlc).await.map_err(|err| {
                        StrategyExecutionError::Feed(format!(
                            "indicator {} calculation failed: {}",
                            name, err
                        ))
                    })?
                }
                IndicatorSourceSpec::Formula { expression } => {
                    return Err(StrategyExecutionError::Feed(format!(
                        "formula indicators are not supported: {}",
                        expression
                    )));
                }
            };
            match self.context.timeframe_mut(&binding.timeframe) {
                Ok(data) => data.insert_indicator(binding.alias.clone(), values),
                Err(_) => {
                    let mut data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
                    data.insert_indicator(binding.alias.clone(), values);
                    self.context
                        .insert_timeframe(binding.timeframe.clone(), data);
                }
            }
        }
        Ok(())
    }

    fn collect_report(&mut self, report: &crate::strategy::positions::ExecutionReport) {
        for trade in &report.closed_trades {
            self.trades.push(StrategyTrade::from(trade));
        }
    }
}

struct HistoricalFeed {
    frames: HashMap<TimeFrame, Arc<QuoteFrame>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: TimeFrame,
}

impl HistoricalFeed {
    fn new(frames: HashMap<TimeFrame, QuoteFrame>) -> Self {
        let mut arc_frames = HashMap::new();
        let mut primary_timeframe = None;
        let mut max_len = 0usize;
        for (timeframe, frame) in frames {
            let len = frame.len();
            if len > max_len {
                max_len = len;
                primary_timeframe = Some(timeframe.clone());
            }
            arc_frames.insert(timeframe, Arc::new(frame));
        }
        let primary_timeframe =
            primary_timeframe.expect("HistoricalFeed requires at least one timeframe");
        Self {
            frames: arc_frames,
            indices: HashMap::new(),
            primary_timeframe,
        }
    }

    fn initialize_context(&self) -> StrategyContext {
        let mut map = HashMap::new();
        for (timeframe, frame) in &self.frames {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            map.insert(timeframe.clone(), data);
        }
        StrategyContext::with_timeframes(map)
    }

    fn reset(&mut self) {
        self.indices.clear();
    }

    fn step(&mut self, context: &mut StrategyContext) -> bool {
        let primary_frame = match self.frames.get(&self.primary_timeframe) {
            Some(frame) => frame,
            None => return false,
        };
        let primary_len = primary_frame.len();
        if primary_len == 0 {
            return false;
        }
        let current_primary_index = {
            let entry = self
                .indices
                .entry(self.primary_timeframe.clone())
                .or_insert(0);
            if *entry >= primary_len {
                return false;
            }
            *entry
        };
        for (timeframe, frame) in &self.frames {
            let len = frame.len();
            if len == 0 {
                continue;
            }
            let idx = if timeframe == &self.primary_timeframe {
                current_primary_index.min(len - 1)
            } else {
                let entry = self.indices.entry(timeframe.clone()).or_insert(0);
                *entry = (*entry).min(len - 1);
                *entry
            };
            match context.timeframe_mut(timeframe) {
                Ok(data) => {
                    data.set_index(idx);
                    if data.symbol().is_none() {
                        data.set_symbol(frame.symbol().clone());
                    }
                }
                Err(_) => {
                    let mut data = TimeframeData::with_quote_frame(frame.as_ref(), idx);
                    data.set_index(idx);
                    context.insert_timeframe(timeframe.clone(), data);
                }
            }
        }
        {
            let entry = self
                .indices
                .entry(self.primary_timeframe.clone())
                .or_insert(0);
            *entry = current_primary_index.saturating_add(1);
        }
        for (timeframe, frame) in &self.frames {
            if timeframe == &self.primary_timeframe {
                continue;
            }
            let len = frame.len();
            let entry = self.indices.entry(timeframe.clone()).or_insert(0);
            if *entry + 1 < len {
                *entry += 1;
            }
        }
        true
    }
}

impl From<&ClosedTrade> for StrategyTrade {
    fn from(trade: &ClosedTrade) -> Self {
        StrategyTrade {
            position_id: trade.position_id.clone(),
            symbol: trade.symbol.clone(),
            timeframe: trade.timeframe.clone(),
            direction: trade.direction.clone(),
            quantity: trade.quantity,
            entry_price: trade.entry_price,
            exit_price: trade.exit_price,
            entry_time: trade.entry_time.clone(),
            exit_time: trade.exit_time.clone(),
            pnl: trade.pnl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::quote::Quote;
    use crate::data_model::quote_frame::QuoteFrame;
    use crate::strategy::context::StrategyContext;
    use crate::strategy::types::{
        PriceField, SignalStrength, StrategyDecision, StrategyId, StrategyMetadata,
        StrategyParameterMap, TimeframeRequirement, TrendDirection,
    };
    use async_trait::async_trait;

    #[derive(Clone)]
    struct SimpleStrategy {
        id: StrategyId,
        metadata: StrategyMetadata,
        timeframe: TimeFrame,
        requirements: Vec<TimeframeRequirement>,
        parameters: StrategyParameterMap,
    }

    impl SimpleStrategy {
        fn new(timeframe: TimeFrame) -> Self {
            let id = "SIMPLE_ENTRY_EXIT".to_string();
            let metadata = StrategyMetadata::with_id(&id, "Simple Entry Exit");
            let requirements = vec![TimeframeRequirement {
                alias: "primary".to_string(),
                timeframe: timeframe.clone(),
            }];
            Self {
                id,
                metadata,
                timeframe,
                requirements,
                parameters: StrategyParameterMap::new(),
            }
        }
    }

    #[async_trait]
    impl Strategy for SimpleStrategy {
        fn id(&self) -> &StrategyId {
            &self.id
        }

        fn metadata(&self) -> &StrategyMetadata {
            &self.metadata
        }

        fn parameters(&self) -> &StrategyParameterMap {
            &self.parameters
        }

        fn indicator_bindings(&self) -> &[super::types::IndicatorBindingSpec] {
            &[]
        }

        fn conditions(&self) -> &[super::types::PreparedCondition] {
            &[]
        }

        fn entry_rules(&self) -> &[super::types::StrategyRuleSpec] {
            &[]
        }

        fn exit_rules(&self) -> &[super::types::StrategyRuleSpec] {
            &[]
        }

        fn timeframe_requirements(&self) -> &[TimeframeRequirement] {
            &self.requirements
        }

        async fn evaluate(
            &self,
            context: &StrategyContext,
        ) -> Result<StrategyDecision, StrategyError> {
            let data = context.timeframe(&self.timeframe)?;
            let idx = data.index();
            let series_len = data
                .price_series_slice(&PriceField::Close)
                .map(|slice| slice.len())
                .unwrap_or(0);
            let mut decision = StrategyDecision::empty();
            if idx == 0 {
                let signal = StrategySignal {
                    rule_id: "enter".to_string(),
                    signal_type: StrategySignalType::Entry,
                    direction: PositionDirection::Long,
                    timeframe: self.timeframe.clone(),
                    strength: SignalStrength::Strong,
                    trend: TrendDirection::Up,
                    quantity: Some(1.0),
                    tags: Vec::new(),
                };
                decision.entries.push(signal);
            } else if series_len > 0 && idx + 1 == series_len {
                let signal = StrategySignal {
                    rule_id: "exit".to_string(),
                    signal_type: StrategySignalType::Exit,
                    direction: PositionDirection::Long,
                    timeframe: self.timeframe.clone(),
                    strength: SignalStrength::Strong,
                    trend: TrendDirection::Down,
                    quantity: Some(1.0),
                    tags: Vec::new(),
                };
                decision.exits.push(signal);
            }
            Ok(decision)
        }

        fn clone_box(&self) -> Box<dyn Strategy> {
            Box::new(self.clone())
        }
    }

    fn build_frame(prices: &[f32], timeframe: &TimeFrame) -> QuoteFrame {
        let symbol = Symbol::from_descriptor("TEST.TEST");
        let mut frame = QuoteFrame::new(symbol.clone(), timeframe.clone());
        for (idx, price) in prices.iter().enumerate() {
            let quote = Quote::from_parts(
                symbol.clone(),
                timeframe.clone(),
                Utc::now() + chrono::Duration::minutes(idx as i64),
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

    #[tokio::test]
    async fn backtest_executor_produces_trade_and_metrics() {
        let timeframe = TimeFrame::minutes(1);
        let frame = build_frame(&[100.0, 102.0, 105.0], &timeframe);
        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), frame);
        let strategy: Box<dyn Strategy> = Box::new(SimpleStrategy::new(timeframe.clone()));
        let mut executor = BacktestExecutor::new(strategy, frames).expect("executor creation");
        let report = executor.run_backtest().await.expect("backtest run");
        assert_eq!(report.trades.len(), 1);
        let trade = &report.trades[0];
        assert!((trade.pnl - 5.0).abs() < 1e-6);
        assert_eq!(report.metrics.total_trades, 1);
        assert!((report.metrics.total_pnl - 5.0).abs() < 1e-6);
        assert!((report.metrics.win_rate - 1.0).abs() < 1e-6);
        assert_eq!(report.equity_curve.last().copied().unwrap_or_default(), 5.0);
    }
}
