use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index};
use try_partialord::*;
extern crate chrono;
use crate::core::agt::{
    candles,
    indicators::common::{IndicatorData, IndicatorsMeta, OptimizationParam},
};
use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};

#[derive(Clone, Debug)]
pub struct SimpleIndicators {
    data: Vec<f32>,
    result: Vec<f32>,
    meta: IndicatorsMeta,
}
#[derive(Debug, PartialEq, Sequence, Clone, Copy, Ord, PartialOrd, Eq)]
pub enum SimpleIndicatorsEnum {
    RSI,
    SMA,
    MAXFOR,
    MINFOR,
    VTRAND,
    GEOMEAN,
    AMMA,
    SQWMA,
    SINEWMA,
    AMA,
    ZLEMA,
    EMA,
    TPBF,
    WMA,
    SUPERTRAND,
    SIGNALLINE,
}

impl SimpleIndicators {
    pub async fn new(data: Vec<f32>) -> Self {
        return SimpleIndicators {
            data: data,
            result: vec![],
            meta: IndicatorsMeta {
                current_param: HashMap::new(),
                multi_indicator: false,
                optimization_param: HashMap::new(),
                name: String::from(""),
                name_param: vec![],
                value_param: vec![],
            },
        };
    }
    pub async fn get_indicator(
        &mut self,
        name: SimpleIndicatorsEnum,
        period: f32,
        coeff_atr: f32,
        meta: bool,
    ) -> IndicatorData {
        let result = match name {
            SimpleIndicatorsEnum::RSI => self.get_rsi(period, meta).await,
            SimpleIndicatorsEnum::SMA => self.get_sma(period, meta).await,
            SimpleIndicatorsEnum::MAXFOR => self.get_maxfor(period, meta).await,
            SimpleIndicatorsEnum::MINFOR => self.get_minfor(period, meta).await,
            SimpleIndicatorsEnum::VTRAND => self.get_vtrand(period, meta).await,
            SimpleIndicatorsEnum::GEOMEAN => self.get_geomean(period, meta).await,
            SimpleIndicatorsEnum::AMMA => self.get_amma(period, meta).await,
            SimpleIndicatorsEnum::SQWMA => self.get_sqwma(period, meta).await,
            SimpleIndicatorsEnum::SINEWMA => self.get_sinewma(period, meta).await,
            SimpleIndicatorsEnum::AMA => self.get_ama(period, meta).await,
            SimpleIndicatorsEnum::ZLEMA => self.get_zlema(period, meta).await,
            SimpleIndicatorsEnum::EMA => self.get_ema(period, meta).await,
            SimpleIndicatorsEnum::TPBF => self.get_tpbf(period, meta).await,
            SimpleIndicatorsEnum::WMA => self.get_wma(period, meta).await,
            SimpleIndicatorsEnum::SUPERTRAND => self.get_super_trend(period, coeff_atr, meta).await,
            SimpleIndicatorsEnum::SIGNALLINE => self.get_super_trend(period, coeff_atr, meta).await,
        };
        return result;
    }
    pub async fn get_super_trend(
        &mut self,
        period: f32,
        coeff_atr: f32,
        meta: bool,
    ) -> IndicatorData {
        self.meta.current_param = HashMap::from([
            ("period".to_string(), period),
            ("coeff_atr".to_string(), coeff_atr),
        ]);
        self.meta.optimization_param = HashMap::from([
            (
                "period".to_string(),
                OptimizationParam {
                    start: 10.0,
                    stop: 300.0,
                    step: 5.0,
                },
            ),
            (
                "coeff_atr".to_string(),
                OptimizationParam {
                    start: 1.5,
                    stop: 8.0,
                    step: 0.2,
                },
            ),
        ]);
        self.meta.name = String::from("SUPERTREND");
        self.meta.name_param = vec!["period".to_string(), "coeff_atr".to_string()];
        self.meta.value_param = vec![period, coeff_atr];
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
    async fn calculate_super_trend(&self, period: f32, coeff_atr: f32) -> Vec<f32> {
        let list = self.calculate_atr(period).await;
        let data = &self.data;
        let count = *&self.data.len();
        let mut array = vec![0.0; count];
        for i in 2..count {
            let num = data[i] + list[i] * coeff_atr;
            let num2 = data[i] - list[i] * coeff_atr;
            if self.data[i] >= array[i - 1] {
                if self.data[i - 1] < array[i - 1] {
                    array[i] = num2;
                } else {
                    array[i] = if num2 > array[i - 1] {
                        num2
                    } else {
                        array[i - 1]
                    }
                }
            } else if self.data[i] < array[i - 1] {
                if self.data[i - 1] > array[i - 1] {
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

    pub async fn get_atr(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("ATR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
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
        let count = *&self.data.len();
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
            let num = (self.data[i - 1] - self.data[i]).abs();
            list.push(num);
        }

        return list;
    }
    pub async fn get_sma(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("SMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_sma(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_sma(&mut self, period: f32) {
        let data = &self.data;
        let count = self.data.len();
        let mut num2: f32 = 0.0;
        let num = min(count, period as usize);
        let mut result: Vec<f32> = vec![];
        for i in 0..num {
            num2 += data.index(i);
            let value = num2 / (i as f32 + 1 as f32);
            result.push(value);
        }
        for i in num..count {
            let num3: &f32 = data.index(i);
            let num4: &f32 = data.index(i - period as usize);
            let num5: &f32 = result.index(i - 1);
            result.push(num5 + (num3 - num4) / period as f32)
        }
        self.result = result;
    }
    pub async fn get_maxfor(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("MAXFOR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_maxfor(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_maxfor(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let num = min(count, period as usize);
        let mut result: Vec<f32> = Vec::new();
        for i in 0..num {
            let value = *data[0..i + 1].iter().try_max().unwrap().expect("msg");
            result.push(value);
        }
        for i in num..count {
            let value = *data[i - period as usize..i]
                .iter()
                .try_max()
                .unwrap()
                .expect("msg");
            result.push(value)
        }
        self.result = result;
    }

    pub async fn get_minfor(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("MINFOR");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_minfor(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_minfor(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let num = min(count, period as usize);
        let mut result: Vec<f32> = Vec::new();
        for i in 0..num {
            let value = *data[0..i + 1].iter().try_min().unwrap().expect("msg");
            result.push(value);
        }
        for i in num..count {
            let value = *data[i - period as usize..i]
                .iter()
                .try_min()
                .unwrap()
                .expect("msg");
            result.push(value)
        }
        self.result = result;
    }
    pub async fn get_vtrand(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("GEOMEAN");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_minfor(period).await;
        let llv = self.result.clone();
        self.calculate_maxfor(period).await;
        let hhv = self.result.clone();
        self.result = self.calculate_vtrand(hhv, llv, period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_vtrand(&self, hhv: Vec<f32>, llv: Vec<f32>, _period: f32) -> Vec<f32> {
        hhv.into_iter()
            .zip(llv)
            .map(|(a, b)| (a + b) / 2.0)
            .collect()
    }

    pub async fn get_geomean(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("GEOMEAN");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_geomean(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_geomean(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        for i in 0..count {
            if i < period as usize {
                result.push(data[i])
            } else {
                let mut num = data[i].powf(1.0 / period);
                for j in 1..period as usize {
                    num *= data[i - j as usize].powf(1.0 / period);
                }
                result.push(num);
            }
        }
        self.result = result;
    }

    pub async fn get_amma(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("AMMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_amma(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_amma(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        for i in 0..count {
            if i < period as usize {
                result.push(data[i])
            } else {
                let num = ((period - 1.0) * result[i - 1] + data[i]) / period;
                result.push(num);
            }
        }
        self.result = result;
    }

    pub async fn get_sqwma(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("SQWMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_sqwma(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_sqwma(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        let num = period * (period - 1.0) / 2.0;
        let num2 = period * (period - 1.0) * (2.0 * period - 1.0) / 6.0;
        for i in 0..count {
            if i < period as usize {
                result.push(data[i])
            } else {
                let mut num3 = 0.0;
                let mut num4 = 0.0;
                for j in 0..period as usize {
                    let num5 = data[i - j as usize];
                    num3 += num5;
                    num4 += num5 * j as f32;
                }
                let num6 = num2 * period - num * num;
                let num7 = (num4 * period - num * num3) / num6;
                let num8 = (num3 - num * num7) / period;
                result.push(num8);
            }
        }
        self.result = result;
    }
    pub async fn get_sinewma(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("SINEWMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_sinewma(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_sinewma(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        for i in 0..count {
            if i < period as usize {
                result.push(data[i])
            } else {
                let num = 3.1415926535;
                let mut num2 = 0.0;
                let mut num3 = 0.0;
                for j in 0..period as usize - 1 {
                    num3 += (num * (j as f32 + 1.0) / (period + 1.0)).sin();
                    num2 += data[i - j as usize] * (num * (j as f32 + 1.0) / (period + 1.0)).sin();
                }
                let mut result2 = 0.0;
                if num3 > 0.0 {
                    result2 = num2 / num3;
                }
                result.push(result2 as f32);
            }
        }
        self.result = result;
    }

    pub async fn get_ama(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("AMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_ama(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_ama(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        let mut num: f32 = if count < period as usize {
            0.0
        } else {
            data[(period - 1.0) as usize] as f32
        };
        for i in 0..count {
            if i < (period + 2.0) as usize {
                result.push(data[i]);
            } else {
                let num2 = (data[i] - data[i - period as usize]).abs();
                let mut num3 = 1E-09;
                for j in 0..period as usize {
                    num3 += (data[i - j as usize] - data[i - j as usize - 1]).abs() as f32;
                }
                let num4 = num2 as f32 / num3;

                let x = num4 * 0.60215 + 0.06452;
                let num5 = x.powf(2.0);

                let num6 = num + num5 * (data[i] - num as f32) as f32;
                result.push(num6 as f32);
                num = num6;
            }
        }
        self.result = result;
    }
    pub async fn get_zlema(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("ZLEMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        let result = self.calculate_zlema(&self.data, period).await;
        println!("{:?}", result);
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_zlema(&self, data: &Vec<f32>, period: f32) -> Vec<f32> {
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        let num: f32 = 2.0 / (period + 1.0);
        let num2: i32 = (period as i32 - 1) / 2;
        for i in 0..count {
            if i < period as usize {
                result.push(data[i]);
            } else {
                result.push(
                    result[i - 1]
                        + num * (data[i] + (data[i] - data[i - num2 as usize]) - result[i - 1]),
                )
            }
        }
        return result;
    }

    pub async fn get_ema(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("EMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_ema(period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_ema(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = Vec::new();
        // let mut num: f32 = 2.0 / (period as f32 + 1.0);
        let num: f32 = 2.0 / (period + 1.0);
        for i in 0..count {
            if i < period as usize {
                result.push(data[i]);
            } else {
                result.push(result[i - 1] + num * (data[i] - result[i - 1]))
            }
        }
        self.result = result;
    }
    pub async fn get_tpbf(&mut self, period: f32, meta: bool) -> IndicatorData {
        // Сделать версию для инструмента h l
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("TPBF");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        let h = self.get_maxfor(period, false).await;
        let l = self.get_minfor(period, false).await;
        self.calculate_tpbf(&h.data, &l.data, period).await;
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_tpbf(&mut self, h: &Vec<f32>, l: &Vec<f32>, period: f32) {
        let count = h.len();
        let mut result: Vec<f32> = Vec::new();
        let num = 1.0_f32.atan() as f32;
        let num2 = 45.0 / num;
        let num3 = 1.0 / num2;
        let num4 = 1.0_f32.atan() * 4.0_f32;
        let num5 = (-num4 as f32 / period as f32).exp();
        let num6 = 2.0 * num5 * (num3 * (3.0_f32.sqrt() as f32) * 180.0 / period as f32).cos();
        let num7 = num5 * num5;
        let num8 = num6 + num7;
        let num9 = -(num7 + num6 * num7);
        let num10 = num7 * num7;
        let num11 = (1.0 - num6 + num7) * (1.0 - num7) / 8.0;
        for i in 0..count {
            if i < 4 as usize {
                result.push((*&l[i] + *&h[i]) / 2.0);
            } else {
                result.push(
                    (num11
                        * ((*&l[i] + *&h[i]) / 2.0
                            + 3.0 * ((*&l[i - 1] + *&h[i - 1]) / 2.0)
                            + 3.0 * ((*&l[i - 2] + *&h[i - 2]) / 2.0)
                            + (*&l[i - 3] + *&h[i - 3]) / 2.0) as f32
                        + num8 * result[i - 1] as f32
                        + num9 * result[i - 2] as f32
                        + num10 * result[i - 3] as f32) as f32,
                )
            }
        }
        self.result = result;
    }
    pub async fn get_wma(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("WMA");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_wma(period).await;
        // println!("{:?}", result);
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_wma(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = vec![0.0; count];
        let mut num: f32 = 0.0;
        let mut num2: f32 = 0.0;
        for i in 0..period as usize {
            num2 = num2 + i as f32 + 1.0
        }
        for j in period as usize..count {
            let mut num3 = 1.0;
            for k in j - period as usize + 1..j + 1 as usize {
                num += num3 * data[k];
                num3 += 1.0;
            }
            result.push(num / num2);
            num = 0.0
        }
        self.result = result;
    }
    pub async fn get_rsi(&mut self, period: f32, meta: bool) -> IndicatorData {
        self.meta.current_param = HashMap::from([("period".to_string(), period)]);
        self.meta.name = String::from("RSI");
        self.meta.name_param = vec!["period".to_string()];
        self.meta.value_param = vec![period];
        self.meta.optimization_param = HashMap::from([(
            "period".to_string(),
            OptimizationParam {
                start: 10.0,
                stop: 300.0,
                step: 5.0,
            },
        )]);
        if meta {
            return IndicatorData {
                data: self.result.clone(),
                meta: self.meta.clone(),
            };
        }
        self.calculate_rsi(period).await;
        // println!("{:?}", result);
        return IndicatorData {
            data: self.result.clone(),
            meta: self.meta.clone(),
        };
    }
    async fn calculate_rsi(&mut self, period: f32) {
        let data = &self.data;
        let count = data.len();
        let mut result: Vec<f32> = vec![0.0; count];
        if count > 0 {
            let mut array2 = vec![0.0; count];
            let mut array3 = vec![0.0; count];
            array2[0] = 0.0;
            array3[0] = 0.0;
            for i in 1..count {
                let mut num = 0.0;
                let mut num2 = 0.0;
                if self.data[i - 1] < self.data[i] {
                    num = self.data[i] - self.data[i - 1]
                } else if self.data[i - 1] > self.data[i] {
                    num2 = self.data[i - 1] - self.data[i]
                }
                array2[i] = num;
                array3[i] = num2;
            }
            let list = SimpleIndicators::new(array2)
                .await
                .get_ema(period, false)
                .await
                .data;
            let list2 = SimpleIndicators::new(array3)
                .await
                .get_ema(period, false)
                .await
                .data;
            for i in 0..count {
                if list2[i] == 0.0 {
                    result[i] = 100.0;
                } else if list[i] / list2[i] == 1.0 {
                    result[i] = 0.0;
                } else {
                    result[i] = 100.0 - 100.0 / (1.0 + list[i] / list2[i])
                }
            }
        }
        self.result = result;
    }
    // pub async fn get_ma_to_hl(
    //     &self,
    //     h: &Vec<f32>,
    //     l: &Vec<f32>,
    //     ma: MAIndicators,
    //     period: i32,
    // ) -> Vec<f32> {
    //     let result = match ma {
    //         MAIndicators::SMA => {
    //             let hhi = self.calculate_sma(h, period).await;
    //             let lli = self.calculate_sma(l, period).await;
    //             self.calculate_ma_to_hl(hhi, lli).await
    //         }
    //         MAIndicators::VTREND => todo!("Недоступно"),
    //         MAIndicators::GEOMEAN => {
    //             let hhi = self.calculate_geomean(h, period).await;
    //             let lli = self.calculate_geomean(l, period).await;
    //             self.calculate_ma_to_hl(hhi, lli).await
    //         }
    //         MAIndicators::AMMA => todo!("Реализовать"),
    //         MAIndicators::SQWMA => todo!("Реализовать"),
    //         MAIndicators::SINEWMA => todo!("Реализовать"),
    //         MAIndicators::AMA => todo!("Реализовать"),
    //         MAIndicators::TPBF => todo!("Реализовать"),
    //         MAIndicators::ZLEMA => todo!("Реализовать"),
    //         MAIndicators::EMA => todo!("Реализовать"),
    //     };
    //     println!("{:?}", result);
    //     return result;
    // }
    // async fn calculate_ma_to_hl(&self, hhi: Vec<f32>, lli: Vec<f32>) -> Vec<f32> {
    //     let count = hhi.len();
    //     let mut result: Vec<f32> = Vec::new();
    //     let mut num = 0.0;
    //     for i in 1..count {
    //         let mut num2 = 1.0;
    //         if self.data[i] > hhi[i] {
    //             num2 = 1.0
    //         } else if self.data[i] < lli[i] {
    //             num2 = -1.0
    //         } else {
    //             num2 = 0.0
    //         }
    //         if num2 != 0.0 {
    //             num = num2
    //         }
    //         if num == -1.0 {
    //             result.push(hhi[i - 1])
    //         } else {
    //             result.push(lli[i - 1])
    //         }
    //     }
    //     return result;
    // }
}
