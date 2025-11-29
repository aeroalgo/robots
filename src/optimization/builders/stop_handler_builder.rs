use crate::discovery::types::{StopHandlerConfig, StopHandlerInfo};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;

use crate::optimization::candidate_builder_config::CandidateBuilderConfig;

pub struct StopHandlerBuilder<'a> {
    config: &'a CandidateBuilderConfig,
    rng: &'a mut rand::rngs::ThreadRng,
}

impl<'a> StopHandlerBuilder<'a> {
    pub fn new(config: &'a CandidateBuilderConfig, rng: &'a mut rand::rngs::ThreadRng) -> Self {
        Self { config, rng }
    }

    pub fn select_stop_handler(
        &mut self,
        available: &[StopHandlerConfig],
    ) -> Option<StopHandlerInfo> {
        let excluded_stop_handlers: HashSet<&str> = self
            .config
            .rules
            .excluded_stop_handlers
            .iter()
            .map(|s| s.as_str())
            .collect();

        let stop_loss_configs: Vec<&StopHandlerConfig> = available
            .iter()
            .filter(|c| c.stop_type == "stop_loss")
            .filter(|c| !excluded_stop_handlers.contains(c.handler_name.as_str()))
            .collect();

        if stop_loss_configs.is_empty() {
            return None;
        }

        stop_loss_configs
            .choose(&mut *self.rng)
            .map(|config| StopHandlerInfo {
                id: format!("stop_{}", self.rng.gen::<u32>()),
                name: config.handler_name.clone(),
                handler_name: config.handler_name.clone(),
                stop_type: config.stop_type.clone(),
                optimization_params: Self::make_handler_params(config),
                priority: config.priority,
            })
    }

    pub fn select_take_handler(
        &mut self,
        available: &[StopHandlerConfig],
    ) -> Option<StopHandlerInfo> {
        let take_configs: Vec<&StopHandlerConfig> = available
            .iter()
            .filter(|c| c.stop_type == "take_profit")
            .collect();

        take_configs
            .choose(&mut *self.rng)
            .map(|config| StopHandlerInfo {
                id: format!("take_{}", self.rng.gen::<u32>()),
                name: config.handler_name.clone(),
                handler_name: config.handler_name.clone(),
                stop_type: config.stop_type.clone(),
                optimization_params: Self::make_handler_params(config),
                priority: config.priority,
            })
    }

    pub fn make_handler_params(config: &StopHandlerConfig) -> Vec<crate::discovery::ConditionParamInfo> {
        if config.parameter_name.is_empty() {
            Vec::new()
        } else {
            vec![crate::discovery::ConditionParamInfo {
                name: config.parameter_name.clone(),
                optimizable: true,
                global_param_name: config.global_param_name.clone(),
            }]
        }
    }
}
