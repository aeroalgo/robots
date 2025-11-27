use std::collections::HashMap;

use crate::position::view::ActivePosition;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::auxiliary::AuxiliaryIndicatorSpec;
use crate::risk::context::{StopEvaluationContext, StopValidationContext};
use crate::risk::traits::{StopHandler, StopOutcome, StopValidationResult};
use crate::risk::utils::{calculate_stop_exit_price, get_price_at_index, is_stop_triggered};

pub struct IndicatorStopHandler {
    pub indicator_name: String,
    pub indicator_params: HashMap<String, f64>,
    pub offset_percent: f64,
    pub trailing: bool,
}

impl IndicatorStopHandler {
    pub fn new(
        indicator_name: String,
        indicator_params: HashMap<String, f64>,
        offset_percent: f64,
        trailing: bool,
    ) -> Self {
        Self {
            indicator_name,
            indicator_params,
            offset_percent,
            trailing,
        }
    }

    pub fn auxiliary_alias(&self) -> String {
        let mut params: Vec<_> = self.indicator_params.iter().collect();
        params.sort_by_key(|(k, _)| k.as_str());

        let params_str: String = params
            .iter()
            .map(|(k, v)| format!("{}_{}", k, **v as u32))
            .collect::<Vec<_>>()
            .join("_");

        format!(
            "aux_stop_{}_{}",
            self.indicator_name.to_uppercase(),
            params_str
        )
    }

    fn indicator_description(&self) -> String {
        let params_str: String = self
            .indicator_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", self.indicator_name, params_str)
    }

    fn get_indicator_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index)
    }

    fn get_indicator_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index)
    }

    fn calculate_stop_level(&self, indicator_value: f64, direction: &PositionDirection) -> f64 {
        let offset = indicator_value * self.offset_percent;
        match direction {
            PositionDirection::Long => indicator_value + offset,
            PositionDirection::Short => indicator_value - offset,
            _ => indicator_value,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: &PositionDirection,
    ) -> f64 {
        if !self.trailing {
            return new_stop;
        }

        let stop_key = format!("{}_current_stop", self.name());

        if let Some(current_stop) = position
            .metadata
            .get(&stop_key)
            .and_then(|s| s.parse::<f64>().ok())
        {
            match direction {
                PositionDirection::Long => new_stop.max(current_stop),
                PositionDirection::Short => new_stop.min(current_stop),
                _ => new_stop,
            }
        } else {
            new_stop
        }
    }
}

impl StopHandler for IndicatorStopHandler {
    fn name(&self) -> &str {
        "IndicatorStop"
    }

    fn validate_before_entry(&self, ctx: &StopValidationContext<'_>) -> Option<StopValidationResult> {
        let indicator_value = self.get_indicator_value_for_validation(ctx)? as f64;
        let stop_level = self.calculate_stop_level(indicator_value, &ctx.direction);

        let is_valid = match ctx.direction {
            PositionDirection::Long => ctx.current_price > stop_level,
            PositionDirection::Short => ctx.current_price < stop_level,
            _ => false,
        };

        let indicator_desc = self.indicator_description();
        let reason = if !is_valid {
            match ctx.direction {
                PositionDirection::Long => Some(format!(
                    "Цена {} не выше уровня стопа {} ({} = {})",
                    ctx.current_price, stop_level, indicator_desc, indicator_value
                )),
                PositionDirection::Short => Some(format!(
                    "Цена {} не ниже уровня стопа {} ({} = {})",
                    ctx.current_price, stop_level, indicator_desc, indicator_value
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
        let indicator_value = self.get_indicator_value(ctx)? as f64;
        let new_stop = self.calculate_stop_level(indicator_value, &ctx.position.direction);

        let current_stop = self.compute_updated_stop(ctx.position, new_stop, &ctx.position.direction);

        let low_price = get_price_at_index(ctx.timeframe_data, &PriceField::Low, ctx.index, ctx.current_price);
        let high_price = get_price_at_index(ctx.timeframe_data, &PriceField::High, ctx.index, ctx.current_price);

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
            metadata.insert("indicator_name".to_string(), self.indicator_name.clone());
            metadata.insert("indicator_description".to_string(), self.indicator_description());
            metadata.insert("indicator_value".to_string(), indicator_value.to_string());

            for (key, value) in &self.indicator_params {
                metadata.insert(format!("indicator_{}", key), value.to_string());
            }

            metadata.insert("triggered_price".to_string(), exit_price.to_string());
            metadata.insert(format!("{}_current_stop", self.name()), current_stop.to_string());

            return Some(StopOutcome {
                exit_price,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![AuxiliaryIndicatorSpec {
            indicator_name: self.indicator_name.clone(),
            parameters: self.indicator_params.clone(),
            alias: self.auxiliary_alias(),
        }]
    }
}

