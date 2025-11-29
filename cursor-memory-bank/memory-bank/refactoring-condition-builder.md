# Рефакторинг дублирования логики генерации условий

## Дата: 2025-11-29

## Проблема

Логика генерации условий дублировалась в нескольких местах:

| Файл | Метод | Описание |
|------|-------|----------|
| `genetic.rs` | `create_condition_for_indicator()` | ~270 строк генерации условий для мутации |
| `genetic.rs` | `can_compare_indicators_for_mutation()` | ~35 строк проверки совместимости индикаторов |
| `condition_builder.rs` | `build_condition()` | Основная логика генерации условий |
| `condition_builder.rs` | `can_compare_indicators()` | Основная логика проверки совместимости |

### Нарушение DRY
Одна и та же логика выбора типа условия, оператора, генерации констант и проверки совместимости индикаторов была реализована дважды.

## Решение

### 1. Расширение ConditionBuilder

Добавлены новые публичные методы для работы со `StrategyCandidate`:

```rust
pub fn create_for_candidate_indicator(
    indicator: &IndicatorInfo,
    candidate: &StrategyCandidate,
    is_entry: bool,
    probabilities: &ConditionProbabilities,
) -> Option<ConditionInfo>
```

Также добавлены статические методы для генерации условий без зависимости от `&mut self`:
- `generate_condition_details_static()`
- `generate_indicator_constant_static()`
- `generate_trend_condition_static()`
- `generate_indicator_indicator_static()`
- `generate_indicator_price_static()`

### 2. Обновление genetic.rs

**Удалено:**
- `create_condition_for_indicator()` - 270 строк дублированного кода
- `can_compare_indicators_for_mutation()` - 35 строк дублированного кода

**Заменено на делегирование:**

```rust
fn create_condition_for_indicator(
    indicator: &crate::discovery::IndicatorInfo,
    candidate: &StrategyCandidate,
    is_entry: bool,
    config: &GeneticAlgorithmConfig,
    _price_fields: &[PriceField],
    _operators: &[ConditionOperator],
) -> Option<crate::discovery::ConditionInfo> {
    let default_probabilities = ConditionProbabilities::default();
    let probabilities = config
        .candidate_builder_config
        .as_ref()
        .map(|c| &c.probabilities.conditions)
        .unwrap_or(&default_probabilities);

    ConditionBuilder::create_for_candidate_indicator(indicator, candidate, is_entry, probabilities)
}

fn can_compare_indicators_for_mutation(
    primary: &crate::discovery::IndicatorInfo,
    secondary: &crate::discovery::IndicatorInfo,
    nested_indicators: &[crate::discovery::NestedIndicator],
    all_indicators: &[crate::discovery::IndicatorInfo],
) -> bool {
    ConditionBuilder::can_compare_indicators(primary, secondary, nested_indicators, all_indicators)
}
```

## Результат

| Файл | Строк до | Строк после | Изменение |
|------|----------|-------------|-----------|
| `genetic.rs` | 1713 | 1431 | -282 (-16%) |
| `condition_builder.rs` | 860 | 1249 | +389 |

### Преимущества
1. **Единая точка изменений** - вся логика генерации условий теперь в `ConditionBuilder`
2. **Консистентность** - один и тот же код используется везде
3. **Тестируемость** - можно тестировать `ConditionBuilder` отдельно

### Обратная совместимость
- Все публичные API сохранены
- Методы в `genetic.rs` остались как тонкие обёртки над `ConditionBuilder`

## Изменённые файлы

1. `src/optimization/builders/condition_builder.rs` - добавлены статические методы
2. `src/optimization/genetic.rs` - удалено дублирование, добавлено делегирование
