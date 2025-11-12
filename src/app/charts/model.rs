use hex;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TickerCandle {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub candles: Vec<f32>,
    pub timestamp: i32,
    pub tf: i16,
    pub key: String,
    pub meta: String,
}

impl TickerCandle {
    pub fn get_key(timestamp: i32, tf: i16) -> String {
        let tf = tf.to_string();
        let tstmp = timestamp.to_string();
        let mut key: String = "".to_string();
        key.push_str(tstmp.as_str());
        key.push_str(tf.as_str());
        let result = hex::encode(Sha1::digest(key.as_bytes()));
        return result[..24].to_string();
    }
}
