use crate::data_model::types::{Symbol, TimeFrame};
use crate::position::{ClosedTrade, ExecutionReport};
use crate::strategy::types::PositionDirection;
use chrono::{DateTime, Utc};

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
    pub entry_rule_id: Option<String>,
    pub exit_rule_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct BacktestMetrics {
    pub total_pnl: f64,
    pub total_trades: usize,
    pub win_rate: f64,
    pub average_trade: f64,
}

impl BacktestMetrics {
    pub fn from_data(trades: &[StrategyTrade], realized_pnl: f64) -> Self {
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

impl BacktestReport {
    pub fn new(
        trades: Vec<StrategyTrade>,
        metrics: BacktestMetrics,
        equity_curve: Vec<f64>,
    ) -> Self {
        Self {
            trades,
            metrics,
            equity_curve,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BacktestAnalytics {
    trades: Vec<StrategyTrade>,
    equity_curve: Vec<f64>,
}

impl BacktestAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.trades.clear();
        self.equity_curve.clear();
    }

    pub fn push_equity_point(&mut self, equity: f64) {
        self.equity_curve.push(equity);
    }

    pub fn absorb_execution_report(&mut self, report: &ExecutionReport) {
        for trade in &report.closed_trades {
            self.trades.push(StrategyTrade::from(trade));
        }
    }

    pub fn trades(&self) -> &[StrategyTrade] {
        &self.trades
    }

    pub fn equity_curve(&self) -> &[f64] {
        &self.equity_curve
    }

    pub fn build_report(&self, realized_pnl: f64) -> BacktestReport {
        let metrics = BacktestMetrics::from_data(&self.trades, realized_pnl);
        BacktestReport::new(self.trades.clone(), metrics, self.equity_curve.clone())
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
            entry_rule_id: trade.entry_rule_id.clone(),
            exit_rule_id: trade.exit_rule_id.clone(),
        }
    }
}
