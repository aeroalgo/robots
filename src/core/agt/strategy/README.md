# Strategy Module Documentation

## Обзор

Модуль `strategy` содержит реализации для работы с торговыми стратегиями и условиями. Включает в себя как базовые, так и оптимизированные версии для работы с условиями.

## Структуры

### StrategyCondition

Базовая структура для работы с условиями стратегий.

```rust
pub struct StrategyCondition {
    data: Vec<f64>,
    indicator: Vec<f64>,
    constant: Vec<f64>,
    condition: ConditionEnum,
    result: Vec<bool>,
    name_indicator: String,
}
```

### OptimizedStrategyCondition

Оптимизированная структура, использующая слайсы вместо владения данными для экономии памяти.

```rust
pub struct OptimizedStrategyCondition<'a> {
    data: &'a [f64],
    indicator: &'a [f64],
    constant: f64,
    condition: ConditionEnum,
    result: Vec<bool>,
    name_indicator: String,
}
```

## Условия (ConditionEnum)

Поддерживаются следующие типы условий:

- `ABOVE` - значение выше порога
- `BELOW` - значение ниже порога
- `CROSSESABOVE` - пересечение выше порога
- `CROSSESBELOW` - пересечение ниже порога
- `LOWERPERCENTBARS` - снижение на процент за N баров
- `GREATERPERCENTBARS` - повышение на процент за N баров
- `FAILINGDATABARS` - падающие данные за N баров
- `FAILINGINDICATORSBARS` - падающие индикаторы за N баров
- `FALLINGTORISINGDATA` - переход от падения к росту данных
- `FALLINGTORISINGINDICATORS` - переход от падения к росту индикаторов
- `RISINGDATABARS` - растущие данные за N баров
- `RISINGINDICATORSBARS` - растущие индикаторы за N баров
- `RISINGTOFALLINGDATA` - переход от роста к падению данных
- `RISINGTOFALLINGINDICATORS` - переход от роста к падению индикаторов

## Оптимизации памяти

### Проблемы в оригинальном коде:

1. **Множественные клоны векторов** - данные копировались при создании структур
2. **Неэффективное использование памяти** - каждый экземпляр владел копией данных
3. **Отсутствие предварительного выделения памяти** - векторы росли динамически

### Решения:

1. **Использование слайсов** - `OptimizedStrategyCondition` использует ссылки на данные
2. **Предварительное выделение памяти** - `Vec::with_capacity()` для результатов
3. **Избежание клонирования** - использование `move` вместо `clone()`
4. **Ленивые вычисления** - сигналы вычисляются только при необходимости

## Использование

### Базовое использование:

```rust
use crate::core::agt::strategy::condition::StrategyCondition;
use crate::core::agt::opt::iterating::conditions::ConditionEnum;

let mut condition = StrategyCondition::new(
    data,
    indicator,
    ConditionEnum::ABOVE,
    50.0,
    "RSI".to_string(),
).await;

let signals = condition.generate_signals().await;
```

### Оптимизированное использование:

```rust
use crate::core::agt::strategy::optimized_condition::{ConditionFactory, ConditionUtils};

if let Some(mut optimized_condition) = ConditionFactory::create_optimized_condition(
    &data,
    &indicator,
    ConditionEnum::ABOVE,
    50.0,
    "RSI".to_string(),
) {
    let signals = optimized_condition.generate_signals();
    let stats = ConditionUtils::calculate_signal_stats(signals);
}
```

### Работа с несколькими условиями:

```rust
let conditions = vec![
    ConditionEnum::ABOVE,
    ConditionEnum::BELOW,
    ConditionEnum::CROSSESABOVE,
];

let optimized_conditions = ConditionFactory::create_multiple_conditions(
    &data,
    &indicator,
    conditions,
    50.0,
    "RSI".to_string(),
);
```

## Утилиты

### ConditionUtils

- `combine_signals_and()` - объединение сигналов с логическим И
- `combine_signals_or()` - объединение сигналов с логическим ИЛИ
- `find_signal_crossings()` - поиск пересечений сигналов
- `calculate_signal_stats()` - вычисление статистики сигналов

### ConditionFactory

- `create_optimized_condition()` - создание оптимизированного условия
- `create_multiple_conditions()` - создание нескольких условий

## Производительность

Оптимизированная версия показывает значительное улучшение производительности:

- **Меньше аллокаций памяти** - использование слайсов
- **Быстрее создание** - отсутствие клонирования
- **Эффективнее вычисления** - предварительное выделение памяти

## Тестирование

Модуль включает comprehensive тесты:

```bash
cargo test --package robots --lib strategy
```

## Примеры

См. `example_usage.rs` для полных примеров использования всех функций модуля. 