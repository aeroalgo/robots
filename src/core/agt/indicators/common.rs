use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct OptimizationParam {
    pub start: f64,
    pub stop: f64,
    pub step: f64,
}
#[derive(Debug, Clone)]
pub struct IndicatorsMeta {
    pub current_param: HashMap<String, f64>,
    pub optimization_param: HashMap<String, OptimizationParam>,
    pub name: String,
    pub name_param: Vec<String>,
    pub value_param: Vec<f64>,
    pub multi_indicator: bool,
}
#[derive(Debug, Clone)]
pub struct IndicatorData {
    pub data: Vec<f64>,
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
