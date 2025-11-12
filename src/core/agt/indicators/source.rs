use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index, vec};
use try_partialord::*;
extern crate chrono;
use crate::core::agt::{
    candles::source::Source,
    indicators::common::{IndicatorData, IndicatorsEnum, IndicatorsMeta, OptimizationParam},
};

use super::any::SimpleIndicators;

pub struct SourceIndicators {
    pub open: Vec<f32>,
    pub high: Vec<f32>,
    pub low: Vec<f32>,
    pub close: Vec<f32>,
    pub volume: Vec<f32>,
    pub timestamp: Vec<i32>,
    pub timeframe: i16,
    pub meta: IndicatorsMeta,
}

impl SourceIndicators {
    pub async fn new(data: &Source) -> Self {
        return SourceIndicators {
            open: data.open.clone(),
            high: data.high.clone(),
            low: data.low.clone(),
            close: data.close.clone(),
            volume: data.open.clone(),
            timestamp: data.timestamp.clone(),
            timeframe: data.timeframe.clone(),
            meta: IndicatorsMeta {
                current_param: HashMap::new(),
                optimization_param: HashMap::new(),
                multi_indicator: false,
                name: String::from(""),
                name_param: vec![],
                value_param: vec![],
            },
        };
    }
    pub async fn get_indicator(
        &mut self,
        name: &IndicatorsEnum,
        period: f32,
        coeff_atr: f32,
        meta: bool,
    ) -> IndicatorData {
        let mut simple_indicators = SimpleIndicators::new(self.close.clone()).await;
        let result = match name {
            IndicatorsEnum::RSI => simple_indicators.get_rsi(period, meta).await,
            IndicatorsEnum::STOCHASTIC => self.get_stochastic(period, meta).await,
            IndicatorsEnum::ATR => self.get_atr(period, meta).await,
            IndicatorsEnum::ATROLD => self.get_atr_old(period, meta).await,
            IndicatorsEnum::WATR => self.get_watr(period, meta).await,
            IndicatorsEnum::SMA => simple_indicators.get_sma(period, meta).await,
            IndicatorsEnum::MAXFOR => self.get_maxfor(period, meta).await,
            IndicatorsEnum::MINFOR => self.get_minfor(period, meta).await,
            IndicatorsEnum::VTRAND => self.get_vtrand(period, meta).await,
            IndicatorsEnum::GEOMEAN => simple_indicators.get_geomean(period, meta).await,
            IndicatorsEnum::AMMA => simple_indicators.get_amma(period, meta).await,
            IndicatorsEnum::SQWMA => simple_indicators.get_sqwma(period, meta).await,
            IndicatorsEnum::SINEWMA => simple_indicators.get_sinewma(period, meta).await,
            IndicatorsEnum::AMA => simple_indicators.get_ama(period, meta).await,
            IndicatorsEnum::ZLEMA => simple_indicators.get_zlema(period, meta).await,
            IndicatorsEnum::EMA => simple_indicators.get_ema(period, meta).await,
            IndicatorsEnum::TPBF => simple_indicators.get_tpbf(period, meta).await,
            IndicatorsEnum::WMA => simple_indicators.get_wma(period, meta).await,
            IndicatorsEnum::SUPERTRAND => self.get_super_trend(period, coeff_atr, meta).await,
        };
        return result;
    }
    pub async fn get_vtrand(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("GEOMEAN");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: Vec::new(),
                meta: self.meta.clone(),
            };
        }
        let llv = self.calculate_minfor(period).await;
        let hhv = self.calculate_maxfor(period).await;
        let result = self.calculate_vtrand(hhv, llv, period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_vtrand(&self, hhv: Vec<f32>, llv: Vec<f32>, _period: f32) -> Vec<f32> {
        hhv.into_iter()
            .zip(llv)
            .map(|(a, b)| (a + b) / 2.0)
            .collect()
    }
    pub async fn get_atr_old(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("ATROLD");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: Vec::new(),
                meta: self.meta.clone(),
            };
        }
        let result = self.calculate_atr_old(period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_atr_old(&self, period: f32) -> Vec<f32> {
        let period = period;
        let mut prev_atr: f32 = 0.0;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 1..count {
            let delta = i as i128 - period as i128;
            let true_range = self.true_range_old(i).await;
            if delta > 0 {
                let data = (prev_atr * (period - 1.0) + true_range) / period;
                array[i] = data;
                prev_atr = data;
            } else {
                let mut num = prev_atr * i as f32;
                num += true_range;
                prev_atr = num / (i + 1) as f32;
                array[i] = num / (i + 1) as f32;
            }
        }
        // println!("{:?}", array);
        return array;
    }
    async fn true_range_old(&self, i: usize) -> f32 {
        let high = self.high[i];
        let low = self.low[i];
        let close = self.close[i];
        let num = (high - low).abs();
        let num = num.max((close - high).abs());
        let num = num.max((close - low).abs());
        return num;
    }
    pub async fn get_atr(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("ATR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: Vec::new(),
                meta: self.meta.clone(),
            };
        }
        let result = self.calculate_atr(period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_atr(&self, period: f32) -> Vec<f32> {
        let period = period;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 0..count {
            let list = self.true_range(period as usize, i).await;
            let mut list2 = SimpleIndicators::new(list.clone()).await;
            let list2 = list2.get_sma(period, false).await;
            array[i] = list2.data[list2.data.len() - 1]
        }
        // println!("{:?}", array);
        return array;
    }
    async fn true_range(&self, period: usize, bar_num: usize) -> Vec<f32> {
        let mut list = vec![0.0; period as usize];
        let mut new_period = period;
        if bar_num < period {
            new_period = bar_num
        }
        for i in bar_num - new_period + 1..bar_num + 1 {
            let high = self.high[i];
            let low = self.low[i];
            let close = self.close[i - 1];
            let num = high.max(close) - low.min(close);
            list.push(num);
        }

        return list;
    }

    pub async fn get_super_trend(
        &mut self,
        period: f32,
        coeff_atr: f32,
        meta: bool,
    ) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("SUPERTREND");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: Vec::new(),
                meta: self.meta.clone(),
            };
        }
        println!("{:?}", coeff_atr);
        let result = self.calculate_super_trend(period, coeff_atr).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_super_trend(&mut self, period: f32, coeff_atr: f32) -> Vec<f32> {
        let list = self.get_watr(period, false).await;
        let list = list.data;
        let data = self.calculate_median_price().await;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 2..count {
            let num = data[i] + list[i] * coeff_atr as f32;
            let num2 = data[i] - list[i] * coeff_atr as f32;
            if self.close[i] >= array[i - 1] {
                if self.close[i - 1] < array[i - 1] {
                    array[i] = num2;
                } else {
                    array[i] = if num2 > array[i - 1] {
                        num2
                    } else {
                        array[i - 1]
                    }
                }
            } else if self.close[i] < array[i - 1] {
                if self.close[i - 1] > array[i - 1] {
                    array[i] = num;
                } else {
                    array[i] = if num < array[i - 1] {
                        num
                    } else {
                        array[i - 1]
                    }
                }
            }
        }
        return array;
    }

    async fn calculate_median_price(&self) -> Vec<f32> {
        let count = *&self.close.len();
        let mut array = vec![];
        for i in 0..count {
            array.push((self.high[i] + self.low[i]) / 2.0);
        }
        // println!("{:?}", array);
        return array;
    }
    pub async fn get_watr(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("WATR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: Vec::new(),
                meta: self.meta.clone(),
            };
        }
        let result = self.calculate_watr(period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_watr(&self, period: f32) -> Vec<f32> {
        let period = period;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 0..count {
            let list = self.true_range(period as usize, i).await;
            let mut list2 = SimpleIndicators::new(list.clone()).await;
            let list2 = list2.get_wma(period, false).await;
            array[i] = list2.data[list2.data.len() - 1];
        }

        return array;
    }
    pub async fn get_maxfor(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("MAXFOR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: vec![],
                meta: self.meta.clone(),
            };
        }
        let result = self.calculate_maxfor(period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_maxfor(&mut self, period: f32) -> Vec<f32> {
        let data = &self.high;
        let count = data.len();
        let num = min(count, period as usize);
        let mut result: Vec<f32> = Vec::new();
        for i in 0..num {
            let value = *data[0..i + 1].iter().try_max().unwrap().expect("msg");
            result.push(value);
        }
        for i in num..count {
            let value = *data[i - period as usize..i + 1]
                .iter()
                .try_max()
                .unwrap()
                .expect("msg");
            result.push(value)
        }
        result
    }
    pub async fn get_minfor(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("MINFOR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: vec![],
                meta: self.meta.clone(),
            };
        }
        let result = self.calculate_minfor(period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_minfor(&mut self, period: f32) -> Vec<f32> {
        let data = &self.low;
        let count = data.len();
        let num = min(count, period as usize);
        let mut result: Vec<f32> = Vec::new();
        for i in 0..num {
            let value = *data[0..i + 1].iter().try_min().unwrap().expect("msg");
            result.push(value);
        }
        for i in num..count {
            let value = *data[i + 1 - period as usize..i]
                .iter()
                .try_min()
                .unwrap()
                .expect("msg");
            result.push(value)
        }
        result
    }
    pub async fn get_stochastic(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("STOCHASTIC");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: vec![],
                meta: self.meta.clone(),
            };
        }
        let hhv = self.get_maxfor(period, meta).await;
        let llv = self.get_minfor(period, meta).await;
        let result = self.calculate_stochastic(hhv.data, llv.data, period).await;
        return IndicatorData {
            data: result,
            meta: self.meta.clone(),
        };
    }
    async fn calculate_stochastic(
        &mut self,
        hhv: Vec<f32>,
        llv: Vec<f32>,
        _period: f32,
    ) -> Vec<f32> {
        let data = hhv;
        let data2 = llv;
        let mut result: Vec<f32> = vec![0.0; self.close.len()];
        for i in 0..self.close.len() {
            let num = data[i] - data2[i];
            result[i] = if num == 0.0 {
                0.0
            } else {
                100.0 * ((self.close[i] - data2[i]) / num)
            }
        }
        // println!("{:?}", &self.close[23890..]);
        result
    }
}
