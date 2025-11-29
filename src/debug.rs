use crate::data_model::types::TimeFrame;
use crate::metrics::backtest::{BacktestMetrics, BacktestReport, StrategyTrade};
use crate::strategy::context::StrategyContext;

pub struct DebugConfig {
    pub show_metrics: bool,
    pub show_indicators: bool,
    pub indicator_count: usize,
    pub show_first_trades: usize,
    pub show_last_trades: usize,
    pub show_stop_take_details: usize,
    pub show_conditions: bool,
    pub condition_signals_count: usize,
    pub only_triggered_conditions: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            show_metrics: true,
            show_indicators: true,
            indicator_count: 100,
            show_first_trades: 100,
            show_last_trades: 100,
            show_stop_take_details: 10,
            show_conditions: true,
            condition_signals_count: 50,
            only_triggered_conditions: false,
        }
    }
}

pub fn print_strategy_debug(
    report: &BacktestReport,
    context: &StrategyContext,
    strategy_name: &str,
    symbol: &str,
    timeframe: &TimeFrame,
    config: &DebugConfig,
) {
    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                        ОТЛАДКА СТРАТЕГИИ                                     ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║ Стратегия: {:66} ║", strategy_name);
    println!("║ Символ: {:69} ║", symbol);
    println!("║ Таймфрейм: {:66} ║", timeframe.identifier());
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");

    if config.show_metrics {
        print_metrics_table(&report.metrics);
    }

    if config.show_indicators {
        print_indicators(context, timeframe, config.indicator_count);
    }

    if config.show_conditions {
        print_conditions_summary(context, timeframe);
        print_conditions_signals(
            context,
            timeframe,
            config.condition_signals_count,
            config.only_triggered_conditions,
        );
    }

    print_trades_summary(
        &report.trades,
        config.show_first_trades,
        config.show_last_trades,
    );

    print_stop_take_details(
        &report.trades,
        context,
        timeframe,
        config.show_stop_take_details,
    );
}

