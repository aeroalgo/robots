use std::collections::HashMap;

use crate::indicators::types::ParameterSet;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind};

use crate::risk::{normalize_indicator_params, AuxiliaryIndicatorSpec};
use crate::risk::context::{StopEvaluationContext, StopValidationContext};
use crate::risk::parameters::create_stop_percentage_parameter;
use crate::risk::traits::{StopHandler, StopOutcome, StopValidationResult};
use crate::risk::utils::{calculate_stop_exit_price, get_price_at_index, is_stop_triggered, validate_indicator_before_entry};

pub struct PercentTrailIndicatorStopHandler {
    pub percentage: f64,
    pub indicator_name: String,
    pub indicator_params: HashMap<String, f64>,
    parameters: ParameterSet,
}

impl PercentTrailIndicatorStopHandler {
    pub fn new(
        percentage: f64,
        indicator_name: String,
        indicator_params: HashMap<String, f64>,
    ) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_stop_percentage_parameter(
            "percentage",
            percentage as f32,
            "Процент для trailing стопа",
        ));
        Self {
            percentage,
            indicator_name,
            indicator_params,
            parameters: params,
        }
    }

    fn auxiliary_alias(&self) -> String {
        let indicator_params =
            normalize_indicator_params(&self.indicator_name, &self.indicator_params);
        let mut params: Vec<_> = indicator_params.iter().collect();
        params.sort_by_key(|(k, _)| k.as_str());
        let params_str: String = params
            .iter()
            .map(|(k, v)| format!("{}_{}", k, **v as u32))
            .collect::<Vec<_>>()
            .join("_");
        format!("aux_stop_ind_{}_{}", self.indicator_name, params_str)
    }

    fn get_indicator_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index)
    }

    fn get_indicator_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index)
    }

    fn indicator_description(&self) -> String {
        format!("{}", self.indicator_name)
    }
}

impl StopHandler for PercentTrailIndicatorStopHandler {
    fn name(&self) -> &str {
        "PercentTrailIndicatorStop"
    }

    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        let indicator_value = self.get_indicator_value_for_validation(ctx)? as f64;
        let indicator_desc = self.indicator_description();
        Some(validate_indicator_before_entry(ctx, indicator_value, &indicator_desc))
    }

    fn compute_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        let indicator_value = self.get_indicator_value(ctx)? as f64;
        let ratio = self.percentage / 100.0;

        let new_stop = match ctx.position.direction {
            PositionDirection::Long => indicator_value * (1.0 - ratio),
            PositionDirection::Short => indicator_value * (1.0 + ratio),
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

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        let indicator_params =
            normalize_indicator_params(&self.indicator_name, &self.indicator_params);
        let mut params: Vec<_> = indicator_params.iter().collect();
        params.sort_by_key(|(k, _)| k.as_str());
        let params_str: String = params
            .iter()
            .map(|(k, v)| format!("{}_{}", k, **v as u32))
            .collect::<Vec<_>>()
            .join("_");
        let alias = format!("aux_stop_ind_{}_{}", self.indicator_name, params_str);

        vec![AuxiliaryIndicatorSpec {
            indicator_name: self.indicator_name.clone(),
            parameters: indicator_params,
            alias,
        }]
    }
}
