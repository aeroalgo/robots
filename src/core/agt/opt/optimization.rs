use crate::{
    app::charts::model::TickerCandle,
    core::agt::{
        candles::source::Source,
        indicators::{
            any::SimpleIndicatorsEnum,
            common::{IndicatorData, IndicatorsEnum},
            source::SourceIndicators,
        },
    },
};

use super::iterating::{
    conditions::SourceCombinationCondition,
    indicators::{QuantityIndicators, SimpleCombinationIndicators, SourceCombinationIndicators},
};

pub struct MainOptimization {}

impl MainOptimization {
    pub async fn execute(
        quntity_source_indicators: QuantityIndicators,
        quntity_simple_indicators: QuantityIndicators,
        source_data: Vec<TickerCandle>,
    ) {
        let source_data = Source::new(source_data).await;
        let mut source_indicators = SourceIndicators::new(&source_data).await;

        // Конвертируем QuantityIndicators в usize
        let source_quantity = match quntity_source_indicators {
            QuantityIndicators::One => 1,
            QuantityIndicators::Two => 2,
            QuantityIndicators::Three => 3,
            QuantityIndicators::Four => 4,
        };

        let simple_quantity = match quntity_simple_indicators {
            QuantityIndicators::One => 1,
            QuantityIndicators::Two => 2,
            QuantityIndicators::Three => 3,
            QuantityIndicators::Four => 4,
        };

        let source_indicatiors_combination: Vec<Vec<IndicatorsEnum>> =
            SourceCombinationIndicators::execute(source_quantity);
        let z: Vec<Vec<SimpleIndicatorsEnum>> =
            SimpleCombinationIndicators::execute(simple_quantity);

        let source_conditions = SourceCombinationCondition::execute(source_quantity);
        let mut x = 0;
        for indicators in source_indicatiors_combination.iter() {
            for condition in source_conditions.iter() {
                let mut indicator_data: Vec<IndicatorData> = vec![];
                let mut condition_data: Vec<IndicatorData> = vec![];

                // for indicator in indicators.iter() {
                //     indicator_data.push(
                //         source_indicators
                //             .get_indicator(indicator, 20, 1.0, false)
                //             .await,
                //     );

                // }
                x = x + 1
                // println!("{:?}", indicators);
            }
        }
        println!("{:?}", x);
    }
}