pub fn print_metrics_table(metrics: &BacktestMetrics) {
    println!("\n┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                              МЕТРИКИ БЭКТЕСТА                                │");
    println!("├──────────────────────────────────────────────────────────────────────────────┤");

    println!("│ {:^76} │", "═══ БАЗОВЫЕ МЕТРИКИ ═══");
    print_metric_row("Total Trades", Some(metrics.total_trades as f64));
    print_metric_row("Number of Wins", Some(metrics.number_of_wins as f64));
    print_metric_row("Number of Losses", Some(metrics.number_of_losses as f64));
    print_metric_row("Total Profit", Some(metrics.total_profit));
    print_metric_row("Gross Profit", Some(metrics.gross_profit));
    print_metric_row("Gross Loss", Some(metrics.gross_loss));
    print_metric_row("Average Trade", Some(metrics.average_trade));
    print_metric_row("Average Win", metrics.average_win);
    print_metric_row("Average Loss", metrics.average_loss);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ МЕТРИКИ РИСКА И ДОХОДНОСТИ ═══");
    print_metric_row_pct(
        "Winning Percentage",
        Some(metrics.winning_percentage * 100.0),
    );
    print_metric_row("Profit Factor", metrics.profit_factor);
    print_metric_row("Sharpe Ratio", metrics.sharpe_ratio);
    print_metric_row("Return/DD Ratio", metrics.return_dd_ratio);
    print_metric_row("Wins/Losses Ratio", metrics.wins_losses_ratio);
    print_metric_row("Payout Ratio", metrics.payout_ratio);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ МЕТРИКИ ПРОСАДКИ ═══");
    print_metric_row("Max Drawdown", metrics.drawdown);
    print_metric_row_pct("Max Drawdown %", metrics.drawdown_percent);
    print_metric_row("Max Consecutive Wins", Some(metrics.max_consec_wins as f64));
    print_metric_row(
        "Max Consecutive Losses",
        Some(metrics.max_consec_losses as f64),
    );

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ ВРЕМЕННЫЕ МЕТРИКИ ═══");
    print_metric_row("Yearly Avg Profit", metrics.yearly_avg_profit);
    print_metric_row_pct("Yearly Avg % Return", metrics.yearly_avg_percent_return);
    print_metric_row_pct("CAGR", metrics.cagr);
    print_metric_row("Monthly Avg Profit", metrics.monthly_avg_profit);
    print_metric_row("Daily Avg Profit", metrics.daily_avg_profit);
    print_metric_row_pct("AHPR", metrics.ahpr);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ СТАТИСТИЧЕСКИЕ МЕТРИКИ ═══");
    print_metric_row("Expectancy", metrics.expectancy);
    print_metric_row("R Expectancy", metrics.r_expectancy);
    print_metric_row("R Expectancy Score", metrics.r_expectancy_score);
    print_metric_row("Deviation", metrics.deviation);
    print_metric_row("STR Quality Number", metrics.str_quality_number);
    print_metric_row("SQN Score", metrics.sqn_score);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ ПРОДВИНУТЫЕ МЕТРИКИ ═══");
    print_metric_row_pct("Exposure", metrics.exposure.map(|e| e * 100.0));
    print_metric_row("Stability", metrics.stability);
    print_metric_row("Z-Score", metrics.z_score);
    print_metric_row("Z-Probability", metrics.z_probability);
    print_metric_row("Symmetry", metrics.symmetry);
    print_metric_row("Trades Symmetry", metrics.trades_symmetry);
    print_metric_row("NSymmetry", metrics.nsymmetry);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ МЕТРИКИ ЗАСТОЯ ═══");
    print_metric_row(
        "Stagnation In Days",
        metrics.stagnation_in_days.map(|d| d as f64),
    );
    print_metric_row_pct("Stagnation %", metrics.stagnation_percent);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ ДОПОЛНИТЕЛЬНЫЕ МЕТРИКИ ═══");
    print_metric_row("Profit In Pips", metrics.profit_in_pips);
    print_metric_row("Annual % / Max DD %", metrics.annual_percent_max_dd_ratio);

    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {:^76} │", "═══ ИНФОРМАЦИЯ О BACKTEST ═══");
    print_metric_row("Initial Capital", Some(metrics.initial_capital));
    print_metric_row("Ending Capital", Some(metrics.ending_capital));
    print_metric_row("Total Bars", Some(metrics.total_bars as f64));
    print_metric_row("Bars In Positions", Some(metrics.bars_in_positions as f64));

    if let Some(sd) = metrics.start_date {
        println!(
            "│ {:40} {:>35} │",
            "Start Date",
            sd.format("%Y-%m-%d %H:%M:%S")
        );
    }
    if let Some(ed) = metrics.end_date {
        println!(
            "│ {:40} {:>35} │",
            "End Date",
            ed.format("%Y-%m-%d %H:%M:%S")
        );
    }

    println!("└──────────────────────────────────────────────────────────────────────────────┘");
}

fn print_metric_row(name: &str, value: Option<f64>) {
    match value {
        Some(v) if v.is_finite() => {
            println!("│ {:40} {:>35.4} │", name, v);
        }
        Some(_) => {
            println!("│ {:40} {:>35} │", name, "∞");
        }
        None => {
            println!("│ {:40} {:>35} │", name, "N/A");
        }
    }
}

fn print_metric_row_pct(name: &str, value: Option<f64>) {
    match value {
        Some(v) if v.is_finite() => {
            println!("│ {:40} {:>34.2}% │", name, v);
        }
        Some(_) => {
            println!("│ {:40} {:>35} │", name, "∞");
        }
        None => {
            println!("│ {:40} {:>35} │", name, "N/A");
        }
    }
}

