use chrono::Utc;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::{DataSource, Database};
use robots::data_model::quote_frame::QuoteFrame;
use robots::data_model::types::{Symbol, TimeFrame};
use robots::indicators::{IndicatorFactory, OHLCData};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Подключение
    let config = ClickHouseConfig::default();
    let mut connector = ClickHouseConnector::with_config(config);
    connector.connect().await.unwrap();
    if let Err(err) = connector.ping().await {
        eprintln!("Ошибка ping ClickHouse: {}", err);
        return;
    }

    // Вставка данных
    let start = Utc::now() - chrono::Duration::days(30);
    let end = Utc::now();
    let symbol = Symbol::from_descriptor("AFLT.MM");
    let timeframe = TimeFrame::from_identifier("60");
    let symbol_descriptor = symbol.descriptor();
    let timeframe_identifier = timeframe.identifier();
    // Запрос данных
    let candles = connector
        .get_ohlcv(&symbol_descriptor, &timeframe_identifier, start, end, None)
        .await
        .unwrap();
    if candles.is_empty() {
        println!(
            "Нет данных для {} {}",
            symbol_descriptor, timeframe_identifier
        );
        return;
    }
    let frame = match QuoteFrame::try_from_ohlcv(candles, symbol.clone(), timeframe.clone()) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Ошибка формирования QuoteFrame: {}", err);
            return;
        }
    };
    println!(
        "Получено {} свечей для {} {}",
        frame.len(),
        symbol_descriptor,
        timeframe_identifier
    );
    let open: Vec<f64> = frame.opens().iter().map(|value| value as f64).collect();
    let high: Vec<f64> = frame.highs().iter().map(|value| value as f64).collect();
    let low: Vec<f64> = frame.lows().iter().map(|value| value as f64).collect();
    let close: Vec<f64> = frame.closes().iter().map(|value| value as f64).collect();
    let volume: Vec<f64> = frame.volumes().iter().map(|value| value as f64).collect();
    let timestamps = frame.timestamp_millis();
    let ohlc = OHLCData::new(open, high, low, close)
        .with_volume(volume)
        .with_timestamp(timestamps);
    let mut params = HashMap::new();
    params.insert("period".to_string(), 10.0);
    params.insert("coeff_atr".to_string(), 3.0);
    let indicator = match IndicatorFactory::create_indicator("SUPERTREND", params) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Ошибка создания SuperTrend: {}", err);
            return;
        }
    };
    match indicator.calculate_with_ohlc(&ohlc).await {
        Ok(values) => {
            if let Some(last) = values.last() {
                println!("SuperTrend значение: {}", last);
            } else {
                println!("SuperTrend вернул пустой результат");
            }
        }
        Err(err) => {
            eprintln!("Ошибка расчета SuperTrend: {}", err);
        }
    }
}
