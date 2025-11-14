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
    let start = Utc::now() - chrono::Duration::days(91);
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
    println!(
        "Всего сделок: {} | PnL: {:.2} | Win rate: {:.2}% | Средняя сделка: {:.2}",
        report.metrics.total_trades,
        report.metrics.total_pnl,
        report.metrics.win_rate * 100.0,
        report.metrics.average_trade
    );

    let close_prices: Vec<f32> = candles.iter().map(|candle| candle.close as f32).collect();
    let sma_period = 10;
    let mut sma_params = HashMap::new();
    sma_params.insert("period".to_string(), sma_period as f32);
    let sma_indicator = IndicatorFactory::create_indicator("SMA", sma_params)
        .context("Не удалось создать индикатор SMA")?;
    let sma_values = sma_indicator
        .calculate_simple(&close_prices)
        .await
        .context("Не удалось рассчитать SMA")?;
    println!("SMA values: {:?}", sma_values);
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
            println!(
                "- {:?} qty {:.2} вход {:.2} ({}) выход {:.2} ({}) pnl {:.2}",
                trade.direction,
                trade.quantity,
                trade.entry_price,
                entry_time,
                trade.exit_price,
                exit_time,
                trade.pnl
            );
        }
    }

    if let Some(last_equity) = report.equity_curve.last() {
        println!("Финальная equity: {:.2}", last_equity);
    }

    Ok(())
}
