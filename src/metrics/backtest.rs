use crate::data_model::types::{Symbol, TimeFrame};
use crate::data_model::vector_ops::unsafe_ops;
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

/// Полный набор метрик производительности стратегии
#[derive(Clone, Debug, Default)]
pub struct BacktestMetrics {
    // ===== БАЗОВЫЕ МЕТРИКИ ПРОИЗВОДИТЕЛЬНОСТИ =====
    /// TOTAL PROFIT = ENDING CAPITAL – INITIAL CAPITAL
    /// ENDING CAPITAL = INITIAL CAPITAL + equity (где equity = realized_pnl + unrealized_pnl)
    pub total_profit: f64,
    /// Profit In Pips = TOTAL PROFIT / 1 PIP VALUE
    pub profit_in_pips: Option<f64>,
    /// YEARLY AVG PROFIT = TOTAL PROFIT / NUMBER OF YEARS
    pub yearly_avg_profit: Option<f64>,
    /// YEARLY AVG % RETURN = YEARLY AVG PROFIT / INITIAL CAPITAL
    pub yearly_avg_percent_return: Option<f64>,
    /// CAGR = ((ENDING CAPITAL / INITIAL CAPITAL)^(1 / NUMBER OF YEARS)) - 1
    pub cagr: Option<f64>,
    
    // ===== МЕТРИКИ РИСКА И ДОХОДНОСТИ =====
    /// Sharpe Ratio (требует расчета стандартного отклонения доходности)
    pub sharpe_ratio: Option<f64>,
    /// PROFIT FACTOR = SUM OF WINNING TRADES / SUM OF LOSING TRADES
    pub profit_factor: Option<f64>,
    /// RETURN/DD RATIO = TOTAL PROFIT / DRAWDOWN
    pub return_dd_ratio: Option<f64>,
    /// WINNING PERCENTAGE = Percentage of winning trades in all trades
    pub winning_percentage: f64,
    
    // ===== МЕТРИКИ ПРОСАДКИ =====
    /// DRAWDOWN = Max (EQUITY PEAK – FOLLOWING TROUGH)
    pub drawdown: Option<f64>,
    /// % DRAW DOWN = DRAW DOWN / PREVIOUS CAPITAL PEAK
    pub drawdown_percent: Option<f64>,
    /// MAX CONSEC WINS = Maximum number of wins in a row
    pub max_consec_wins: usize,
    /// MAX CONSEC LOSSES = Maximum number of losses in a row
    pub max_consec_losses: usize,
    
    // ===== СТАТИСТИЧЕСКИЕ МЕТРИКИ =====
    /// EXPECTANCY = (PROPORTION OF WINS × AVERAGE WIN) - (PROPORTION OF LOSSES × AVERAGE LOSS)
    pub expectancy: Option<f64>,
    /// R EXPECTANCY = EXPECTANCY / RISK
    pub r_expectancy: Option<f64>,
    /// R EXPECTANCY SCORE = R EXPECTANCY × (NUMBER OF TRADES / NUMBER OF YEARS)
    pub r_expectancy_score: Option<f64>,
    /// STR Quality Number (требует уточнения формулы)
    pub str_quality_number: Option<f64>,
    /// SQN Score (требует уточнения формулы)
    pub sqn_score: Option<f64>,
    
    // ===== ПРОДВИНУТЫЕ МЕТРИКИ =====
    /// Z-Score (требует расчета)
    pub z_score: Option<f64>,
    /// Z-Probability (требует расчета)
    pub z_probability: Option<f64>,
    /// DEVIATION = Average difference between P(L) of each trade and average trade
    pub deviation: Option<f64>,
    /// EXPOSURE = NUMBER OF BARS IN ALL POSITIONS / TOTAL NUMBER OF BARS IN THE SAMPLE
    pub exposure: Option<f64>,
    
    // ===== МЕТРИКИ СИММЕТРИИ И СТАБИЛЬНОСТИ =====
    /// Symmetry (требует уточнения формулы)
    pub symmetry: Option<f64>,
    /// Trades Symmetry (требует уточнения формулы)
    pub trades_symmetry: Option<f64>,
    /// NSymmetry (требует уточнения формулы)
    pub nsymmetry: Option<f64>,
    /// Stability (требует уточнения формулы)
    pub stability: Option<f64>,
    
