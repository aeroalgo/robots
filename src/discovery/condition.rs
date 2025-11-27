use crate::data_model::types::TimeFrame;
use crate::discovery::types::{ConditionInfo, ConditionParamInfo, IndicatorInfo};
use crate::strategy::types::{
    ConditionBindingSpec, ConditionInputSpec, ConditionOperator, DataSeriesSource, PriceField,
};
use std::collections::HashMap;

/// Генератор комбинаций условий
pub struct ConditionCombinationGenerator;

impl ConditionCombinationGenerator {
    /// Генерирует комбинации условий между индикаторами и ценой
    ///
    /// # Аргументы
    /// * `indicators` - список индикаторов для использования
    /// * `price_fields` - список полей цены (Open, High, Low, Close)
    /// * `operators` - список операторов для условий
    /// * `timeframes` - опциональный список таймфреймов для генерации мультитаймфреймовых условий
    ///
    /// # Возвращает
    /// Вектор комбинаций условий
    pub fn generate_indicator_price_conditions(
        indicators: &[IndicatorInfo],
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
        timeframes: Option<&[TimeFrame]>,
    ) -> Vec<ConditionInfo> {
        let mut conditions = Vec::new();

        for indicator in indicators {
            for price_field in price_fields {
                for operator in operators {
                    // Проверяем, применим ли оператор к комбинации индикатор-цена
                    if Self::is_valid_operator_for_indicator_price(operator) {
                        // Если таймфреймы указаны, генерируем комбинации с разными таймфреймами
                        if let Some(tfs) = timeframes {
                            for primary_tf in tfs {
                                for secondary_tf in tfs {
                                    let condition = ConditionInfo {
                                        id: format!(
                                            "ind_price_{}_{:?}_{:?}_tf{:?}_tf{:?}",
                                            indicator.alias, price_field, operator, primary_tf, secondary_tf
                                        ),
                                        name: format!(
                                            "{} ({:?}) {} {:?} ({:?})",
                                            indicator.name,
                                            primary_tf,
                                            Self::operator_to_str(operator),
                                            price_field,
                                            secondary_tf
                                        ),
                                        operator: operator.clone(),
                                        condition_type: "indicator_price".to_string(),
                                        optimization_params: Vec::new(),
                                        constant_value: None,
                                        primary_timeframe: Some(primary_tf.clone()),
                                        secondary_timeframe: Some(secondary_tf.clone()),
                                        price_field: None,
                                    };
                                    conditions.push(condition);
                                }
                            }
                        } else {
                            // Без таймфреймов - используем базовый таймфрейм стратегии
                            let condition = ConditionInfo {
                                id: format!(
                                    "ind_price_{}_{:?}_{:?}",
                                    indicator.alias, price_field, operator
                                ),
                                name: format!(
                                    "{} {} {:?}",
                                    indicator.name,
                                    Self::operator_to_str(operator),
                                    price_field
                                ),
                                operator: operator.clone(),
                                condition_type: "indicator_price".to_string(),
                                optimization_params: Vec::new(),
                                constant_value: None,
                                primary_timeframe: None,
                                secondary_timeframe: None,
                                price_field: Some(format!("{:?}", price_field)),
                            };
                            conditions.push(condition);
                        }
                    }
                }
            }
        }

        conditions
    }

