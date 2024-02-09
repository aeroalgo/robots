use mongodb::{Client, Collection};

const DB_NAME: &str = "charts";
const COLLECTION_NAME: &str = "MICEX20";

#[derive(Debug)]
pub struct MongoDbClient {
    pub client: Client,
}

impl MongoDbClient {
    pub async fn new(mongodb_uri: String) -> Self {
        let mongodb_client = Client::with_uri_str(mongodb_uri)
            .await
            .expect("Failed to create MongoDB client");

        return MongoDbClient {
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
    pub async fn get_collections(&self, db_name:&str) -> Vec<String> {
        let x = self.client.database(db_name);
        let collections = x.list_collection_names(None).await.expect("msg");
        return collections;
    }
}
