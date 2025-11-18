use crate::metrics::backtest::{BacktestMetrics, BacktestReport};

#[derive(Clone, Debug)]
pub struct FitnessThresholds {
    pub min_sharpe_ratio: Option<f64>,
    pub max_drawdown_pct: Option<f64>,
    pub min_win_rate: Option<f64>,
    pub min_profit_factor: Option<f64>,
    pub min_total_profit: Option<f64>,
    pub min_trades_count: Option<usize>,
    pub min_cagr: Option<f64>,
    pub max_max_drawdown: Option<f64>,
}

impl Default for FitnessThresholds {
    fn default() -> Self {
        Self {
            min_sharpe_ratio: Some(1.0),
            max_drawdown_pct: Some(20.0),
            min_win_rate: Some(0.45),
            min_profit_factor: Some(1.5),
            min_total_profit: Some(1000.0),
            min_trades_count: Some(30),
            min_cagr: Some(10.0),
            max_max_drawdown: Some(5000.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FitnessWeights {
    pub sharpe_ratio_weight: f64,
    pub profit_factor_weight: f64,
    pub win_rate_weight: f64,
    pub cagr_weight: f64,
    pub drawdown_penalty: f64,
    pub trades_count_bonus: f64,
}

impl Default for FitnessWeights {
    fn default() -> Self {
        Self {
            sharpe_ratio_weight: 0.3,
            profit_factor_weight: 0.25,
            win_rate_weight: 0.15,
            cagr_weight: 0.2,
            drawdown_penalty: 0.05,
            trades_count_bonus: 0.05,
        }
    }
}

pub struct FitnessFunction;

impl FitnessFunction {
    pub fn passes_thresholds(report: &BacktestReport, thresholds: &FitnessThresholds) -> bool {
        let metrics = &report.metrics;
        let trades_count = report.trades.len();

        if let Some(min_sharpe) = thresholds.min_sharpe_ratio {
            if let Some(sharpe) = metrics.sharpe_ratio {
                if sharpe < min_sharpe {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(max_dd_pct) = thresholds.max_drawdown_pct {
            if let Some(dd_pct) = metrics.drawdown_percent {
                if dd_pct > max_dd_pct {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(min_win_rate) = thresholds.min_win_rate {
            if metrics.winning_percentage < min_win_rate {
                return false;
            }
        }

        if let Some(min_profit_factor) = thresholds.min_profit_factor {
            if let Some(pf) = metrics.profit_factor {
                if pf < min_profit_factor {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(min_profit) = thresholds.min_total_profit {
            if metrics.total_profit < min_profit {
                return false;
            }
        }

        if let Some(min_trades) = thresholds.min_trades_count {
            if trades_count < min_trades {
                return false;
            }
        }

        if let Some(min_cagr) = thresholds.min_cagr {
            if let Some(cagr) = metrics.cagr {
                if cagr < min_cagr {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(max_dd) = thresholds.max_max_drawdown {
            if let Some(dd) = metrics.drawdown {
                if dd.abs() > max_dd {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn calculate_fitness(report: &BacktestReport, weights: &FitnessWeights) -> f64 {
        let metrics = &report.metrics;
        let trades_count = report.trades.len();

        let sharpe_score = Self::normalize_sharpe_ratio(metrics.sharpe_ratio);
        let profit_factor_score = Self::normalize_profit_factor(metrics.profit_factor);
        let win_rate_score = metrics.winning_percentage;
        let cagr_score = Self::normalize_cagr(metrics.cagr);
        let drawdown_penalty = Self::calculate_drawdown_penalty(metrics.drawdown_percent);
        let trades_bonus = Self::calculate_trades_bonus(trades_count);

        let total_weight = weights.sharpe_ratio_weight
            + weights.profit_factor_weight
            + weights.win_rate_weight
            + weights.cagr_weight
            + weights.drawdown_penalty
            + weights.trades_count_bonus;

        if total_weight == 0.0 {
            return 0.0;
        }

        let fitness = (sharpe_score * weights.sharpe_ratio_weight
            + profit_factor_score * weights.profit_factor_weight
            + win_rate_score * weights.win_rate_weight
            + cagr_score * weights.cagr_weight
            - drawdown_penalty * weights.drawdown_penalty
            + trades_bonus * weights.trades_count_bonus)
            / total_weight;

        fitness.max(0.0)
    }

    pub fn evaluate_strategy(
        report: &BacktestReport,
        thresholds: &FitnessThresholds,
        weights: &FitnessWeights,
    ) -> Option<f64> {
        if !Self::passes_thresholds(report, thresholds) {
            return None;
        }

        Some(Self::calculate_fitness(report, weights))
    }

    fn normalize_sharpe_ratio(sharpe: Option<f64>) -> f64 {
        match sharpe {
            Some(s) if s >= 0.0 => (s / 3.0).min(1.0),
            Some(s) => 0.0,
            None => 0.0,
        }
    }

    fn normalize_profit_factor(pf: Option<f64>) -> f64 {
        match pf {
            Some(p) if p > 0.0 => (p / 5.0).min(1.0),
            Some(_) => 0.0,
            None => 0.0,
        }
    }

    fn normalize_cagr(cagr: Option<f64>) -> f64 {
        match cagr {
            Some(c) if c >= 0.0 => (c / 100.0).min(1.0),
            Some(_) => 0.0,
            None => 0.0,
        }
    }

    fn calculate_drawdown_penalty(dd_pct: Option<f64>) -> f64 {
        match dd_pct {
            Some(dd) if dd > 0.0 => (dd / 50.0).min(1.0),
            Some(_) => 0.0,
            None => 0.0,
        }
    }

    fn calculate_trades_bonus(trades_count: usize) -> f64 {
        if trades_count >= 100 {
            1.0
        } else if trades_count >= 50 {
            0.75
        } else if trades_count >= 30 {
            0.5
        } else {
            (trades_count as f64 / 30.0).min(0.5)
        }
    }
}