pub fn print_indicators(context: &StrategyContext, timeframe: &TimeFrame, count: usize) {
    println!("\n┌──────────────────────────────────────────────────────────────────────────────┐");
    println!(
        "│                    ПОСЛЕДНИЕ {} ЗНАЧЕНИЙ ИНДИКАТОРОВ                        │",
        count
    );
    println!("└──────────────────────────────────────────────────────────────────────────────┘");

    let data = match context.timeframe(timeframe) {
        Ok(d) => d,
        Err(_) => {
            println!(
                "  [!] Данные для таймфрейма {} не найдены",
                timeframe.identifier()
            );
            return;
        }
    };

    let mut indicators: Vec<(String, Vec<f32>)> = Vec::new();

    let price_fields = [
        (crate::strategy::types::PriceField::Open, "Open"),
        (crate::strategy::types::PriceField::High, "High"),
        (crate::strategy::types::PriceField::Low, "Low"),
        (crate::strategy::types::PriceField::Close, "Close"),
        (crate::strategy::types::PriceField::Volume, "Volume"),
    ];

    for (field, name) in price_fields.iter() {
        if let Some(series) = data.price_series_slice(field) {
            let start = series.len().saturating_sub(count);
            let values: Vec<f32> = series[start..].to_vec();
            if !values.is_empty() {
                indicators.push((format!("PRICE:{}", name), values));
            }
        }
    }

    let indicator_aliases = data.indicator_aliases();
    for (i, alias) in indicator_aliases.iter().enumerate() {
        if let Some(series) = data.indicator_by_index(i) {
            let start = series.len().saturating_sub(count);
            let values: Vec<f32> = series[start..].to_vec();
            if !values.is_empty() {
                indicators.push((format!("IND:{}", alias), values));
            }
        }
    }

    for alias in data.auxiliary_aliases() {
        if let Some(series) = data.auxiliary_series_slice(alias) {
            let start = series.len().saturating_sub(count);
            let values: Vec<f32> = series[start..].to_vec();
            if !values.is_empty() {
                indicators.push((format!("AUX:{}", alias), values));
            }
        }
    }

    if indicators.is_empty() {
        println!("  [!] Индикаторы не найдены");
        return;
    }

    println!("\nНайдено {} серий данных:\n", indicators.len());

    for (name, values) in &indicators {
        println!("┌─ {} ({} значений) ─", name, values.len());
        for (i, chunk) in values.chunks(10).enumerate() {
            let start_idx = i * 10;
            let chunk_str: Vec<String> = chunk.iter().map(|v| format!("{:.4}", v)).collect();
            println!(
                "│ [{:3}-{:3}]: {}",
                start_idx,
                start_idx + chunk.len() - 1,
                chunk_str.join(" | ")
            );
        }
        println!("└─");
    }
}

pub fn print_indicators_from_executor(
    context: &StrategyContext,
    timeframe: &TimeFrame,
    count: usize,
) {
    print_indicators(context, timeframe, count);
}

pub fn print_trades_summary(trades: &[StrategyTrade], first_count: usize, last_count: usize) {
    println!("\n┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                              СДЕЛКИ                                          │");
    println!("├──────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Всего сделок: {:62} │", trades.len());
    println!("└──────────────────────────────────────────────────────────────────────────────┘");

    if trades.is_empty() {
        println!("  [!] Сделки отсутствуют");
        return;
    }

    let first_trades: Vec<&StrategyTrade> = trades.iter().take(first_count).collect();
    let last_trades: Vec<&StrategyTrade> = trades.iter().rev().take(last_count).rev().collect();

    if !first_trades.is_empty() {
        println!(
            "\n┌── ПЕРВЫЕ {} СДЕЛОК ──────────────────────────────────────────────────────────┐",
            first_trades.len()
        );
        print_trades_table(&first_trades);
    }

    if trades.len() > first_count {
        println!(
            "\n┌── ПОСЛЕДНИЕ {} СДЕЛОК ─────────────────────────────────────────────────────────┐",
            last_trades.len()
        );
        print_trades_table(&last_trades);
    }
}

fn print_trades_table(trades: &[&StrategyTrade]) {
    println!(
        "│ {:3} │ {:7} │ {:>10} │ {:>10} │ {:>12} │ {:16} │ {:16} │ {:15} │",
        "#", "Dir", "Entry", "Exit", "PnL", "Entry Time", "Exit Time", "Exit Rule"
    );
    println!("├─────┼─────────┼────────────┼────────────┼──────────────┼──────────────────┼──────────────────┼─────────────────┤");

    for (i, trade) in trades.iter().enumerate() {
        let entry_time = trade
            .entry_time
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let exit_time = trade
            .exit_time
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let exit_rule = trade.exit_rule_id.as_deref().unwrap_or("N/A");
        let direction = format!("{:?}", trade.direction);

        let pnl_str = if trade.pnl >= 0.0 {
            format!("+{:.2}", trade.pnl)
        } else {
            format!("{:.2}", trade.pnl)
        };

        println!(
            "│ {:3} │ {:7} │ {:>10.2} │ {:>10.2} │ {:>12} │ {:16} │ {:16} │ {:15} │",
            i + 1,
            direction,
            trade.entry_price,
            trade.exit_price,
            pnl_str,
            entry_time,
            exit_time,
            &exit_rule[..exit_rule.len().min(15)]
        );
    }
    println!("└─────┴─────────┴────────────┴────────────┴──────────────┴──────────────────┴──────────────────┴─────────────────┘");
}

