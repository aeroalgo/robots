# Настройка параметров генерации стратегий

Параметры стратегий, которые будут оптимизироваться, задаются через `StrategyDiscoveryConfig`. Эта конфигурация определяет, какие типы стратегий будут генерироваться и оптимизироваться.

## Параметры конфигурации

### `max_optimization_params: usize`

Максимальное количество параметров оптимизации в стратегии. **Учитываются ВСЕ параметры оптимизации:**

1. **Параметры индикаторов** (базовые + вложенные)
   - Например: период SMA, период EMA, период RSI
2. **Параметры условий входа (entry conditions)**
   - Параметры условий, если они оптимизируемые
3. **Параметры условий выхода (exit conditions)**
   - Параметры exit rules, если они оптимизируемые
4. **Параметры стоп-обработчиков (stop handlers)**
   - Стоп-лосс: обычно 1 параметр (размер стопа)
   - Тейк-профит: обычно 1 параметр (размер профита)
   - **Итого: обычно 2 параметра (стоп + тейк)**

**Примеры расчета:**
- `8` параметров может быть:
  - 3 индикатора по 2 параметра = 6 параметров
  - + стоп-лосс (1) + тейк-профит (1) = 2 параметра
  - **Итого: 6 + 2 = 8 параметров**
- `10` параметров может быть:
  - 4 индикатора по 2 параметра = 8 параметров
  - + стоп-лосс (1) + тейк-профит (1) = 2 параметра
  - **Итого: 8 + 2 = 10 параметров**
- `12` параметров может быть:
  - 4 индикатора по 2 параметра = 8 параметров
  - + 2 условия входа по 1 параметру = 2 параметра
  - + стоп-лосс (1) + тейк-профит (1) = 2 параметра
  - **Итого: 8 + 2 + 2 = 12 параметров**

**Влияние:**
- Больше параметров = больше вариантов стратегий, но дольше оптимизация
- Меньше параметров = быстрее оптимизация, но меньше разнообразия
- **Важно:** При расчете учитываются ВСЕ компоненты стратегии (индикаторы + условия + стопы)

### `timeframe_count: usize`

Количество таймфреймов для использования в стратегиях.

**Примеры:**
- `1` - только базовый таймфрейм (например, 60 минут)
- `2` - базовый + один дополнительный (например, 60 и 120 минут)
- `3` - базовый + два дополнительных (например, 60, 120, 180 минут)
- `4` - мультитаймфреймовые стратегии (например, 60, 120, 240, 480 минут)

**Влияние:**
- Больше таймфреймов = более сложные стратегии, но больше комбинаций
- Меньше таймфреймов = проще стратегии, быстрее генерация

### `base_timeframe: TimeFrame`

Базовый таймфрейм для генерации комбинаций.

**Примеры:**
```rust
TimeFrame::Minutes(60)   // 60 минут
TimeFrame::Minutes(240)  // 4 часа
TimeFrame::Hours(1)      // 1 час
```

**Влияние:**
- Определяет основной таймфрейм для стратегий
- Дополнительные таймфреймы генерируются как кратные базовому

### `allow_indicator_on_indicator: bool`

Разрешить построение индикаторов по другим индикаторам.

**Примеры:**
- `false` - индикаторы строятся только по цене (SMA от Close, RSI от Close)
- `true` - индикаторы могут строиться по другим индикаторам (SMA от RSI, RSI от SMA)

**Влияние:**
- `true` = больше разнообразия, но сложнее стратегии
- `false` = проще стратегии, быстрее вычисления

### `max_indicator_depth: usize`

Максимальная глубина вложенности индикаторов (если `allow_indicator_on_indicator = true`).

**Примеры:**
- `1` - индикаторы только первого уровня (SMA от цены, RSI от цены)
- `2` - индикаторы могут быть вложенными (SMA от RSI, где RSI от цены)
- `3` - более глубокая вложенность (SMA от RSI от SMA от цены)

**Влияние:**
- Больше глубина = более сложные стратегии, но дольше вычисления
- Меньше глубина = проще стратегии, быстрее

**Примечание:** Пороги для осцилляторов (например, RSI < 30, RSI > 70) автоматически генерируются на основе значений из `get_oscillator_threshold_range()` в `src/indicators/implementations.rs`. Для каждого осциллятора используются его специфичные диапазоны оптимизации.

## Примеры конфигураций

### Простая конфигурация (для быстрого тестирования)

```rust
let discovery_config = StrategyDiscoveryConfig {
    max_optimization_params: 6,
    timeframe_count: 1,
    base_timeframe: TimeFrame::Minutes(60),
    allow_indicator_on_indicator: false,
    max_indicator_depth: 1,
};
```

