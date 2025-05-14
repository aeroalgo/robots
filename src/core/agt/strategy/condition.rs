use crate::core::agt::{
    candles,
    indicators::common::{IndicatorData, IndicatorsEnum, IndicatorsMeta, OptimizationParam},
    opt::iterating::conditions::ConditionEnum,
};
use std::{array, cmp::min, collections::HashMap, convert::TryInto, ops::Index, vec};

#[derive(Clone, Debug)]
pub struct StrategyCondition {
    data: Vec<f32>,
    indicator: Vec<f32>,
    constant: Vec<f32>,
    condition: ConditionEnum,
    result: Vec<f32>,
    name_indicator: String,
}

impl StrategyCondition {
    pub async fn new(
        data: Vec<f32>,
        indicator: Vec<f32>,
        condition: ConditionEnum,
        constant: f32,
        name_indicator: String,
    ) -> Self {
        return StrategyCondition {
            data: data.clone(),
            indicator: indicator.clone(),
            constant: vec![constant; data.len()],
            condition: condition,
            result: vec![],
            name_indicator: name_indicator,
        };
    }

    pub async fn get_signal(&self, condition: ConditionEnum) {
        let result = match condition {
            ConditionEnum::ABOVE => todo!(),
            ConditionEnum::BELOW => todo!(),
            ConditionEnum::CROSSESABOVE => todo!(),
            ConditionEnum::CROSSESBELOW => todo!(),
            ConditionEnum::LOWERPERCENTBARS => todo!(),
            ConditionEnum::GREATERPERCENTBARS => todo!(),
            ConditionEnum::FAILINGDATABARS => todo!(),
            ConditionEnum::FAILINGINDICATORSBARS => todo!(),
            ConditionEnum::FALLINGTORISINGDATA => todo!(),
            ConditionEnum::FALLINGTORISINGINDICATORS => todo!(),
            ConditionEnum::RISINGDATABARS => todo!(),
            ConditionEnum::RISINGINDICATORSBARS => todo!(),
            ConditionEnum::RISINGTOFALLINGDATA => todo!(),
            ConditionEnum::RISINGTOFALLINGINDICATORS => todo!(),
        };
    }
}