pub fn print_stop_take_details(
    trades: &[StrategyTrade],
    context: &StrategyContext,
    timeframe: &TimeFrame,
    count: usize,
) {
    println!("\n┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                  ДЕТАЛИ СТОП-ЛОССОВ НА БАРАХ СДЕЛОК                          │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘");

    if trades.is_empty() {
        println!("  [!] Сделки отсутствуют");
        return;
    }

    let first_trades: Vec<&StrategyTrade> = trades.iter().take(count).collect();
    let last_trades: Vec<&StrategyTrade> = trades.iter().rev().take(count).rev().collect();

    println!(
        "\n┌── ПЕРВЫЕ {} СДЕЛОК (значения стопов на каждом баре) ───────────────────────┐",
        first_trades.len().min(count)
    );
    print_stop_values_per_bar(&first_trades, context, timeframe);

    if trades.len() > count {
        println!(
            "\n┌── ПОСЛЕДНИЕ {} СДЕЛОК (значения стопов на каждом баре) ──────────────────────┐",
            last_trades.len().min(count)
        );
        print_stop_values_per_bar(&last_trades, context, timeframe);
    }
}

fn print_stop_values_per_bar(
    trades: &[&StrategyTrade],
    context: &StrategyContext,
    timeframe: &TimeFrame,
) {
    let data = match context.timeframe(timeframe) {
        Ok(d) => d,
        Err(_) => {
            println!(
                "  [!] Данные для таймфрейма {} не найдены",
                timeframe.identifier()
            );
            return;
        }
    };

    let timestamps = data.timestamps_slice();
    let closes = data.price_series_slice(&crate::strategy::types::PriceField::Close);
    let highs = data.price_series_slice(&crate::strategy::types::PriceField::High);
    let lows = data.price_series_slice(&crate::strategy::types::PriceField::Low);

    for (trade_idx, trade) in trades.iter().enumerate() {
        let entry_time = trade.entry_time;
        let exit_time = trade.exit_time;

        let entry_bar_idx = find_bar_index_by_time(timestamps, entry_time);
        let exit_bar_idx = find_bar_index_by_time(timestamps, exit_time);

        println!(
            "\n═══════════════════════════════════════════════════════════════════════════════"
        );
        println!(
            "СДЕЛКА #{}: {:?} | Вход: {:.2} ({}) → Выход: {:.2} ({}) | PnL: {:+.2}",
            trade_idx + 1,
            trade.direction,
            trade.entry_price,
            entry_time
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or("N/A".to_string()),
            trade.exit_price,
            exit_time
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or("N/A".to_string()),
            trade.pnl
        );
        println!(
            "Exit Rule: {}",
            trade.exit_rule_id.as_deref().unwrap_or("N/A")
        );
        println!("───────────────────────────────────────────────────────────────────────────────");

        match (entry_bar_idx, exit_bar_idx) {
            (Some(start), Some(end)) if start <= end => {
                println!(
                    "│ {:5} │ {:16} │ {:>10} │ {:>10} │ {:>10} │ {:>10} │ {:>12} │",
                    "Bar", "Дата/Время", "Close", "High", "Low", "MaxHigh", "Trail Stop"
                );
                println!("├───────┼──────────────────┼────────────┼────────────┼────────────┼────────────┼──────────────┤");

                let stop_history_map: std::collections::HashMap<usize, &crate::position::StopHistoryEntry> = 
                    trade.stop_history.iter().map(|h| (h.bar_index, h)).collect();

                for bar_idx in start..=end {
                    let time_str = timestamps
                        .and_then(|ts| ts.get(bar_idx))
                        .and_then(|millis| crate::data_model::types::timestamp_from_millis(*millis))
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| format!("{}", bar_idx));

                    let close = closes.and_then(|c| c.get(bar_idx).copied()).unwrap_or(0.0);
                    let high = highs.and_then(|h| h.get(bar_idx).copied()).unwrap_or(0.0);
                    let low = lows.and_then(|l| l.get(bar_idx).copied()).unwrap_or(0.0);

                    let (max_high, trail_stop) = if let Some(history) = stop_history_map.get(&bar_idx) {
                        (history.max_high as f32, history.stop_level as f32)
                    } else {
                        let prev_history = trade.stop_history.iter()
                            .filter(|h| h.bar_index <= bar_idx)
                            .max_by_key(|h| h.bar_index);
                        match prev_history {
                            Some(h) => (h.max_high as f32, h.stop_level as f32),
                            None => (high, 0.0),
                        }
                    };

                    let bar_marker = if bar_idx == start {
                        "→"
                    } else if bar_idx == end {
                        "←"
                    } else {
                        " "
                    };

                    println!(
                        "│{}{:5} │ {:16} │ {:>10.2} │ {:>10.2} │ {:>10.2} │ {:>10.2} │ {:>12.2} │",
                        bar_marker, bar_idx, time_str, close, high, low, max_high, trail_stop
                    );
                }
                println!("└───────┴──────────────────┴────────────┴────────────┴────────────┴────────────┴──────────────┘");
                if trade.stop_history.is_empty() {
                    println!("  [!] История стопов не записана");
                } else {
                    println!("  → = бар входа, ← = бар выхода | Trail Stop из сохранённой истории");
                }
            }
            _ => {
                println!("  [!] Не удалось определить бары входа/выхода для сделки");
                println!(
                    "  Entry bar: {:?}, Exit bar: {:?}",
                    entry_bar_idx, exit_bar_idx
                );
            }
        }
    }
}

