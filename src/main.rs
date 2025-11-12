use chrono::Utc;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::indicators::IndicatorFactory;
use std::collections::HashMap;
use std::error::Error;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let mut connector = ClickHouseConnector::with_config(ClickHouseConfig::default());
    connector.connect().await?;
    connector.ping().await?;

    let symbol = Symbol::from_descriptor("AFLT.MM");
    let timeframe = TimeFrame::from_identifier("60");
    let start = Utc::now() - chrono::Duration::days(30);
    let end = Utc::now() + chrono::Duration::hours(3);

    let candles = connector
        .get_ohlcv_typed(&symbol, &timeframe, start, end, None)
        .await?;
    println!("DB candles fetched: {}", candles.len());
    if let Some(last) = candles.last() {
        println!(
            "DB last candle: close={}, ts={:?}",
            last.close, last.timestamp
        );
    }
    if candles.is_empty() {
        println!(
            "Нет данных для {} {}",
            symbol.descriptor(),
            timeframe.identifier()
        );
        return Ok(());
    }

    let frame = QuoteFrame::try_from_ohlcv(candles, symbol.clone(), timeframe.clone())?;

    println!(
        "Получено {} свечей для {} {}",
        frame.len(),
        symbol.descriptor(),
        timeframe.identifier()
    );

    let ohlc = frame.to_indicator_ohlc();
    let input_len = ohlc.close.len();

    let params = HashMap::from([("period".to_string(), 10.0), ("coeff_atr".to_string(), 3.0)]);

    let indicator = IndicatorFactory::create_indicator("WMA", params.clone())?;
    let values = indicator.calculate_ohlc(&ohlc).await?;
    println!("{:?}", values);
    Ok(())
}
