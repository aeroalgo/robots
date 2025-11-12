use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct OptimizationParam {
    pub start: f32,
    pub stop: f32,
    pub step: f32,
}
#[derive(Debug, Clone)]
pub struct IndicatorsMeta {
    pub current_param: HashMap<String, f32>,
    pub optimization_param: HashMap<String, OptimizationParam>,
    pub name: String,
    pub name_param: Vec<String>,
    pub value_param: Vec<f32>,
    pub multi_indicator: bool,
}
#[derive(Debug, Clone)]
pub struct IndicatorData {
    pub data: Vec<f32>,
    pub meta: IndicatorsMeta,
}

#[derive(Debug, PartialEq, Sequence, Clone, Copy, Hash, Eq)]
pub enum IndicatorsEnum {
    RSI,
    STOCHASTIC,
    ATR,
    ATROLD,
    WATR,
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
}
