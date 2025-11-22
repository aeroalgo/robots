use crate::discovery::types::StopHandlerConfig;
use crate::indicators::implementations::OptimizationRange;
use crate::risk::stops;
use crate::risk::takes;
use std::sync::OnceLock;
use tokio::sync::RwLock;

pub struct StopHandlerRegistry {
    handlers: Vec<HandlerInfo>,
}

struct HandlerInfo {
    handler_name: String,
    stop_type: String,
    parameter_name: String,
    optimization_range: OptimizationRange,
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
        if let Some(range) = stops::get_optimization_range("StopLossPct", "percentage") {
            println!("      [StopHandlerRegistry] Зарегистрирован StopLossPct (диапазон: {:.1}-{:.1}, шаг: {:.2})", 
                     range.start, range.end, range.step);
            self.handlers.push(HandlerInfo {
                handler_name: "StopLossPct".to_string(),
                stop_type: "stop_loss".to_string(),
                parameter_name: "percentage".to_string(),
                optimization_range: range,
                priority: 100,
            });
        } else {
            eprintln!(
                "      [StopHandlerRegistry] ОШИБКА: Не удалось зарегистрировать StopLossPct"
            );
        }

        if let Some(range) = takes::get_optimization_range("TakeProfitPct", "percentage") {
            println!("      [StopHandlerRegistry] Зарегистрирован TakeProfitPct (диапазон: {:.1}-{:.1}, шаг: {:.2})", 
                     range.start, range.end, range.step);
            self.handlers.push(HandlerInfo {
                handler_name: "TakeProfitPct".to_string(),
                stop_type: "take_profit".to_string(),
                parameter_name: "percentage".to_string(),
                optimization_range: range,
                priority: 100,
            });
        } else {
            eprintln!(
                "      [StopHandlerRegistry] ОШИБКА: Не удалось зарегистрировать TakeProfitPct"
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
}

pub static GLOBAL_REGISTRY: OnceLock<RwLock<StopHandlerRegistry>> = OnceLock::new();

pub fn get_global_registry() -> &'static RwLock<StopHandlerRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| RwLock::new(StopHandlerRegistry::new()))
}
