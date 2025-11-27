use std::collections::HashMap;

use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::auxiliary::AuxiliaryIndicatorSpec;
use crate::risk::context::StopEvaluationContext;
use crate::risk::traits::{StopHandler, StopOutcome};
use crate::risk::utils::{calculate_stop_exit_price, get_price_at_index, is_stop_triggered};

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
}

impl StopHandler for ATRTrailStopHandler {
    fn name(&self) -> &str {
        "ATRTrailStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let atr_value = self.get_atr_value(ctx)?;

        let max_high = ctx.max_high_since_entry;
        let min_low = ctx.min_low_since_entry;

        let offset = atr_value as f64 * self.coeff_atr;
        let new_stop = match ctx.position.direction {
            PositionDirection::Long => max_high - offset,
            PositionDirection::Short => min_low + offset,
            _ => ctx.current_price,
        };

        let prev_stop = ctx.current_stop;

        // Trailing: стоп может только улучшаться
        let effective_stop = match ctx.position.direction {
            PositionDirection::Long => {
                // Для Long стоп растёт (подтягивается вверх)
                prev_stop.map(|p| new_stop.max(p)).unwrap_or(new_stop)
            }
            PositionDirection::Short => {
                // Для Short стоп падает (подтягивается вниз)
                prev_stop.map(|p| new_stop.min(p)).unwrap_or(new_stop)
            }
            _ => new_stop,
        };

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

        if is_stop_triggered(
            &ctx.position.direction,
            low_price,
            high_price,
            effective_stop,
        ) {
            let open_price = get_price_at_index(
                ctx.timeframe_data,
                &PriceField::Open,
                ctx.index,
                ctx.current_price,
            );

            let exit_price = calculate_stop_exit_price(
                &ctx.position.direction,
                effective_stop,
                open_price,
                ctx.current_price,
            );

            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), effective_stop.to_string());
            metadata.insert("triggered_price".to_string(), exit_price.to_string());
            metadata.insert("atr_value".to_string(), atr_value.to_string());
            metadata.insert("max_high".to_string(), max_high.to_string());
            metadata.insert("min_low".to_string(), min_low.to_string());

            return Some(StopOutcome {
                exit_price,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![AuxiliaryIndicatorSpec::atr(self.period as u32)]
    }

    fn get_trailing_updates(&self, _ctx: &StopEvaluationContext<'_>) -> HashMap<String, String> {
        HashMap::new()
    }

    fn compute_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        let atr_value = self.get_atr_value(ctx)?;

        let max_high = ctx.max_high_since_entry;
        let min_low = ctx.min_low_since_entry;

        let offset = atr_value as f64 * self.coeff_atr;
        let new_stop = match ctx.position.direction {
            PositionDirection::Long => max_high - offset,
            PositionDirection::Short => min_low + offset,
            _ => return None,
        };

        Some(new_stop)
    }
}
