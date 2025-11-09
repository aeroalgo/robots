use crate::app::charts::model::TickerCandle;
use itertools::Itertools;
use mongodb::bson::oid::ObjectId;
use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index};
use try_partialord::*;
extern crate chrono;

pub struct Source {
    data: Vec<TickerCandle>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
    pub timestamp: Vec<i32>,
    pub timeframe: i16,
}

pub enum Element {
    Open,
    High,
    Low,
    Close,
    Volume,
}
pub enum MAIndicators {
    SMA,
    VTrend,
    GEOMEAN,
    AMMA,
    SQWMA,
    SINEWMA,
    AMA,
    TPBF,
    ZLEMA,
    EMA,
}

impl Source {
    pub async fn new(data: Vec<TickerCandle>) -> Self {
        let _timeframe = data[0].tf; // Префикс _ для неиспользуемой переменной
        let mut open: Vec<f64> = Vec::new();
        let mut high: Vec<f64> = Vec::new();
        let mut low: Vec<f64> = Vec::new();
        let mut close: Vec<f64> = Vec::new();
        let mut volume: Vec<f64> = Vec::new();
        let mut timestamp: Vec<i32> = Vec::new();
        for d in &data {
            open.push(d.candles[0]);
            high.push(d.candles[1]);
            low.push(d.candles[2]);
            close.push(d.candles[3]);
            timestamp.push(d.timestamp);
            volume.push(*d.candles.last().expect("msg"))
        }
        return Source {
            data: data.clone(),
            open: open,
            high: high,
            low: low,
            close: close,
            volume: volume,
            timestamp: timestamp,
            timeframe: data[0].tf,
        };
    }

    // pub async fn compres(&mut self) {
    //     self.get_data().await;
    //     for (idx, data) in self.timestamp.iter().enumerate() {
    //         if idx > 0 {
    //             let x = self.timestamp[idx] - self.timestamp[idx - 1];
    //             println!("{:?}", x);
    //         }
    //     }
    // }
}
