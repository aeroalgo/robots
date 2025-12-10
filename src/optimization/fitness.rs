use crate::metrics::backtest::BacktestReport;

#[derive(Clone, Debug)]
pub struct FitnessThresholds {
    pub min_sharpe_ratio: Option<f64>,
    pub max_drawdown_pct: Option<f64>,
    pub min_win_rate: Option<f64>,
    pub min_profit_factor: Option<f64>,
    pub min_trades_count: Option<usize>,
    pub min_cagr: Option<f64>,
    pub min_recovery_factor: Option<f64>,
}

impl Default for FitnessThresholds {
    fn default() -> Self {
        Self {
            min_sharpe_ratio: Some(1.0),
            max_drawdown_pct: Some(20.0),
            min_win_rate: Some(0.45),
            min_profit_factor: Some(1.5),
            min_trades_count: Some(30),
            min_cagr: Some(10.0),
            min_recovery_factor: Some(1.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FitnessWeights {
    pub sharpe_ratio_weight: f64,
    pub profit_factor_weight: f64,
    pub win_rate_weight: f64,
    pub cagr_weight: f64,
    pub recovery_factor_weight: f64,
    pub drawdown_penalty: f64,
    pub trades_count_bonus: f64,
}

impl Default for FitnessWeights {
    fn default() -> Self {
        Self {
            sharpe_ratio_weight: 0.25,
            profit_factor_weight: 0.20,
            win_rate_weight: 0.10,
            cagr_weight: 0.15,
            recovery_factor_weight: 0.20,
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

        if let Some(min_rf) = thresholds.min_recovery_factor {
            match (metrics.cagr, metrics.drawdown_percent) {
                (Some(cagr), Some(dd)) if dd > 0.0 => {
                    let rf = cagr / dd;
                    if rf < min_rf {
                        return false;
                    }
                }
                _ => return false,
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
        let recovery_factor_score =
            Self::normalize_recovery_factor(metrics.cagr, metrics.drawdown_percent);
        let drawdown_penalty = Self::calculate_drawdown_penalty(metrics.drawdown_percent);
        let trades_bonus = Self::calculate_trades_bonus(trades_count);

        let total_weight = weights.sharpe_ratio_weight
            + weights.profit_factor_weight
            + weights.win_rate_weight
            + weights.cagr_weight
            + weights.recovery_factor_weight
            + weights.drawdown_penalty
            + weights.trades_count_bonus;

        if total_weight == 0.0 {
            return 0.0;
        }

        let fitness = (sharpe_score * weights.sharpe_ratio_weight
            + profit_factor_score * weights.profit_factor_weight
            + win_rate_score * weights.win_rate_weight
            + cagr_score * weights.cagr_weight
            + recovery_factor_score * weights.recovery_factor_weight
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
            Some(_) => 0.0,
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

    fn normalize_recovery_factor(cagr: Option<f64>, drawdown_pct: Option<f64>) -> f64 {
        match (cagr, drawdown_pct) {
            (Some(c), Some(dd)) if c > 0.0 && dd > 0.0 => {
                let rf = c / dd;
                (rf / 5.0).min(1.0)
            }
            _ => 0.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::types::{Symbol, TimeFrame};
    use crate::metrics::backtest::{BacktestMetrics, BacktestReport, StrategyTrade};
    use crate::strategy::types::PositionDirection;
    use chrono::Utc;

    fn create_test_report(trades: Vec<StrategyTrade>, metrics: BacktestMetrics) -> BacktestReport {
        BacktestReport::new(
            trades,
            metrics,
            vec![1000.0, 1100.0, 1200.0, 1150.0, 1300.0],
        )
    }

    fn create_test_metrics() -> BacktestMetrics {
        let mut metrics = BacktestMetrics::default();
        metrics.sharpe_ratio = Some(2.0);
        metrics.profit_factor = Some(2.5);
        metrics.winning_percentage = 0.6;
        metrics.cagr = Some(15.0);
        metrics.drawdown_percent = Some(10.0);
        metrics.total_trades = 50;
        metrics.total_profit = 5000.0;
        metrics
    }

    fn create_test_trade(pnl: f64) -> StrategyTrade {
        StrategyTrade {
            position_id: "test_1".to_string(),
            symbol: Symbol::new("BTCUSDT".to_string()),
            timeframe: TimeFrame::from_identifier("60"),
            direction: PositionDirection::Long,
            quantity: 1.0,
            entry_price: 50000.0,
            exit_price: 51000.0,
            entry_time: Some(Utc::now()),
            exit_time: Some(Utc::now()),
            pnl,
            entry_rule_id: None,
            exit_rule_id: None,
            stop_history: vec![],
        }
    }

    #[test]
    fn test_passes_thresholds_all_pass() {
        let metrics = create_test_metrics();
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_sharpe_fails() {
        let mut metrics = create_test_metrics();
        metrics.sharpe_ratio = Some(0.5);
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_drawdown_fails() {
        let mut metrics = create_test_metrics();
        metrics.drawdown_percent = Some(25.0);
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_win_rate_fails() {
        let mut metrics = create_test_metrics();
        metrics.winning_percentage = 0.4;
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_profit_factor_fails() {
        let mut metrics = create_test_metrics();
        metrics.profit_factor = Some(1.2);
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_trades_count_fails() {
        let metrics = create_test_metrics();
        let trades = vec![create_test_trade(100.0); 20];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_cagr_fails() {
        let mut metrics = create_test_metrics();
        metrics.cagr = Some(5.0);
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_recovery_factor_fails() {
        let mut metrics = create_test_metrics();
        metrics.cagr = Some(5.0);
        metrics.drawdown_percent = Some(10.0);
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_passes_thresholds_missing_sharpe() {
        let mut metrics = create_test_metrics();
        metrics.sharpe_ratio = None;
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();

        assert!(!FitnessFunction::passes_thresholds(&report, &thresholds));
    }

    #[test]
    fn test_calculate_fitness() {
        let metrics = create_test_metrics();
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let weights = FitnessWeights::default();

        let fitness = FitnessFunction::calculate_fitness(&report, &weights);

        assert!(fitness > 0.0);
        assert!(fitness <= 1.0);
    }

    #[test]
    fn test_calculate_fitness_zero_weights() {
        let metrics = create_test_metrics();
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let weights = FitnessWeights {
            sharpe_ratio_weight: 0.0,
            profit_factor_weight: 0.0,
            win_rate_weight: 0.0,
            cagr_weight: 0.0,
            recovery_factor_weight: 0.0,
            drawdown_penalty: 0.0,
            trades_count_bonus: 0.0,
        };

        let fitness = FitnessFunction::calculate_fitness(&report, &weights);

        assert_eq!(fitness, 0.0);
    }

    #[test]
    fn test_evaluate_strategy_passes() {
        let metrics = create_test_metrics();
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();
        let weights = FitnessWeights::default();

        let result = FitnessFunction::evaluate_strategy(&report, &thresholds, &weights);

        assert!(result.is_some());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_evaluate_strategy_fails() {
        let mut metrics = create_test_metrics();
        metrics.sharpe_ratio = Some(0.5);
        let trades = vec![create_test_trade(100.0); 50];
        let report = create_test_report(trades, metrics);
        let thresholds = FitnessThresholds::default();
        let weights = FitnessWeights::default();

        let result = FitnessFunction::evaluate_strategy(&report, &thresholds, &weights);

        assert!(result.is_none());
    }

    #[test]
    fn test_normalize_sharpe_ratio() {
        assert_eq!(FitnessFunction::normalize_sharpe_ratio(Some(3.0)), 1.0);
        assert_eq!(FitnessFunction::normalize_sharpe_ratio(Some(1.5)), 0.5);
        assert_eq!(FitnessFunction::normalize_sharpe_ratio(Some(-1.0)), 0.0);
        assert_eq!(FitnessFunction::normalize_sharpe_ratio(None), 0.0);
    }

    #[test]
    fn test_normalize_profit_factor() {
        assert_eq!(FitnessFunction::normalize_profit_factor(Some(5.0)), 1.0);
        assert_eq!(FitnessFunction::normalize_profit_factor(Some(2.5)), 0.5);
        assert_eq!(FitnessFunction::normalize_profit_factor(Some(-1.0)), 0.0);
        assert_eq!(FitnessFunction::normalize_profit_factor(None), 0.0);
    }

    #[test]
    fn test_normalize_cagr() {
        assert_eq!(FitnessFunction::normalize_cagr(Some(100.0)), 1.0);
        assert_eq!(FitnessFunction::normalize_cagr(Some(50.0)), 0.5);
        assert_eq!(FitnessFunction::normalize_cagr(Some(-10.0)), 0.0);
        assert_eq!(FitnessFunction::normalize_cagr(None), 0.0);
    }

    #[test]
    fn test_normalize_recovery_factor() {
        assert_eq!(
            FitnessFunction::normalize_recovery_factor(Some(10.0), Some(2.0)),
            1.0
        );
        assert_eq!(
            FitnessFunction::normalize_recovery_factor(Some(5.0), Some(2.0)),
            0.5
        );
        assert_eq!(
            FitnessFunction::normalize_recovery_factor(Some(0.0), Some(2.0)),
            0.0
        );
        assert_eq!(
            FitnessFunction::normalize_recovery_factor(None, Some(2.0)),
            0.0
        );
    }

    #[test]
    fn test_calculate_drawdown_penalty() {
        assert_eq!(FitnessFunction::calculate_drawdown_penalty(Some(50.0)), 1.0);
        assert_eq!(FitnessFunction::calculate_drawdown_penalty(Some(25.0)), 0.5);
        assert_eq!(FitnessFunction::calculate_drawdown_penalty(Some(0.0)), 0.0);
        assert_eq!(FitnessFunction::calculate_drawdown_penalty(None), 0.0);
    }

    #[test]
    fn test_calculate_trades_bonus() {
        assert_eq!(FitnessFunction::calculate_trades_bonus(100), 1.0);
        assert_eq!(FitnessFunction::calculate_trades_bonus(75), 0.75);
        assert_eq!(FitnessFunction::calculate_trades_bonus(40), 0.5);
        assert_eq!(FitnessFunction::calculate_trades_bonus(15), 0.5);
        assert_eq!(FitnessFunction::calculate_trades_bonus(0), 0.0);
    }

    #[test]
    fn test_fitness_thresholds_default() {
        let thresholds = FitnessThresholds::default();
        assert_eq!(thresholds.min_sharpe_ratio, Some(1.0));
        assert_eq!(thresholds.max_drawdown_pct, Some(20.0));
        assert_eq!(thresholds.min_win_rate, Some(0.45));
        assert_eq!(thresholds.min_profit_factor, Some(1.5));
        assert_eq!(thresholds.min_trades_count, Some(30));
        assert_eq!(thresholds.min_cagr, Some(10.0));
        assert_eq!(thresholds.min_recovery_factor, Some(1.0));
    }

    #[test]
    fn test_fitness_weights_default() {
        let weights = FitnessWeights::default();
        assert_eq!(weights.sharpe_ratio_weight, 0.25);
        assert_eq!(weights.profit_factor_weight, 0.20);
        assert_eq!(weights.win_rate_weight, 0.10);
        assert_eq!(weights.cagr_weight, 0.15);
        assert_eq!(weights.recovery_factor_weight, 0.20);
        assert_eq!(weights.drawdown_penalty, 0.05);
        assert_eq!(weights.trades_count_bonus, 0.05);
    }
}
