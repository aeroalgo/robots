use crate::core::settings::Settings;
use mongodb::{Client, Collection};
#[derive(Debug)]
pub struct MongoDbClient {
    pub db_name: String,
    pub collection: String,
    pub client: Client,
}

impl MongoDbClient {
    pub async fn new(db_name: &String, collection: &String) -> Self {
        let settings = Settings::new("local".to_string());
        let mongodb_client = Client::with_uri_str(settings.mongodb_uri)
            .await
            .expect("Failed to create MongoDB client");

        return MongoDbClient {
            db_name: db_name.to_string(),
            collection: collection.to_string(),
            client: mongodb_client,
        };
    }
    pub async fn get_dbs(&self) -> Vec<String> {
        let x = self
            .client
            .list_database_names(None, None)
            .await
            .expect("msg");
        return x;
    }
    pub async fn get_collections(&self, db_name: &str) -> Vec<String> {
        let x = self.client.database(db_name);
        let collections = x.list_collection_names(None).await.expect("msg");
        return collections;
    }
    pub fn get_session<T>(&self) -> Collection<T> {
        return self
            .client
            .database(&self.db_name)
            .collection(&self.collection);
    }
}
