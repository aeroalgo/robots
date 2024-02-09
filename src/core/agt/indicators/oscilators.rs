// use crate::app::charts::model::TickerCandle;
// use itertools::Itertools;
// use mongodb::bson::oid::ObjectId;
// use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index};
// use try_partialord::*;
// extern crate chrono;

// pub struct Oscilators {
//     data: Vec<f32>,
// }
// pub enum OscilatorsIndicators {
//     SMA,
//     VTrend,
//     GEOMEAN,
//     AMMA,
//     SQWMA,
//     SINEWMA,
//     AMA,
//     TPBF,
//     ZLEMA,
//     EMA,
// }

// impl Oscilators {
//     pub async fn new(data: Vec<f32>) -> Self {
//         return Oscilators { data: data };
//     }
//     pub async fn get_sma(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_sma(&self.data, period).await;
//         return result;
//     }
//     async fn calculate_sma(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut num2: f32 = 0.0;
//         let num = min(count, period.try_into().unwrap());
//         let mut result: Vec<f32> = Vec::new();
//         for i in 0..num {
//             num2 += data.index(i);
//             let value = num2 / (i as f32 + 1 as f32);
//             result.push(value);
//         }
//         for i in num..count {
//             let num3: &f32 = data.index(i);
//             let num4: &f32 = data.index(i - period as usize);
//             let num5: &f32 = result.index(i - 1);
//             result.push(num5 + (num3 - num4) / period as f32)
//         }
//         return result;
//     }
//     pub async fn get_maxfor(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_maxfor(&self.data, period).await;
//         return result;
//     }
//     async fn calculate_maxfor(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let num = min(count, period.try_into().unwrap());
//         let mut result: Vec<f32> = Vec::new();
//         for i in 0..num {
//             let value = *data[0..i + 1].iter().try_max().unwrap().expect("msg");
//             result.push(value);
//         }
//         for i in num..count {
//             let value = *data[i - period as usize..i]
//                 .iter()
//                 .try_max()
//                 .unwrap()
//                 .expect("msg");
//             result.push(value)
//         }
//         return result;
//     }

//     pub async fn get_minfor(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_minfor(&self.data, period).await;
//         return result;
//     }
//     async fn calculate_minfor(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let num = min(count, period.try_into().unwrap());
//         let mut result: Vec<f32> = Vec::new();
//         for i in 0..num {
//             let value = *data[0..i + 1].iter().try_min().unwrap().expect("msg");
//             result.push(value);
//         }
//         for i in num..count {
//             let value = *data[i - period as usize..i]
//                 .iter()
//                 .try_min()
//                 .unwrap()
//                 .expect("msg");
//             result.push(value)
//         }
//         return result;
//     }
//     pub async fn get_vtrand(&self, h: &Vec<f32>, l: &Vec<f32>, period: i32) -> Vec<f32> {
//         // Передать максимум за, минимум за
//         let llv = self.calculate_minfor(l, period).await;
//         let hhv = self.calculate_maxfor(h, period).await;
//         let result = self.calculate_vtrand(hhv, llv, period).await;
//         return result;
//     }
//     async fn calculate_vtrand(&self, hhv: Vec<f32>, llv: Vec<f32>, period: i32) -> Vec<f32> {
//         hhv.into_iter()
//             .zip(llv)
//             .map(|(a, b)| (a + b) / 2.0)
//             .collect()
//     }

//     pub async fn get_geomean(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_geomean(&self.data, period).await;
//         println!("{:?}", result);
//         return result;
//     }
//     async fn calculate_geomean(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         for i in 0..count {
//             if i < period as usize {
//                 result.push(data[i])
//             } else {
//                 let mut num = data[i].powf(1.0 / period as f32);
//                 for j in 1..period {
//                     num *= data[i - j as usize].powf(1.0 / period as f32);
//                 }
//                 result.push(num);
//             }
//         }
//         return result;
//     }

//     pub async fn get_amma(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_amma(&self.data, period).await;
//         return result;
//     }
//     async fn calculate_amma(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         for i in 0..count {
//             if i < period as usize {
//                 result.push(data[i])
//             } else {
//                 let num = ((period - 1) as f32 * result[i - 1] + data[i]) / period as f32;
//                 result.push(num);
//             }
//         }
//         return result;
//     }

