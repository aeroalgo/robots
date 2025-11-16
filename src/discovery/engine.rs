use crate::data_model::types::TimeFrame;
use crate::discovery::config::{GlobalParamRange, StrategyDiscoveryConfig};
use crate::discovery::condition::ConditionCombinationGenerator;
use crate::discovery::indicator::IndicatorCombinationGenerator;
use crate::discovery::stop_handler::StopHandlerCombinationGenerator;
use crate::discovery::timeframe::TimeFrameGenerator;
use crate::discovery::strategy_converter::{StrategyConversionError, StrategyConverter};
use crate::discovery::types::{
    ConditionInfo, IndicatorInfo, IndicatorParamInfo, NestedIndicator, StopHandlerConfig,
    StopHandlerInfo,
};
use crate::strategy::types::StrategyDefinition;
use crate::strategy::types::{ConditionOperator, PriceField};

/// Основной генератор стратегий
pub struct StrategyDiscoveryEngine {
    config: StrategyDiscoveryConfig,
}

impl StrategyDiscoveryEngine {
    pub fn new(config: StrategyDiscoveryConfig) -> Self {
        Self { config }
    }

    /// Генерирует все возможные комбинации стратегий на основе конфигурации
    pub fn generate_strategies(
        &self,
        available_indicators: &[IndicatorInfo],
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
        stop_handler_configs: &[StopHandlerConfig],
    ) -> Vec<StrategyCandidate> {
        let mut candidates = Vec::new();

        // Генерируем комбинации таймфреймов
        let timeframe_combinations = TimeFrameGenerator::generate_combinations(
            self.config.base_timeframe.clone(),
            self.config.timeframe_count,
        );

        // Генерируем комбинации индикаторов (не учитываем стопы, т.к. они теперь отдельно)
        let indicator_combinations: Vec<(Vec<IndicatorInfo>, Vec<NestedIndicator>)> =
            if self.config.allow_indicator_on_indicator {
                // Генерируем комбинации с поддержкой вложенных индикаторов
                IndicatorCombinationGenerator::generate_with_indicator_inputs(
                    available_indicators,
                    self.config.max_optimization_params,
                    false, // НЕ включаем стопы, они генерируются отдельно
                    self.config.max_indicator_depth,
                )
                .into_iter()
                .map(|ic| (ic.base_indicators, ic.nested_indicators))
                .collect()
            } else {
                // Генерируем только базовые комбинации без вложенных индикаторов
                IndicatorCombinationGenerator::generate_combinations(
                    available_indicators,
                    self.config.max_optimization_params,
                    false, // НЕ включаем стопы, они генерируются отдельно
                )
                .into_iter()
                .map(|indicators| (indicators, Vec::new()))
                .collect()
            };

        // Генерируем комбинации условий (включая условия индикатор-константа для осцилляторов)
        let all_conditions = ConditionCombinationGenerator::generate_all_conditions_with_constants(
            available_indicators,
            price_fields,
            operators,
            self.config.allow_indicator_on_indicator, // разрешаем условия индикатор-индикатор, если разрешено строить индикаторы по индикаторам
            &self.config.oscillator_thresholds, // пороги для осцилляторов (например, [30, 50, 70] для RSI)
        );

        // Генерируем комбинации стопов и тейкпрофитов
        let stop_combinations = StopHandlerCombinationGenerator::generate_combinations_from_configs(
            stop_handler_configs,
        );

        // Комбинируем таймфреймы, индикаторы, условия и стопы
        for timeframes in timeframe_combinations {
            for (base_indicators, nested_indicators) in &indicator_combinations {
                // Собираем все доступные индикаторы (базовые + вложенные) для фильтрации условий
                let all_available_indicator_aliases: Vec<String> = base_indicators
                    .iter()
                    .map(|ind| ind.alias.clone())
                    .chain(nested_indicators.iter().map(|nested| nested.indicator.alias.clone()))
                    .collect();

                // Генерируем подмножества условий для каждой комбинации индикаторов
                // Используем только те условия, которые используют индикаторы из текущей комбинации
                let relevant_conditions: Vec<ConditionInfo> = all_conditions
                    .iter()
                    .filter(|cond| {
                        // Проверяем, используется ли индикатор из текущей комбинации в условии
                        match &cond.condition_type[..] {
                            "indicator_price" => {
                                // Условия индикатор-цена актуальны, если индикатор в комбинации (базовый или вложенный)
                                all_available_indicator_aliases
                                    .iter()
                                    .any(|alias| cond.id.contains(alias))
                            }
                            "indicator_indicator" => {
                                // Для условий индикатор-индикатор проверяем, что оба индикатора в комбинации
                                let matching_count = all_available_indicator_aliases
                                    .iter()
                                    .filter(|alias| cond.id.contains(*alias))
                                    .count();
                                matching_count >= 2
                            }
                            "indicator_constant" => {
                                // Условия индикатор-константа актуальны, если осциллятор в комбинации
                                // Проверяем базовые индикаторы
                                let base_oscillator_match = base_indicators.iter().any(|ind| {
                                    ind.indicator_type == "oscillator" && cond.id.contains(&ind.alias)
                                });
                                // Проверяем вложенные индикаторы
                                let nested_oscillator_match = nested_indicators.iter().any(|nested| {
                                    nested.indicator.indicator_type == "oscillator"
                                        && cond.id.contains(&nested.indicator.alias)
                                });
                                base_oscillator_match || nested_oscillator_match
                            }
                            _ => false,
                        }
                    })
                    .cloned()
                    .collect();

                // Генерируем комбинации условий с учетом оставшихся параметров (учитываем стопы)
                // Параметры базовых индикаторов
                let base_indicator_params: usize = base_indicators
                    .iter()
                    .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
                    .sum();

                // Параметры вложенных индикаторов
                let nested_indicator_params: usize = nested_indicators
                    .iter()
                    .map(|nested| {
                        nested
                            .indicator
                            .parameters
                            .iter()
                            .filter(|p| p.optimizable)
                            .count()
                    })
                    .sum();

                let indicator_params = base_indicator_params + nested_indicator_params;

                // Вычисляем максимальное количество параметров стопов из всех комбинаций
                let max_stop_params: usize = stop_combinations
                    .iter()
                    .map(|stops| {
                        stops
                            .iter()
                            .map(|stop| {
                                stop.optimization_params
                                    .iter()
                                    .filter(|p| p.optimizable)
                                    .count()
                            })
                            .sum::<usize>()
                    })
                    .max()
                    .unwrap_or(0);

                let remaining_params_for_conditions = self
                    .config
                    .max_optimization_params
                    .saturating_sub(indicator_params + max_stop_params);

                // Генерируем комбинации entry условий
                let entry_condition_combinations = Self::generate_condition_combinations_with_limit(
                    &relevant_conditions,
                    remaining_params_for_conditions,
                );

                // Генерируем комбинации exit условий (используем те же условия, что и для entry)
                // Можно также генерировать инвертированные условия или отдельный набор
                let exit_condition_combinations = Self::generate_condition_combinations_with_limit(
                    &relevant_conditions,
                    remaining_params_for_conditions,
                );

                // Комбинируем entry условия, exit условия, стопы
                // Генерируем стратегии с разными комбинациями:
                // 1. Только entry условия + стопы
                // 2. Entry условия + exit условия + стопы
                // 3. Entry условия + exit условия (без стопов)
                // 4. Entry условия + стопы (без exit условий) - уже есть выше
                for entry_conditions in &entry_condition_combinations {
                    for stop_handlers in &stop_combinations {
                        // Вариант 1: Entry условия + стопы (без exit условий)
                        let candidate = StrategyCandidate {
                            indicators: base_indicators.clone(),
                            nested_indicators: nested_indicators.clone(),
                            conditions: entry_conditions.clone(),
                            exit_conditions: vec![],
                            stop_handlers: stop_handlers.clone(),
                            timeframes: timeframes.clone(),
                            config: self.config.clone(),
                        };
                        if candidate.is_valid() {
                            candidates.push(candidate);
                        }

                        // Вариант 2: Entry условия + exit условия + стопы
                        for exit_conditions in &exit_condition_combinations {
                            let entry_params: usize = entry_conditions
                                .iter()
                                .map(|c| c.optimization_params.iter().filter(|p| p.optimizable).count())
                                .sum();
                            let exit_params: usize = exit_conditions
                                .iter()
                                .map(|c| c.optimization_params.iter().filter(|p| p.optimizable).count())
                                .sum();
                            let stop_params: usize = stop_handlers
                                .iter()
                                .map(|s| s.optimization_params.iter().filter(|p| p.optimizable).count())
                                .sum();
                            
                            if indicator_params + entry_params + exit_params + stop_params <= self.config.max_optimization_params {
                                let candidate = StrategyCandidate {
                                    indicators: base_indicators.clone(),
                                    nested_indicators: nested_indicators.clone(),
                                    conditions: entry_conditions.clone(),
                                    exit_conditions: exit_conditions.clone(),
                                    stop_handlers: stop_handlers.clone(),
                                    timeframes: timeframes.clone(),
                                    config: self.config.clone(),
                                };
                                if candidate.is_valid() {
                                    candidates.push(candidate);
                                }
                            }
                        }
                    }

                    // Вариант 3: Entry условия + exit условия (без стопов)
                    for exit_conditions in &exit_condition_combinations {
                        let entry_params: usize = entry_conditions
                            .iter()
                            .map(|c| c.optimization_params.iter().filter(|p| p.optimizable).count())
                            .sum();
                        let exit_params: usize = exit_conditions
                            .iter()
                            .map(|c| c.optimization_params.iter().filter(|p| p.optimizable).count())
                            .sum();
                        
                        if indicator_params + entry_params + exit_params <= self.config.max_optimization_params {
                            let candidate = StrategyCandidate {
                                indicators: base_indicators.clone(),
                                nested_indicators: nested_indicators.clone(),
                                conditions: entry_conditions.clone(),
                                exit_conditions: exit_conditions.clone(),
                                stop_handlers: vec![],
                                timeframes: timeframes.clone(),
                                config: self.config.clone(),
                            };
                            if candidate.is_valid() {
                                candidates.push(candidate);
                            }
                        }
                    }
                }
            }
        }

        candidates
    }

