use crate::app::charts::crud::TickerRepository;
use crate::core::agt::candles::source::Source;
use crate::core::agt::indicators::any::{MAIndicators, MovingAverage};
use crate::core::agt::indicators::source::SourceIndicators;
use crate::core::database::mongodb::MongoDbClient;
use app::charts::model::TickerCandle;
mod app;
mod core;

#[tokio::main]
async fn main() {
    let ticker = TickerRepository::new("charts".to_string(), "ALRS.MM".to_string()).await;
    let all_data: Vec<TickerCandle> = ticker.get_all().await;
    let mut m5 = Source::new(all_data).await;
    let mut x = SourceIndicators::new(&m5).await;
    let data = x.get_super_trend(10, 2, false).await;
    println!("{:?}", data.data);
    // h60.compres().await;
    // h60.get_sma(Element::Close, 20).await;
    // x.get_ema(Element::Close, 20).await;
    // let x  = Ticker::get_key("ALRS.MM".to_string(), 60);
    // println!("{:?}", x);

    // let dbases = client.get_dbs().await;
    // let db_name: &str = "charts";
    // let dbases = client.get_collections(db_name).await;
    // println!("{:?}", ticker);
}