    /// Генерирует комбинации условий между индикаторами
    ///
    /// # Аргументы
    /// * `indicators` - список индикаторов для использования
    /// * `operators` - список операторов для условий
    /// * `timeframes` - опциональный список таймфреймов для генерации мультитаймфреймовых условий
    ///
    /// # Возвращает
    /// Вектор комбинаций условий индикатор-индикатор
    pub fn generate_indicator_indicator_conditions(
        indicators: &[IndicatorInfo],
        operators: &[ConditionOperator],
        timeframes: Option<&[TimeFrame]>,
    ) -> Vec<ConditionInfo> {
        let mut conditions = Vec::new();

        // Генерируем пары индикаторов (без дубликатов и без одинаковых)
        for i in 0..indicators.len() {
            for j in (i + 1)..indicators.len() {
                let primary = &indicators[i];
                let secondary = &indicators[j];

                for operator in operators {
                    // Проверяем, применим ли оператор к комбинации индикатор-индикатор
                    if Self::is_valid_operator_for_indicator_indicator(operator) {
                        // Если таймфреймы указаны, генерируем комбинации с разными таймфреймами
                        if let Some(tfs) = timeframes {
                            for primary_tf in tfs {
                                for secondary_tf in tfs {
                                    let condition = ConditionInfo {
                                        id: format!(
                                            "ind_ind_{}_{}_{:?}_tf{:?}_tf{:?}",
                                            primary.alias, secondary.alias, operator, primary_tf, secondary_tf
                                        ),
                                        name: format!(
                                            "{} ({:?}) {} {} ({:?})",
                                            primary.name,
                                            primary_tf,
                                            Self::operator_to_str(operator),
                                            secondary.name,
                                            secondary_tf
                                        ),
                                        operator: operator.clone(),
                                        condition_type: "indicator_indicator".to_string(),
                                        optimization_params: Vec::new(),
                                        constant_value: None,
                                        primary_timeframe: Some(primary_tf.clone()),
                                        secondary_timeframe: Some(secondary_tf.clone()),
                                        price_field: None,
                                    };
                                    conditions.push(condition);
                                }
                            }
                        } else {
                            // Без таймфреймов - используем базовый таймфрейм стратегии
                            let condition = ConditionInfo {
                                id: format!(
                                    "ind_ind_{}_{}_{:?}",
                                    primary.alias, secondary.alias, operator
                                ),
                                name: format!(
                                    "{} {} {}",
                                    primary.name,
                                    Self::operator_to_str(operator),
                                    secondary.name
                                ),
                                operator: operator.clone(),
                                condition_type: "indicator_indicator".to_string(),
                                optimization_params: Vec::new(),
                                constant_value: None,
                                primary_timeframe: None,
                                secondary_timeframe: None,
                                price_field: None,
                            };
                            conditions.push(condition);
                        }
                    }
                }
            }
        }

        conditions
    }

    /// Генерирует комбинации условий между индикатором и константой
    /// Используется для осцилляторов (RSI > 70, RSI < 30, Stochastic > 80 и т.д.)
    ///
    /// # Аргументы
    /// * `indicators` - список индикаторов (только осцилляторы)
    /// * `operators` - список операторов (обычно GreaterThan и LessThan)
    /// * `constant_values` - значения констант для условий (например, [30, 50, 70] для RSI)
    /// * `timeframes` - опциональный список таймфреймов для генерации мультитаймфреймовых условий
    ///
    /// # Возвращает
    /// Вектор комбинаций условий индикатор-константа
    pub fn generate_indicator_constant_conditions(
        indicators: &[IndicatorInfo],
        operators: &[ConditionOperator],
        timeframes: Option<&[TimeFrame]>,
    ) -> Vec<ConditionInfo> {
        use crate::indicators::parameters::ParameterPresets;
        
        let mut conditions = Vec::new();

        // Фильтруем только осцилляторы
        let oscillators: Vec<&IndicatorInfo> = indicators
            .iter()
            .filter(|ind| ind.indicator_type == "oscillator")
            .collect();

        for oscillator in oscillators {
            for operator in operators {
                // Для условий индикатор-константа обычно используются только > и <
                if Self::is_valid_operator_for_indicator_constant(operator) {
                    // Получаем диапазон оптимизации для этого осциллятора
                    if let Some(range) = ParameterPresets::get_oscillator_threshold_range(&oscillator.name, "threshold") {
                        // Генерируем значения из диапазона с шагом
                        let mut constant = range.start;
                        while constant <= range.end {
                            // Если таймфреймы указаны, генерируем комбинации с разными таймфреймами
                            if let Some(tfs) = timeframes {
                                for primary_tf in tfs {
                                    let condition = ConditionInfo {
                                        id: format!(
                                            "ind_const_{}_{:?}_{}_tf{:?}",
                                            oscillator.alias, operator, constant, primary_tf
                                        ),
                                        name: format!(
                                            "{} ({:?}) {} {:.1}",
                                            oscillator.name,
                                            primary_tf,
                                            Self::operator_to_str(operator),
                                            constant
                                        ),
                                        operator: operator.clone(),
                                        condition_type: "indicator_constant".to_string(),
                                        optimization_params: vec![ConditionParamInfo {
                                            name: "threshold".to_string(),
                                            optimizable: true,
                                            global_param_name: None,
                                        }],
                                        constant_value: Some(constant as f64),
                                        primary_timeframe: Some(primary_tf.clone()),
                                        secondary_timeframe: None, // Константа не имеет таймфрейма
                                        price_field: None,
                                    };
                                    conditions.push(condition);
                                }
                            } else {
                                // Без таймфреймов - используем базовый таймфрейм стратегии
                                let condition = ConditionInfo {
                                    id: format!(
                                        "ind_const_{}_{:?}_{}",
                                        oscillator.alias, operator, constant
                                    ),
                                    name: format!(
                                        "{} {} {:.1}",
                                        oscillator.name,
                                        Self::operator_to_str(operator),
                                        constant
                                    ),
                                    operator: operator.clone(),
                                    condition_type: "indicator_constant".to_string(),
                                    optimization_params: vec![ConditionParamInfo {
                                        name: "threshold".to_string(),
                                        optimizable: true,
                                        global_param_name: None,
                                    }],
                                    constant_value: Some(constant as f64),
                                    primary_timeframe: None,
                                    secondary_timeframe: None,
                                    price_field: None,
                                };
                                conditions.push(condition);
                            }
                            
                            constant += range.step;
                        }
                    }
                }
            }
        }

        conditions
    }

