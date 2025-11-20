use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::Utc;
use robots::candles::aggregator::TimeFrameAggregator;
use robots::data_access::database::clickhouse::OhlcvData;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::discovery::StrategyDiscoveryConfig;
use robots::indicators::registry::IndicatorFactory;
use robots::optimization::*;
use robots::strategy::executor::BacktestExecutor;
use robots::strategy::presets::default_strategy_definitions;
use robots::strategy::types::PriceField;

#[cfg(feature = "profiling")]
use pprof::ProfilerGuard;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

async fn run() -> Result<()> {
    let mut connector = ClickHouseConnector::with_config(ClickHouseConfig::default());
    connector
        .connect()
        .await
        .context("Не удалось подключиться к ClickHouse")?;
    connector
        .ping()
        .await
        .context("ClickHouse не отвечает на ping")?;

    let symbol = Symbol::from_descriptor("AFLT.MM");
    let timeframe = TimeFrame::from_identifier("60");
    let start = Utc::now() - chrono::Duration::days(1000);
    let end = Utc::now() + chrono::Duration::hours(3);

    let candles: Vec<_> = connector
        .get_ohlcv_typed(&symbol, &timeframe, start, end, None)
        .await
        .context("Не удалось получить свечи из ClickHouse")?;

    println!(
        "Получено {} свечей для {} {}",
        candles.len(),
        symbol.descriptor(),
        timeframe.identifier()
    );
    if let Some(last) = candles.last() {
        println!(
            "Последняя свеча: close={}, ts={}",
            last.close, last.timestamp
        );
    }
    if candles.is_empty() {
        println!(
            "Нет данных для {} {} за указанный период",
            symbol.descriptor(),
            timeframe.identifier()
        );
        return Ok(());
    }

    let frame = QuoteFrame::try_from_ohlcv(candles.clone(), symbol.clone(), timeframe.clone())
        .context("Не удалось построить QuoteFrame из данных ClickHouse")?;

    // Расчет индикаторов на базовом таймфрейме 60 минут для проверки
    let close_values: Vec<f32> = frame.closes().iter().collect();

    // Trend SMA (period = 40)
    let trend_sma =
        IndicatorFactory::create_indicator("SMA", HashMap::from([("period".to_string(), 40.0)]))?;
    let trend_sma_values = trend_sma.calculate_simple(&close_values)?;

    let source_frame = frame.clone();
    let mut frames = HashMap::new();
    frames.insert(timeframe.clone(), frame);

    let definition = default_strategy_definitions()
        .into_iter()
        .find(|def| def.metadata.id == "SMA_CROSSOVER_LONG")
        .context("Стратегия SMA_CROSSOVER_LONG не найдена")?;

    let mut executor =
        BacktestExecutor::from_definition(definition, None, frames).map_err(anyhow::Error::new)?;

    #[cfg(feature = "profiling")]
    let _guard = {
        std::fs::create_dir_all("profiling").ok();
        ProfilerGuard::new(100).expect("Failed to start profiler")
    };
    let start_time = std::time::Instant::now();
    let report = executor.run_backtest().map_err(anyhow::Error::new)?;
    let elapsed = start_time.elapsed();
    #[cfg(feature = "profiling")]
    {
        if let Ok(report) = _guard.report().build() {
            let file_path = "profiling/flamegraph-pprof.svg";
            std::fs::remove_file(file_path).ok();
            match std::fs::File::create(file_path) {
                Ok(file) => {
                    if let Err(e) = report.flamegraph(file) {
                        eprintln!("⚠️  Ошибка при записи flamegraph: {}", e);
                    } else {
                        println!("\n✅ Профиль сохранен в {}", file_path);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Ошибка при создании файла {}: {}", file_path, e);
                    eprintln!("   Проверьте права доступа к папке profiling/");
                }
            }
        }
    }

    println!("\n=== ВРЕМЯ ВЫПОЛНЕНИЯ БЭКТЕСТА ===");
    println!(
        "Время выполнения: {:.2} секунд ({:.2} миллисекунд)",
        elapsed.as_secs_f64(),
        elapsed.as_millis() as f64
    );

    println!("Стратегия: SMA_CROSSOVER_LONG");
    println!("Символ: {}", symbol.descriptor());
    println!(
        "Таймфрейм: {} минут",
        timeframe.total_minutes().unwrap_or_default()
    );

    let ema_timeframe = TimeFrame::minutes(240);

    // Расчет EMA 50 на базовом таймфрейме
    let close_values: Vec<f32> = executor
        .context()
        .timeframe(&timeframe)
        .map_err(|e| anyhow::anyhow!("Не удалось получить данные базового таймфрейма: {}", e))?
        .price_series_slice(&robots::strategy::types::PriceField::Close)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены закрытия"))?
        .to_vec();

    println!("\n=== БАЗОВЫЕ МЕТРИКИ ===");
    println!(
        "Всего сделок: {} | Прибыльных: {} | Убыточных: {}",
        report.metrics.total_trades, report.metrics.number_of_wins, report.metrics.number_of_losses
    );
    println!(
        "Total Profit: {:.2} | Win Rate: {:.2}% | Average Trade: {:.2}",
        report.metrics.total_profit,
        report.metrics.winning_percentage * 100.0,
        report.metrics.average_trade
    );

    if let Some(aw) = report.metrics.average_win {
        println!("Average Win: {:.2}", aw);
    }
    if let Some(al) = report.metrics.average_loss {
        println!("Average Loss: {:.2}", al);
    }
    println!(
        "Gross Profit: {:.2} | Gross Loss: {:.2}",
        report.metrics.gross_profit, report.metrics.gross_loss
    );

    println!("\n=== МЕТРИКИ РИСКА И ДОХОДНОСТИ ===");
    if let Some(pf) = report.metrics.profit_factor {
        println!("Profit Factor: {:.2}", pf);
    }
    if let Some(sr) = report.metrics.sharpe_ratio {
        println!("Sharpe Ratio: {:.2}", sr);
    }
    if let Some(rdd) = report.metrics.return_dd_ratio {
        println!("Return/DD Ratio: {:.2}", rdd);
    }
    if let Some(wlr) = report.metrics.wins_losses_ratio {
        println!("Wins/Losses Ratio: {:.2}", wlr);
    }
    if let Some(pr) = report.metrics.payout_ratio {
        println!("Payout Ratio: {:.2}", pr);
    }

    println!("\n=== МЕТРИКИ ПРОСАДКИ ===");
    if let Some(dd) = report.metrics.drawdown {
        println!("Max Drawdown: {:.2}", dd);
    }
    if let Some(dd_pct) = report.metrics.drawdown_percent {
        println!("Max Drawdown %: {:.2}%", dd_pct);
    }
    println!(
        "Max Consecutive Wins: {} | Max Consecutive Losses: {}",
        report.metrics.max_consec_wins, report.metrics.max_consec_losses
    );

    println!("\n=== ВРЕМЕННЫЕ МЕТРИКИ ===");
    if let Some(yap) = report.metrics.yearly_avg_profit {
        println!("Yearly Avg Profit: {:.2}", yap);
    }
    if let Some(yapr) = report.metrics.yearly_avg_percent_return {
        println!("Yearly Avg % Return: {:.2}%", yapr);
    }
    if let Some(cagr) = report.metrics.cagr {
        println!("CAGR: {:.2}%", cagr);
    }
    if let Some(map) = report.metrics.monthly_avg_profit {
        println!("Monthly Avg Profit: {:.2}", map);
    }
    if let Some(dap) = report.metrics.daily_avg_profit {
        println!("Daily Avg Profit: {:.2}", dap);
    }
    if let Some(ahpr) = report.metrics.ahpr {
        println!("AHPR: {:.2}%", ahpr);
    }

    println!("\n=== СТАТИСТИЧЕСКИЕ МЕТРИКИ ===");
    if let Some(exp) = report.metrics.expectancy {
        println!("Expectancy: {:.2}", exp);
    }
    if let Some(re) = report.metrics.r_expectancy {
        println!("R Expectancy: {:.2}", re);
    }
    if let Some(res) = report.metrics.r_expectancy_score {
        println!("R Expectancy Score: {:.2}", res);
    }
    if let Some(dev) = report.metrics.deviation {
        println!("Deviation: {:.2}", dev);
    }

    println!("\n=== ПРОДВИНУТЫЕ МЕТРИКИ ===");
    if let Some(exp) = report.metrics.exposure {
        println!("Exposure: {:.2}%", exp * 100.0);
    }
    if let Some(stab) = report.metrics.stability {
        println!("Stability: {:.4}", stab);
    }

    println!("\n=== МЕТРИКИ ЗАСТОЯ ===");
    if let Some(sid) = report.metrics.stagnation_in_days {
        println!("Stagnation In Days: {}", sid);
    }
    if let Some(sp) = report.metrics.stagnation_percent {
        println!("Stagnation %: {:.2}%", sp);
    }

    println!("\n=== ДОПОЛНИТЕЛЬНЫЕ МЕТРИКИ ===");
    if let Some(apmdd) = report.metrics.annual_percent_max_dd_ratio {
        println!("Annual % / Max DD %: {:.2}", apmdd);
    }
    if let Some(pp) = report.metrics.profit_in_pips {
        println!("Profit In Pips: {:.2}", pp);
    }

    println!("\n=== ИНФОРМАЦИЯ О BACKTEST ===");
    println!(
        "Initial Capital: {:.2} | Ending Capital: {:.2}",
        report.metrics.initial_capital, report.metrics.ending_capital
    );
    if let Some(sd) = report.metrics.start_date {
        println!("Start Date: {}", sd.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(ed) = report.metrics.end_date {
        println!("End Date: {}", ed.format("%Y-%m-%d %H:%M:%S"));
    }
    println!(
        "Total Bars: {} | Bars In Positions: {}",
        report.metrics.total_bars, report.metrics.bars_in_positions
    );

    if report.trades.is_empty() {
        println!("Сделки отсутствуют");
    } else {
        println!("Сделки:");
        for trade in &report.trades {
            let entry_time = trade
                .entry_time
                .map(|ts| ts.to_rfc3339())
                .unwrap_or_else(|| "n/a".to_string());
            let exit_time = trade
                .exit_time
                .map(|ts| ts.to_rfc3339())
                .unwrap_or_else(|| "n/a".to_string());
            let entry_rule = trade.entry_rule_id.as_deref().unwrap_or("n/a");
            let exit_rule = trade.exit_rule_id.as_deref().unwrap_or("n/a");
            println!(
                "- {:?} qty {:.2} вход {:.2} ({}) выход {:.2} ({}) pnl {:.2} [entry_rule: {} | exit_rule: {}]",
                trade.direction,
                trade.quantity,
                trade.entry_price,
                entry_time,
                trade.exit_price,
                exit_time,
                trade.pnl,
                entry_rule,
                exit_rule
            );
        }
    }

    if let Some(last_equity) = report.equity_curve.last() {
        println!("Финальная equity: {:.2}", last_equity);
    }

    println!("\n=== ТАБЛИЦА СИГНАЛОВ И ИНДИКАТОРОВ ===");
    print_signals_table(&executor, &timeframe, &symbol).await?;

    println!("\n=== СЖАТЫЕ СВЕЧИ (1h → 4h) ===");
    print_compressed_candles(&executor, &source_frame).await?;

    println!("\n=== ГЕНЕТИЧЕСКАЯ ОПТИМИЗАЦИЯ ===");
    // run_genetic_optimization(&symbol, &timeframe, candles).await?;

    Ok(())
}

fn align_timestamp_millis_to_240min(timestamp_millis: i64) -> i64 {
    let total_minutes = timestamp_millis / (60 * 1000);
    let aligned_minutes = (total_minutes / 240) * 240;
    aligned_minutes * 60 * 1000
}

#[allow(dead_code)]
fn align_timestamp_to_240min(
    timestamp: chrono::DateTime<chrono::Utc>,
) -> chrono::DateTime<chrono::Utc> {
    let timestamp_millis = timestamp.timestamp_millis();
    let aligned_millis = align_timestamp_millis_to_240min(timestamp_millis);
    chrono::DateTime::from_timestamp(
        aligned_millis / 1000,
        ((aligned_millis % 1000) * 1_000_000) as u32,
    )
    .unwrap_or(timestamp)
}

async fn print_signals_table(
    executor: &BacktestExecutor,
    base_timeframe: &TimeFrame,
    _symbol: &robots::data_model::types::Symbol,
) -> Result<()> {
    use robots::strategy::types::PriceField;

    let context = executor.context();
    let higher_timeframe = TimeFrame::minutes(240);

    let base_data = context
        .timeframe(base_timeframe)
        .map_err(|e| anyhow::anyhow!("Не удалось получить данные базового таймфрейма: {}", e))?;
    let higher_data = context
        .timeframe(&higher_timeframe)
        .map_err(|e| anyhow::anyhow!("Не удалось получить данные старшего таймфрейма: {}", e))?;

    let definition = default_strategy_definitions()
        .into_iter()
        .find(|def| def.metadata.id == "SMA_CROSSOVER_LONG")
        .ok_or_else(|| anyhow::anyhow!("Стратегия SMA_CROSSOVER_LONG не найдена"))?;

    let conditions = &definition.condition_bindings;

    let close_60 = base_data
        .price_series_slice(&PriceField::Close)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены закрытия для 60 минут"))?;
    let close_240 = higher_data
        .price_series_slice(&PriceField::Close)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены закрытия для 240 минут"))?;

    let fast_sma = base_data
        .indicator_series_slice("fast_sma")
        .ok_or_else(|| anyhow::anyhow!("Не найден индикатор fast_sma"))?;
    let slow_sma = base_data
        .indicator_series_slice("slow_sma")
        .ok_or_else(|| anyhow::anyhow!("Не найден индикатор slow_sma"))?;
    let trend_sma = base_data
        .indicator_series_slice("trend_sma")
        .ok_or_else(|| anyhow::anyhow!("Не найден индикатор trend_sma"))?;
    let ema_240 = higher_data
        .indicator_series_slice("ema_240")
        .ok_or_else(|| anyhow::anyhow!("Не найден индикатор ema_240"))?;

    let max_len = close_60
        .len()
        .min(fast_sma.len())
        .min(slow_sma.len())
        .min(trend_sma.len());

    let condition_names: Vec<String> = conditions.iter().map(|c| c.id.clone()).collect();
    let header_width = 20
        + 10 * 6
        + condition_names.len() * 4
        + condition_names.iter().map(|s| s.len()).sum::<usize>();

    println!("\n{:-<1$}", "", header_width.max(200));
    print!(
        "{:<20} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | ",
        "Timestamp", "Close 60", "Close 240", "Fast SMA", "Slow SMA", "Trend SMA", "EMA 240"
    );
    for name in &condition_names {
        print!("{:>4} | ", name);
    }
    println!();
    println!("{:-<1$}", "", header_width.max(200));

    let mut condition_signals: Vec<Vec<bool>> = vec![vec![]; conditions.len()];

    for i in 0..max_len {
        let close_240_idx_for_stats = {
            if let Some(current_ts_millis) = base_data.timestamp_millis_at(i) {
                let aligned_ts_millis = align_timestamp_millis_to_240min(current_ts_millis);
                let mut best_idx = 0;
                for idx in 0..close_240.len() {
                    if let Some(ts_240_millis) = higher_data.timestamp_millis_at(idx) {
                        if ts_240_millis <= aligned_ts_millis {
                            best_idx = idx;
                        } else if ts_240_millis > aligned_ts_millis {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                best_idx
            } else {
                (i / 4).min(close_240.len().saturating_sub(1))
            }
        };

        for (cond_idx, condition) in conditions.iter().enumerate() {
            let signal = if condition.timeframe == *base_timeframe {
                let cond_data = base_data.condition_result(&condition.id);
                cond_data
                    .and_then(|data| data.signals.get(i).copied())
                    .unwrap_or(false)
            } else if condition.timeframe == higher_timeframe {
                let cond_data = higher_data.condition_result(&condition.id);
                cond_data
                    .and_then(|data| data.signals.get(close_240_idx_for_stats).copied())
                    .unwrap_or(false)
            } else {
                false
            };
            if condition_signals[cond_idx].len() <= i {
                condition_signals[cond_idx].push(signal);
            }
        }
    }

    let start_idx = if max_len > 200 { max_len - 200 } else { 0 };
    let end_idx = max_len;

    for i in start_idx..end_idx {
        let ts = base_data
            .timestamp_at(i)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let close_60_val = close_60.get(i).copied().unwrap_or(0.0);
        let fast_sma_val = fast_sma.get(i).copied().unwrap_or(0.0);
        let slow_sma_val = slow_sma.get(i).copied().unwrap_or(0.0);
        let trend_sma_val = trend_sma.get(i).copied().unwrap_or(0.0);

        let close_240_idx = {
            if let Some(current_ts_millis) = base_data.timestamp_millis_at(i) {
                let aligned_ts_millis = align_timestamp_millis_to_240min(current_ts_millis);
                let mut best_idx = 0;
                for idx in 0..close_240.len() {
                    if let Some(ts_240_millis) = higher_data.timestamp_millis_at(idx) {
                        if ts_240_millis <= aligned_ts_millis {
                            best_idx = idx;
                        } else if ts_240_millis > aligned_ts_millis {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                best_idx
            } else {
                (i / 4).min(close_240.len().saturating_sub(1))
            }
        };
        let close_240_val = close_240.get(close_240_idx).copied().unwrap_or(0.0);
        let ema_240_val = ema_240.get(close_240_idx).copied().unwrap_or(0.0);

        let mut signals = Vec::new();
        for condition in conditions.iter() {
            let signal = if condition.timeframe == *base_timeframe {
                let cond_data = base_data.condition_result(&condition.id);
                cond_data
                    .and_then(|data| data.signals.get(i).copied())
                    .unwrap_or(false)
            } else if condition.timeframe == higher_timeframe {
                let cond_data = higher_data.condition_result(&condition.id);
                cond_data
                    .and_then(|data| data.signals.get(close_240_idx).copied())
                    .unwrap_or(false)
            } else {
                false
            };
            signals.push(signal);
        }

        print!(
            "{:<20} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | ",
            ts, close_60_val, close_240_val, fast_sma_val, slow_sma_val, trend_sma_val, ema_240_val
        );
        for &signal in &signals {
            print!("{:>4} | ", if signal { "✓" } else { "" });
        }
        println!();
    }

    if max_len > 200 {
        println!("... (показаны последние 200 строк из {})", max_len);
    }

    println!("{:-<1$}", "", header_width.max(200));

    println!("\n=== СТАТИСТИКА СИГНАЛОВ ===");
    for (cond_idx, condition) in conditions.iter().enumerate() {
        let total_signals: usize = condition_signals[cond_idx]
            .iter()
            .map(|&s| if s { 1 } else { 0 })
            .sum();
        let total = condition_signals[cond_idx].len();
        let percentage = if total > 0 {
            (total_signals as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "{}: {} сигналов из {} ({:.2}%)",
            condition.id, total_signals, total, percentage
        );
    }

    Ok(())
}

async fn print_compressed_candles(
    executor: &BacktestExecutor,
    source_frame: &QuoteFrame,
) -> Result<()> {
    let target_timeframe = TimeFrame::minutes(240);

    let aggregated = TimeFrameAggregator::aggregate(source_frame, target_timeframe.clone())
        .map_err(|e| anyhow::anyhow!("Не удалось агрегировать таймфрейм: {}", e))?;

    let compressed_frame = aggregated.frame();
    let context = executor.context();

    let compressed_data = context
        .timeframe(&target_timeframe)
        .map_err(|e| anyhow::anyhow!("Не удалось получить данные сжатого таймфрейма: {}", e))?;

    let close_240 = compressed_data
        .price_series_slice(&PriceField::Close)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены закрытия для 240 минут"))?;
    let open_240 = compressed_data
        .price_series_slice(&PriceField::Open)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены открытия для 240 минут"))?;
    let high_240 = compressed_data
        .price_series_slice(&PriceField::High)
        .ok_or_else(|| anyhow::anyhow!("Не найдены максимальные цены для 240 минут"))?;
    let low_240 = compressed_data
        .price_series_slice(&PriceField::Low)
        .ok_or_else(|| anyhow::anyhow!("Не найдены минимальные цены для 240 минут"))?;
    let volume_240 = compressed_data
        .price_series_slice(&PriceField::Volume)
        .ok_or_else(|| anyhow::anyhow!("Не найдены объемы для 240 минут"))?;

    let ema_240 = compressed_data
        .indicator_series_slice("ema_240")
        .ok_or_else(|| anyhow::anyhow!("Не найден индикатор ema_240"))?;

    let max_len = close_240.len().min(ema_240.len());

    let header_width = 180;
    println!("\n{:-<1$}", "", header_width);
    print!(
        "{:<20} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | {:>10} | ",
        "Timestamp", "Open", "High", "Low", "Close", "Volume", "EMA 240", "Source Count"
    );
    println!();
    println!("{:-<1$}", "", header_width);

    let start_idx = if max_len > 100 { max_len - 100 } else { 0 };
    let end_idx = max_len;

    for i in start_idx..end_idx {
        let ts = compressed_frame
            .get(i)
            .and_then(|q| Some(q.timestamp().format("%Y-%m-%d %H:%M").to_string()))
            .unwrap_or_else(|| "N/A".to_string());

        let open_val = open_240.get(i).copied().unwrap_or(0.0);
        let high_val = high_240.get(i).copied().unwrap_or(0.0);
        let low_val = low_240.get(i).copied().unwrap_or(0.0);
        let close_val = close_240.get(i).copied().unwrap_or(0.0);
        let volume_val = volume_240.get(i).copied().unwrap_or(0.0);
        let ema_val = ema_240.get(i).copied().unwrap_or(0.0);

        let source_count = aggregated
            .get_source_indices(i)
            .map(|indices| indices.len())
            .unwrap_or(0);

        print!(
            "{:<20} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.2} | {:>10.0} | {:>10.2} | {:>10} | ",
            ts, open_val, high_val, low_val, close_val, volume_val, ema_val, source_count
        );
        println!();
    }

    if max_len > 100 {
        println!("... (показаны последние 100 строк из {})", max_len);
    }

    println!("{:-<1$}", "", header_width);

    Ok(())
}

async fn run_genetic_optimization(
    symbol: &Symbol,
    base_timeframe: &TimeFrame,
    candles: Vec<OhlcvData>,
) -> Result<()> {
    println!("\n🧬 Запуск генетической оптимизации...");
    println!("   Символ: {}", symbol.descriptor());
    println!(
        "   Базовый таймфрейм: {} минут",
        base_timeframe.total_minutes().unwrap_or(60)
    );
    println!("   Количество свечей: {}\n", candles.len());

    let frame = QuoteFrame::try_from_ohlcv(candles, symbol.clone(), base_timeframe.clone())
        .context("Не удалось построить QuoteFrame")?;

    let mut frames = HashMap::new();
    frames.insert(base_timeframe.clone(), frame);

    println!("⚙️  Создание конфигурации генетического алгоритма...");
    let config = GeneticAlgorithmConfig {
        population_size: 30,
        max_generations: 5,
        crossover_rate: 0.7,
        mutation_rate: 0.1,
        elitism_count: 3,
        islands_count: 5,
        migration_interval: 3,
        migration_rate: 0.05,
        fitness_thresholds: FitnessThresholds {
            min_sharpe_ratio: Some(0.5),
            max_drawdown_pct: None,
            min_win_rate: Some(0.40),
            min_profit_factor: Some(1.1),
            min_total_profit: None,
            min_trades_count: Some(70),
            min_cagr: None,
            max_max_drawdown: None,
        },
        fitness_weights: FitnessWeights {
            sharpe_ratio_weight: 0.3,
            profit_factor_weight: 0.25,
            win_rate_weight: 0.15,
            cagr_weight: 0.2,
            drawdown_penalty: 0.05,
            trades_count_bonus: 0.05,
        },
        use_existing_strategies: false,
        decimation_coefficient: 2.0,
        filter_initial_population: true,
        restart_on_finish: false,
        restart_on_stagnation: true,
        fresh_blood_rate: 0.1,
        detect_duplicates: true,
        param_mutation_min_percent: 0.03,
        param_mutation_max_percent: 0.05,
    };

    println!("   Размер популяции: {}", config.population_size);
    println!("   Максимум поколений: {}", config.max_generations);
    println!("   Количество островов: {}", config.islands_count);
    println!("   Элитизм: {} особей", config.elitism_count);
    println!(
        "   Вероятность скрещивания: {:.1}%",
        config.crossover_rate * 100.0
    );
    println!(
        "   Вероятность мутации: {:.1}%\n",
        config.mutation_rate * 100.0
    );

    println!("🧬 Генерация начальной популяции...");
    let generator =
        InitialPopulationGenerator::new(config.clone(), frames.clone(), base_timeframe.clone());

    let initial_population = generator.generate(None).await?;
    println!(
        "   Сгенерировано {} особей\n",
        initial_population.individuals.len()
    );

    println!("🏝️  Создание островов...");
    let mut initial_populations = vec![initial_population.clone()];
    for i in 1..config.islands_count {
        let mut pop = initial_population.clone();
        pop.island_id = Some(i);
        initial_populations.push(pop);
    }

    let mut island_manager = IslandManager::new(config.clone(), initial_populations);
    println!("   Создано {} островов\n", island_manager.islands_count());

    println!("🧬 Создание генетического алгоритма...");
    let discovery_config = StrategyDiscoveryConfig {
        max_optimization_params: 8,
        timeframe_count: 2,
        base_timeframe: base_timeframe.clone(),
        allow_indicator_on_indicator: true,
        max_indicator_depth: 1,
    };
    let mut genetic_algorithm = GeneticAlgorithmV3::new(
        config.clone(),
        frames.clone(),
        base_timeframe.clone(),
        discovery_config,
    );

    println!("📈 Создание менеджеров эволюции...");
    let mut evolution_manager = EvolutionManager::new(config.clone());
    let migration_system = MigrationSystem::new(config.clone());
    let fresh_blood = FreshBloodSystem::new(config.clone());

    println!("\n🚀 Запуск эволюции...\n");

    for generation in 0..config.max_generations {
        println!("═══════════════════════════════════════════════════════");
        println!("Поколение {}/{}", generation + 1, config.max_generations);
        println!("═══════════════════════════════════════════════════════");

        let islands = island_manager.get_all_islands_mut();

        for (island_idx, island) in islands.iter_mut().enumerate() {
            println!(
                "\n🏝️  Остров {} (поколение {})",
                island_idx, island.generation
            );

            genetic_algorithm.evolve_generation(island).await?;

            let best = island.individuals.iter().max_by(|a, b| {
                let fitness_a = a.strategy.fitness.unwrap_or(0.0);
                let fitness_b = b.strategy.fitness.unwrap_or(0.0);
                fitness_a
                    .partial_cmp(&fitness_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if let Some(best_individual) = best {
                let fitness = best_individual.strategy.fitness.unwrap_or(0.0);
                println!("   Лучший fitness: {:.4}", fitness);

                if let Some(ref report) = best_individual.strategy.backtest_report {
                    println!("   Total Profit: {:.2}", report.metrics.total_profit);
                    if let Some(sharpe) = report.metrics.sharpe_ratio {
                        println!("   Sharpe Ratio: {:.2}", sharpe);
                    }
                    if let Some(pf) = report.metrics.profit_factor {
                        println!("   Profit Factor: {:.2}", pf);
                    }
                    println!(
                        "   Win Rate: {:.1}%",
                        report.metrics.winning_percentage * 100.0
                    );
                    println!("   Trades: {}", report.trades.len());
                }

                if let Some(ref candidate) = best_individual.strategy.candidate {
                    print_strategy_info(candidate);
                }

                evolution_manager.update_fitness_history(fitness);
            }
        }

        if generation > 0 && (generation + 1) % config.migration_interval == 0 {
            println!("\n🔄 Миграция между островами...");
            let islands = island_manager.get_all_islands_mut();
            migration_system.migrate_between_islands(islands)?;
            println!("   Миграция завершена");
        }

        if generation > 0 && generation % 3 == 0 {
            println!("\n🩸 Инъекция свежей крови...");
            let islands = island_manager.get_all_islands_mut();
            for island in islands.iter_mut() {
                fresh_blood.inject_fresh_blood(island, &generator).await?;
            }
            println!("   Инъекция завершена");
        }

        if evolution_manager.should_restart() {
            println!("\n⚠️  Обнаружен застой! Перезапуск эволюции...");
            evolution_manager.reset_stagnation();
        }

        println!();
    }

    println!("═══════════════════════════════════════════════════════");
    println!("✅ Эволюция завершена!");
    println!("═══════════════════════════════════════════════════════\n");

    println!("🏆 Лучшие стратегии по островам:\n");
    let islands = island_manager.get_all_islands();
    for (island_idx, island) in islands.iter().enumerate() {
        let best = island.individuals.iter().max_by(|a, b| {
            let fitness_a = a.strategy.fitness.unwrap_or(0.0);
            let fitness_b = b.strategy.fitness.unwrap_or(0.0);
            fitness_a
                .partial_cmp(&fitness_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(best_individual) = best {
            println!("═══════════════════════════════════════════════════════");
            println!("Остров {} - Лучшая стратегия:", island_idx);
            println!("═══════════════════════════════════════════════════════");
            println!(
                "Fitness: {:.4}",
                best_individual.strategy.fitness.unwrap_or(0.0)
            );

            if let Some(ref report) = best_individual.strategy.backtest_report {
                print_backtest_metrics(report);
            }

            if let Some(ref candidate) = best_individual.strategy.candidate {
                print_strategy_info(candidate);
            }

            println!();
        }
    }

    Ok(())
}

fn print_strategy_info(candidate: &robots::discovery::StrategyCandidate) {
    println!("\n📋 Информация о стратегии:");
    println!("   Индикаторы:");
    for indicator in &candidate.indicators {
        println!("     - {} ({})", indicator.name, indicator.alias);
    }

    if !candidate.nested_indicators.is_empty() {
        println!("   Вложенные индикаторы:");
        for nested in &candidate.nested_indicators {
            println!(
                "     - {} ({})",
                nested.indicator.name, nested.indicator.alias
            );
        }
    }

    if !candidate.conditions.is_empty() {
        println!("   Условия входа:");
        for condition in &candidate.conditions {
            println!("     - {} ({})", condition.name, condition.id);
        }
    }

    if !candidate.exit_conditions.is_empty() {
        println!("   Условия выхода:");
        for condition in &candidate.exit_conditions {
            println!("     - {} ({})", condition.name, condition.id);
        }
    }

    if !candidate.stop_handlers.is_empty() {
        println!("   Stop handlers:");
        for stop in &candidate.stop_handlers {
            println!("     - {} ({})", stop.name, stop.handler_name);
        }
    }

    if !candidate.take_handlers.is_empty() {
        println!("   Take handlers:");
        for take in &candidate.take_handlers {
            println!("     - {} ({})", take.name, take.handler_name);
        }
    }

    if !candidate.timeframes.is_empty() {
        println!("   Таймфреймы:");
        for tf in &candidate.timeframes {
            println!("     - {}", tf.identifier());
        }
    }
}

fn print_backtest_metrics(report: &robots::metrics::backtest::BacktestReport) {
    println!("\n📊 Метрики бэктеста:");
    println!("   === БАЗОВЫЕ МЕТРИКИ ===");
    println!(
        "   Всего сделок: {} | Прибыльных: {} | Убыточных: {}",
        report.metrics.total_trades, report.metrics.number_of_wins, report.metrics.number_of_losses
    );
    println!(
        "   Total Profit: {:.2} | Win Rate: {:.2}% | Average Trade: {:.2}",
        report.metrics.total_profit,
        report.metrics.winning_percentage * 100.0,
        report.metrics.average_trade
    );

    if let Some(aw) = report.metrics.average_win {
        println!("   Average Win: {:.2}", aw);
    }
    if let Some(al) = report.metrics.average_loss {
        println!("   Average Loss: {:.2}", al);
    }
    println!(
        "   Gross Profit: {:.2} | Gross Loss: {:.2}",
        report.metrics.gross_profit, report.metrics.gross_loss
    );

    println!("   === МЕТРИКИ РИСКА И ДОХОДНОСТИ ===");
    if let Some(pf) = report.metrics.profit_factor {
        println!("   Profit Factor: {:.2}", pf);
    }
    if let Some(sr) = report.metrics.sharpe_ratio {
        println!("   Sharpe Ratio: {:.2}", sr);
    }
    if let Some(rdd) = report.metrics.return_dd_ratio {
        println!("   Return/DD Ratio: {:.2}", rdd);
    }
    if let Some(cagr) = report.metrics.cagr {
        println!("   CAGR: {:.2}%", cagr);
    }

    println!("   === МЕТРИКИ ПРОСАДКИ ===");
    if let Some(dd) = report.metrics.drawdown {
        println!("   Max Drawdown: {:.2}", dd);
    }
    if let Some(dd_pct) = report.metrics.drawdown_percent {
        println!("   Max Drawdown %: {:.2}%", dd_pct);
    }
    println!(
        "   Max Consecutive Wins: {} | Max Consecutive Losses: {}",
        report.metrics.max_consec_wins, report.metrics.max_consec_losses
    );
}
