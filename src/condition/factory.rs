use crate::condition::{
    base::Condition,
    conditions::*,
    types::{ConditionConfig, ConditionError},
};
use std::collections::HashMap;

/// Фабрика для создания условий
pub struct ConditionFactory;

impl ConditionFactory {
    /// Создать условие по имени и параметрам
    pub fn create_condition(
        name: &str,
        parameters: HashMap<String, f32>,
    ) -> Result<Box<dyn Condition + Send + Sync>, ConditionError> {
        match name.to_uppercase().as_str() {
            // Условия сравнения
            "ABOVE" => Ok(Box::new(AboveCondition::new()?)),

            // Условия пересечения
            "CROSSESABOVE" => Ok(Box::new(CrossesAboveCondition::new()?)),

            // Трендовые условия
            "RISINGTREND" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(RisingTrendCondition::new(period)?))
            }
            "GREATERPERCENT" => Ok(Box::new(GreaterPercentCondition::new()?)),

            _ => Err(ConditionError::UnknownCondition(name.to_string())),
        }
    }

    /// Получить список всех доступных условий
    pub fn get_available_conditions() -> Vec<&'static str> {
        vec![
            // Условия сравнения
            "Above",
            // Условия пересечения
            "CrossesAbove",
            // Трендовые условия
            "RisingTrend",
            // Процентные условия
            "GreaterPercent",
        ]
    }

    /// Получить информацию об условии
    pub fn get_condition_info(name: &str) -> Option<ConditionConfig> {
        match name.to_uppercase().as_str() {
            "ABOVE" => Some(ConditionConfig {
                name: "Above".to_string(),
                description: "Проверяет, что первый вектор выше второго".to_string(),
                condition_type: crate::condition::types::ConditionType::Comparison,
                category: crate::condition::types::ConditionCategory::Filter,
                min_data_points: 2,
                is_reversible: true,
            }),
            "CROSSESABOVE" => Some(ConditionConfig {
                name: "CrossesAbove".to_string(),
                description: "Проверяет пересечение линии выше".to_string(),
                condition_type: crate::condition::types::ConditionType::Crossover,
                category: crate::condition::types::ConditionCategory::Entry,
                min_data_points: 2,
                is_reversible: false,
            }),
            "RISINGTREND" => Some(ConditionConfig {
                name: "RisingTrend".to_string(),
                description: "Проверяет растущий тренд".to_string(),
                condition_type: crate::condition::types::ConditionType::Trend,
                category: crate::condition::types::ConditionCategory::Filter,
                min_data_points: 20,
                is_reversible: true,
            }),
            "GREATERPERCENT" => Some(ConditionConfig {
                name: "GreaterPercent".to_string(),
                description: "Проверяет, что первый вектор выше второго на указанный процент"
                    .to_string(),
                condition_type: crate::condition::types::ConditionType::Percentage,
                category: crate::condition::types::ConditionCategory::Filter,
                min_data_points: 2,
                is_reversible: true,
            }),
            _ => None,
        }
    }

    /// Создать условие с параметрами по умолчанию
    pub fn create_condition_default(
        name: &str,
    ) -> Result<Box<dyn Condition + Send + Sync>, ConditionError> {
        let empty_params = HashMap::new();
        Self::create_condition(name, empty_params)
    }

    /// Создать условие из конфигурации
    pub fn create_from_config(
        config: &ConditionConfig,
    ) -> Result<Box<dyn Condition + Send + Sync>, ConditionError> {
        // Создаем условие с параметрами по умолчанию
        match config.name.to_uppercase().as_str() {
            "ABOVE" => Ok(Box::new(AboveCondition::new()?)),
            "CROSSESABOVE" => Ok(Box::new(CrossesAboveCondition::new()?)),
            "RISINGTREND" => Ok(Box::new(RisingTrendCondition::new(20.0)?)),
            "GREATERPERCENT" => Ok(Box::new(GreaterPercentCondition::new()?)),
            _ => Err(ConditionError::UnknownCondition(config.name.clone())),
        }
    }
}

/// Реестр условий
pub struct ConditionRegistry {
    conditions: HashMap<String, Box<dyn Condition + Send + Sync>>,
}

impl ConditionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            conditions: HashMap::new(),
        };
        registry.register_default_conditions();
        registry
    }

    /// Зарегистрировать условие
    pub fn register_condition(&mut self, name: &str, condition: Box<dyn Condition + Send + Sync>) {
        self.conditions.insert(name.to_string(), condition);
    }

    /// Получить условие по имени
    pub fn get_condition(&self, name: &str) -> Option<&Box<dyn Condition + Send + Sync>> {
        self.conditions.get(name)
    }

    /// Получить все зарегистрированные условия
    pub fn get_all_conditions(&self) -> Vec<&Box<dyn Condition + Send + Sync>> {
        self.conditions.values().collect()
    }

    /// Зарегистрировать стандартные условия
    fn register_default_conditions(&mut self) {
        // Регистрируем стандартные условия
        if let Ok(above) = AboveCondition::new() {
            self.register_condition("Above", Box::new(above));
        }

        if let Ok(crosses_above) = CrossesAboveCondition::new() {
            self.register_condition("CrossesAbove", Box::new(crosses_above));
        }

        if let Ok(rising_trend) = RisingTrendCondition::new(20.0) {
            self.register_condition("RisingTrend", Box::new(rising_trend));
        }

        if let Ok(greater_percent) = GreaterPercentCondition::new() {
            self.register_condition("GreaterPercent", Box::new(greater_percent));
        }
    }
}

/// Глобальный реестр условий
pub static GLOBAL_CONDITION_REGISTRY: std::sync::OnceLock<std::sync::RwLock<ConditionRegistry>> =
    std::sync::OnceLock::new();

pub fn get_global_condition_registry() -> &'static std::sync::RwLock<ConditionRegistry> {
    GLOBAL_CONDITION_REGISTRY.get_or_init(|| std::sync::RwLock::new(ConditionRegistry::new()))
}