    /// Генерирует комбинации условий с учетом ограничения на количество параметров оптимизации
    fn generate_condition_combinations_with_limit(
        conditions: &[ConditionInfo],
        max_params: usize,
    ) -> Vec<Vec<ConditionInfo>> {
        let mut result = Vec::new();

        // Генерируем комбинации условий разной длины
        for combo_len in 0..=conditions.len().min(max_params) {
            let combinations = Self::combinations_of_length(conditions, combo_len);
            for combo in combinations {
                let condition_params: usize = combo
                    .iter()
                    .map(|cond| {
                        cond.optimization_params
                            .iter()
                            .filter(|p| p.optimizable)
                            .count()
                    })
                    .sum();

                if condition_params <= max_params {
                    result.push(combo);
                }
            }
        }

        result
    }

    /// Генерирует комбинации заданной длины
    fn combinations_of_length<T: Clone>(items: &[T], k: usize) -> Vec<Vec<T>> {
        if k == 0 {
            return vec![vec![]];
        }
        if k > items.len() {
            return vec![];
        }

        let mut result = Vec::new();
        for i in 0..=items.len() - k {
            let first = items[i].clone();
            let rest_combinations = Self::combinations_of_length(&items[i + 1..], k - 1);
            for mut combo in rest_combinations {
                combo.insert(0, first.clone());
                result.push(combo);
            }
        }
        result
    }