    /// Генерирует все возможные комбинации условий для набора индикаторов
    pub fn generate_all_conditions(
        indicators: &[IndicatorInfo],
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
        allow_indicator_indicator: bool,
        timeframes: Option<&[TimeFrame]>,
    ) -> Vec<ConditionInfo> {
        Self::generate_all_conditions_with_constants(
            indicators,
            price_fields,
            operators,
            allow_indicator_indicator,
            timeframes,
        )
    }

    /// Генерирует все возможные комбинации условий для набора индикаторов с поддержкой констант
    ///
    /// # Аргументы
    /// * `indicators` - список индикаторов для использования
    /// * `price_fields` - список полей цены (Open, High, Low, Close)
    /// * `operators` - список операторов для условий
    /// * `allow_indicator_indicator` - разрешить ли условия индикатор-индикатор
    /// Пороги для осцилляторов автоматически берутся из get_oscillator_threshold_range()
    /// * `timeframes` - опциональный список таймфреймов для генерации мультитаймфреймовых условий
    ///
    /// # Возвращает
    /// Вектор всех возможных комбинаций условий
    pub fn generate_all_conditions_with_constants(
        indicators: &[IndicatorInfo],
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
        allow_indicator_indicator: bool,
        timeframes: Option<&[TimeFrame]>,
    ) -> Vec<ConditionInfo> {
        let mut all_conditions = Vec::new();

        // Условия индикатор-цена
        let indicator_price =
            Self::generate_indicator_price_conditions(indicators, price_fields, operators, timeframes);
        all_conditions.extend(indicator_price);

        // Условия индикатор-индикатор (если разрешено)
        if allow_indicator_indicator {
            let indicator_indicator =
                Self::generate_indicator_indicator_conditions(indicators, operators, timeframes);
            all_conditions.extend(indicator_indicator);
        }

        // Условия индикатор-константа для осцилляторов
        // Используем значения из get_oscillator_threshold_range
        let indicator_constant = Self::generate_indicator_constant_conditions(
            indicators,
            operators,
            timeframes,
        );
        all_conditions.extend(indicator_constant);

        all_conditions
    }

    /// Проверяет, применим ли оператор к комбинации индикатор-цена
    fn is_valid_operator_for_indicator_price(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::GreaterThan
                | ConditionOperator::LessThan
                | ConditionOperator::CrossesAbove
                | ConditionOperator::CrossesBelow
        )
    }

    /// Проверяет, применим ли оператор к комбинации индикатор-индикатор
    fn is_valid_operator_for_indicator_indicator(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::GreaterThan
                | ConditionOperator::LessThan
                | ConditionOperator::CrossesAbove
                | ConditionOperator::CrossesBelow
                | ConditionOperator::Between
        )
    }

    /// Проверяет, применим ли оператор к комбинации индикатор-константа
    /// Для осцилляторов обычно используются только > и <
    fn is_valid_operator_for_indicator_constant(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::GreaterThan | ConditionOperator::LessThan
        )
    }

    /// Преобразует оператор в строковое представление
    fn operator_to_str(operator: &ConditionOperator) -> &'static str {
        match operator {
            ConditionOperator::GreaterThan => ">",
            ConditionOperator::LessThan => "<",
            ConditionOperator::CrossesAbove => "Crosses Above",
            ConditionOperator::CrossesBelow => "Crosses Below",
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

