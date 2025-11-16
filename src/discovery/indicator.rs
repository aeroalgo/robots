use crate::discovery::types::{IndicatorCombination, IndicatorInfo, NestedIndicator};

/// Генератор комбинаций индикаторов
pub struct IndicatorCombinationGenerator;

impl IndicatorCombinationGenerator {
    /// Генерирует комбинации индикаторов с учетом ограничения на количество параметров оптимизации
    ///
    /// # Аргументы
    /// * `available_indicators` - список доступных индикаторов
    /// * `max_params` - максимальное количество параметров оптимизации
    /// * `include_stops` - включать ли стоп-лосс/тейк-профит в подсчет параметров
    ///
    /// # Возвращает
    /// Вектор комбинаций, где каждая комбинация - это список индикаторов
    pub fn generate_combinations(
        available_indicators: &[IndicatorInfo],
        max_params: usize,
        include_stops: bool,
    ) -> Vec<Vec<IndicatorInfo>> {
        let mut result = Vec::new();
        let stop_params = if include_stops { 2 } else { 0 }; // стоп-лосс и тейк-профит

        // Генерируем комбинации разной длины
        for combo_len in 1..=available_indicators.len() {
            let combinations = Self::generate_combinations_of_length(
                available_indicators,
                combo_len,
                max_params - stop_params,
            );
            result.extend(combinations);
        }

        result
    }

    /// Генерирует комбинации индикаторов заданной длины
    fn generate_combinations_of_length(
        indicators: &[IndicatorInfo],
        length: usize,
        max_params: usize,
    ) -> Vec<Vec<IndicatorInfo>> {
        if length == 0 {
            return vec![vec![]];
        }
        if length > indicators.len() {
            return vec![];
        }

        let mut result = Vec::new();
        for i in 0..=indicators.len() - length {
            let first = indicators[i].clone();
            let rest_combinations =
                Self::generate_combinations_of_length(&indicators[i + 1..], length - 1, max_params);

            for mut combo in rest_combinations {
                let total_params: usize = combo
                    .iter()
                    .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
                    .sum::<usize>()
                    + first.parameters.iter().filter(|p| p.optimizable).count();

                if total_params <= max_params {
                    combo.insert(0, first.clone());
                    result.push(combo);
                }
            }
        }
        result
    }

    /// Генерирует комбинации индикаторов с поддержкой построения индикаторов по индикаторам
    pub fn generate_with_indicator_inputs(
        available_indicators: &[IndicatorInfo],
        max_params: usize,
        include_stops: bool,
        max_depth: usize,
    ) -> Vec<IndicatorCombination> {
        let mut result = Vec::new();
        let stop_params = if include_stops { 2 } else { 0 };

        // Генерируем комбинации с разной глубиной вложенности
        for depth in 0..=max_depth {
            let combinations = Self::generate_nested_combinations(
                available_indicators,
                max_params - stop_params,
                depth,
            );
            result.extend(combinations);
        }

        result
    }

    /// Генерирует вложенные комбинации индикаторов
    ///
    /// Алгоритм:
    /// 1. Генерирует все комбинации базовых индикаторов (строящихся по цене)
    /// 2. Для каждой комбинации базовых индикаторов генерирует возможные вложенные индикаторы
    /// 3. Вложенные индикаторы могут строиться только по индикаторам, которые могут быть входными данными
    /// 4. Учитывает глубину вложенности и ограничение на параметры оптимизации
    fn generate_nested_combinations(
        indicators: &[IndicatorInfo],
        max_params: usize,
        max_depth: usize,
    ) -> Vec<IndicatorCombination> {
        let mut result = Vec::new();

        // Разделяем индикаторы на базовые (строящиеся по цене) и те, что могут строиться по индикаторам
        let base_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .filter(|ind| ind.input_type == "price" || ind.can_use_indicator_input)
            .collect();

        let nested_capable_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .filter(|ind| ind.can_use_indicator_input)
            .collect();

        if base_indicators.is_empty() {
            return result;
        }

        // Генерируем все комбинации базовых индикаторов
        for combo_len in 1..=base_indicators.len().min(max_params) {
            let base_combinations = Self::generate_combinations_of_length(
                &base_indicators
                    .iter()
                    .map(|&ind| (*ind).clone())
                    .collect::<Vec<_>>(),
                combo_len,
                max_params,
            );

            for base_combo in base_combinations {
                // Для каждой комбинации базовых индикаторов генерируем вложенные
                let base_params: usize = base_combo
                    .iter()
                    .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
                    .sum();

                let remaining_params = max_params.saturating_sub(base_params);

                // Генерируем вложенные индикаторы с учетом глубины
                let nested_combinations = Self::generate_nested_for_base(
                    &base_combo,
                    &nested_capable_indicators,
                    remaining_params,
                    max_depth,
                    1, // текущая глубина
                );

                // Добавляем все комбинации (включая пустую, если нет вложенных индикаторов)
                for nested_combo in nested_combinations {
                    result.push(IndicatorCombination {
                        base_indicators: base_combo.clone(),
                        nested_indicators: nested_combo,
                    });
                }
            }
        }

        result
    }

