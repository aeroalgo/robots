use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::Utc;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::indicators::registry::IndicatorFactory;
use robots::strategy::executor::BacktestExecutor;
use robots::strategy::presets::default_strategy_definitions;

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
    let start = Utc::now() - chrono::Duration::days(94);
    let end = Utc::now() + chrono::Duration::hours(3);

    let candles = connector
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
    let trend_sma_values = trend_sma.calculate_simple(&close_values).await?;

    let mut frames = HashMap::new();
    frames.insert(timeframe.clone(), frame);

    let definition = default_strategy_definitions()
        .into_iter()
        .find(|def| def.metadata.id == "SMA_CROSSOVER_LONG")
        .context("Стратегия SMA_CROSSOVER_LONG не найдена")?;

    let mut executor =
        BacktestExecutor::from_definition(definition, None, frames).map_err(anyhow::Error::new)?;

    let report = executor.run_backtest().await.map_err(anyhow::Error::new)?;

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

    let ema_50_indicator =
        IndicatorFactory::create_indicator("EMA", HashMap::from([("period".to_string(), 40.0)]))?;
    let ema_50_values = ema_50_indicator
        .calculate_simple(&close_values)
        .await
        .context("Не удалось рассчитать EMA 50")?;
    print_strategy_data_table(&executor, &timeframe, &ema_timeframe, &ema_50_values)?;

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

    Ok(())
}

fn print_strategy_data_table(
    executor: &BacktestExecutor,
    base_timeframe: &TimeFrame,
    higher_timeframe: &TimeFrame,
    ema_50_values: &[f32],
) -> Result<()> {
    use robots::strategy::types::PriceField;

    let context = executor.context();
    let base_data = context
        .timeframe(base_timeframe)
        .map_err(|e| anyhow::anyhow!("Не удалось получить данные базового таймфрейма: {}", e))?;

    let higher_data = context
        .timeframe(higher_timeframe)
        .map_err(|e| anyhow::anyhow!("Не удалось получить данные старшего таймфрейма: {}", e))?;

    let close_prices = base_data
        .price_series_slice(&PriceField::Close)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены закрытия"))?;

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

    let timestamps = base_data
        .ohlc_ref()
        .and_then(|ohlc| ohlc.timestamp.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Не найдены временные метки"))?;

    let higher_close = higher_data
        .price_series_slice(&PriceField::Close)
        .ok_or_else(|| anyhow::anyhow!("Не найдены цены закрытия старшего таймфрейма"))?;

    let len = close_prices
        .len()
        .min(fast_sma.len())
        .min(slow_sma.len())
        .min(trend_sma.len())
        .min(timestamps.len())
        .min(ema_50_values.len());

    println!("\nТаблица данных стратегии:");
    println!("{:-<150}", "");
    println!(
        "{:<20} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10} | {:<8} | {:<8}",
        "Дата",
        "Close(60)",
        "Close(240)",
        "EMA_240",
        "EMA_50",
        "Fast_SMA",
        "Slow_SMA",
        "Close>EMA",
        "Fast>Trend"
    );
    println!("{:-<150}", "");

    let ratio = higher_timeframe.total_minutes().unwrap_or(240)
        / base_timeframe.total_minutes().unwrap_or(60);

    for i in 0..len {
        let timestamp =
            robots::data_model::types::timestamp_from_millis(timestamps[i]).unwrap_or_default();
        let date_str = timestamp.format("%Y-%m-%d %H:%M").to_string();

        let close_60 = close_prices[i];
        let fast = fast_sma[i];
        let slow = slow_sma[i];
        let trend = trend_sma[i];
        let ema_50 = ema_50_values[i];

        let close_240 = if i < higher_close.len() {
            higher_close[i]
        } else {
            higher_close[higher_close.len().saturating_sub(1)]
        };

        let ema_val = if i < ema_240.len() {
            ema_240[i]
        } else {
            ema_240[ema_240.len().saturating_sub(1)]
        };

        let close_above_ema = close_240 > ema_val;
        let fast_cross_above_trend =
            i > 0 && fast_sma[i] > trend_sma[i] && fast_sma[i - 1] <= trend_sma[i - 1];

        println!(
            "{:<20} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2} | {:<8} | {:<8}",
            date_str,
            close_60,
            close_240,
            ema_val,
            ema_50,
            fast,
            slow,
            if close_above_ema { "ДА" } else { "НЕТ" },
            if fast_cross_above_trend {
                "ДА"
            } else {
                "НЕТ"
            }
        );
    }

    println!("{:-<150}", "");

    Ok(())
}