    // ===== МЕТРИКИ ЗАСТОЯ =====
    /// STAGNATION IN DAYS = The longest period of making a new high on the equity
    pub stagnation_in_days: Option<usize>,
    /// STAGNATION IN % = STAGNATION IN DAYS / TOTAL NUMBER OF DAYS
    pub stagnation_percent: Option<f64>,
    /// GROSS PROFIT = SUM OF WINS
    pub gross_profit: f64,
    /// GROSS LOSS = SUM OF LOSSES
    pub gross_loss: f64,
    
    // ===== ДОПОЛНИТЕЛЬНЫЕ МЕТРИКИ =====
    /// AHPR = Arithmetic average of yearly profit in %
    pub ahpr: Option<f64>,
    /// MONTHLY AVG PROFIT = TOTAL PROFIT / NUMBER OF MONTHS
    pub monthly_avg_profit: Option<f64>,
    /// DAILY AVG PROFIT = TOTAL PROFIT / NUMBER OF DAYS
    pub daily_avg_profit: Option<f64>,
    /// WINS/LOSSES RATIO = NUMBER OF WINS / NUMBER OF LOSSES
    pub wins_losses_ratio: Option<f64>,
    /// PAYOUT RATIO = AVERAGE WIN / AVERAGE LOSS
    pub payout_ratio: Option<f64>,
    /// ANNUAL % / MAX DD % = CAGR / % DRAW DOWN
    pub annual_percent_max_dd_ratio: Option<f64>,
    /// AVERAGE WIN = GROSS PROFIT / NUMBER OF WINS
    pub average_win: Option<f64>,
    /// AVERAGE LOSS = GROSS LOSS / NUMBER OF LOSSES
    pub average_loss: Option<f64>,
    /// AVERAGE TRADE = TOTAL PROFIT / NUMBER OF TRADES
    pub average_trade: f64,
    
    // ===== БАЗОВАЯ ИНФОРМАЦИЯ =====
    /// Общее количество сделок
    pub total_trades: usize,
    /// Количество прибыльных сделок
    pub number_of_wins: usize,
    /// Количество убыточных сделок
    pub number_of_losses: usize,
    /// Начальный капитал
    pub initial_capital: f64,
    /// Конечный капитал
    pub ending_capital: f64,
    /// Начальная дата backtest
    pub start_date: Option<DateTime<Utc>>,
    /// Конечная дата backtest
    pub end_date: Option<DateTime<Utc>>,
    /// Общее количество баров в выборке
    pub total_bars: usize,
    /// Количество баров в позициях
    pub bars_in_positions: usize,
}