//     pub async fn get_sqwma(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_sqwma(&self.data, period).await;
//         return result;
//     }
//     async fn calculate_sqwma(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         let num = period as f32 * (period - 1) as f32 / 2.0;
//         let num2 = period as f32 * (period - 1) as f32 * (2 * period - 1) as f32 / 6.0;
//         for i in 0..count {
//             if i < period as usize {
//                 result.push(data[i])
//             } else {
//                 let mut num3 = 0.0;
//                 let mut num4 = 0.0;
//                 for j in 0..period {
//                     let num5 = data[i - j as usize];
//                     num3 += num5;
//                     num4 += num5 * j as f32;
//                 }
//                 let num6 = num2 * period as f32 - num * num;
//                 let num7 = (num4 * period as f32 - num * num3) / num6;
//                 let num8 = (num3 - num * num7) / period as f32;
//                 result.push(num8);
//             }
//         }
//         return result;
//     }
//     pub async fn get_sinewma(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_sinewma(&self.data, period).await;
//         println!("{:?}", result);
//         return result;
//     }
//     async fn calculate_sinewma(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         for i in 0..count {
//             if i < period as usize {
//                 result.push(data[i])
//             } else {
//                 let num = 3.1415926535;
//                 let mut num2 = 0.0;
//                 let mut num3 = 0.0;
//                 for j in 0..period - 1 {
//                     num3 += (num * (j as f64 + 1.0) / (period + 1) as f64).sin();
//                     num2 += data[i - j as usize] as f64
//                         * (num * (j as f64 + 1.0) / (period + 1) as f64).sin();
//                 }
//                 let mut result2 = 0.0;
//                 if num3 > 0.0 {
//                     result2 = num2 / num3;
//                 }
//                 result.push(result2 as f32);
//             }
//         }
//         return result;
//     }

//     pub async fn get_ama(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_ama(&self.data, period).await;
//         println!("{:?}", result);
//         return result;
//     }
//     async fn calculate_ama(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         let mut num: f64 = if count < period as usize {
//             0.0
//         } else {
//             data[(period - 1) as usize] as f64
//         };
//         for i in 0..count {
//             if i < (period + 2) as usize {
//                 result.push(data[i]);
//             } else {
//                 let num2 = (data[i] - data[i - period as usize]).abs();
//                 let mut num3 = 1E-09;
//                 for j in 0..period {
//                     num3 += (data[i - j as usize] - data[i - j as usize - 1]).abs() as f64;
//                 }
//                 let num4 = num2 as f64 / num3;

//                 let x = num4 * 0.60215 + 0.06452;
//                 let num5 = x.powf(2.0);

//                 let num6 = num + num5 * (data[i] - num as f32) as f64;
//                 result.push(num6 as f32);
//                 num = num6;
//             }
//         }
//         return result;
//     }
//     pub async fn get_zlema(&self, period: i32) -> Vec<f32> {
//         let result = self.calculate_zlema(&self.data, period).await;
//         println!("{:?}", result);
//         return result;
//     }
//     async fn calculate_zlema(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         let num: f32 = 2.0 / (period as f32 + 1.0);
//         let num2: i32 = (period - 1) / 2;
//         for i in 0..count {
//             if i < period as usize {
//                 result.push(data[i]);
//             } else {
//                 result.push(
//                     result[i - 1]
//                         + num * (data[i] + (data[i] - data[i - num2 as usize]) - result[i - 1]),
//                 )
//             }
//         }
//         return result;
//     }

