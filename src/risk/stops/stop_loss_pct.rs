use std::collections::HashMap;

use crate::position::view::ActivePosition;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::context::StopEvaluationContext;
use crate::risk::traits::{StopHandler, StopOutcome};
use crate::risk::utils::{calculate_stop_exit_price, get_price_at_index};

pub struct StopLossPctHandler {
    pub percentage: f64,
}

impl StopLossPctHandler {
    pub fn new(percentage: f64) -> Self {
        Self { percentage }
    }

    fn level(&self, position: &ActivePosition) -> Option<f64> {
        let ratio = self.percentage / 100.0;
        match position.direction {
            PositionDirection::Long => Some(position.entry_price * (1.0 - ratio)),
            PositionDirection::Short => Some(position.entry_price * (1.0 + ratio)),
            _ => None,
        }
    }
}

impl StopHandler for StopLossPctHandler {
    fn name(&self) -> &str {
        "StopLossPct"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let level = self.level(ctx.position)?;

        let low_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::Low,
            ctx.index,
            ctx.current_price,
        );

        let high_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::High,
            ctx.index,
            ctx.current_price,
        );

        let triggered = match ctx.position.direction {
            PositionDirection::Long => low_price <= level,
            PositionDirection::Short => high_price >= level,
            _ => false,
        };

        if triggered {
            let open_price = get_price_at_index(
                ctx.timeframe_data,
                &PriceField::Open,
                ctx.index,
                ctx.current_price,
            );
            
            let exit_price = calculate_stop_exit_price(
                &ctx.position.direction,
                level,
                open_price,
                ctx.current_price,
            );
            
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            metadata.insert("triggered_price".to_string(), exit_price.to_string());
            return Some(StopOutcome {
                exit_price,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