impl BacktestMetrics {
    /// Создает метрики из данных backtest
    /// 
    /// # Аргументы
    /// * `trades` - список закрытых сделок
    /// * `equity_curve` - кривая капитала (equity по времени)
    /// * `initial_capital` - начальный капитал
    /// * `start_date` - начальная дата backtest
    /// * `end_date` - конечная дата backtest
    /// * `total_bars` - общее количество баров в выборке
    /// * `bars_in_positions` - количество баров, когда была открыта позиция
    /// * `pip_value` - значение одного пипса (опционально, для расчета profit_in_pips)
    pub fn from_data(
        trades: &[StrategyTrade],
        equity_curve: &[f64],
        initial_capital: f64,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        total_bars: usize,
        bars_in_positions: usize,
        pip_value: Option<f64>,
    ) -> Self {
        let total_trades = trades.len();
        let ending_capital = equity_curve.last().copied().unwrap_or(initial_capital);
        let total_profit = ending_capital - initial_capital;
        
        let (number_of_wins, number_of_losses, gross_profit, gross_loss, max_consec_wins, max_consec_losses, sum_sq_diff) = 
            Self::calculate_trades_metrics_in_single_pass(trades);
        
        // WINNING PERCENTAGE
        let winning_percentage = if total_trades == 0 {
            0.0
        } else {
            number_of_wins as f64 / total_trades as f64
        };
        
        let average_trade = if total_trades == 0 {
            0.0
        } else {
            total_profit / total_trades as f64
        };
        
        let average_win = if number_of_wins == 0 {
            None
        } else {
            Some(gross_profit / number_of_wins as f64)
        };
        
        let average_loss = if number_of_losses == 0 {
            None
        } else {
            Some(gross_loss / number_of_losses as f64)
        };
        
        let profit_factor = if gross_loss == 0.0 {
            if gross_profit > 0.0 {
                Some(f64::INFINITY)
            } else {
                Some(0.0)
            }
        } else {
            Some(gross_profit / gross_loss)
        };
        
        let wins_losses_ratio = if number_of_losses == 0 {
            if number_of_wins > 0 {
                Some(f64::INFINITY)
            } else {
                None
            }
        } else {
            Some(number_of_wins as f64 / number_of_losses as f64)
        };
        
        let payout_ratio = match (average_win, average_loss) {
            (Some(aw), Some(al)) if al != 0.0 => Some(aw / al),
            (Some(_), Some(0.0)) => Some(f64::INFINITY),
            _ => None,
        };
        
        let proportion_of_wins = winning_percentage;
        let proportion_of_losses = 1.0 - winning_percentage;
        let expectancy = match (average_win, average_loss) {
            (Some(aw), Some(al)) => Some((proportion_of_wins * aw) - (proportion_of_losses * al)),
            _ => None,
        };
        
        // Расчет временных метрик
        let number_of_years = Self::calculate_years(start_date, end_date);
        let number_of_months = Self::calculate_months(start_date, end_date);
        let number_of_days = Self::calculate_days(start_date, end_date);
        
        // YEARLY AVG PROFIT
        let yearly_avg_profit = if number_of_years > 0.0 {
            Some(total_profit / number_of_years)
        } else {
            None
        };
        
        // YEARLY AVG % RETURN
        let yearly_avg_percent_return = if initial_capital > 0.0 {
            yearly_avg_profit.map(|yap| (yap / initial_capital) * 100.0)
        } else {
            None
        };
        
        // CAGR
        let cagr = if initial_capital > 0.0 && number_of_years > 0.0 && ending_capital > 0.0 {
            let ratio = ending_capital / initial_capital;
            if ratio > 0.0 {
                Some((ratio.powf(1.0 / number_of_years) - 1.0) * 100.0)
            } else {
                None
            }
        } else {
            None
        };
        
        // MONTHLY AVG PROFIT
        let monthly_avg_profit = if number_of_months > 0.0 {
            Some(total_profit / number_of_months)
        } else {
            None
        };
        
        // DAILY AVG PROFIT
        let daily_avg_profit = if number_of_days > 0.0 {
            Some(total_profit / number_of_days)
        } else {
            None
        };
        
        // AHPR (Arithmetic average of yearly profit in %)
        let ahpr = if initial_capital > 0.0 && number_of_years > 0.0 {
            yearly_avg_profit.map(|yap| (yap / initial_capital) * 100.0)
        } else {
            None
        };
        
        // EXPOSURE
        let exposure = if total_bars > 0 {
            Some(bars_in_positions as f64 / total_bars as f64)
        } else {
            None
        };
        
        // PROFIT IN PIPS
        let profit_in_pips = pip_value.and_then(|pv| {
            if pv > 0.0 {
                Some(total_profit / pv)
            } else {
                None
            }
        });
        
        // R EXPECTANCY (требует RISK - средний риск на сделку)
        // Пока используем average_loss как proxy для RISK
        let r_expectancy = match (expectancy, average_loss) {
            (Some(exp), Some(al)) if al > 0.0 => Some(exp / al),
            _ => None,
        };
        
        // R EXPECTANCY SCORE
        let r_expectancy_score = if number_of_years > 0.0 {
            r_expectancy.map(|re| re * (total_trades as f64 / number_of_years))
        } else {
            None
        };
        
        let deviation = if total_trades > 0 {
            Some((sum_sq_diff / total_trades as f64).sqrt())
        } else {
            None
        };
        
        let (drawdown, drawdown_percent, stagnation_in_days, stagnation_percent, sharpe_ratio, stability) = 
            Self::calculate_equity_metrics_in_single_pass(equity_curve, initial_capital, start_date, end_date, number_of_years);
        
        // RETURN/DD RATIO
        let return_dd_ratio = drawdown.and_then(|dd| {
            if dd > 0.0 {
                Some(total_profit / dd)
            } else {
                None
            }
        });
        
        // ANNUAL % / MAX DD %
        let annual_percent_max_dd_ratio = match (cagr, drawdown_percent) {
            (Some(c), Some(dd)) if dd > 0.0 => Some(c / dd),
            _ => None,
        };
        
        // Z-Score и Z-Probability (требуют уточнения формулы)
        // Пока оставляем None, нужно уточнить формулу
        
        // STR Quality Number и SQN Score (требуют уточнения формулы)
        // Пока оставляем None, нужно уточнить формулу
        
        // Symmetry, Trades Symmetry, NSymmetry (требуют уточнения формулы)
        // Пока оставляем None, нужно уточнить формулу
        
        Self {
            // Базовые метрики
            total_profit,
            profit_in_pips,
            yearly_avg_profit,
            yearly_avg_percent_return,
            cagr,
            
            // Метрики риска и доходности
            sharpe_ratio,
            profit_factor,
            return_dd_ratio,
            winning_percentage,
            
            // Метрики просадки
            drawdown,
            drawdown_percent,
            max_consec_wins,
            max_consec_losses,
            
            // Статистические метрики
            expectancy,
            r_expectancy,
            r_expectancy_score,
            str_quality_number: None, // Требует уточнения
            sqn_score: None, // Требует уточнения
            
            // Продвинутые метрики
            z_score: None, // Требует уточнения
            z_probability: None, // Требует уточнения
            deviation,
            exposure,
            
            // Метрики симметрии и стабильности
            symmetry: None, // Требует уточнения
            trades_symmetry: None, // Требует уточнения
            nsymmetry: None, // Требует уточнения
            stability,
            
            // Метрики застоя
            stagnation_in_days,
            stagnation_percent,
            gross_profit,
            gross_loss,
            
            // Дополнительные метрики
            ahpr,
            monthly_avg_profit,
            daily_avg_profit,
            wins_losses_ratio,
            payout_ratio,
            annual_percent_max_dd_ratio,
            average_win,
            average_loss,
            average_trade,
            
            // Базовая информация
            total_trades,
            number_of_wins,
            number_of_losses,
            initial_capital,
            ending_capital,
            start_date,
            end_date,
            total_bars,
            bars_in_positions,
        }
    }
    
