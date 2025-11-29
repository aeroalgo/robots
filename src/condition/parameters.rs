/// Диапазон параметра условия
#[derive(Debug, Clone)]
pub struct ConditionParameterRange {
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl ConditionParameterRange {
    pub fn new(min: f32, max: f32, step: f32) -> Self {
        Self { min, max, step }
    }
}

/// Тип параметра условия
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionParameterType {
    Period,     // Период для трендовых условий (RisingTrend, FallingTrend)
    Percentage, // Процент для условий GreaterPercent, LowerPercent
}

/// Пресеты параметров условий
pub struct ConditionParameterPresets;

impl ConditionParameterPresets {
    /// Диапазон периода для трендовых условий (RisingTrend, FallingTrend)
    /// min: 2, max: 4, step: 1
    pub fn trend_period() -> ConditionParameterRange {
        ConditionParameterRange::new(2.0, 4.0, 1.0)
    }

    /// Диапазон процента для условий GreaterPercent, LowerPercent
    /// min: 0.5%, max: 10%, step: 0.5%
    pub fn percentage() -> ConditionParameterRange {
        ConditionParameterRange::new(0.5, 10.0, 0.5)
    }

    /// Получить диапазон для параметра по типу
    pub fn get_range(param_type: ConditionParameterType) -> ConditionParameterRange {
        match param_type {
            ConditionParameterType::Period => Self::trend_period(),
            ConditionParameterType::Percentage => Self::percentage(),
        }
    }

    /// Получить диапазон для параметра по имени условия
    pub fn get_range_for_condition(condition_name: &str) -> Option<ConditionParameterRange> {
        match condition_name.to_uppercase().as_str() {
            "RISINGTREND" | "FALLINGTREND" => Some(Self::trend_period()),
            "GREATERPERCENT" | "LOWERPERCENT" => Some(Self::percentage()),
            _ => None,
        }
    }

    /// Получить тип параметра по имени условия
    pub fn get_parameter_type(condition_name: &str) -> Option<ConditionParameterType> {
        match condition_name.to_uppercase().as_str() {
            "RISINGTREND" | "FALLINGTREND" => Some(ConditionParameterType::Period),
            "GREATERPERCENT" | "LOWERPERCENT" => Some(ConditionParameterType::Percentage),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_period_range() {
        let range = ConditionParameterPresets::trend_period();
        assert_eq!(range.min, 2.0);
        assert_eq!(range.max, 4.0);
        assert_eq!(range.step, 1.0);
    }

    #[test]
    fn test_percentage_range() {
        let range = ConditionParameterPresets::percentage();
        assert_eq!(range.min, 0.5);
        assert_eq!(range.max, 10.0);
        assert_eq!(range.step, 0.5);
    }

    #[test]
    fn test_get_range_for_condition() {
        assert!(ConditionParameterPresets::get_range_for_condition("RisingTrend").is_some());
        assert!(ConditionParameterPresets::get_range_for_condition("FallingTrend").is_some());
        assert!(ConditionParameterPresets::get_range_for_condition("GreaterPercent").is_some());
        assert!(ConditionParameterPresets::get_range_for_condition("LowerPercent").is_some());
        assert!(ConditionParameterPresets::get_range_for_condition("Above").is_none());
    }
}
