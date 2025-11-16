use crate::data_model::types::TimeFrame;
use crate::discovery::types::{ConditionParamInfo, StopHandlerConfig, StopHandlerInfo};
use crate::strategy::types::{PriceField, StrategyParameterMap};

/// Генератор комбинаций стопов и тейкпрофитов
pub struct StopHandlerCombinationGenerator;

impl StopHandlerCombinationGenerator {
    /// Генерирует все возможные комбинации стопов и тейкпрофитов из конфигураций
    ///
    /// # Аргументы
    /// * `configs` - конфигурации стоп-обработчиков (могут быть разные типы: процентные, фиксированные и т.д.)
    ///
    /// # Возвращает
    /// Вектор комбинаций стоп-обработчиков
    pub fn generate_combinations_from_configs(
        configs: &[StopHandlerConfig],
    ) -> Vec<Vec<StopHandlerInfo>> {
        let mut all_combinations = Vec::new();

        // Разделяем конфигурации по типам стопов
        let mut stop_losses: Vec<StopHandlerInfo> = Vec::new();
        let mut take_profits: Vec<StopHandlerInfo> = Vec::new();

        for config in configs {
            for (i, &param_value) in config.parameter_values.iter().enumerate() {
                let stop_info = StopHandlerInfo {
                    id: format!("{}_{}_{}", config.handler_name, config.stop_type, i),
                    name: format!("{} {:.2}", config.handler_name, param_value),
                    handler_name: config.handler_name.clone(),
                    stop_type: config.stop_type.clone(),
                    optimization_params: vec![ConditionParamInfo {
                        name: config.parameter_name.clone(),
                        optimizable: true,
                        global_param_name: config.global_param_name.clone(),
                    }],
                    priority: config.priority,
                };

                match config.stop_type.as_str() {
                    "stop_loss" => stop_losses.push(stop_info),
                    "take_profit" => take_profits.push(stop_info),
                    _ => {}
                }
            }
        }

        // Генерируем комбинации: каждый стоп-лосс с каждым тейк-профитом
        for stop_loss in &stop_losses {
            for take_profit in &take_profits {
                all_combinations.push(vec![stop_loss.clone(), take_profit.clone()]);
            }
        }

        // Если нет стоп-лоссов или тейк-профитов, добавляем только те, что есть
        if stop_losses.is_empty() && !take_profits.is_empty() {
            for take_profit in &take_profits {
                all_combinations.push(vec![take_profit.clone()]);
            }
        }
        if !stop_losses.is_empty() && take_profits.is_empty() {
            for stop_loss in &stop_losses {
                all_combinations.push(vec![stop_loss.clone()]);
            }
        }

        all_combinations
    }

    /// Генерирует все возможные комбинации стопов и тейкпрофитов (старый метод для обратной совместимости)
    ///
    /// # Аргументы
    /// * `stop_loss_percentages` - значения процентов для стоп-лосса (например, [0.1, 0.2, 0.3])
    /// * `take_profit_percentages` - значения процентов для тейк-профита (например, [0.5, 1.0, 1.5])
    /// * `allow_multiple_stops` - разрешить ли несколько стопов/тейкпрофитов
    ///
    /// # Возвращает
    /// Вектор комбинаций стоп-обработчиков
    pub fn generate_combinations(
        stop_loss_percentages: &[f64],
        take_profit_percentages: &[f64],
        _allow_multiple_stops: bool,
    ) -> Vec<Vec<StopHandlerInfo>> {
        let mut configs = Vec::new();

        // Создаем конфигурации для процентных стопов
        if !stop_loss_percentages.is_empty() {
            configs.push(StopHandlerConfig {
                handler_name: "StopLossPct".to_string(),
                stop_type: "stop_loss".to_string(),
                parameter_values: stop_loss_percentages.to_vec(),
                parameter_name: "percentage".to_string(),
                global_param_name: Some("pct".to_string()),
                priority: 100,
            });
        }

        if !take_profit_percentages.is_empty() {
            configs.push(StopHandlerConfig {
                handler_name: "TakeProfitPct".to_string(),
                stop_type: "take_profit".to_string(),
                parameter_values: take_profit_percentages.to_vec(),
                parameter_name: "percentage".to_string(),
                global_param_name: Some("pct".to_string()),
                priority: 90,
            });
        }

        Self::generate_combinations_from_configs(&configs)
    }

    /// Генерирует простые комбинации (по одному стоп-лоссу и тейк-профиту)
    pub fn generate_simple_combinations(
        stop_loss_percentages: &[f64],
        take_profit_percentages: &[f64],
    ) -> Vec<Vec<StopHandlerInfo>> {
        Self::generate_combinations(stop_loss_percentages, take_profit_percentages, false)
    }

    /// Создает StopHandlerSpec из StopHandlerInfo
    pub fn create_stop_handler_spec(
        stop: &StopHandlerInfo,
        timeframe: TimeFrame,
        price_field: PriceField,
        direction: crate::strategy::types::PositionDirection,
    ) -> crate::strategy::types::StopHandlerSpec {
        let mut parameters = StrategyParameterMap::new();
        // Значение процента будет установлено из optimization_params
        // Пока используем значение из id или устанавливаем по умолчанию
        let percentage = Self::extract_percentage_from_id(&stop.id).unwrap_or(0.2);
        parameters.insert(
            "percentage".to_string(),
            crate::strategy::types::StrategyParamValue::Number(percentage),
        );

        crate::strategy::types::StopHandlerSpec {
            id: stop.id.clone(),
            name: stop.name.clone(),
            handler_name: stop.handler_name.clone(),
            timeframe,
            price_field,
            parameters,
            direction,
            priority: stop.priority,
            tags: vec![],
            target_entry_ids: vec![],
        }
    }

    /// Извлекает значение процента из ID стопа
    fn extract_percentage_from_id(id: &str) -> Option<f64> {
        // ID формат: "stop_loss_0_0.2" или "take_profit_1_0.5"
        if let Some(last_underscore) = id.rfind('_') {
            if let Some(prev_underscore) = id[..last_underscore].rfind('_') {
                let percentage_str = &id[prev_underscore + 1..];
                percentage_str.parse::<f64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}
