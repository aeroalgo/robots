use std::collections::HashMap;

use crate::position::view::ActivePosition;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::context::TakeEvaluationContext;
use crate::risk::traits::{TakeHandler, TakeOutcome};
use crate::risk::utils::get_price_at_index;

pub struct TakeProfitPctHandler {
    pub percentage: f64,
}

impl TakeProfitPctHandler {
    pub fn new(percentage: f64) -> Self {
        Self { percentage }
    }

    fn level(&self, position: &ActivePosition) -> Option<f64> {
        let ratio = self.percentage / 100.0;
        match position.direction {
            PositionDirection::Long => Some(position.entry_price * (1.0 + ratio)),
            PositionDirection::Short => Some(position.entry_price * (1.0 - ratio)),
            _ => None,
        }
    }
}

impl TakeHandler for TakeProfitPctHandler {
    fn name(&self) -> &str {
        "TakeProfitPct"
    }

    fn evaluate(&self, ctx: &TakeEvaluationContext<'_>) -> Option<TakeOutcome> {
        let level = self.level(ctx.position)?;

        let high_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::High,
            ctx.index,
            ctx.current_price,
        );

        let low_price = get_price_at_index(
            ctx.timeframe_data,
            &PriceField::Low,
            ctx.index,
            ctx.current_price,
        );

        let triggered = match ctx.position.direction {
            PositionDirection::Long => high_price >= level,
            PositionDirection::Short => low_price <= level,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            let triggered_price = match ctx.position.direction {
                PositionDirection::Long => high_price,
                PositionDirection::Short => low_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            return Some(TakeOutcome {
                exit_price: level,
                kind: StopSignalKind::TakeProfit,
                metadata,
            });
        }
        None
    }
}
