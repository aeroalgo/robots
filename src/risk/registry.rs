use crate::discovery::types::StopHandlerConfig;
use crate::indicators::types::ParameterRange;
use crate::risk::factory::StopHandlerFactory;
use crate::risk::stops::{
    ATRTrailStopHandler, HILOTrailingStopHandler, PercentTrailingStopHandler, StopLossPctHandler,
};
use crate::risk::takes::TakeProfitPctHandler;
use crate::risk::traits::{StopHandler, TakeHandler};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

pub struct StopHandlerRegistry {
    handlers: Vec<HandlerInfo>,
}

struct HandlerInfo {
    handler_name: String,
    stop_type: String,
    parameter_name: String,
    optimization_range: ParameterRange,
    priority: i32,
}

impl StopHandlerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: Vec::new(),
        };
        registry.register_all_handlers();
        registry
    }

    fn register_all_handlers(&mut self) {
        // Регистрируем Stop Handlers
        self.register_stop_handler(Box::new(StopLossPctHandler::new(5.0)));
        self.register_stop_handler(Box::new(ATRTrailStopHandler::new(14.0, 5.0)));
        self.register_stop_handler(Box::new(HILOTrailingStopHandler::new(14.0)));
        self.register_stop_handler(Box::new(PercentTrailingStopHandler::new(1.0)));

        // ATRTrailIndicatorStop требует indicator_name, создаем с дефолтным
        let mut atr_trail_ind_params = HashMap::new();
        atr_trail_ind_params.insert(
            "indicator_name".to_string(),
            crate::strategy::types::StrategyParamValue::Text("SMA".to_string()),
        );
        atr_trail_ind_params.insert(
            "period".to_string(),
            crate::strategy::types::StrategyParamValue::Number(14.0),
        );
        atr_trail_ind_params.insert(
            "coeff_atr".to_string(),
            crate::strategy::types::StrategyParamValue::Number(5.0),
        );
        atr_trail_ind_params.insert(
            "indicator_period".to_string(),
            crate::strategy::types::StrategyParamValue::Number(20.0),
        );
        if let Ok(handler) =
            StopHandlerFactory::create("ATRTrailIndicatorStop", &atr_trail_ind_params)
        {
            self.register_stop_handler(handler);
        }

        // PercentTrailIndicatorStop требует indicator_name
        let mut percent_trail_ind_params = HashMap::new();
        percent_trail_ind_params.insert(
            "indicator_name".to_string(),
            crate::strategy::types::StrategyParamValue::Text("SMA".to_string()),
        );
        percent_trail_ind_params.insert(
            "percentage".to_string(),
            crate::strategy::types::StrategyParamValue::Number(1.0),
        );
        percent_trail_ind_params.insert(
            "indicator_period".to_string(),
            crate::strategy::types::StrategyParamValue::Number(20.0),
        );
        if let Ok(handler) =
            StopHandlerFactory::create("PercentTrailIndicatorStop", &percent_trail_ind_params)
        {
            self.register_stop_handler(handler);
        }

        // Регистрируем Take Handlers
        self.register_take_handler(Box::new(TakeProfitPctHandler::new(10.0)));
    }

    /// Автоматически регистрирует стоп-обработчик, извлекая информацию из его ParameterSet
    fn register_stop_handler(&mut self, handler: Box<dyn StopHandler>) {
        let handler_name = handler.name().to_string();
        let handler_type = handler.handler_type().to_string();
        let optimization_ranges = handler.parameters().get_optimization_ranges();
        let ranges_count = optimization_ranges.len();

        for (param_name, range) in optimization_ranges {
            println!(
                "      [StopHandlerRegistry] Зарегистрирован {} (параметр {}: {:.1}-{:.1}, шаг: {:.2})",
                handler_name, param_name, range.start, range.end, range.step
            );
            self.handlers.push(HandlerInfo {
                handler_name: handler_name.clone(),
                stop_type: handler_type.clone(),
                parameter_name: param_name,
                optimization_range: range,
                priority: 100,
            });
        }

        if ranges_count == 0 {
            eprintln!(
                "      [StopHandlerRegistry] ПРЕДУПРЕЖДЕНИЕ: {} не имеет параметров для оптимизации",
                handler_name
            );
        }
    }

    /// Автоматически регистрирует тейк-обработчик, извлекая информацию из его ParameterSet
    fn register_take_handler(&mut self, handler: Box<dyn TakeHandler>) {
        let handler_name = handler.name().to_string();
        let handler_type = handler.handler_type().to_string();
        let optimization_ranges = handler.parameters().get_optimization_ranges();
        let ranges_count = optimization_ranges.len();

        for (param_name, range) in optimization_ranges {
            println!(
                "      [StopHandlerRegistry] Зарегистрирован {} (параметр {}: {:.1}-{:.1}, шаг: {:.2})",
                handler_name, param_name, range.start, range.end, range.step
            );
            self.handlers.push(HandlerInfo {
                handler_name: handler_name.clone(),
                stop_type: handler_type.clone(),
                parameter_name: param_name,
                optimization_range: range,
                priority: 100,
            });
        }

        if ranges_count == 0 {
            eprintln!(
                "      [StopHandlerRegistry] ПРЕДУПРЕЖДЕНИЕ: {} не имеет параметров для оптимизации",
                handler_name
            );
        }
    }

    pub fn get_all_configs(&self) -> Vec<StopHandlerConfig> {
        self.handlers
            .iter()
            .map(|handler| {
                let range = &handler.optimization_range;
                let mut parameter_values = Vec::new();
                let mut value = range.start as f64;
                let end = range.end as f64;
                let step = range.step as f64;
                while value <= end {
                    parameter_values.push(value);
                    value += step;
                }

                StopHandlerConfig {
                    handler_name: handler.handler_name.clone(),
                    stop_type: handler.stop_type.clone(),
                    parameter_values,
                    parameter_name: handler.parameter_name.clone(),
                    global_param_name: Some(format!(
                        "{}_{}",
                        handler.handler_name.to_lowercase(),
                        handler.parameter_name
                    )),
                    priority: handler.priority,
                }
            })
            .collect()
    }

    pub fn get_stop_loss_configs(&self) -> Vec<StopHandlerConfig> {
        self.get_all_configs()
            .into_iter()
            .filter(|config| config.stop_type == "stop_loss")
            .collect()
    }

    pub fn get_take_profit_configs(&self) -> Vec<StopHandlerConfig> {
        self.get_all_configs()
            .into_iter()
            .filter(|config| config.stop_type == "take_profit")
            .collect()
    }

    /// Получить диапазон оптимизации для параметра обработчика
    pub fn get_parameter_range(
        &self,
        handler_name: &str,
        param_name: &str,
    ) -> Option<ParameterRange> {
        self.handlers
            .iter()
            .find(|h| {
                h.handler_name.eq_ignore_ascii_case(handler_name)
                    && h.parameter_name.eq_ignore_ascii_case(param_name)
            })
            .map(|h| h.optimization_range.clone())
    }
}

pub static GLOBAL_REGISTRY: OnceLock<RwLock<StopHandlerRegistry>> = OnceLock::new();

pub fn get_global_registry() -> &'static RwLock<StopHandlerRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| RwLock::new(StopHandlerRegistry::new()))
}

/// Получить диапазон оптимизации для параметра стоп-обработчика (синхронная обертка)
pub fn get_stop_optimization_range(handler_name: &str, param_name: &str) -> Option<ParameterRange> {
    let registry = GLOBAL_REGISTRY.get_or_init(|| RwLock::new(StopHandlerRegistry::new()));
    // Используем обычное чтение для синхронного доступа (std::sync::RwLock)
    registry
        .read()
        .ok()?
        .get_parameter_range(handler_name, param_name)
}

/// Получить диапазон оптимизации для параметра тейк-обработчика (синхронная обертка)
pub fn get_take_optimization_range(handler_name: &str, param_name: &str) -> Option<ParameterRange> {
    get_stop_optimization_range(handler_name, param_name)
}