fn find_bar_index_by_time(
    timestamps: Option<&[i64]>,
    target_time: Option<chrono::DateTime<chrono::Utc>>,
) -> Option<usize> {
    let ts = timestamps?;
    let target = target_time?;
    let target_millis = target.timestamp_millis();

    ts.iter()
        .enumerate()
        .min_by_key(|(_, &ts_millis)| (ts_millis - target_millis).abs())
        .map(|(idx, _)| idx)
}

pub fn print_quick_summary(report: &BacktestReport, strategy_name: &str) {
    let m = &report.metrics;

    println!("\n╔═══════════════════════════════════════════════════════════════════╗");
    println!("║ КРАТКИЙ ОТЧЕТ: {:51} ║", strategy_name);
    println!("╠═══════════════════════════════════════════════════════════════════╣");
    println!(
        "║ Сделок: {:5} │ Win: {:5} │ Loss: {:5} │ Win%: {:6.2}%         ║",
        m.total_trades,
        m.number_of_wins,
        m.number_of_losses,
        m.winning_percentage * 100.0
    );
    println!(
        "║ Profit: {:+10.2} │ PF: {:6} │ Sharpe: {:6} │ DD%: {:6} ║",
        m.total_profit,
        m.profit_factor
            .map(|v| format!("{:.2}", v))
            .unwrap_or("N/A".to_string()),
        m.sharpe_ratio
            .map(|v| format!("{:.2}", v))
            .unwrap_or("N/A".to_string()),
        m.drawdown_percent
            .map(|v| format!("{:.2}%", v))
            .unwrap_or("N/A".to_string())
    );
    println!("╚═══════════════════════════════════════════════════════════════════╝");
}

