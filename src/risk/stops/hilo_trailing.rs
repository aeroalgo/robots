use std::collections::HashMap;

use crate::indicators::types::ParameterSet;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::auxiliary::AuxiliaryIndicatorSpec;
use crate::risk::context::{StopEvaluationContext, StopValidationContext};
use crate::risk::parameters::create_stop_period_parameter;
use crate::risk::traits::{StopHandler, StopOutcome, StopValidationResult};
use crate::risk::utils::{calculate_stop_exit_price, get_price_at_index, is_stop_triggered};

pub struct HILOTrailingStopHandler {
    pub period: f64,
    parameters: ParameterSet,
}

impl HILOTrailingStopHandler {
    pub fn new(period: f64) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_stop_period_parameter(
            "period",
            period as f32,
            "Период для расчета MINFOR/MAXFOR",
        ));
        Self {
            period,
            parameters: params,
        }
    }

    fn minfor_auxiliary_alias(&self) -> String {
        format!("aux_MINFOR_{}", self.period as u32)
    }

    fn maxfor_auxiliary_alias(&self) -> String {
        format!("aux_MAXFOR_{}", self.period as u32)
    }

    fn get_minfor_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let aux_alias = self.minfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        let minfor_alias = format!("MINFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&minfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MINFOR", ctx.index))
    }

    fn get_maxfor_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let aux_alias = self.maxfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        let maxfor_alias = format!("MAXFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&maxfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MAXFOR", ctx.index))
    }

    fn get_minfor_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let aux_alias = self.minfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        let minfor_alias = format!("MINFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&minfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MINFOR", ctx.index))
    }

    fn get_maxfor_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let aux_alias = self.maxfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        let maxfor_alias = format!("MAXFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&maxfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MAXFOR", ctx.index))
    }

    fn calculate_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        match ctx.position.direction {
            PositionDirection::Long => {
                let minfor = self.get_minfor_value(ctx)?;
                Some(minfor as f64)
            }
            PositionDirection::Short => {
                let maxfor = self.get_maxfor_value(ctx)?;
                Some(maxfor as f64)
            }
            _ => None,
        }
    }
}

impl StopHandler for HILOTrailingStopHandler {
    fn name(&self) -> &str {
        "HILOTrailingStop"
    }

    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }

    fn compute_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        self.calculate_stop_level(ctx)
    }

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        let stop_level = match ctx.direction {
            PositionDirection::Long => {
                let minfor = self.get_minfor_value_for_validation(ctx)?;
                minfor as f64
            }
            PositionDirection::Short => {
                let maxfor = self.get_maxfor_value_for_validation(ctx)?;
                maxfor as f64
            }
            _ => return None,
        };

        let is_valid = match ctx.direction {
            PositionDirection::Long => ctx.current_price > stop_level,
            PositionDirection::Short => ctx.current_price < stop_level,
            _ => false,
        };

        let reason = if !is_valid {
            match ctx.direction {
                PositionDirection::Long => Some(format!(
                    "Цена {} не выше MINFOR уровня {}",
                    ctx.current_price, stop_level
                )),
                PositionDirection::Short => Some(format!(
                    "Цена {} не ниже MAXFOR уровня {}",
                    ctx.current_price, stop_level
                )),
                _ => None,
            }
        } else {
            None
        };

        Some(StopValidationResult {
            stop_level,
            is_valid,
            reason,
        })
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

            if let Some(minfor) = self.get_minfor_value(ctx) {
                metadata.insert("minfor_value".to_string(), minfor.to_string());
            }
            if let Some(maxfor) = self.get_maxfor_value(ctx) {
                metadata.insert("maxfor_value".to_string(), maxfor.to_string());
            }

            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            return Some(StopOutcome {
                exit_price,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![
            AuxiliaryIndicatorSpec::minfor(self.period as u32),
            AuxiliaryIndicatorSpec::maxfor(self.period as u32),
        ]
    }
}
