use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct OptimizationParam {
    pub start: i16,
    pub stop: i16,
    pub step: i16,
}
#[derive(Debug, Clone)]
pub struct IndicatorsMeta {
    pub current_param: HashMap<String, i16>,
    pub optimization_param: HashMap<String, OptimizationParam>,
    pub name: String,
    pub name_param: Vec<String>,
    pub value_param: Vec<i16>,
}
#[derive(Debug, Clone)]
pub struct IndicatorData {
    pub data: Vec<f32>,
    pub meta: IndicatorsMeta,
}
