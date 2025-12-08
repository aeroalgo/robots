use crate::data_model::types::TimeFrame;
use crate::discovery::types::{ConditionInfo, ConditionParamInfo};
use crate::strategy::types::{
    ConditionBindingSpec, ConditionInputSpec, ConditionOperator, DataSeriesSource,
};
use std::collections::HashMap;

/// Генератор комбинаций условий
///
/// Примечание: Методы генерации всех комбинаций удалены, так как основная генерация
/// кандидатов происходит через `candidate_builder.rs` с рандомным выбором и вероятностями.
/// Оставлены только вспомогательные методы, используемые в других модулях.
pub struct ConditionCombinationGenerator;

impl ConditionCombinationGenerator {
    /// Создает optimization_params для условия в зависимости от оператора
    pub fn create_optimization_params_for_operator(
        operator: &ConditionOperator,
    ) -> Vec<ConditionParamInfo> {
        match operator {
            ConditionOperator::LowerPercent | ConditionOperator::GreaterPercent => {
                vec![ConditionParamInfo {
                    name: "percent".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }]
            }
            ConditionOperator::RisingTrend | ConditionOperator::FallingTrend => {
                vec![ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }]
            }
            _ => Vec::new(),
        }
    }

    /// Проверяет, применим ли оператор к комбинации индикатор-цена
    fn is_valid_operator_for_indicator_price(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::Above
                | ConditionOperator::Below
                | ConditionOperator::GreaterPercent
                | ConditionOperator::LowerPercent
        )
    }

    /// Проверяет, применим ли оператор к комбинации индикатор-индикатор
    fn is_valid_operator_for_indicator_indicator(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::Above
                | ConditionOperator::Below
                | ConditionOperator::Between
                | ConditionOperator::GreaterPercent
                | ConditionOperator::LowerPercent
        )
    }

    /// Проверяет, применим ли оператор к комбинации индикатор-константа
    /// Для осцилляторов обычно используются только > и <
    fn is_valid_operator_for_indicator_constant(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::Above | ConditionOperator::Below
        )
    }

    /// Преобразует оператор в строковое представление
    fn operator_to_str(operator: &ConditionOperator) -> &'static str {
        match operator {
            ConditionOperator::Above => ">",
            ConditionOperator::Below => "<",
            ConditionOperator::RisingTrend => "RisingTrend",
            ConditionOperator::FallingTrend => "FallingTrend",
            ConditionOperator::GreaterPercent => ">%",
            ConditionOperator::LowerPercent => "<%",
            ConditionOperator::Between => "Between",
        }
    }

    /// Создает ConditionBindingSpec из ConditionInfo
    pub fn create_condition_binding(
        condition: &ConditionInfo,
        timeframe: TimeFrame,
        primary_source: DataSeriesSource,
        secondary_source: Option<DataSeriesSource>,
    ) -> ConditionBindingSpec {
        let input = match secondary_source {
            Some(secondary) => ConditionInputSpec::Dual {
                primary: primary_source,
                secondary,
            },
            None => ConditionInputSpec::Single {
                source: primary_source,
            },
        };

        ConditionBindingSpec {
            id: condition.id.clone(),
            name: condition.name.clone(),
            timeframe,
            declarative: crate::strategy::types::ConditionDeclarativeSpec::from_input(
                condition.operator.clone(),
                &input,
            ),
            parameters: HashMap::new(),
            input,
            weight: 1.0,
            tags: vec![],
            user_formula: None,
        }
    }
}
