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
    let start = Utc::now() - chrono::Duration::days(30);
    let end = Utc::now();
    // Запрос данных
    let candles = connector
        .get_ohlcv("AFLT.MM", "60", start, end, None)
        .await
        .unwrap();
    println!("{:?}", candles);
}
