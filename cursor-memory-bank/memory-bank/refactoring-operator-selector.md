# Рефакторинг: Strategy Pattern для выбора операторов

## Дата: 2025-11-29

## Проблема

Большие цепочки if/else для выбора операторов и типов условий в `condition_builder.rs`:

```rust
let condition_type = if primary_indicator.indicator_type == "oscillator"
    && !is_oscillator_used_in_nested
{
    "indicator_constant"
} else if primary_indicator.indicator_type == "volatility" {
    "indicator_constant"
} else if is_built_on_oscillator {
    // ...
} else {
    // ...
};

let operator = if condition_type == "trend_condition" {
    // ...
} else if primary_indicator.indicator_type == "volatility" {
    // ...
} else if self.should_add(probabilities.use_crosses_operator) {
    // ...
} else {
    // ...
};
```

### Нарушение SOLID
- **OCP (Open/Closed)**: Код открыт для модификации при добавлении нового типа индикатора

## Решение

Создан модуль `operator_selector.rs` с enum-based Strategy Pattern:

### Использование существующего IndicatorCategory

Используется `IndicatorCategory` из `src/indicators/types.rs`:

```rust
pub enum IndicatorCategory {
    Trend,             // Трендовые индикаторы
    Oscillator,        // Осцилляторы
    Channel,           // Канальные индикаторы
    Volume,            // Объемные индикаторы
    SupportResistance, // Поддержка и сопротивление
    Custom,            // Пользовательские индикаторы
    Volatility,        // Волатильность
}
```

### Функции выбора

```rust
pub fn category_from_str(indicator_type: &str) -> IndicatorCategory { ... }
pub fn select_condition_type(category: &IndicatorCategory, ...) -> &'static str { ... }
pub fn select_operator(category: &IndicatorCategory, ...) -> ConditionOperator { ... }
```

### OperatorSelectorFactory

```rust
pub struct OperatorSelectorFactory;

impl OperatorSelectorFactory {
    pub fn select_operator_and_condition_type(
        indicator: &IndicatorInfo,
        nested_indicators: &[NestedIndicator],
        all_indicators: &[&IndicatorInfo],
        probabilities: &ConditionProbabilities,
        rng: &mut impl Rng,
    ) -> (ConditionOperator, &'static str)
}
```

## Логика выбора по категориям

### Oscillator
- **Condition Type**: `indicator_constant` (если не используется в nested, иначе `indicator_indicator`)
- **Operators**: `Above`, `Below`

### Volatility
- **Condition Type**: `indicator_constant`
- **Operators**: `GreaterPercent`, `LowerPercent`

### Trend / Channel / Volume / Default
- **Condition Type**: 
  - `indicator_indicator` (если built on oscillator)
  - `trend_condition` (по вероятности)
  - `indicator_indicator` (по вероятности)
  - `indicator_price` (по умолчанию)
- **Operators**:
  - `RisingTrend`, `FallingTrend` (для trend_condition)
  - `CrossesAbove`, `CrossesBelow` (по вероятности)
  - `Above`, `Below` (по умолчанию)

## Результат

| Файл | Строк до | Строк после | Изменение |
|------|----------|-------------|-----------|
| `condition_builder.rs` | 1249 | 1151 | -98 (-8%) |
| `operator_selector.rs` (новый) | 0 | 140 | +140 |

### Преимущества

1. **OCP соблюдён**: Для нового типа индикатора достаточно добавить вариант в enum
2. **Читаемость**: Логика для каждой категории изолирована в match
3. **Расширяемость**: Легко добавить новые категории индикаторов
4. **Тестируемость**: Каждая категория может тестироваться отдельно

## Изменённые файлы

1. `src/optimization/builders/operator_selector.rs` - новый
2. `src/optimization/builders/mod.rs` - добавлен экспорт
3. `src/optimization/builders/condition_builder.rs` - использует `OperatorSelectorFactory`