pub fn print_conditions_signals(
    context: &StrategyContext,
    timeframe: &TimeFrame,
    last_n: usize,
    only_triggered: bool,
) {
    println!("\n┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                      СРАБАТЫВАНИЯ УСЛОВИЙ                                    │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘");

    let data = match context.timeframe(timeframe) {
        Ok(d) => d,
        Err(_) => {
            println!(
                "  [!] Данные для таймфрейма {} не найдены",
                timeframe.identifier()
            );
            return;
        }
    };

    let condition_ids = data.condition_ids_ordered();
    if condition_ids.is_empty() {
        println!("  [!] Условия не найдены");
        return;
    }

    let timestamps = data.timestamps_slice();

    let mut condition_results: Vec<(
        &String,
        Option<&crate::condition::types::ConditionResultData>,
    )> = Vec::new();
    for (cond_id, _idx) in &condition_ids {
        let result = data.condition_result(cond_id);
        condition_results.push((cond_id, result));
    }

    let data_len = condition_results
        .iter()
        .filter_map(|(_, r)| r.map(|res| res.signals.len()))
        .max()
        .unwrap_or(0);

    if data_len == 0 {
        println!("  [!] Нет данных о срабатывании условий");
        return;
    }

    println!("\nУсловия ({}):", condition_ids.len());
    for (i, (cond_id, _)) in condition_ids.iter().enumerate() {
        println!("  C{}: {}", i + 1, cond_id);
    }
    println!();

    let mut header = format!("│ {:5} │ {:19} │", "#", "Дата/Время");
    for i in 0..condition_ids.len() {
        header.push_str(&format!(" C{:1} │", i + 1));
    }
    header.push_str(" All │");

    let separator_len = header.chars().count();
    let separator: String = "─".repeat(separator_len.saturating_sub(2));

    println!("┌{}┐", separator);
    println!("{}", header);
    println!("├{}┤", separator);

    let mut rows_to_show: Vec<usize> = Vec::new();

    if only_triggered {
        for bar_idx in 0..data_len {
            let any_triggered = condition_results.iter().any(|(_, res)| {
                res.map(|r| r.signals.get(bar_idx).copied().unwrap_or(false))
                    .unwrap_or(false)
            });
            if any_triggered {
                rows_to_show.push(bar_idx);
            }
        }
        let start = rows_to_show.len().saturating_sub(last_n);
        rows_to_show = rows_to_show[start..].to_vec();
    } else {
        let start = data_len.saturating_sub(last_n);
        rows_to_show = (start..data_len).collect();
    }

    for bar_idx in rows_to_show {
        let time_str = timestamps
            .and_then(|ts| ts.get(bar_idx))
            .and_then(|millis| crate::data_model::types::timestamp_from_millis(*millis))
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| format!("Bar {}", bar_idx));

        let mut row = format!("│ {:5} │ {:19} │", bar_idx, time_str);

        let mut all_true = true;
        for (_, res) in &condition_results {
            let signal = res
                .map(|r| r.signals.get(bar_idx).copied().unwrap_or(false))
                .unwrap_or(false);

            let symbol = if signal { " ✓ " } else { " · " };
            row.push_str(&format!("{}│", symbol));

            if !signal {
                all_true = false;
            }
        }

        let all_symbol = if all_true { " ✓ " } else { " · " };
        row.push_str(&format!(" {} │", all_symbol));

        println!("{}", row);
    }

    println!("└{}┘", separator);

    println!("\nСтатистика срабатываний:");
    for (i, (cond_id, res)) in condition_results.iter().enumerate() {
        if let Some(result) = res {
            let total = result.signals.len();
            let triggered: usize = result.signals.iter().filter(|&&s| s).count();
            let pct = if total > 0 {
                triggered as f64 / total as f64 * 100.0
            } else {
                0.0
            };
            println!(
                "  C{} [{}]: {} / {} ({:.2}%)",
                i + 1,
                cond_id,
                triggered,
                total,
                pct
            );
        } else {
            println!("  C{} [{}]: N/A", i + 1, cond_id);
        }
    }

    let all_triggered: usize = (0..data_len)
        .filter(|&bar_idx| {
            condition_results.iter().all(|(_, res)| {
                res.map(|r| r.signals.get(bar_idx).copied().unwrap_or(false))
                    .unwrap_or(false)
            })
        })
        .count();
    let all_pct = if data_len > 0 {
        all_triggered as f64 / data_len as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "  Все условия вместе: {} / {} ({:.2}%)",
        all_triggered, data_len, all_pct
    );
}

