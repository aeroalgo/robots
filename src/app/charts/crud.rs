use crate::app::charts::model::TickerCandle;
use crate::core::database::mongodb::MongoDbClient;
use mongodb::bson::{doc, oid::ObjectId, Document};
use mongodb::options::FindOptions;
use mongodb::{Client, Collection};
use std::env;
use tokio_stream::StreamExt;

pub struct TickerRepository {
    pub db_name: String,
    pub collection: String,
    pub client: Client,
    pub session: Collection<TickerCandle>,
}

impl TickerRepository {
    pub async fn new(db_name: String, collection: String) -> Self {
        let client = MongoDbClient::new(&db_name, &collection).await;
        let session = client.get_session();
        return TickerRepository {
            db_name: db_name,
            collection: collection,
            session: session,
            client: client.client,
        };
    }
    // pub async fn get_all_tickers(&self) -> Vec<String> {
    //     let filter = doc! {};
    //     let mut tickers = self.session.find(filter, None).await.expect("msg");
    //     let mut result: Vec<String> = Vec::new();
    //     while let Some(ticker) = tickers.next().await {
    //         let data_ticker = ticker.expect("msg");
    //         result.push(data_ticker.ticker);
    //     }
    //     return result;
    // }

    pub async fn get_all(&self) -> Vec<TickerCandle> {
        let filter = doc! {"tf": 60};
        let find_options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();
        let mut tickers = self.session.find(filter, find_options).await.expect("msg");
        let mut result: Vec<TickerCandle> = Vec::new();
        while let Some(ticker) = tickers.next().await {
            let data_ticker = ticker.expect("msg");

            result.push(data_ticker);
        }
        return result;
    }
    // pub async fn get_by_key(&self, ticker: String, tf: i32) -> Ticker {
    //     let key = Ticker::get_key(ticker, tf);
    //     let filter = doc! {"key": key};
    //     let mut tickers = self.session.find(filter, None).await.expect("msg");
    //     let result = tickers.next().await.unwrap().expect("msg");
    //     return result;

    // }

    pub async fn get_count(&self, filter: Document) -> u64 {
        let x = self
            .session
            .count_documents(filter, None)
            .await
            .expect("msg");
        return x;
    }
}