    fn calculate_trades_metrics_in_single_pass(
        trades: &[StrategyTrade],
    ) -> (usize, usize, f64, f64, usize, usize, f64) {
        let mut number_of_wins = 0;
        let mut number_of_losses = 0;
        let mut gross_profit = 0.0;
        let mut gross_loss = 0.0;
        let mut max_wins = 0;
        let mut max_losses = 0;
        let mut current_wins = 0;
        let mut current_losses = 0;
        let mut sum_pnl = 0.0;
        
        for trade in trades {
            let pnl = trade.pnl;
            sum_pnl += pnl;
            
            if pnl > 0.0 {
                number_of_wins += 1;
                gross_profit += pnl;
                current_wins += 1;
                current_losses = 0;
                max_wins = max_wins.max(current_wins);
            } else if pnl < 0.0 {
                number_of_losses += 1;
                gross_loss += pnl.abs();
                current_losses += 1;
                current_wins = 0;
                max_losses = max_losses.max(current_losses);
            }
        }
        
        let total_trades = trades.len();
        let average_trade = if total_trades > 0 {
            sum_pnl / total_trades as f64
        } else {
            0.0
        };
        
        let pnl_values: Vec<f64> = trades.iter().map(|t| t.pnl).collect();
        let pnl_values_f32: Vec<f32> = pnl_values.iter().map(|&x| x as f32).collect();
        let average_trade_f32 = average_trade as f32;
        let sum_sq_diff = unsafe_ops::sum_sq_diff_f32_fast(&pnl_values_f32, average_trade_f32) as f64;
        
        (number_of_wins, number_of_losses, gross_profit, gross_loss, max_wins, max_losses, sum_sq_diff)
    }
    
