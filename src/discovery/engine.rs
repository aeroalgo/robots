use crate::data_model::types::TimeFrame;
use crate::discovery::condition::ConditionCombinationGenerator;
use crate::discovery::config::StrategyDiscoveryConfig;
use crate::discovery::indicator::IndicatorCombinationGenerator;
use crate::discovery::stop_handler::StopHandlerCombinationGenerator;
use crate::discovery::strategy_converter::{StrategyConversionError, StrategyConverter};
use crate::discovery::timeframe::TimeFrameGenerator;
use crate::discovery::types::{
    ConditionInfo, ConditionParamInfo, IndicatorInfo, IndicatorParamInfo, NestedIndicator,
    StopHandlerConfig, StopHandlerInfo,
};
use crate::strategy::types::StrategyDefinition;
use crate::strategy::types::{ConditionOperator, PriceField};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Состояние генерации для возобновления работы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationState {
    /// Уже обработанные комбинации (хеш для быстрой проверки)
    pub processed_combinations: HashSet<String>,
    /// Семя для рандомизации
    pub random_seed: u64,
    /// Общее количество сгенерированных стратегий
    pub total_generated: usize,
}

impl GenerationState {
    pub fn new(random_seed: u64) -> Self {
        Self {
            processed_combinations: HashSet::new(),
            random_seed,
            total_generated: 0,
        }
    }

    pub fn combination_hash(
        timeframes: &[TimeFrame],
        indicators: &[IndicatorInfo],
        nested: &[NestedIndicator],
    ) -> String {
        let mut parts = Vec::new();
        parts.push(format!("tf:{}", timeframes.len()));
        for tf in timeframes {
            parts.push(format!("{:?}", tf));
        }
        parts.push(format!("ind:{}", indicators.len()));
        for ind in indicators {
            parts.push(ind.alias.clone());
        }
        parts.push(format!("nested:{}", nested.len()));
        for n in nested {
            parts.push(format!("{}:{}", n.indicator.alias, n.input_indicator_alias));
        }
        parts.join("|")
    }
}

/// Основной генератор стратегий
pub struct StrategyDiscoveryEngine {
    config: StrategyDiscoveryConfig,
    state: Option<GenerationState>,
}

impl StrategyDiscoveryEngine {
    pub fn new(config: StrategyDiscoveryConfig) -> Self {
        Self {
            config,
            state: None,
        }
    }

    pub fn with_state(config: StrategyDiscoveryConfig, state: GenerationState) -> Self {
        Self {
            config,
            state: Some(state),
        }
    }

    pub fn get_state(&self) -> Option<&GenerationState> {
        self.state.as_ref()
    }

    pub fn take_state(self) -> Option<GenerationState> {
        self.state
    }

