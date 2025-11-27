use crate::position::view::ActivePosition;
use crate::strategy::context::TimeframeData;
use crate::strategy::types::{PositionDirection, PriceField};

pub struct StopEvaluationContext<'a> {
    pub position: &'a ActivePosition,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> StopEvaluationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct StopValidationContext<'a> {
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> StopValidationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct TakeEvaluationContext<'a> {
    pub position: &'a ActivePosition,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> TakeEvaluationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