    /// Применяет глобальные настройки параметров к параметрам индикатора
    pub fn apply_global_param_ranges(
        &self,
        param: &IndicatorParamInfo,
    ) -> Option<GlobalParamRange> {
        if let Some(global_name) = &param.global_param_name {
            self.config.global_param_ranges.get(global_name).cloned()
        } else {
            None
        }
    }
}

/// Кандидат стратегии для дальнейшей оптимизации
#[derive(Debug, Clone)]
pub struct StrategyCandidate {
    /// Базовые индикаторы (строящиеся по цене)
    pub indicators: Vec<IndicatorInfo>,
    /// Вложенные индикаторы (строящиеся по другим индикаторам)
    pub nested_indicators: Vec<NestedIndicator>,
    /// Условия входа (entry conditions)
    pub conditions: Vec<ConditionInfo>,
    /// Условия выхода (exit conditions)
    pub exit_conditions: Vec<ConditionInfo>,
    pub stop_handlers: Vec<StopHandlerInfo>,
    pub timeframes: Vec<TimeFrame>,
    pub config: StrategyDiscoveryConfig,
}

impl StrategyCandidate {
    /// Вычисляет общее количество параметров оптимизации для этой стратегии
    pub fn total_optimization_params(&self) -> usize {
        // Параметры базовых индикаторов
        let base_indicator_params: usize = self
            .indicators
            .iter()
            .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
            .sum();

        // Параметры вложенных индикаторов
        let nested_indicator_params: usize = self
            .nested_indicators
            .iter()
            .map(|nested| {
                nested
                    .indicator
                    .parameters
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let indicator_params = base_indicator_params + nested_indicator_params;

        let entry_condition_params: usize = self
            .conditions
            .iter()
            .map(|cond| {
                cond.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let exit_condition_params: usize = self
            .exit_conditions
            .iter()
            .map(|cond| {
                cond.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let condition_params = entry_condition_params + exit_condition_params;

        // Параметры стоп-обработчиков (стоп-лосс и тейк-профит)
        let stop_params: usize = self
            .stop_handlers
            .iter()
            .map(|stop| {
                stop.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        indicator_params + condition_params + stop_params
    }

    /// Проверяет, соответствует ли кандидат ограничениям конфигурации
    pub fn is_valid(&self) -> bool {
        self.total_optimization_params() <= self.config.max_optimization_params
            && self.timeframes.len() <= self.config.timeframe_count
    }

    /// Возвращает все индикаторы кандидата (базовые + вложенные) для удобства работы
    pub fn all_indicators(&self) -> Vec<&IndicatorInfo> {
        let mut result: Vec<&IndicatorInfo> = self.indicators.iter().collect();
        result.extend(
            self.nested_indicators
                .iter()
                .map(|nested| &nested.indicator),
        );
        result
    }

    /// Возвращает все алиасы индикаторов кандидата (базовые + вложенные)
    pub fn all_indicator_aliases(&self) -> Vec<String> {
        let mut result: Vec<String> = self.indicators.iter().map(|ind| ind.alias.clone()).collect();
        result.extend(
            self.nested_indicators
                .iter()
                .map(|nested| nested.indicator.alias.clone()),
        );
        result
    }

    /// Получает информацию о вложенном индикаторе по его алиасу
    pub fn get_nested_indicator(&self, alias: &str) -> Option<&NestedIndicator> {
        self.nested_indicators
            .iter()
            .find(|nested| nested.indicator.alias == alias)
    }

    /// Получает алиас индикатора-источника для вложенного индикатора
    pub fn get_nested_indicator_source(&self, nested_alias: &str) -> Option<&str> {
        self.get_nested_indicator(nested_alias)
            .map(|nested| nested.input_indicator_alias.as_str())
    }

    /// Преобразует кандидата стратегии в StrategyDefinition для использования с StrategyBuilder
    pub fn to_strategy_definition(
        &self,
        base_timeframe: TimeFrame,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        StrategyConverter::candidate_to_definition(self, base_timeframe)
    }
}