    /// Генерирует вложенные индикаторы для заданной комбинации базовых индикаторов
    ///
    /// # Аргументы
    /// * `base_indicators` - базовые индикаторы, по которым могут строиться вложенные
    /// * `nested_capable` - индикаторы, которые могут строиться по другим индикаторам
    /// * `remaining_params` - оставшееся количество параметров оптимизации
    /// * `max_depth` - максимальная глубина вложенности
    /// * `current_depth` - текущая глубина вложенности
    fn generate_nested_for_base(
        base_indicators: &[IndicatorInfo],
        nested_capable: &[&IndicatorInfo],
        remaining_params: usize,
        max_depth: usize,
        current_depth: usize,
    ) -> Vec<Vec<NestedIndicator>> {
        if current_depth > max_depth || remaining_params == 0 || nested_capable.is_empty() {
            return vec![vec![]];
        }

        let mut result = Vec::new();

        // Получаем все доступные индикаторы-источники (базовые индикаторы)
        let available_sources: Vec<String> = base_indicators
            .iter()
            .map(|ind| ind.alias.clone())
            .collect();

        if available_sources.is_empty() {
            return vec![vec![]];
        }

        // Генерируем комбинации вложенных индикаторов
        for nested_indicator in nested_capable {
            let nested_params: usize = nested_indicator
                .parameters
                .iter()
                .filter(|p| p.optimizable)
                .count();

            if nested_params > remaining_params {
                continue;
            }

            // Генерируем вложенные индикаторы для каждого доступного источника
            for source_alias in &available_sources {
                let nested = NestedIndicator {
                    indicator: (*nested_indicator).clone(),
                    input_indicator_alias: source_alias.clone(),
                    depth: current_depth,
                };

                let new_remaining = remaining_params.saturating_sub(nested_params);

                // Рекурсивно генерируем следующие уровни вложенности
                // (если текущий вложенный индикатор может быть источником для других)
                let deeper_nested = if nested_indicator.can_use_indicator_input
                    && current_depth < max_depth
                    && new_remaining > 0
                {
                    // Создаем расширенный список базовых индикаторов, включая текущий вложенный
                    // как потенциальный источник для следующих уровней
                    let mut extended_base = base_indicators.to_vec();
                    extended_base.push((*nested_indicator).clone());

                    // Рекурсивно генерируем более глубокие уровни
                    Self::generate_nested_for_base(
                        &extended_base,
                        nested_capable,
                        new_remaining,
                        max_depth,
                        current_depth + 1,
                    )
                } else {
                    vec![vec![]]
                };

                // Комбинируем текущий вложенный индикатор с более глубокими уровнями
                for mut deeper_combo in deeper_nested {
                    deeper_combo.insert(0, nested.clone());
                    result.push(deeper_combo);
                }

                // Также добавляем только текущий вложенный индикатор без более глубоких уровней
                result.push(vec![nested]);
            }
        }

        // Добавляем пустую комбинацию (без вложенных индикаторов на этом уровне)
        result.push(vec![]);

        result
    }
}