**Характеристики:**
- Простые стратегии с 6 параметрами
- Один таймфрейм
- Индикаторы только от цены
- Без условий с константами

### Стандартная конфигурация

```rust
let discovery_config = StrategyDiscoveryConfig {
    max_optimization_params: 8,
    timeframe_count: 3,
    base_timeframe: TimeFrame::Minutes(60),
    allow_indicator_on_indicator: false,
    max_indicator_depth: 1,
};
```

**Характеристики:**
- Средняя сложность (8 параметров)
- Мультитаймфреймовые стратегии (3 таймфрейма)
- Индикаторы только от цены
- Условия с порогами осцилляторов

### Продвинутая конфигурация

```rust
let discovery_config = StrategyDiscoveryConfig {
    max_optimization_params: 12,
    timeframe_count: 4,
    base_timeframe: TimeFrame::Minutes(60),
    allow_indicator_on_indicator: true,
    max_indicator_depth: 2,
};
```

**Характеристики:**
- Сложные стратегии (12 параметров)
- Мультитаймфреймовые стратегии (4 таймфрейма)
- Вложенные индикаторы (до 2 уровней)
- Множество порогов для осцилляторов

### Конфигурация для трендовых стратегий

```rust
let discovery_config = StrategyDiscoveryConfig {
    max_optimization_params: 10,
    timeframe_count: 3,
    base_timeframe: TimeFrame::Minutes(240),  // 4 часа
    allow_indicator_on_indicator: false,
    max_indicator_depth: 1,
};
```

**Характеристики:**
- Фокус на трендовых индикаторах (SMA, EMA)
- Более высокий таймфрейм
- Без осцилляторов

### Конфигурация для скальпинга

```rust
let discovery_config = StrategyDiscoveryConfig {
    max_optimization_params: 6,
    timeframe_count: 2,
    base_timeframe: TimeFrame::Minutes(5),  // 5 минут
    allow_indicator_on_indicator: false,
    max_indicator_depth: 1,
};
```

**Характеристики:**
- Низкий таймфрейм (5 минут)
- Меньше параметров для быстрой оптимизации
- Использование осцилляторов

## Как считается количество параметров

Метод `StrategyCandidate::total_optimization_params()` учитывает:

```rust
// 1. Параметры базовых индикаторов
let base_indicator_params = indicators.iter()
    .map(|ind| ind.parameters.iter().filter(|p| p.optimizable).count())
    .sum();

// 2. Параметры вложенных индикаторов
let nested_indicator_params = nested_indicators.iter()
    .map(|nested| nested.indicator.parameters.iter().filter(|p| p.optimizable).count())
    .sum();

// 3. Параметры условий входа (entry conditions)
let entry_condition_params = conditions.iter()
    .map(|cond| cond.optimization_params.iter().filter(|p| p.optimizable).count())
    .sum();

// 4. Параметры условий выхода (exit conditions)
let exit_condition_params = exit_conditions.iter()
    .map(|cond| cond.optimization_params.iter().filter(|p| p.optimizable).count())
    .sum();

// 5. Параметры стоп-обработчиков (стоп-лосс и тейк-профит)
let stop_params = stop_handlers.iter()
    .map(|stop| stop.optimization_params.iter().filter(|p| p.optimizable).count())
    .sum();

// Итого:
total = base_indicator_params + nested_indicator_params 
      + entry_condition_params + exit_condition_params 
      + stop_params
```

**Важно:** При генерации стратегий система автоматически учитывает все эти компоненты и гарантирует, что `total_optimization_params <= max_optimization_params`.

## Использование в коде

### Базовое использование (с настройками по умолчанию)

```rust
let generator = InitialPopulationGenerator::new(
    config.clone(),
    frames.clone(),
    base_timeframe.clone(),
);
```

### С кастомной конфигурацией

```rust
let discovery_config = StrategyDiscoveryConfig {
    max_optimization_params: 8,
    timeframe_count: 3,
    base_timeframe: base_timeframe.clone(),
    allow_indicator_on_indicator: false,
    max_indicator_depth: 1,
};

let generator = InitialPopulationGenerator::with_discovery_config(
    config.clone(),
    frames.clone(),
    base_timeframe.clone(),
    discovery_config,
);
```

## Рекомендации

1. **Начните с простой конфигурации** для тестирования
2. **Увеличивайте сложность постепенно** по мере необходимости
3. **Используйте `timeframe_count: 1`** для быстрой оптимизации
4. **Включайте `allow_indicator_on_indicator: true`** только если нужны сложные стратегии

## Влияние на производительность

- **Больше `max_optimization_params`** → больше времени на генерацию и оптимизацию
- **Больше `timeframe_count`** → больше комбинаций, дольше генерация
- **`allow_indicator_on_indicator: true`** → значительно больше комбинаций