    /// Рассчитывает количество лет между датами
    fn calculate_years(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> f64 {
        match (start, end) {
            (Some(s), Some(e)) => {
                let duration = e.signed_duration_since(s);
                duration.num_seconds() as f64 / (365.25 * 24.0 * 3600.0)
            }
            _ => 0.0,
        }
    }
    
    /// Рассчитывает количество месяцев между датами
    fn calculate_months(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> f64 {
        match (start, end) {
            (Some(s), Some(e)) => {
                let duration = e.signed_duration_since(s);
                duration.num_seconds() as f64 / (30.44 * 24.0 * 3600.0) // Среднее количество дней в месяце
            }
            _ => 0.0,
        }
    }
    
    /// Рассчитывает количество дней между датами
    fn calculate_days(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> f64 {
        match (start, end) {
            (Some(s), Some(e)) => {
                let duration = e.signed_duration_since(s);
                duration.num_days() as f64
            }
            _ => 0.0,
        }
    }
    
    fn calculate_equity_metrics_in_single_pass(
        equity_curve: &[f64],
        initial_capital: f64,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        years: f64,
    ) -> (Option<f64>, Option<f64>, Option<usize>, Option<f64>, Option<f64>, Option<f64>) {
        if equity_curve.is_empty() {
            return (None, None, None, None, None, None);
        }
        
        let n = equity_curve.len() as f64;
        let mut max_drawdown = 0.0;
        let mut max_drawdown_percent = 0.0;
        let mut peak = initial_capital.max(equity_curve[0]);
        
        let mut max_stagnation_bars = 0;
        let mut current_stagnation_bars = 0;
        let mut stagnation_peak = equity_curve[0];
        
        let mut returns_sum = 0.0;
        let mut returns_count = 0;
        let mut prev_equity = equity_curve[0];
        
        let mut y_sum = 0.0;
        let x_mean = (n - 1.0) / 2.0;
        
        for (i, &equity) in equity_curve.iter().enumerate() {
            if equity > peak {
                peak = equity;
            }
            let drawdown = peak - equity;
            let drawdown_pct = if peak > 0.0 {
                (drawdown / peak) * 100.0
            } else {
                0.0
            };
            
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
            if drawdown_pct > max_drawdown_percent {
                max_drawdown_percent = drawdown_pct;
            }
            
            if equity > stagnation_peak {
                stagnation_peak = equity;
                current_stagnation_bars = 0;
            } else {
                current_stagnation_bars += 1;
                max_stagnation_bars = max_stagnation_bars.max(current_stagnation_bars);
            }
            
            if i > 0 && prev_equity > 0.0 {
                let ret = (equity - prev_equity) / prev_equity;
                returns_sum += ret;
                returns_count += 1;
            }
            prev_equity = equity;
            
            y_sum += equity;
        }
        
        let drawdown = if max_drawdown > 0.0 { Some(max_drawdown) } else { None };
        let drawdown_percent = if max_drawdown_percent > 0.0 { Some(max_drawdown_percent) } else { None };
        
        let total_days = Self::calculate_days(start_date, end_date);
        let stagnation_in_days = if n > 0.0 && total_days > 0.0 {
            let days_per_bar = total_days / n;
            Some((max_stagnation_bars as f64 * days_per_bar).round() as usize)
        } else {
            None
        };
        
        let stagnation_percent = match (stagnation_in_days, total_days) {
            (Some(sid), td) if td > 0.0 => {
                let percent = (sid as f64 / td) * 100.0;
                Some(percent.min(100.0))
            }
            _ => None,
        };
        
        let sharpe_ratio = if returns_count > 0 && years > 0.0 {
            let avg_return = returns_sum / returns_count as f64;
            let annualized_return = avg_return * (252.0 / years);
            
            let returns: Vec<f32> = (1..equity_curve.len())
                .filter_map(|i| {
                    let prev = equity_curve[i - 1];
                    if prev > 0.0 {
                        Some(((equity_curve[i] - prev) / prev) as f32)
                    } else {
                        None
                    }
                })
                .collect();
            
            let returns_mean = if returns.is_empty() {
                0.0
            } else {
                unsafe_ops::mean_f32_fast(&returns).unwrap_or(0.0)
            };
            let variance_sum = unsafe_ops::sum_sq_diff_f32_fast(&returns, returns_mean) as f64;
            
            let variance = variance_sum / returns_count as f64;
            let std_dev = variance.sqrt();
            let annualized_std_dev = std_dev * (252.0_f64.sqrt() / years.sqrt());
            
            if annualized_std_dev > 0.0 {
                Some(annualized_return / annualized_std_dev)
            } else {
                None
            }
        } else {
            None
        };
        
        let stability = if n >= 2.0 {
            let y_mean = y_sum / n;
            
            let mut numerator = 0.0;
            let mut denominator = 0.0;
            let mut ss_tot = 0.0;
            
            for (i, &y) in equity_curve.iter().enumerate() {
                let x = i as f64;
                let x_diff = x - x_mean;
                let y_diff = y - y_mean;
                numerator += x_diff * y_diff;
                denominator += x_diff * x_diff;
                ss_tot += y_diff * y_diff;
            }
            
            if denominator == 0.0 {
                None
            } else {
                let slope = numerator / denominator;
                let intercept = y_mean - slope * x_mean;
                
                let mut ss_res = 0.0;
                for (i, &y) in equity_curve.iter().enumerate() {
                    let x = i as f64;
                    let y_pred = slope * x + intercept;
                    let residual = y - y_pred;
                    ss_res += residual * residual;
                }
                
                if ss_tot == 0.0 {
                    Some(1.0)
                } else {
                    let r_squared = 1.0 - (ss_res / ss_tot);
                    Some(r_squared.max(0.0).min(1.0))
                }
            }
        } else {
            None
        };
        
        (drawdown, drawdown_percent, stagnation_in_days, stagnation_percent, sharpe_ratio, stability)
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
    bars_in_positions: usize,
}

impl BacktestAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.trades.clear();
        self.equity_curve.clear();
        self.bars_in_positions = 0;
    }
    
    /// Увеличивает счетчик баров в позициях, если есть открытые позиции
    pub fn increment_bars_in_positions_if_has_positions(&mut self, has_open_positions: bool) {
        if has_open_positions {
            self.bars_in_positions += 1;
        }
    }
    
    /// Возвращает количество баров, когда была открыта позиция
    pub fn bars_in_positions(&self) -> usize {
        self.bars_in_positions
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

    /// Создает отчет с полными метриками
    /// 
    /// # Аргументы
    /// * `initial_capital` - начальный капитал
    /// * `start_date` - начальная дата backtest
    /// * `end_date` - конечная дата backtest
    /// * `total_bars` - общее количество баров в выборке
    /// * `bars_in_positions` - количество баров, когда была открыта позиция
    /// * `pip_value` - значение одного пипса (опционально)
    pub fn build_report(
        &self,
        initial_capital: f64,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        total_bars: usize,
        bars_in_positions: usize,
        pip_value: Option<f64>,
    ) -> BacktestReport {
        let metrics = BacktestMetrics::from_data(
            &self.trades,
            &self.equity_curve,
            initial_capital,
            start_date,
            end_date,
            total_bars,
            bars_in_positions,
            pip_value,
        );
        
        BacktestReport::new(self.trades.clone(), metrics, self.equity_curve.clone())
    }
    
    /// Создает отчет с упрощенными параметрами (для обратной совместимости)
    pub fn build_report_simple(&self, _realized_pnl: f64) -> BacktestReport {
        let initial_capital = self.equity_curve.first().copied().unwrap_or(10000.0);
        
        // Пытаемся определить даты из сделок
        let start_date = self.trades.first()
            .and_then(|t| t.entry_time);
        let end_date = self.trades.last()
            .and_then(|t| t.exit_time);
        
        // Оценка количества баров (предполагаем, что equity_curve соответствует барам)
        let total_bars = self.equity_curve.len();
        let bars_in_positions = self.bars_in_positions;
        
        self.build_report(
            initial_capital,
            start_date,
            end_date,
            total_bars,
            bars_in_positions,
            None,
        )
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
