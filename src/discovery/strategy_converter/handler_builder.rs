use std::collections::HashMap;

use crate::data_model::types::TimeFrame;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::StopHandlerInfo;
use crate::strategy::types::{
    PositionDirection, PriceField, StopHandlerSpec, StrategyParamValue, TakeHandlerSpec,
};

use super::main::StrategyConversionError;
use super::parameter_extractor::ParameterExtractor;

pub struct HandlerBuilder;

impl HandlerBuilder {
    pub fn create_stop_handlers(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<StopHandlerSpec>, StrategyConversionError> {
        let mut stop_handlers = Vec::new();

        for stop_handler in &candidate.stop_handlers {
            let (normalized_name_for_defaults, _) =
                crate::risk::extract_indicator_from_handler_name(&stop_handler.handler_name);
            let mut parameters =
                ParameterExtractor::get_default_stop_params(&normalized_name_for_defaults);

            let normalized_handler_name = crate::risk::process_stop_handler_indicator(
                &stop_handler.handler_name,
                &mut parameters,
            );

            stop_handlers.push(StopHandlerSpec {
                id: stop_handler.id.clone(),
                name: stop_handler.name.clone(),
                handler_name: normalized_handler_name,
                timeframe: base_timeframe.clone(),
                price_field: PriceField::Close,
                parameters,
                direction: PositionDirection::Long,
                priority: stop_handler.priority,
                tags: vec!["stop_loss".to_string()],
                target_entry_ids: vec![],
            });
        }

        Ok(stop_handlers)
    }

    pub fn create_take_handlers(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<TakeHandlerSpec>, StrategyConversionError> {
        let mut take_handlers = Vec::new();

        for take_handler in &candidate.take_handlers {
            let parameters = HashMap::new();

            take_handlers.push(TakeHandlerSpec {
                id: take_handler.id.clone(),
                name: take_handler.name.clone(),
                handler_name: take_handler.handler_name.clone(),
                timeframe: base_timeframe.clone(),
                price_field: PriceField::Close,
                parameters,
                direction: PositionDirection::Long,
                priority: take_handler.priority,
                tags: vec!["take_profit".to_string()],
                target_entry_ids: vec![],
            });
        }

        Ok(take_handlers)
    }

    pub fn set_target_entry_ids(
        stop_handlers: &mut [StopHandlerSpec],
        take_handlers: &mut [TakeHandlerSpec],
        entry_rule_ids: &[String],
    ) {
        for stop_handler in stop_handlers.iter_mut() {
            if stop_handler.target_entry_ids.is_empty() {
                stop_handler.target_entry_ids = entry_rule_ids.to_vec();
            }
        }
        for take_handler in take_handlers.iter_mut() {
            if take_handler.target_entry_ids.is_empty() {
                take_handler.target_entry_ids = entry_rule_ids.to_vec();
            }
        }
    }
}