//     pub async fn get_ema(&self, period: i32) -> Vec<f32> {
//         let result: Vec<f32> = self.calculate_ema(&self.data, period).await;
//         return result;
//     }
//     async fn calculate_ema(&self, data: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = data.len();
//         let mut result: Vec<f32> = Vec::new();
//         // let mut num: f32 = 2.0 / (period as f32 + 1.0);
//         let num: f32 = 2.0 / (period + 1) as f32;
//         for i in 0..count {
//             if i < period as usize {
//                 result.push(data[i]);
//             } else {
//                 result.push(result[i - 1] + num * (data[i] - result[i - 1]))
//             }
//         }
//         return result;
//     }
//     pub async fn get_tpbf(&self, h: &Vec<f32>, l: &Vec<f32>, period: i32) -> Vec<f32> {
//         let result = self.calculate_tpbf(&h, &l, period).await;
//         println!("{:?}", result);
//         return result;
//     }
//     async fn calculate_tpbf(&self, h: &Vec<f32>, l: &Vec<f32>, period: i32) -> Vec<f32> {
//         let count = h.len();
//         let mut result: Vec<f32> = Vec::new();
//         let num = 1.0_f32.atan() as f64;
//         let num2 = 45.0 / num;
//         let num3 = 1.0 / num2;
//         let num4 = 1.0_f32.atan() * 4.0_f32;
//         let num5 = (-num4 / period as f32).exp() as f64;
//         let num6 =
//             2.0 * num5 as f64 * (num3 * 3.0_f32.sqrt() as f64 * 180.0 / period as f64).cos() as f64;
//         let num7 = num5 * num5;
//         let num8 = num6 + num7;
//         let num9 = -(num7 + num6 * num7);
//         let num10 = num7 * num7;
//         let num11 = (1.0 - num6 + num7) * (1.0 - num7) / 8.0;
//         for i in 0..count {
//             if i < 4 as usize {
//                 result.push((*&l[i] + *&h[i]) / 2.0);
//             } else {
//                 result.push(
//                     (num11
//                         * ((*&l[i] + *&h[i]) / 2.0
//                             + 3.0 * ((*&l[i - 1] + *&h[i - 1]) / 2.0)
//                             + 3.0 * ((*&l[i - 2] + *&h[i - 2]) / 2.0)
//                             + (*&l[i - 3] + *&h[i - 3]) / 2.0) as f64
//                         + num8 * result[i - 1] as f64
//                         + num9 * result[i - 2] as f64
//                         + num10 * result[i - 3] as f64) as f32,
//                 )
//             }
//         }
//         return result;
//     }
//     pub async fn get_ma_to_hl(
//         &self,
//         h: &Vec<f32>,
//         l: &Vec<f32>,
//         ma: MAIndicators,
//         period: i32,
//     ) -> Vec<f32> {
//         let result = match ma {
//             MAIndicators::SMA => {
//                 let hhi = self.calculate_sma(h, period).await;
//                 let lli = self.calculate_sma(l, period).await;
//                 self.calculate_ma_to_hl(hhi, lli).await
//             }
//             MAIndicators::VTrend => todo!("Недоступно"),
//             MAIndicators::GEOMEAN => {
//                 let hhi = self.calculate_geomean(h, period).await;
//                 let lli = self.calculate_geomean(l, period).await;
//                 self.calculate_ma_to_hl(hhi, lli).await
//             }
//             MAIndicators::AMMA => todo!("Реализовать"),
//             MAIndicators::SQWMA => todo!("Реализовать"),
//             MAIndicators::SINEWMA => todo!("Реализовать"),
//             MAIndicators::AMA => todo!("Реализовать"),
//             MAIndicators::TPBF => todo!("Реализовать"),
//             MAIndicators::ZLEMA => todo!("Реализовать"),
//             MAIndicators::EMA => todo!("Реализовать"),
//         };
//         println!("{:?}", result);
//         return result;
//     }
//     async fn calculate_ma_to_hl(&self, hhi: Vec<f32>, lli: Vec<f32>) -> Vec<f32> {
//         let count = hhi.len();
//         let mut result: Vec<f32> = Vec::new();
//         let mut num = 0.0;
//         for i in 1..count {
//             let mut num2 = 1.0;
//             if self.data[i] > hhi[i] {
//                 num2 = 1.0
//             } else if self.data[i] < lli[i] {
//                 num2 = -1.0
//             } else {
//                 num2 = 0.0
//             }
//             if num2 != 0.0 {
//                 num = num2
//             }
//             if num == -1.0 {
//                 result.push(hhi[i - 1])
//             } else {
//                 result.push(lli[i - 1])
//             }
//         }
//         return result;
//     }
// }