pub fn print_conditions_summary(context: &StrategyContext, timeframe: &TimeFrame) {
    println!("\n┌──────────────────────────────────────────────────────────────────────────────┐");
    println!("│                      СВОДКА ПО УСЛОВИЯМ                                      │");
    println!("└──────────────────────────────────────────────────────────────────────────────┘");

    let data = match context.timeframe(timeframe) {
        Ok(d) => d,
        Err(_) => {
            println!(
                "  [!] Данные для таймфрейма {} не найдены",
                timeframe.identifier()
            );
            return;
        }
    };

    let condition_ids = data.condition_ids_ordered();
    if condition_ids.is_empty() {
        println!("  [!] Условия не найдены");
        return;
    }

    println!(
        "\n│ {:3} │ {:40} │ {:>10} │ {:>10} │ {:>8} │",
        "#", "ID Условия", "Сработало", "Всего", "Процент"
    );
    println!(
        "├─────┼──────────────────────────────────────────┼────────────┼────────────┼──────────┤"
    );

    let mut total_bars = 0usize;
    let mut all_conditions_triggered = 0usize;

    for (i, (cond_id, _)) in condition_ids.iter().enumerate() {
        if let Some(result) = data.condition_result(cond_id) {
            let total = result.signals.len();
            let triggered: usize = result.signals.iter().filter(|&&s| s).count();
            let pct = if total > 0 {
                triggered as f64 / total as f64 * 100.0
            } else {
                0.0
            };

            println!(
                "│ {:3} │ {:40} │ {:>10} │ {:>10} │ {:>7.2}% │",
                i + 1,
                &cond_id[..cond_id.len().min(40)],
                triggered,
                total,
                pct
            );

            if total > total_bars {
                total_bars = total;
            }
        } else {
            println!(
                "│ {:3} │ {:40} │ {:>10} │ {:>10} │ {:>8} │",
                i + 1,
                &cond_id[..cond_id.len().min(40)],
                "N/A",
                "N/A",
                "N/A"
            );
        }
    }

    if total_bars > 0 {
        let condition_results: Vec<Option<&crate::condition::types::ConditionResultData>> =
            condition_ids
                .iter()
                .map(|(cond_id, _)| data.condition_result(cond_id))
                .collect();

        all_conditions_triggered = (0..total_bars)
            .filter(|&bar_idx| {
                condition_results.iter().all(|res| {
                    res.map(|r| r.signals.get(bar_idx).copied().unwrap_or(false))
                        .unwrap_or(false)
                })
            })
            .count();
    }

    println!(
        "├─────┴──────────────────────────────────────────┴────────────┴────────────┴──────────┤"
    );
    let all_pct = if total_bars > 0 {
        all_conditions_triggered as f64 / total_bars as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "│ ВСЕ УСЛОВИЯ ВМЕСТЕ: {:>10} / {:>10} ({:>7.2}%)                           │",
        all_conditions_triggered, total_bars, all_pct
    );
    println!(
        "└──────────────────────────────────────────────────────────────────────────────────────┘"
    );
}

pub fn print_equity_curve_summary(equity_curve: &[f64]) {
    if equity_curve.is_empty() {
        println!("  [!] Кривая эквити пуста");
        return;
    }

    println!("\n┌── КРИВАЯ ЭКВИТИ ────────────────────────────────────────────────────────────┐");

    let first = equity_curve.first().copied().unwrap_or(0.0);
    let last = equity_curve.last().copied().unwrap_or(0.0);
    let min = equity_curve.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = equity_curve
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    println!(
        "│ Начальная: {:>12.2} │ Конечная: {:>12.2} │ Изменение: {:+.2}%{:>4}│",
        first,
        last,
        if first > 0.0 {
            (last - first) / first * 100.0
        } else {
            0.0
        },
        ""
    );
    println!(
        "│ Минимум:   {:>12.2} │ Максимум: {:>12.2} │ Точек: {:>12} │",
        min,
        max,
        equity_curve.len()
    );

    let last_10: Vec<String> = equity_curve
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|v| format!("{:.0}", v))
        .collect();
    println!("│ Последние 10: {} │", last_10.join(" → "));

    println!("└─────────────────────────────────────────────────────────────────────────────┘");
}
