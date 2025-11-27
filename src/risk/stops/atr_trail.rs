use std::collections::HashMap;

use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::auxiliary::AuxiliaryIndicatorSpec;
use crate::risk::context::StopEvaluationContext;
use crate::risk::traits::{StopHandler, StopOutcome};
use crate::risk::utils::{
    compute_trailing_stop, get_bar_extremes, get_price_at_index, is_stop_triggered,
};

pub struct ATRTrailStopHandler {
    pub period: f64,
    pub coeff_atr: f64,
}

impl ATRTrailStopHandler {
    pub fn new(period: f64, coeff_atr: f64) -> Self {
        Self { period, coeff_atr }
    }

    fn auxiliary_alias(&self) -> String {
        format!("aux_ATR_{}", self.period as u32)
    }

    fn get_atr_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        let atr_alias = format!("ATR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&atr_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("ATR", ctx.index))
    }

    fn calculate_stop_level(
        &self,
        direction: &PositionDirection,
        min_price: f64,
        max_price: f64,
        atr: f32,
        current_price: f64,
    ) -> f64 {
        let atr_f64 = atr as f64;
        let offset = atr_f64 * self.coeff_atr;

        match direction {
            PositionDirection::Long => min_price - offset,
            PositionDirection::Short => max_price + offset,
            _ => current_price,
        }
    }
}

impl StopHandler for ATRTrailStopHandler {
    fn name(&self) -> &str {
        "ATRTrailStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let atr_value = self.get_atr_value(ctx)?;
        let (min_price, max_price) =
            get_bar_extremes(ctx.timeframe_data, ctx.index, ctx.current_price);

        let new_stop = self.calculate_stop_level(
            &ctx.position.direction,
            min_price,
            max_price,
            atr_value,
            ctx.current_price,
        );

        let current_stop =
            compute_trailing_stop(ctx.position, new_stop, &ctx.position.direction, self.name());

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
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match ctx.position.direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert("atr_value".to_string(), atr_value.to_string());
            metadata.insert("min_price".to_string(), min_price.to_string());
            metadata.insert("max_price".to_string(), max_price.to_string());
            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
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

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![AuxiliaryIndicatorSpec::atr(self.period as u32)]
    }
}
