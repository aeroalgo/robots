use crate::core::agt::{
    indicators::{any::SimpleIndicatorsEnum, common::IndicatorsEnum},
    opt::iterating::conditions::ConditionEnum,
};

pub struct StrategyParametrs {
    source_indicators: Vec<IndicatorsEnum>,
    simple_indicators: Vec<SimpleIndicatorsEnum>,
    source_condition: Vec<ConditionEnum>,
    simple_condition: Vec<ConditionEnum>,
}