    /// Генерирует стратегии рандомно с возможностью возобновления
    /// Возвращает итератор, который генерирует стратегии по требованию
    pub fn generate_strategies_random(
        &mut self,
        available_indicators: &[IndicatorInfo],
        price_fields: &[PriceField],
        operators: &[ConditionOperator],
        stop_handler_configs: &[StopHandlerConfig],
        available_timeframes: Option<&[TimeFrame]>,
    ) -> StrategyIterator {
        println!(
            "      [generate_strategies_random] Начало случайной генерации стратегий на лету..."
        );
        let state = self
            .state
            .get_or_insert_with(|| GenerationState::new(rand::thread_rng().gen()));

        use rand::SeedableRng;
        let rng = rand::rngs::StdRng::seed_from_u64(state.random_seed);

        println!("      [generate_strategies_random] Подготовка доступных таймфреймов...");
        let available_timeframes = if let Some(timeframes) = available_timeframes {
            let tf_vec: Vec<TimeFrame> = timeframes.iter().cloned().collect();
            println!(
                "      [generate_strategies_random] Используются переданные таймфреймы: {}",
                tf_vec.len()
            );
            tf_vec
        } else {
            let base_minutes = match &self.config.base_timeframe {
                TimeFrame::Minutes(m) => *m,
                _ => 60,
            };
            let mut timeframes = vec![self.config.base_timeframe.clone()];
            let mut multiplier = 2;
            loop {
                let minutes = base_minutes * multiplier;
                if minutes > self.config.max_timeframe_minutes {
                    break;
                }
                timeframes.push(TimeFrame::Minutes(minutes));
                multiplier += 1;
            }
            println!(
                "      [generate_strategies_random] Сгенерировано таймфреймов: {}",
                timeframes.len()
            );
            timeframes
        };

        println!("      [generate_strategies_random] Извлечение стоп-обработчиков...");
        let (stop_losses, take_profits) = Self::extract_stop_handlers(stop_handler_configs);
        println!(
            "      [generate_strategies_random] Стоп-лоссов: {}, Тейк-профитов: {}",
            stop_losses.len(),
            take_profits.len()
        );

        println!("      [generate_strategies_random] Генерация стратегий будет происходить случайным образом на лету...");

        let state_clone = state.clone();
        let state_arc = Arc::new(std::sync::Mutex::new(state_clone.clone()));
        self.state = Some(state_clone);

        StrategyIterator {
            state: state_arc,
            available_indicators: available_indicators.to_vec(),
            price_fields: price_fields.to_vec(),
            operators: operators.to_vec(),
            available_timeframes,
            stop_losses,
            take_profits,
            config: self.config.clone(),
            rng,
            max_attempts: 10000,
            attempts: 0,
        }
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
            self.config.max_timeframe_minutes,
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

        // Собираем все уникальные таймфреймы из всех комбинаций для генерации мультитаймфреймовых условий
        let all_timeframes: Vec<TimeFrame> = timeframe_combinations
            .iter()
            .flat_map(|tfs| tfs.iter())
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let timeframes_for_conditions = if all_timeframes.is_empty() {
            None
        } else {
            Some(all_timeframes.as_slice())
        };

        // Генерируем комбинации условий (включая условия индикатор-константа для осцилляторов)
        let all_conditions = ConditionCombinationGenerator::generate_all_conditions_with_constants(
            available_indicators,
            price_fields,
            operators,
            self.config.allow_indicator_on_indicator, // разрешаем условия индикатор-индикатор, если разрешено строить индикаторы по индикаторам
            timeframes_for_conditions, // передаем таймфреймы для генерации мультитаймфреймовых условий
        );

        // Генерируем комбинации стопов и тейкпрофитов
        let stop_combinations = StopHandlerCombinationGenerator::generate_combinations_from_configs(
            stop_handler_configs,
        );

        // Комбинируем таймфреймы, индикаторы, условия и стопы
        for timeframes in timeframe_combinations {
            for (base_indicators, nested_indicators) in &indicator_combinations {
                // Собираем все алиасы индикаторов из текущей комбинации (базовые + вложенные)
                let available_aliases: std::collections::HashSet<String> = {
                    let mut aliases = base_indicators
                        .iter()
                        .map(|ind| ind.alias.clone())
                        .collect::<std::collections::HashSet<String>>();
                    aliases.extend(
                        nested_indicators
                            .iter()
                            .map(|nested| nested.indicator.alias.clone()),
                    );
                    aliases
                };

                // Фильтруем условия: оставляем только те, которые используют индикаторы из текущей комбинации
                let filtered_conditions: Vec<ConditionInfo> = all_conditions
                    .iter()
                    .filter(|condition| {
                        Self::condition_uses_available_indicators(condition, &available_aliases)
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

                // Генерируем комбинации entry условий из отфильтрованных условий
                let entry_condition_combinations = Self::generate_condition_combinations_with_limit(
                    &filtered_conditions,
                    remaining_params_for_conditions,
                );

                // Генерируем комбинации exit условий (используем те же отфильтрованные условия)
                let exit_condition_combinations = Self::generate_condition_combinations_with_limit(
                    &filtered_conditions,
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
                        let (stop_handlers_split, take_handlers_split) =
                            StrategyCandidate::split_handlers(stop_handlers);

                        let candidate = StrategyCandidate {
                            indicators: base_indicators.clone(),
                            nested_indicators: nested_indicators.clone(),
                            conditions: entry_conditions.clone(),
                            exit_conditions: vec![],
                            stop_handlers: stop_handlers_split,
                            take_handlers: take_handlers_split,
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
                                .map(|c| {
                                    c.optimization_params
                                        .iter()
                                        .filter(|p| p.optimizable)
                                        .count()
                                })
                                .sum();
                            let exit_params: usize = exit_conditions
                                .iter()
                                .map(|c| {
                                    c.optimization_params
                                        .iter()
                                        .filter(|p| p.optimizable)
                                        .count()
                                })
                                .sum();
                            let stop_params: usize = stop_handlers
                                .iter()
                                .map(|s| {
                                    s.optimization_params
                                        .iter()
                                        .filter(|p| p.optimizable)
                                        .count()
                                })
                                .sum();

                            let (stop_handlers_split, take_handlers_split) =
                                StrategyCandidate::split_handlers(stop_handlers);
                            let stop_params_split: usize = stop_handlers_split
                                .iter()
                                .map(|s| {
                                    s.optimization_params
                                        .iter()
                                        .filter(|p| p.optimizable)
                                        .count()
                                })
                                .sum();
                            let take_params_split: usize = take_handlers_split
                                .iter()
                                .map(|s| {
                                    s.optimization_params
                                        .iter()
                                        .filter(|p| p.optimizable)
                                        .count()
                                })
                                .sum();

                            if indicator_params
                                + entry_params
                                + exit_params
                                + stop_params_split
                                + take_params_split
                                <= self.config.max_optimization_params
                            {
                                let candidate = StrategyCandidate {
                                    indicators: base_indicators.clone(),
                                    nested_indicators: nested_indicators.clone(),
                                    conditions: entry_conditions.clone(),
                                    exit_conditions: exit_conditions.clone(),
                                    stop_handlers: stop_handlers_split,
                                    take_handlers: take_handlers_split,
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
                            .map(|c| {
                                c.optimization_params
                                    .iter()
                                    .filter(|p| p.optimizable)
                                    .count()
                            })
                            .sum();
                        let exit_params: usize = exit_conditions
                            .iter()
                            .map(|c| {
                                c.optimization_params
                                    .iter()
                                    .filter(|p| p.optimizable)
                                    .count()
                            })
                            .sum();

                        if indicator_params + entry_params + exit_params
                            <= self.config.max_optimization_params
                        {
                            let candidate = StrategyCandidate {
                                indicators: base_indicators.clone(),
                                nested_indicators: nested_indicators.clone(),
                                conditions: entry_conditions.clone(),
                                exit_conditions: exit_conditions.clone(),
                                stop_handlers: vec![],
                                take_handlers: vec![],
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

    /// Проверяет, использует ли условие только доступные индикаторы
    fn condition_uses_available_indicators(
        condition: &ConditionInfo,
        available_aliases: &std::collections::HashSet<String>,
    ) -> bool {
        match condition.condition_type.as_str() {
            "indicator_price" => {
                // Для условий индикатор-цена проверяем, что индикатор есть в доступных
                if let Some(indicator_alias) =
                    Self::extract_indicator_alias_from_condition_id(&condition.id)
                {
                    available_aliases.contains(&indicator_alias)
                } else {
                    false
                }
            }
            "indicator_indicator" => {
                // Для условий индикатор-индикатор проверяем, что оба индикатора есть в доступных
                if let Some(aliases) =
                    Self::extract_indicator_aliases_from_condition_id(&condition.id)
                {
                    aliases.len() >= 2
                        && available_aliases.contains(&aliases[0])
                        && available_aliases.contains(&aliases[1])
                } else {
                    false
                }
            }
            "indicator_constant" => {
                // Для условий индикатор-константа проверяем, что индикатор есть в доступных
                if let Some(indicator_alias) =
                    Self::extract_indicator_alias_from_condition_id(&condition.id)
                {
                    available_aliases.contains(&indicator_alias)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Извлекает алиас индикатора из ID условия типа "indicator_price" или "indicator_constant"
    fn extract_indicator_alias_from_condition_id(condition_id: &str) -> Option<String> {
        if condition_id.starts_with("ind_price_") {
            // Формат: ind_price_{alias}_{price_field}_{operator} или ind_price_{alias}_{price_field}_{operator}_tf...
            let rest = condition_id.strip_prefix("ind_price_")?;
            // Берем первую часть до следующего подчеркивания (это алиас индикатора)
            // Но нужно учесть, что алиас может содержать подчеркивания, поэтому берем до первого вхождения оператора
            // Упрощенный вариант: берем все до первого вхождения известных операторов или таймфрейма
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                // Берем первую часть (алиас) - это все до первого PriceField или Operator
                let parts: Vec<&str> = before_tf.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            } else {
                // Нет таймфрейма, берем первую часть
                let parts: Vec<&str> = rest.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            }
        } else if condition_id.starts_with("ind_const_") {
            // Формат: ind_const_{alias}_{operator}_{constant} или ind_const_{alias}_{operator}_{constant}_tf...
            let rest = condition_id.strip_prefix("ind_const_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string());
                }
            }
        }
        None
    }

    /// Извлекает алиасы индикаторов из ID условия типа "indicator_indicator"
    fn extract_indicator_aliases_from_condition_id(condition_id: &str) -> Option<Vec<String>> {
        if condition_id.starts_with("ind_ind_") {
            // Формат: ind_ind_{alias1}_{alias2}_{operator} или ind_ind_{alias1}_{alias2}_{operator}_tf...
            let rest = condition_id.strip_prefix("ind_ind_")?;
            if let Some(tf_pos) = rest.find("_tf") {
                let before_tf = &rest[..tf_pos];
                let parts: Vec<&str> = before_tf.split('_').collect();
                if parts.len() >= 2 {
                    return Some(vec![parts[0].to_string(), parts[1].to_string()]);
                }
            } else {
                let parts: Vec<&str> = rest.split('_').collect();
                if parts.len() >= 2 {
                    return Some(vec![parts[0].to_string(), parts[1].to_string()]);
                }
            }
        }
        None
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

    /// Извлекает стоп-лоссы и тейк-профиты отдельно из конфигураций
    fn extract_stop_handlers(
        configs: &[StopHandlerConfig],
    ) -> (Vec<StopHandlerInfo>, Vec<StopHandlerInfo>) {
        let mut stop_losses = Vec::new();
        let mut take_profits = Vec::new();

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

        (stop_losses, take_profits)
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
    pub take_handlers: Vec<StopHandlerInfo>,
    pub timeframes: Vec<TimeFrame>,
    pub config: StrategyDiscoveryConfig,
}

impl StrategyCandidate {
    /// Разделяет список обработчиков на stop_handlers и take_handlers
    pub fn split_handlers(
        handlers: &[StopHandlerInfo],
    ) -> (Vec<StopHandlerInfo>, Vec<StopHandlerInfo>) {
        handlers
            .iter()
            .cloned()
            .partition(|h| h.stop_type == "stop_loss")
    }

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

        // Параметры стоп-обработчиков (стоп-лосс)
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

        // Параметры тейк-обработчиков (тейк-профит)
        let take_params: usize = self
            .take_handlers
            .iter()
            .map(|take| {
                take.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        indicator_params + condition_params + stop_params + take_params
    }

    /// Проверяет, соответствует ли кандидат ограничениям конфигурации
    pub fn is_valid(&self) -> bool {
        let has_exit_conditions = !self.exit_conditions.is_empty();
        let has_stop_handlers = !self.stop_handlers.is_empty();
        let has_take_handlers = !self.take_handlers.is_empty();
        let has_any_exit = has_exit_conditions || has_stop_handlers || has_take_handlers;
        let only_take = !has_exit_conditions && !has_stop_handlers && has_take_handlers;

        self.total_optimization_params() <= self.config.max_optimization_params
            && self.timeframes.len() <= self.config.timeframe_count
            && has_any_exit
            && !only_take
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
        let mut result: Vec<String> = self
            .indicators
            .iter()
            .map(|ind| ind.alias.clone())
            .collect();
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

/// Итератор для рандомной генерации стратегий
/// Генерирует стратегии случайным образом на лету, без предгенерации всех комбинаций
pub struct StrategyIterator {
    state: Arc<std::sync::Mutex<GenerationState>>,
    available_indicators: Vec<IndicatorInfo>,
    price_fields: Vec<PriceField>,
    operators: Vec<ConditionOperator>,
    available_timeframes: Vec<TimeFrame>,
    stop_losses: Vec<StopHandlerInfo>,
    take_profits: Vec<StopHandlerInfo>,
    config: StrategyDiscoveryConfig,
    rng: rand::rngs::StdRng,
    max_attempts: usize,
    attempts: usize,
}

impl Iterator for StrategyIterator {
    type Item = StrategyCandidate;

    fn next(&mut self) -> Option<Self::Item> {
        // Генерируем стратегии случайным образом на лету
        // Повторяем до тех пор, пока не сгенерируется валидная стратегия
        loop {
            self.attempts += 1;

            if self.attempts > self.max_attempts {
                return None;
            }

            if let Some(candidate) = self.generate_random_strategy() {
                if let Ok(mut state) = self.state.lock() {
                    state.total_generated += 1;
                }
                self.attempts = 0; // Сбрасываем счетчик при успехе
                return Some(candidate);
            }

            // Если не удалось сгенерировать валидную стратегию, повторяем
        }
    }
}

impl StrategyIterator {
    /// Получает текущее состояние генерации
    pub fn get_state(&self) -> Option<GenerationState> {
        self.state.lock().ok().map(|s| s.clone())
    }

    /// Генерирует одну случайную стратегию на лету поэтапно
    fn generate_random_strategy(&mut self) -> Option<StrategyCandidate> {
        use rand::Rng;

        // ЭТАП 1: БАЗОВАЯ СТРАТЕГИЯ (обязательно)
        // 1.1. Базовый таймфрейм
        let mut timeframes = vec![self.config.base_timeframe.clone()];

        // 1.2. Один индикатор
        if self.available_indicators.is_empty() {
            return None;
        }
        let first_indicator = self.available_indicators.choose(&mut self.rng).cloned()?;
        let mut selected_indicators = vec![first_indicator.clone()];
        let mut nested_indicators = Vec::new();

        // 1.3. Одно условие для этого индикатора
        let base_conditions =
            self.generate_conditions_for_indicators(&selected_indicators, &timeframes)?;
        if base_conditions.is_empty() {
            return None;
        }
        let first_condition = base_conditions.choose(&mut self.rng).cloned()?;
        let mut entry_conditions = vec![first_condition];
        let mut exit_conditions = Vec::new();

        // 1.4. Один стоп обработчик (обязательно)
        if self.stop_losses.is_empty() {
            return None;
        }
        let stop_handler = self.stop_losses.choose(&mut self.rng).cloned()?;
        let mut stop_handlers = vec![stop_handler];
        let mut take_handlers = Vec::new();

        // ЭТАП 2+: ПОЭТАПНОЕ ДОБАВЛЕНИЕ ЭЛЕМЕНТОВ
        loop {
            let current_params = self.calculate_total_params(
                &selected_indicators,
                &nested_indicators,
                &entry_conditions,
                &exit_conditions,
                &stop_handlers,
                &take_handlers,
            );

            if current_params >= self.config.max_optimization_params {
                break;
            }

            // Вероятность продолжить добавление элементов
            if !self.rng.gen_bool(0.6) {
                break;
            }

            let mut added_something = false;

            // 2.1. Дополнительный таймфрейм (вероятность 0.4)
            if timeframes.len() < self.config.timeframe_count
                && timeframes.len() < self.available_timeframes.len()
                && self.rng.gen_bool(0.4)
            {
                let available_tfs: Vec<_> = self
                    .available_timeframes
                    .iter()
                    .filter(|tf| !timeframes.contains(tf))
                    .collect();
                if let Some(new_tf) = available_tfs.choose(&mut self.rng) {
                    timeframes.push((*new_tf).clone());
                    added_something = true;
                }
            }

            // 2.2. Индикатор (вероятность 0.5)
            if self.rng.gen_bool(0.5) {
                let available: Vec<_> = self
                    .available_indicators
                    .iter()
                    .filter(|ind| !selected_indicators.iter().any(|si| si.alias == ind.alias))
                    .collect();
                if let Some(new_indicator) = available.choose(&mut self.rng) {
                    selected_indicators.push((*new_indicator).clone());
                    added_something = true;

                    // ОБЯЗАТЕЛЬНО: добавляем условие для нового индикатора
                    let new_conditions =
                        self.generate_conditions_for_indicators(&selected_indicators, &timeframes)?;

                    // Если это осциллятор, предпочитаем условия с константами
                    let is_oscillator = new_indicator.indicator_type == "oscillator";
                    let preferred_condition = if is_oscillator {
                        new_conditions
                            .iter()
                            .find(|c| {
                                c.condition_type == "indicator_constant"
                                    && self.condition_uses_indicator(c, &new_indicator.alias)
                            })
                            .or_else(|| {
                                new_conditions.iter().find(|c| {
                                    self.condition_uses_indicator(c, &new_indicator.alias)
                                })
                            })
                    } else {
                        new_conditions
                            .iter()
                            .find(|c| self.condition_uses_indicator(c, &new_indicator.alias))
                    };

                    if let Some(new_condition) =
                        preferred_condition.or_else(|| new_conditions.choose(&mut self.rng))
                    {
                        entry_conditions.push(new_condition.clone());
                    }
                }
            }

            // 2.3. Exit rule (вероятность 0.4)
            if self.rng.gen_bool(0.4) {
                let available_exit =
                    self.generate_conditions_for_indicators(&selected_indicators, &timeframes)?;
                if let Some(exit_condition) = available_exit.choose(&mut self.rng) {
                    exit_conditions.push(exit_condition.clone());
                    added_something = true;
                }
            }

            // 2.4. Take profit (вероятность 0.3)
            if !self.take_profits.is_empty() && self.rng.gen_bool(0.3) {
                if let Some(take) = self.take_profits.choose(&mut self.rng) {
                    take_handlers.push(take.clone());
                    added_something = true;
                }
            }

            // 2.5. Индикатор по индикатору (вероятность 0.3, если разрешено)
            if self.config.allow_indicator_on_indicator
                && self.rng.gen_bool(0.3)
                && !selected_indicators.is_empty()
            {
                // Выбираем базовый индикатор для построения нового
                let base_indicator = selected_indicators.choose(&mut self.rng)?;
                if base_indicator.can_use_indicator_input {
                    // Ищем индикаторы, которые могут строиться по индикаторам
                    let available_nested: Vec<_> = self
                        .available_indicators
                        .iter()
                        .filter(|ind| ind.can_use_indicator_input)
                        .filter(|ind| !selected_indicators.iter().any(|si| si.alias == ind.alias))
                        .collect();
                    if let Some(nested_ind) = available_nested.choose(&mut self.rng) {
                        nested_indicators.push(NestedIndicator {
                            indicator: (*nested_ind).clone(),
                            input_indicator_alias: base_indicator.alias.clone(),
                            depth: 1,
                        });
                        added_something = true;

                        // ОБЯЗАТЕЛЬНО: добавляем условие для вложенного индикатора
                        let all_indicators: Vec<_> = selected_indicators
                            .iter()
                            .chain(nested_indicators.iter().map(|n| &n.indicator))
                            .collect();
                        let nested_conditions = self.generate_conditions_for_indicators(
                            &all_indicators
                                .iter()
                                .map(|i| (*i).clone())
                                .collect::<Vec<_>>(),
                            &timeframes,
                        )?;

                        // Если это осциллятор, предпочитаем условия с константами
                        let is_oscillator = nested_ind.indicator_type == "oscillator";
                        let preferred_condition = if is_oscillator {
                            nested_conditions
                                .iter()
                                .find(|c| {
                                    c.condition_type == "indicator_constant"
                                        && self.condition_uses_indicator(c, &nested_ind.alias)
                                })
                                .or_else(|| {
                                    nested_conditions.iter().find(|c| {
                                        self.condition_uses_indicator(c, &nested_ind.alias)
                                    })
                                })
                        } else {
                            nested_conditions
                                .iter()
                                .find(|c| self.condition_uses_indicator(c, &nested_ind.alias))
                        };

                        if let Some(nested_condition) =
                            preferred_condition.or_else(|| nested_conditions.choose(&mut self.rng))
                        {
                            entry_conditions.push(nested_condition.clone());
                        }
                    }
                }
            }

            // Если ничего не добавили, выходим
            if !added_something {
                break;
            }
        }

        // Создаем кандидата стратегии
        let candidate = StrategyCandidate {
            indicators: selected_indicators,
            nested_indicators,
            conditions: entry_conditions,
            exit_conditions,
            stop_handlers,
            take_handlers,
            timeframes,
            config: self.config.clone(),
        };

        // Проверяем валидность и ограничения
        if candidate.is_valid()
            && candidate.total_optimization_params() <= self.config.max_optimization_params
        {
            Some(candidate)
        } else {
            None
        }
    }

    /// Генерирует условия для указанных индикаторов
    fn generate_conditions_for_indicators(
        &self,
        indicators: &[IndicatorInfo],
        timeframes: &[TimeFrame],
    ) -> Option<Vec<ConditionInfo>> {
        let timeframes_slice = if timeframes.is_empty() {
            None
        } else {
            Some(timeframes)
        };
        let conditions = ConditionCombinationGenerator::generate_all_conditions_with_constants(
            indicators,
            &self.price_fields,
            &self.operators,
            self.config.allow_indicator_on_indicator,
            timeframes_slice,
        );
        if conditions.is_empty() {
            None
        } else {
            Some(conditions)
        }
    }

    /// Проверяет, использует ли условие указанный индикатор
    fn condition_uses_indicator(&self, condition: &ConditionInfo, indicator_alias: &str) -> bool {
        if let Some(alias) =
            StrategyDiscoveryEngine::extract_indicator_alias_from_condition_id(&condition.id)
        {
            alias == indicator_alias
        } else {
            false
        }
    }

    /// Вычисляет общее количество параметров оптимизации
    fn calculate_total_params(
        &self,
        indicators: &[IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        entry_conditions: &[ConditionInfo],
        exit_conditions: &[ConditionInfo],
        stop_handlers: &[StopHandlerInfo],
        take_handlers: &[StopHandlerInfo],
    ) -> usize {
        let indicator_params: usize = indicators
            .iter()
            .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
            .sum();

        let nested_params: usize = nested_indicators
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

        let entry_params: usize = entry_conditions
            .iter()
            .map(|c| {
                c.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let exit_params: usize = exit_conditions
            .iter()
            .map(|c| {
                c.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let stop_params: usize = stop_handlers
            .iter()
            .map(|s| {
                s.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        let take_params: usize = take_handlers
            .iter()
            .map(|s| {
                s.optimization_params
                    .iter()
                    .filter(|p| p.optimizable)
                    .count()
            })
            .sum();

        indicator_params + nested_params + entry_params + exit_params + stop_params + take_params
    }
}
