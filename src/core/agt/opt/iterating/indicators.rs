use crate::core::agt::indicators::any::SimpleIndicatorsEnum;
use crate::core::agt::indicators::source::SourceIndicators;
use crate::core::agt::{
    candles,
    indicators::common::{IndicatorData, IndicatorsEnum, IndicatorsMeta, OptimizationParam},
};
use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use itertools::Itertools;

pub struct SourceCombinationIndicators {}
pub struct SimpleCombinationIndicators {}

impl SourceCombinationIndicators {
    pub fn execute(quantity: usize) -> Vec<Vec<IndicatorsEnum>> {
        let indicators: Vec<IndicatorsEnum> = all::<IndicatorsEnum>().collect();
        let sources = vec![indicators; quantity]; // клонируем indicators quantity раз
        sources.into_iter().multi_cartesian_product().collect()
    }
}

// impl SourceCombinationIndicators {
//     pub async fn execute(quntity_indicators: &QuantityIndicators) -> Vec<Vec<IndicatorsEnum>> {
//         let indicators = all::<IndicatorsEnum>().collect::<Vec<_>>();
//         let mut combination_indicator: Vec<Vec<IndicatorsEnum>> = vec![];
//         for indicator1 in indicators.iter() {
//             if let QuantityIndicators::One = quntity_indicators {
//                 combination_indicator.push(vec![indicator1.clone()])
//             }
//             for indicator2 in indicators.iter() {
//                 if let QuantityIndicators::One = quntity_indicators {
//                     break;
//                 }
//                 if let QuantityIndicators::Two = quntity_indicators {
//                     combination_indicator.push(vec![indicator1.clone(), indicator2.clone()])
//                 }
//                 for indicator3 in indicators.iter() {
//                     if let QuantityIndicators::Two = quntity_indicators {
//                         break;
//                     }
//                     if let QuantityIndicators::Three = quntity_indicators {
//                         combination_indicator.push(vec![
//                             indicator1.clone(),
//                             indicator2.clone(),
//                             indicator3.clone(),
//                         ])
//                     }
//                     for indicator4 in indicators.iter() {
//                         if let QuantityIndicators::Three = quntity_indicators {
//                             break;
//                         }
//                         if let QuantityIndicators::Four = quntity_indicators {
//                             combination_indicator.push(vec![
//                                 indicator1.clone(),
//                                 indicator2.clone(),
//                                 indicator3.clone(),
//                                 indicator4.clone(),
//                             ])
//                         }
//                     }
//                 }
//             }
//         }
//         return combination_indicator;
//     }
// }

// impl SimpleCombinationIndicators {
//     pub async fn execute(quntity_indicators: QuantityIndicators) -> Vec<Vec<SimpleIndicatorsEnum>> {
//         let indicators = all::<SimpleIndicatorsEnum>().collect::<Vec<_>>();
//         let mut combination_indicator: Vec<Vec<SimpleIndicatorsEnum>> = vec![];
//         for indicator1 in indicators.iter() {
//             if let QuantityIndicators::One = quntity_indicators {
//                 combination_indicator.push(vec![indicator1.clone()])
//             }
//             for indicator2 in indicators.iter() {
//                 if let QuantityIndicators::One = quntity_indicators {
//                     break;
//                 }
//                 if let QuantityIndicators::Two = quntity_indicators {
//                     combination_indicator.push(vec![indicator1.clone(), indicator2.clone()])
//                 }
//                 for indicator3 in indicators.iter() {
//                     if let QuantityIndicators::Two = quntity_indicators {
//                         break;
//                     }
//                     if let QuantityIndicators::Three = quntity_indicators {
//                         combination_indicator.push(vec![
//                             indicator1.clone(),
//                             indicator2.clone(),
//                             indicator3.clone(),
//                         ])
//                     }
//                     for indicator4 in indicators.iter() {
//                         if let QuantityIndicators::Three = quntity_indicators {
//                             break;
//                         }
//                         if let QuantityIndicators::Four = quntity_indicators {
//                             combination_indicator.push(vec![
//                                 indicator1.clone(),
//                                 indicator2.clone(),
//                                 indicator3.clone(),
//                                 indicator4.clone(),
//                             ])
//                         }
//                     }
//                 }
//             }
//         }
//         return combination_indicator;
//     }
// }
