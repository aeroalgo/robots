use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index, vec};
use try_partialord::*;
extern crate chrono;
use crate::core::agt::{
    candles::source::Source,
    indicators::{
        any::MovingAverage,
        common::{IndicatorData, IndicatorsMeta, OptimizationParam},
    },
};

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

pub enum MAIndicators {
    SMA,
    GEOMEAN,
    AMMA,
    SQWMA,
    SINEWMA,
    AMA,
    ZLEMA,
    EMA,
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
                optimization_param: HashMap::from([(
                    "default".to_string(),
                    OptimizationParam {
                        start: 10,
                        stop: 300,
                        step: 10,
                    },
                )]),
                multi_indicator: false,
                name: String::from(""),
                name_param: vec![],
                value_param: vec![],
            },
        };
    }
    pub async fn get_atr_old(&mut self, period: i16, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("ATR_OLD");
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
    async fn calculate_atr_old(&self, period: i16) -> Vec<f32> {
        let period = period;
        let mut prev_atr: f32 = 0.0;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 1..count {
            let delta = i as i128 - period as i128;
            let true_range = self.true_range_old(i).await;
            if delta > 0 {
                let data = (prev_atr * (period as f32 - 1.0) + true_range) / period as f32;
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
    pub async fn get_atr(&mut self, period: i16, meta: bool) -> IndicatorData {
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
    async fn calculate_atr(&self, period: i16) -> Vec<f32> {
        let period = period;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 0..count {
            let list = self.true_range(period as usize, i).await;
            let mut list2 = MovingAverage::new(list.clone()).await;
            let list2 = list2.get_sma(period as i16, false).await;
            array[i] = list2.data[list2.data.len() - 1]
        }
        println!("{:?}", array);
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
        period: i16,
        coeff_atr: i8,
        meta: bool,
    ) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("SUPER_TREND");
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
    async fn calculate_super_trend(&mut self, period: i16, coeff_atr: i8) -> Vec<f32> {
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
    pub async fn get_watr(&mut self, period: i16, meta: bool) -> IndicatorData {
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
    async fn calculate_watr(&self, period: i16) -> Vec<f32> {
        let period = period;
        let count = *&self.close.len();
        let mut array = vec![0.0; count];
        for i in 0..count {
            let list = self.true_range(period as usize, i).await;
            let mut list2 = MovingAverage::new(list.clone()).await;
            let list2 = list2.get_wma(period as i16, false).await;
            array[i] = list2.data[list2.data.len() - 1];
        }

        return array;
    }
    pub async fn get_maxfor(&mut self, period: i16, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("MAX_FOR");
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
    async fn calculate_maxfor(&mut self, period: i16) -> Vec<f32> {
        let data = &self.high;
        let count = data.len();
        let num = min(count, period.try_into().unwrap());
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
        println!("{:?}", &result[23850..]);
        result
    }
    pub async fn get_minfor(&mut self, period: i16, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("MIN_FOR");
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
    async fn calculate_minfor(&mut self, period: i16) -> Vec<f32> {
        let data = &self.low;
        let count = data.len();
        let num = min(count, period.try_into().unwrap());
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
    pub async fn get_stochastic(&mut self, period: i16, meta: bool) -> IndicatorData {
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
        period: i16,
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
        println!("{:?}", &result[23890..]);
        // println!("{:?}", &self.close[23890..]);
        result
    }
}
// pub async fn get_sma(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("SMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_sma(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_sma(&mut self, period: i16) {
//     let data = &self.data;
//     let count = self.data.len();
//     let mut num2: f32 = 0.0;
//     let num = min(count, period.try_into().unwrap());
//     let mut result: Vec<f32> = vec![];
//     for i in 0..num {
//         num2 += data.index(i);
//         let value = num2 / (i as f32 + 1 as f32);
//         result.push(value);
//     }
//     for i in num..count {
//         let num3: &f32 = data.index(i);
//         let num4: &f32 = data.index(i - period as usize);
//         let num5: &f32 = result.index(i - 1);
//         result.push(num5 + (num3 - num4) / period as f32)
//     }
//     self.result = result;
// }

// pub async fn get_minfor(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("MIN_FOR");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_minfor(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_minfor(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let num = min(count, period.try_into().unwrap());
//     let mut result: Vec<f32> = Vec::new();
//     for i in 0..num {
//         let value = *data[0..i + 1].iter().try_min().unwrap().expect("msg");
//         result.push(value);
//     }
//     for i in num..count {
//         let value = *data[i - period as usize..i]
//             .iter()
//             .try_min()
//             .unwrap()
//             .expect("msg");
//         result.push(value)
//     }
//     self.result = result;
// }
// pub async fn get_vtrand(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("GEOMEAN");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_minfor(period).await;
//     let llv = self.result.clone();
//     self.calculate_maxfor(period).await;
//     let hhv = self.result.clone();
//     self.result = self.calculate_vtrand(hhv, llv, period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_vtrand(&self, hhv: Vec<f32>, llv: Vec<f32>, period: i16) -> Vec<f32> {
//     hhv.into_iter()
//         .zip(llv)
//         .map(|(a, b)| (a + b) / 2.0)
//         .collect()
// }

// pub async fn get_geomean(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("GEOMEAN");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_geomean(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_geomean(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     for i in 0..count {
//         if i < period as usize {
//             result.push(data[i])
//         } else {
//             let mut num = data[i].powf(1.0 / period as f32);
//             for j in 1..period {
//                 num *= data[i - j as usize].powf(1.0 / period as f32);
//             }
//             result.push(num);
//         }
//     }
//     self.result = result;
// }

// pub async fn get_amma(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("AMMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];

//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_amma(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_amma(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     for i in 0..count {
//         if i < period as usize {
//             result.push(data[i])
//         } else {
//             let num = ((period - 1) as f32 * result[i - 1] + data[i]) / period as f32;
//             result.push(num);
//         }
//     }
//     self.result = result;
// }

// pub async fn get_sqwma(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("SQWMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_sqwma(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_sqwma(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     let num = period as f32 * (period - 1) as f32 / 2.0;
//     let num2 = period as f32 * (period - 1) as f32 * (2 * period - 1) as f32 / 6.0;
//     for i in 0..count {
//         if i < period as usize {
//             result.push(data[i])
//         } else {
//             let mut num3 = 0.0;
//             let mut num4 = 0.0;
//             for j in 0..period {
//                 let num5 = data[i - j as usize];
//                 num3 += num5;
//                 num4 += num5 * j as f32;
//             }
//             let num6 = num2 * period as f32 - num * num;
//             let num7 = (num4 * period as f32 - num * num3) / num6;
//             let num8 = (num3 - num * num7) / period as f32;
//             result.push(num8);
//         }
//     }
//     self.result = result;
// }
// pub async fn get_sinewma(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("SINEWMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_sinewma(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_sinewma(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     for i in 0..count {
//         if i < period as usize {
//             result.push(data[i])
//         } else {
//             let num = 3.1415926535;
//             let mut num2 = 0.0;
//             let mut num3 = 0.0;
//             for j in 0..period - 1 {
//                 num3 += (num * (j as f64 + 1.0) / (period + 1) as f64).sin();
//                 num2 += data[i - j as usize] as f64
//                     * (num * (j as f64 + 1.0) / (period + 1) as f64).sin();
//             }
//             let mut result2 = 0.0;
//             if num3 > 0.0 {
//                 result2 = num2 / num3;
//             }
//             result.push(result2 as f32);
//         }
//     }
//     self.result = result;
// }

// pub async fn get_ama(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("AMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_ama(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_ama(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     let mut num: f64 = if count < period as usize {
//         0.0
//     } else {
//         data[(period - 1) as usize] as f64
//     };
//     for i in 0..count {
//         if i < (period + 2) as usize {
//             result.push(data[i]);
//         } else {
//             let num2 = (data[i] - data[i - period as usize]).abs();
//             let mut num3 = 1E-09;
//             for j in 0..period {
//                 num3 += (data[i - j as usize] - data[i - j as usize - 1]).abs() as f64;
//             }
//             let num4 = num2 as f64 / num3;

//             let x = num4 * 0.60215 + 0.06452;
//             let num5 = x.powf(2.0);

//             let num6 = num + num5 * (data[i] - num as f32) as f64;
//             result.push(num6 as f32);
//             num = num6;
//         }
//     }
//     self.result = result;
// }
// pub async fn get_zlema(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("ZLEMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     let result = self.calculate_zlema(&self.data, period).await;
//     println!("{:?}", result);
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_zlema(&self, data: &Vec<f32>, period: i16) -> Vec<f32> {
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     let num: f32 = 2.0 / (period as f32 + 1.0);
//     let num2: i32 = (period as i32 - 1) / 2;
//     for i in 0..count {
//         if i < period as usize {
//             result.push(data[i]);
//         } else {
//             result.push(
//                 result[i - 1]
//                     + num * (data[i] + (data[i] - data[i - num2 as usize]) - result[i - 1]),
//             )
//         }
//     }
//     return result;
// }

// pub async fn get_ema(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("EMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_ema(period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_ema(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = Vec::new();
//     // let mut num: f32 = 2.0 / (period as f32 + 1.0);
//     let num: f32 = 2.0 / (period + 1) as f32;
//     for i in 0..count {
//         if i < period as usize {
//             result.push(data[i]);
//         } else {
//             result.push(result[i - 1] + num * (data[i] - result[i - 1]))
//         }
//     }
//     self.result = result;
// }
// pub async fn get_tpbf(&mut self, period: i16, meta: bool) -> IndicatorData {
//     // Сделать версию для инструмента h l
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("TPBF");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     let h = self.get_maxfor(period, false).await;
//     let l = self.get_minfor(period, false).await;
//     self.calculate_tpbf(&h.data, &l.data, period).await;
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_tpbf(&mut self, h: &Vec<f32>, l: &Vec<f32>, period: i16) {
//     let count = h.len();
//     let mut result: Vec<f32> = Vec::new();
//     let num = 1.0_f32.atan() as f64;
//     let num2 = 45.0 / num;
//     let num3 = 1.0 / num2;
//     let num4 = 1.0_f32.atan() * 4.0_f32;
//     let num5 = (-num4 / period as f32).exp() as f64;
//     let num6 =
//         2.0 * num5 as f64 * (num3 * 3.0_f32.sqrt() as f64 * 180.0 / period as f64).cos() as f64;
//     let num7 = num5 * num5;
//     let num8 = num6 + num7;
//     let num9 = -(num7 + num6 * num7);
//     let num10 = num7 * num7;
//     let num11 = (1.0 - num6 + num7) * (1.0 - num7) / 8.0;
//     for i in 0..count {
//         if i < 4 as usize {
//             result.push((*&l[i] + *&h[i]) / 2.0);
//         } else {
//             result.push(
//                 (num11
//                     * ((*&l[i] + *&h[i]) / 2.0
//                         + 3.0 * ((*&l[i - 1] + *&h[i - 1]) / 2.0)
//                         + 3.0 * ((*&l[i - 2] + *&h[i - 2]) / 2.0)
//                         + (*&l[i - 3] + *&h[i - 3]) / 2.0) as f64
//                     + num8 * result[i - 1] as f64
//                     + num9 * result[i - 2] as f64
//                     + num10 * result[i - 3] as f64) as f32,
//             )
//         }
//     }
//     self.result = result;
// }
// pub async fn get_wma(&mut self, period: i16, meta: bool) -> IndicatorData {
//     self.meta.current_param = HashMap::from([("period".to_string(), period)]);
//     self.meta.name = String::from("WMA");
//     self.meta.name_param = vec!["period".to_string()];
//     self.meta.value_param = vec![period];
//     if meta {
//         return IndicatorData {
//             data: self.result.clone(),
//             meta: self.meta.clone(),
//         };
//     }
//     self.calculate_wma(period).await;
//     // println!("{:?}", result);
//     return IndicatorData {
//         data: self.result.clone(),
//         meta: self.meta.clone(),
//     };
// }
// async fn calculate_wma(&mut self, period: i16) {
//     let data = &self.data;
//     let count = data.len();
//     let mut result: Vec<f32> = vec![0.0; count];
//     let mut num: f32 = 0.0;
//     let mut num2: f32 = 0.0;
//     for i in 0..period {
//         num2 = num2 + i as f32 + 1.0
//     }
//     for j in period as usize..count {
//         let mut num3 = 1.0;
//         for k in j - period as usize + 1..j + 1 as usize {
//             num += num3 * data[k];
//             num3 += 1.0;
//         }
//         result.push(num / num2);
//         num = 0.0
//     }
//     self.result = result;
// }
