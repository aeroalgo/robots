use core::agt::opt::optimization::MainOptimization;

use crate::app::charts::crud::TickerRepository;
use crate::core::agt::candles::source::Source;
use crate::core::agt::indicators::any::SimpleIndicators;
use crate::core::agt::indicators::source::SourceIndicators;
use crate::core::agt::opt::iterating::indicators::{
    QuantityIndicators, SourceCombinationIndicators,
};
use crate::core::database::mongodb::MongoDbClient;
use app::charts::model::TickerCandle;
mod app;
mod core;

#[tokio::main]
async fn main() {
    let ticker = TickerRepository::new("charts".to_string(), "ALRS.MM".to_string()).await;
    let all_data: Vec<TickerCandle> = ticker.get_all().await;
    let source_data = Source::new(all_data).await;

    let mut x = SimpleIndicators::new(source_data.close.clone()).await;
    let data_rsi = x.get_rsi(20, false).await;
    let mut x = SimpleIndicators::new(data_rsi.data.clone()).await;
    let data_sp = x.get_super_trend(20, 3.1, false).await;
    println!("{:?}", data_rsi.data);
    // println!("{:?}", data_sp.data);
    // let c = MainOptimization::execute(
    //     QuantityIndicators::Three,
    //     QuantityIndicators::Three,
    //     all_data,
    // )
    // .await;

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
