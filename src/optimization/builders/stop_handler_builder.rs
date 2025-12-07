use crate::discovery::types::{IndicatorInfo, StopHandlerConfig, StopHandlerInfo};
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
        available_indicators: &[IndicatorInfo],
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
            .map(|config| {
                let mut params = Self::make_handler_params(config, available);
                let mut handler_name = config.handler_name.clone();
                
                // Для новых стопов с индикаторами выбираем случайный трендовый индикатор
                if config.handler_name == "ATRTrailIndicatorStop"
                    || config.handler_name == "PercentTrailIndicatorStop"
                {
                    if let Some(indicator_name) = Self::select_random_trend_indicator(
                        available_indicators,
                        &mut *self.rng,
                    ) {
                        // Сохраняем выбранный индикатор в name для последующего извлечения
                        handler_name = format!("{}:{}", config.handler_name, indicator_name);
                    }
                }
                
                StopHandlerInfo {
                    id: format!("stop_{}", self.rng.gen::<u32>()),
                    name: handler_name.clone(),
                    handler_name: handler_name,
                    stop_type: config.stop_type.clone(),
                    optimization_params: params,
                    priority: config.priority,
                }
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
                optimization_params: Self::make_handler_params(config, available),
                priority: config.priority,
            })
    }

    pub fn make_handler_params(
        config: &StopHandlerConfig,
        all_configs: &[StopHandlerConfig],
    ) -> Vec<crate::discovery::ConditionParamInfo> {
        let handler_name = &config.handler_name;
        let mut params = Vec::new();

        for cfg in all_configs {
            if cfg.handler_name == *handler_name && cfg.stop_type == config.stop_type {
                if !cfg.parameter_name.is_empty() {
                    params.push(crate::discovery::ConditionParamInfo {
                        name: cfg.parameter_name.clone(),
                        optimizable: true,
                        global_param_name: cfg.global_param_name.clone(),
                    });
                }
            }
        }

        params
    }

    /// Выбирает случайный трендовый индикатор из доступных
    fn select_random_trend_indicator(
        available_indicators: &[IndicatorInfo],
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<String> {
        // Фильтруем только трендовые индикаторы
        let trend_indicators: Vec<&IndicatorInfo> = available_indicators
            .iter()
            .filter(|ind| ind.indicator_type == "trend")
            .collect();

        if trend_indicators.is_empty() {
            // Fallback: список популярных трендовых индикаторов
            let default_trend_indicators = vec!["SMA", "EMA", "WMA", "AMA", "ZLEMA"];
            default_trend_indicators
                .choose(rng)
                .map(|s| s.to_string())
        } else {
            trend_indicators
                .choose(rng)
                .map(|ind| ind.name.clone())
        }
    }
}
