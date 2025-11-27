use std::collections::HashMap;

use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::context::StopEvaluationContext;
use crate::risk::traits::{StopHandler, StopOutcome};
use crate::risk::utils::{compute_trailing_stop, get_bar_extremes, get_price_at_index, is_stop_triggered};

pub struct PercentTrailingStopHandler {
    pub percentage: f64,
}

impl PercentTrailingStopHandler {
    pub fn new(percentage: f64) -> Self {
        Self { percentage }
    }

    fn calculate_stop_level(
        &self,
        direction: &PositionDirection,
        min_price: f64,
        max_price: f64,
        current_price: f64,
    ) -> f64 {
        let ratio = self.percentage / 100.0;

        match direction {
            PositionDirection::Long => min_price * (1.0 - ratio),
            PositionDirection::Short => max_price * (1.0 + ratio),
            _ => current_price,
        }
    }
}

impl StopHandler for PercentTrailingStopHandler {
    fn name(&self) -> &str {
        "PercentTrailingStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let (min_price, max_price) = get_bar_extremes(ctx.timeframe_data, ctx.index, ctx.current_price);

        let new_stop = self.calculate_stop_level(
            &ctx.position.direction,
            min_price,
            max_price,
            ctx.current_price,
        );

        let current_stop = compute_trailing_stop(
            ctx.position,
            new_stop,
            &ctx.position.direction,
            self.name(),
        );

        let low_price = get_price_at_index(ctx.timeframe_data, &PriceField::Low, ctx.index, ctx.current_price);
        let high_price = get_price_at_index(ctx.timeframe_data, &PriceField::High, ctx.index, ctx.current_price);

        if is_stop_triggered(&ctx.position.direction, low_price, high_price, current_stop) {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match ctx.position.direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert("percentage".to_string(), self.percentage.to_string());
            metadata.insert("min_price".to_string(), min_price.to_string());
            metadata.insert("max_price".to_string(), max_price.to_string());
            metadata.insert(format!("{}_current_stop", self.name()), current_stop.to_string());
            metadata.insert(format!("{}_min_price", self.name()), min_price.to_string());
            metadata.insert(format!("{}_max_price", self.name()), max_price.to_string());
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

