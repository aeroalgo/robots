use std::collections::HashMap;

use crate::indicators::types::ParameterSet;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::context::StopEvaluationContext;
use crate::risk::parameters::create_stop_percentage_parameter;
use crate::risk::traits::{StopHandler, StopOutcome};
use crate::risk::utils::{calculate_stop_exit_price, get_price_at_index, is_stop_triggered};

pub struct PercentTrailingStopHandler {
    pub percentage: f64,
    parameters: ParameterSet,
}

impl PercentTrailingStopHandler {
    pub fn new(percentage: f64) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_stop_percentage_parameter(
            "percentage",
            percentage as f32,
            "Процент для trailing стопа",
        ));
        Self {
            percentage,
            parameters: params,
        }
    }
}

impl StopHandler for PercentTrailingStopHandler {
    fn name(&self) -> &str {
        "PercentTrailingStop"
    }

    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }

    fn compute_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        let max_high = ctx
            .position
            .max_high_since_entry
            .unwrap_or(ctx.current_price);
        let min_low = ctx
            .position
            .min_low_since_entry
            .unwrap_or(ctx.current_price);
        let ratio = self.percentage / 100.0;

        let new_stop = match ctx.position.direction {
            PositionDirection::Long => max_high * (1.0 - ratio),
            PositionDirection::Short => min_low * (1.0 + ratio),
            _ => return None,
        };

        Some(new_stop)
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let current_stop = ctx.position.current_stop?;

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

        if is_stop_triggered(&ctx.position.direction, low_price, high_price, current_stop) {
            let open_price = get_price_at_index(
                ctx.timeframe_data,
                &PriceField::Open,
                ctx.index,
                ctx.current_price,
            );

            let exit_price = calculate_stop_exit_price(
                &ctx.position.direction,
                current_stop,
                open_price,
                ctx.current_price,
            );

            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            metadata.insert("triggered_price".to_string(), exit_price.to_string());
            metadata.insert("percentage".to_string(), self.percentage.to_string());
            return Some(StopOutcome {
                exit_price,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}
