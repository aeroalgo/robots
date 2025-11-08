use chrono::Utc;
use robots::data_access::database::clickhouse::{ClickHouseConfig, ClickHouseConnector};
use robots::data_access::DataSource;

#[tokio::main]
async fn main() {
    // Подключение
    let config = ClickHouseConfig::default();
    let mut connector = ClickHouseConnector::with_config(config);
    connector.connect().await.unwrap();

    // Вставка данных
    let data = vec![/* ваши данные */];
    connector.insert_ohlcv(&data).await.unwrap();
    let start = Utc::now() - chrono::Duration::hours(24);
    let end = Utc::now();
    // Запрос данных
    let candles = connector
        .get_ohlcv("BTCUSDT", "1h", start, end, Some(100))
        .await
        .unwrap();
}
