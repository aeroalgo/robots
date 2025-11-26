# Руководство по билду стратегий в начальной популяции

## Оглавление
1. [Общая архитектура](#общая-архитектура)
2. [Фазы построения стратегии](#фазы-построения-стратегии)
3. [Индикаторы](#индикаторы)
4. [Условия (Conditions)](#условия-conditions)
5. [Стоп-обработчики](#стоп-обработчики)
6. [Правила валидации](#правила-валидации)
7. [Ограничения и исключения](#ограничения-и-исключения)
8. [Примеры возможных и невозможных стратегий](#примеры-возможных-и-невозможных-стратегий)

---

## Общая архитектура

### Структура кандидата стратегии (`StrategyCandidate`)

```rust
pub struct StrategyCandidate {
    /// Базовые индикаторы (строящиеся по цене)
    pub indicators: Vec<IndicatorInfo>,
    /// Вложенные индикаторы (строящиеся по другим индикаторам)
    pub nested_indicators: Vec<NestedIndicator>,
    /// Условия входа (entry conditions)
    pub conditions: Vec<ConditionInfo>,
    /// Условия выхода (exit conditions)
    pub exit_conditions: Vec<ConditionInfo>,
    /// Стоп-лосс обработчики
    pub stop_handlers: Vec<StopHandlerInfo>,
    /// Тейк-профит обработчики
    pub take_handlers: Vec<StopHandlerInfo>,
    /// Используемые таймфреймы
    pub timeframes: Vec<TimeFrame>,
    /// Конфигурация
    pub config: StrategyDiscoveryConfig,
}
```

### Конфигурация (`StrategyDiscoveryConfig`)

```rust
pub struct StrategyDiscoveryConfig {
    /// Максимум параметров оптимизации (по умолчанию: 10)
    pub max_optimization_params: usize,
    
    /// Количество таймфреймов (по умолчанию: 3)
    pub timeframe_count: usize,
    
    /// Базовый таймфрейм (по умолчанию: 60 минут)
    pub base_timeframe: TimeFrame,
    
    /// Максимальный таймфрейм в минутах (по умолчанию: 1440 = 1 день)
    pub max_timeframe_minutes: u32,
    
    /// Разрешить индикаторы на индикаторах (по умолчанию: false)
    pub allow_indicator_on_indicator: bool,
    
    /// Максимальная глубина вложенности (по умолчанию: 1)
    pub max_indicator_depth: usize,
}
```

---

## Фазы построения стратегии

### Фаза 1: Сбор информации об индикаторах

**Модуль:** `discovery/collector.rs`

Система собирает информацию обо всех зарегистрированных индикаторах из `IndicatorRegistry`:

```
IndicatorInfoCollector::collect_from_registry()
    │
    ├── Trend (SMA, EMA, WMA, AMA, ZLEMA, GEOMEAN, AMMA, SQWMA, SINEWMA, TPBF, SuperTrend, VTRAND, MAXFOR, MINFOR)
    ├── Oscillator (RSI, Stochastic)
    ├── Channel (BBMiddle, BBUpper, BBLower, KCMiddle, KCUpper, KCLower)
    ├── Volatility (ATR, TrueRange, WATR)
    └── Custom/Volume/SupportResistance
```

Для каждого индикатора определяется:
- **Категория** (`indicator_type`): `"trend"`, `"oscillator"`, `"channel"`, `"volatility"`, `"other"`
- **Тип входных данных** (`input_type`): `"price"` или `"indicator"`
- **Возможность вложенности** (`can_use_indicator_input`): только для `Simple` индикаторов
- **Параметры оптимизации** с глобальными именами (`period`, `coeff_atr`, `pct`)

### Фаза 2: Генерация комбинаций индикаторов

**Модуль:** `discovery/indicator.rs`

```
IndicatorCombinationGenerator::generate_with_indicator_inputs()
    │
    ├── generate_nested_combinations()
    │   │
    │   ├── Разделение на base_indicators (по цене)
    │   └── nested_capable_indicators (могут строиться по индикаторам)
    │
    └── Для каждой комбинации базовых:
        ├── Подсчет параметров
        ├── Проверка лимита max_params
        └── Генерация вложенных (если разрешено)
```

**Ограничения:**
- Максимум индикаторов в комбинации: `(params_for_indicators / 2).max(1).min(4)`
- Резерв 2 параметра для стопов/тейков
- Среднее 2 параметра на индикатор

### Фаза 3: Генерация условий

**Модуль:** `discovery/condition.rs`

Генератор создает три типа условий:

#### 3.1 Условия индикатор-цена (`indicator_price`)
```rust
generate_indicator_price_conditions(indicators, price_fields, operators, timeframes)
```
- Индикатор сравнивается с полем цены (Open, High, Low, Close)
- Операторы: `>`, `<`, `CrossesAbove`, `CrossesBelow`
- Мультитаймфреймовые комбинации (если указаны таймфреймы)

#### 3.2 Условия индикатор-индикатор (`indicator_indicator`)
```rust
generate_indicator_indicator_conditions(indicators, operators, timeframes)
```
- Два разных индикатора сравниваются между собой
- Операторы: `>`, `<`, `CrossesAbove`, `CrossesBelow`, `Between`
- Без дубликатов (i < j)

#### 3.3 Условия индикатор-константа (`indicator_constant`)
```rust
generate_indicator_constant_conditions(indicators, operators, timeframes)
```
- **ТОЛЬКО для осцилляторов** (`indicator_type == "oscillator"`)
- Пороговые значения из `get_oscillator_threshold_range()`
- Операторы: `>`, `<`
- Примеры: `RSI > 70`, `RSI < 30`, `Stochastic > 80`

### Фаза 4: Генерация стоп-обработчиков

**Модуль:** `discovery/stop_handler.rs`

```rust
StopHandlerCombinationGenerator::generate_combinations_from_configs()
```

**Типы стопов:**
1. `StopLossPct` - процентный стоп-лосс
2. `TakeProfitPct` - процентный тейк-профит

**Генерируемые комбинации:**
1. Только стоп-лоссы
2. Только тейк-профиты
3. Стоп-лосс + тейк-профит

### Фаза 5: Конвертация в StrategyDefinition

**Модуль:** `discovery/strategy_converter.rs`

```rust
StrategyConverter::candidate_to_definition(candidate, base_timeframe)
    │
    ├── create_metadata() - генерация ID и описания
    ├── extract_parameters() - извлечение параметров оптимизации
    ├── create_indicator_bindings() - биндинги индикаторов для всех TF
    ├── create_condition_bindings() - биндинги условий
    ├── create_stop_and_take_handlers() - стоп/тейк спецификации
    ├── create_entry_rules() - правила входа (RuleLogic::All)
    └── create_exit_rules() - правила выхода
```

---

## Индикаторы

### Типы индикаторов по входным данным

| Тип | Описание | Примеры |
|-----|----------|---------|
| `Simple` | Работают с массивом значений | SMA, EMA, RSI, WMA, AMA, ZLEMA, GEOMEAN, AMMA, SQWMA, SINEWMA, TPBF, BBMiddle, BBUpper, BBLower |
| `OHLC` | Требуют OHLC данные | ATR, TrueRange, SuperTrend, Stochastic, WATR, VTRAND, MAXFOR, MINFOR, KCMiddle, KCUpper, KCLower |
| `Universal` | Могут работать с обоими | ATR, SuperTrend |

### Категории индикаторов

| Категория | Индикаторы |
|-----------|------------|
| **Trend** | SMA, EMA, WMA, AMA, ZLEMA, GEOMEAN, AMMA, SQWMA, SINEWMA, TPBF, SuperTrend, VTRAND, MAXFOR, MINFOR |
| **Oscillator** | RSI, Stochastic |
| **Channel** | BBMiddle, BBUpper, BBLower, KCMiddle, KCUpper, KCLower |
| **Volatility** | ATR, TrueRange, WATR |

### Параметры индикаторов

| Индикатор | Параметры | Тип | Диапазон оптимизации |
|-----------|-----------|-----|---------------------|
| SMA, EMA, WMA, etc. | `period` | Period | start..end, step=10 |
| BBUpper, BBLower, BBMiddle | `period`, `deviation` | Period, Multiplier | period: 5-200, deviation: 0.1-10 |
| SuperTrend | `period`, `coeff_atr` | Period, Multiplier | period: 5-200, coeff_atr: 0.5-10 |
| KCUpper, KCLower | `period`, `atr_period`, `atr_multiplier` | Period, Period, Multiplier | - |
| RSI | `period` | Period | 5-200 (threshold для условий: 20-80) |
| Stochastic | `period` | Period | 5-200 (threshold: 10-90) |

### Индикаторы с поддержкой вложенности

**Могут строиться по другим индикаторам** (`can_use_indicator_input = true`):
- Все индикаторы типа `Simple`
- SMA, EMA, WMA, AMA, ZLEMA, RSI, BBMiddle, BBUpper, BBLower, etc.

**НЕ могут строиться по индикаторам**:
- Индикаторы типа `OHLC` (требуют High, Low, Close)
- ATR, TrueRange, Stochastic, SuperTrend (в OHLC режиме), KCMiddle, KCUpper, KCLower

---

## Условия (Conditions)

### Операторы условий

```rust
pub enum ConditionOperator {
    GreaterThan,    // >
    LessThan,       // <
    CrossesAbove,   // Пересечение вверх
    CrossesBelow,   // Пересечение вниз
    Between,        // Между двумя значениями
}
```

### Допустимые комбинации операторов

| Тип условия | Допустимые операторы |
|-------------|---------------------|
| indicator_price | `>`, `<`, `CrossesAbove`, `CrossesBelow` |
| indicator_indicator | `>`, `<`, `CrossesAbove`, `CrossesBelow`, `Between` |
| indicator_constant | `>`, `<` |

### Типы входных данных для условий

```rust
pub enum ConditionInputSpec {
    Single { source },                              // Один источник
    Dual { primary, secondary },                    // Два источника
    DualWithPercent { primary, secondary, percent }, // С процентом
    Range { source, lower, upper },                  // Диапазон (для Between)
    Indexed { source, index_offset },                // Со смещением индекса
    Ohlc,                                            // OHLC данные
}
```

### Маппинг на фабрику условий

| Оператор | Тренд? | Input Type | Фабрика |
|----------|--------|------------|---------|
| GreaterThan | Да | Single | RISINGTREND |
| GreaterThan | Нет | DualWithPercent | GREATERPERCENT |
| GreaterThan | Нет | Other | ABOVE |
| LessThan | Да | Single | FALLINGTREND |
| LessThan | Нет | DualWithPercent | LOWERPERCENT |
| LessThan | Нет | Other | BELOW |
| CrossesAbove | * | * | CROSSESABOVE |
| CrossesBelow | * | * | CROSSESBELOW |
| Between | * | Range | BETWEEN |

---

## Стоп-обработчики

### Типы стопов

```rust
pub struct StopHandlerInfo {
    pub id: String,
    pub name: String,
    pub handler_name: String,  // Имя обработчика из фабрики
    pub stop_type: String,     // "stop_loss" или "take_profit"
    pub optimization_params: Vec<ConditionParamInfo>,
    pub priority: i32,
}
```

**Доступные handler_name для stop_type = "stop_loss":**
- `StopLossPct` - процентный стоп-лосс
- `ATRTrailStop` - ATR трейлинг стоп
- `HILOTrailingStop` - HILO трейлинг стоп  
- `PercentTrailingStop` - процентный трейлинг стоп

**Доступные handler_name для stop_type = "take_profit":**
- `TakeProfitPct` - процентный тейк-профит

### Доступные Stop Handlers (стоп-лоссы)

| Handler | Алиасы | Параметры | Диапазон оптимизации | Описание |
|---------|--------|-----------|---------------------|----------|
| `StopLossPct` | `STOP_LOSS_PCT`, `STOPLOSS_PCT` | `percentage` | 2.0 - 10.0, step 0.5 | Фиксированный процентный стоп-лосс от цены входа |
| `ATRTrailStop` | `ATR_TRAIL_STOP`, `ATR_TRAIL` | `period`, `coeff_atr` | period: 10-150 (step 10), coeff_atr: 2.0-8.0 (step 0.2) | Трейлинг стоп на основе ATR. Требует индикатор ATR |
| `HILOTrailingStop` | `HILO_TRAIL_STOP`, `HILO_TRAIL` | `period` | 10.0 - 150.0, step 10 | Трейлинг стоп на основе MINFOR/MAXFOR. Требует индикаторы MINFOR, MAXFOR |
| `PercentTrailingStop` | `PERCENT_TRAIL_STOP`, `PERCENT_TRAIL` | `percentage` | 1.0 - 10.0, step 0.5 | Процентный трейлинг стоп |

### Доступные Take Handlers (тейк-профиты)

| Handler | Алиасы | Параметры | Диапазон оптимизации | Описание |
|---------|--------|-----------|---------------------|----------|
| `TakeProfitPct` | `TAKE_PROFIT_PCT` | `percentage` | 2.0 - 10.0, step 0.5 | Фиксированный процентный тейк-профит от цены входа |

### Детали обработчиков

#### StopLossPct
```rust
// Для Long: stop_level = entry_price * (1 - percentage/100)
// Для Short: stop_level = entry_price * (1 + percentage/100)
// Срабатывает когда Low <= stop_level (Long) или High >= stop_level (Short)
```

#### ATRTrailStop
- **Требует**: Индикатор ATR в стратегии
- **Логика**: `stop_level = min_price - (ATR * coeff_atr)` для Long
- **Трейлинг**: Стоп подтягивается только в прибыльную сторону

#### HILOTrailingStop  
- **Требует**: Индикаторы MINFOR и MAXFOR в стратегии
- **Логика**: Для Long стоп = MINFOR, для Short стоп = MAXFOR
- **Валидация перед входом**: Проверяет что цена выше/ниже уровня стопа
- **Трейлинг**: Стоп подтягивается только в прибыльную сторону

#### PercentTrailingStop
- **Логика**: `stop_level = min_price * (1 - percentage/100)` для Long
- **Трейлинг**: Следует за минимумом/максимумом цены

### Логика генерации комбинаций

1. **Только стоп-лоссы** - каждый тип отдельно
2. **Только тейк-профиты** - каждый тип отдельно  
3. **Комбинации SL + TP** - каждый тип SL с каждым типом TP

---

## Правила валидации

### Валидация кандидата (`is_valid`)

```rust
pub fn is_valid(&self) -> bool {
    let has_exit_conditions = !self.exit_conditions.is_empty();
    let has_stop_handlers = !self.stop_handlers.is_empty();
    let has_take_handlers = !self.take_handlers.is_empty();
    let has_any_exit = has_exit_conditions || has_stop_handlers || has_take_handlers;
    let only_take = !has_exit_conditions && !has_stop_handlers && has_take_handlers;

    self.total_optimization_params() <= self.config.max_optimization_params
        && self.timeframes.len() <= self.config.timeframe_count
        && has_any_exit       // ОБЯЗАТЕЛЬНО: хотя бы один выход
        && !only_take         // ЗАПРЕЩЕНО: только тейк без стопа или условия
}
```

### Обязательные требования

1. **Должен быть выход из позиции** - минимум одно из:
   - Exit conditions
   - Stop handlers
   - Take handlers

2. **Нельзя только тейк-профит без стопа** - если есть тейк, должен быть стоп или exit condition

3. **Лимит параметров оптимизации** - `total_optimization_params() <= max_optimization_params`

4. **Лимит таймфреймов** - `timeframes.len() <= timeframe_count`

### Подсчет параметров оптимизации

```rust
total_optimization_params = 
    indicator_params (base + nested) 
    + entry_condition_params 
    + exit_condition_params 
    + stop_params 
    + take_params
```

---

## Ограничения и исключения

### Ограничения по индикаторам

| Ограничение | Значение | Описание |
|-------------|----------|----------|
| Максимум индикаторов в комбинации | 4 | Жесткий лимит |
| Максимум параметров на комбинацию | `max_optimization_params` (10) | Конфигурируется |
| Резерв для стопов | 2 параметра | Вычитается из бюджета |
| Глубина вложенности | `max_indicator_depth` (1) | Только если разрешено |

### Ограничения по условиям

| Ограничение | Описание |
|-------------|----------|
| `indicator_constant` только для осцилляторов | RSI, Stochastic |
| `Between` требует Range input | 3 источника данных |
| Мультитаймфреймы опциональны | Если не указаны - базовый TF |

### Ограничения по стопам

| Ограничение | Описание |
|-------------|----------|
| Минимум 1 выход | Стоп, тейк или exit condition |
| Нельзя только тейк | Требуется стоп или exit condition |
| target_entry_ids | Автоматически заполняются из entry rules |

### Зависимости индикаторов для обработчиков

Стоп-обработчики **автономны** и **автоматически вычисляют** необходимые служебные индикаторы:

| Handler | Служебные индикаторы | Описание |
|---------|---------------------|----------|
| `ATRTrailStop` | `aux_ATR_{period}` | ATR автоматически вычисляется из OHLC данных |
| `HILOTrailingStop` | `aux_MINFOR_{period}`, `aux_MAXFOR_{period}` | MINFOR/MAXFOR автоматически вычисляются |
| `StopLossPct` | - | Не требует индикаторов |
| `PercentTrailingStop` | - | Не требует индикаторов |
| `TakeProfitPct` | - | Не требует индикаторов |

#### Как это работает

1. **При билде стратегии** (`StrategyBuilder::build()`):
   - Для каждого стоп-обработчика собираются `AuxiliaryIndicatorSpec`
   - Учитываются переопределённые параметры (`parameter_overrides`)
   - Specs сохраняются в `DynamicStrategy.auxiliary_specs`

2. **При запуске бэктеста** (`BacktestExecutor::run_backtest()`):
   - Вызывается `populate_auxiliary_indicators()`
   - Индикаторы вычисляются из OHLC данных через `IndicatorFactory`
   - Результаты сохраняются в `TimeframeData.auxiliary_indicators`

3. **При мутации в генетике**:
   - Если изменяется `period` у ATRTrailStop (например с 14 на 20)
   - При следующем билде будет собран `aux_ATR_20` вместо `aux_ATR_14`
   - Новый индикатор автоматически вычислится при бэктесте

#### Преимущества

- **Автономность**: Стоп-обработчики не зависят от торговых индикаторов стратегии
- **Чистота структуры**: ATR для ATRTrailStop не отображается как "торговый индикатор"
- **Эффективность**: Служебные индикаторы вычисляются один раз, а не на каждой свечке
- **Корректная оптимизация**: При мутации параметров стопа индикаторы пересчитываются автоматически

#### Пример в presets.rs

```rust
// Не нужно добавлять ATR как отдельный индикатор!
let stop_handlers = vec![StopHandlerSpec {
    id: "atr_trail_stop".to_string(),
    name: "ATR Trailing Stop".to_string(),
    handler_name: "ATRTrailStop".to_string(),
    timeframe: timeframe.clone(),
    price_field: PriceField::Close,
    parameters: HashMap::from([
        ("period".to_string(), StrategyParamValue::Number(14.0)),
        ("coeff_atr".to_string(), StrategyParamValue::Number(3.0)),
    ]),
    direction: PositionDirection::Long,
    priority: 10,
    tags: vec!["stop".to_string(), "trailing".to_string()],
    target_entry_ids: vec!["enter_long".to_string()],
}];
// ATR с period=14 будет автоматически вычислен как aux_ATR_14
```

### Исключения в конфигурации (candidate_builder_rules_example.json)

```json
{
  "rules": {
    "dependencies": [
      {
        "trigger": { "type": "StopHandler", "name": "StopLossPct" },
        "required": { "type": "TakeHandler", "name": "TakeProfitPct" },
        "strict": true
      }
    ],
    "exclusions": [
      {
        "element": { "type": "StopHandler", "name": "ATRTrailStop" },
        "excluded": { "type": "TakeHandler", "name": "TakeProfitPct" }
      }
    ]
  }
}
```

**Примечание**: Исключение `ATRTrailStop` + `TakeProfitPct` означает, что эти два обработчика не должны использоваться вместе (трейлинг стоп и фиксированный тейк могут конфликтовать в логике выхода).

---

## Примеры возможных и невозможных стратегий

### ✅ Валидные стратегии

#### Пример 1: Простая трендовая
```
Индикаторы: SMA(period=20)
Условие входа: SMA > Close (indicator_price)
Выход: StopLossPct(2%) + TakeProfitPct(4%)
Параметры оптимизации: 3 (period, stop_pct, take_pct)
```

#### Пример 2: Осцилляторная
```
Индикаторы: RSI(period=14)
Условие входа: RSI < 30 (indicator_constant)
Условие выхода: RSI > 70 (indicator_constant)
Выход: StopLossPct(2%)
Параметры: 4 (period, threshold_entry, threshold_exit, stop_pct)
```

#### Пример 3: Мультииндикаторная
```
Индикаторы: SMA(period=20), EMA(period=10), RSI(period=14)
Условие входа: SMA CrossesAbove EMA (indicator_indicator)
Условие входа: RSI < 40 (indicator_constant)
Выход: StopLossPct(2%)
Параметры: 5 (sma_period, ema_period, rsi_period, rsi_threshold, stop_pct)
```

#### Пример 4: С вложенным индикатором
```
Базовый: RSI(period=14)
Вложенный: SMA(period=5) на RSI
Условие: SMA(RSI) CrossesAbove 50
Выход: StopLossPct(2%)
```

#### Пример 5: Мультитаймфреймовая
```
TF1: 60min - SMA(20)
TF2: 240min - SMA(20)
Условие: SMA(60min) > SMA(240min)
Выход: StopLossPct(2%)
```

#### Пример 6: С ATR трейлинг стопом
```
Индикаторы: SMA(20), ATR(14)
Условие входа: SMA > Close
Выход: ATRTrailStop(period=14, coeff_atr=3.0) + TakeProfitPct(5%)
Параметры: 5 (sma_period, atr_period, atr_coeff, take_pct)
Примечание: ATRTrailStop требует ATR индикатор в стратегии!
```

#### Пример 7: С HILO трейлинг стопом  
```
Индикаторы: EMA(20), MINFOR(14), MAXFOR(14)
Условие входа: EMA CrossesAbove Close
Выход: HILOTrailingStop(period=14)
Параметры: 4 (ema_period, minfor_period, maxfor_period, hilo_period)
Примечание: HILOTrailingStop требует MINFOR/MAXFOR индикаторы!
Валидация: Перед входом проверяется что цена > MINFOR (для Long)
```

### ❌ Невалидные стратегии

#### Пример 1: Только тейк-профит
```
Индикаторы: SMA(20)
Условие входа: SMA > Close
Выход: TakeProfitPct(4%)  ← НЕТ СТОПА ИЛИ EXIT CONDITION
Результат: INVALID (only_take = true)
```

#### Пример 2: Превышение параметров
```
Индикаторы: SuperTrend(2) + RSI(1) + BBUpper(2) + KCUpper(3)
Условия: 2 параметра
Стопы: 2 параметра
Итого: 12 параметров > max(10)
Результат: INVALID
```

#### Пример 3: Нет выхода
```
Индикаторы: SMA(20)
Условие входа: SMA > Close
Выход: НИЧЕГО
Результат: INVALID (has_any_exit = false)
```

#### Пример 4: OHLC индикатор как вложенный
```
Базовый: SMA(20)
Вложенный: Stochastic на SMA ← НЕВОЗМОЖНО
Причина: Stochastic требует OHLC данные
```

#### Пример 5: indicator_constant для трендового
```
Индикатор: SMA(20)
Условие: SMA > 100 (indicator_constant)
Результат: НЕ ГЕНЕРИРУЕТСЯ
Причина: indicator_constant только для oscillator типа
```

#### Пример 6: ATRTrailStop без индикатора ATR
```
Индикаторы: SMA(20)
Условие входа: SMA > Close
Выход: ATRTrailStop(period=14, coeff_atr=3.0)
Результат: RUNTIME ERROR
Причина: ATRTrailStop не найдет значение ATR и не сработает
```

#### Пример 7: HILOTrailingStop без MINFOR/MAXFOR
```
Индикаторы: RSI(14)
Условие входа: RSI < 30
Выход: HILOTrailingStop(period=14)
Результат: RUNTIME ERROR
Причина: HILOTrailingStop не найдет MINFOR/MAXFOR и не сработает
```

---

## Диаграмма потока генерации

```
                         ┌─────────────────────────────┐
                         │   IndicatorRegistry         │
                         │   (все доступные индикаторы)│
                         └────────────┬────────────────┘
                                      │
                                      ▼
                         ┌─────────────────────────────┐
                         │ IndicatorInfoCollector      │
                         │ collect_from_registry()     │
                         └────────────┬────────────────┘
                                      │
                                      ▼
                   ┌──────────────────┴──────────────────┐
                   │                                     │
                   ▼                                     ▼
    ┌──────────────────────────┐         ┌──────────────────────────┐
    │ IndicatorCombination     │         │ ConditionCombination     │
    │ Generator                │         │ Generator                │
    │ - base indicators        │         │ - indicator_price        │
    │ - nested indicators      │         │ - indicator_indicator    │
    └───────────┬──────────────┘         │ - indicator_constant     │
                │                        └────────────┬─────────────┘
                │                                     │
                └──────────────────┬──────────────────┘
                                   │
                                   ▼
                         ┌─────────────────────────────┐
                         │ StopHandlerCombination      │
                         │ Generator                   │
                         │ - stop_loss combinations    │
                         │ - take_profit combinations  │
                         │ - mixed combinations        │
                         └────────────┬────────────────┘
                                      │
                                      ▼
                         ┌─────────────────────────────┐
                         │ StrategyCandidate           │
                         │ - indicators                │
                         │ - nested_indicators         │
                         │ - conditions                │
                         │ - exit_conditions           │
                         │ - stop_handlers             │
                         │ - take_handlers             │
                         └────────────┬────────────────┘
                                      │
                                      ▼
                         ┌─────────────────────────────┐
                         │ is_valid() ?                │
                         │ - params <= max             │
                         │ - has exit                  │
                         │ - not only take             │
                         └────────────┬────────────────┘
                                      │
                            ┌─────────┴─────────┐
                            ▼                   ▼
                         ┌──────┐           ┌───────┐
                         │ PASS │           │ FAIL  │
                         └──┬───┘           └───────┘
                            │
                            ▼
                ┌─────────────────────────────┐
                │ StrategyConverter           │
                │ candidate_to_definition()   │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │ StrategyDefinition          │
                │ - metadata                  │
                │ - indicator_bindings        │
                │ - condition_bindings        │
                │ - entry_rules               │
                │ - exit_rules                │
                │ - stop_handlers             │
                │ - take_handlers             │
                └─────────────────────────────┘
```

---

## Таблица совместимости индикаторов

| Индикатор | Может быть источником для вложенного | Может быть вложенным | indicator_constant условия |
|-----------|-------------------------------------|---------------------|---------------------------|
| SMA | ✅ | ✅ | ❌ |
| EMA | ✅ | ✅ | ❌ |
| WMA | ✅ | ✅ | ❌ |
| AMA | ✅ | ✅ | ❌ |
| ZLEMA | ✅ | ✅ | ❌ |
| RSI | ✅ | ✅ | ✅ (threshold: 20-80) |
| Stochastic | ❌ (OHLC) | ❌ | ✅ (threshold: 10-90) |
| ATR | ❌ (OHLC) | ❌ | ❌ |
| SuperTrend | ❌ (OHLC) | ❌ | ❌ |
| BBUpper | ✅ | ✅ | ❌ |
| BBLower | ✅ | ✅ | ❌ |
| BBMiddle | ✅ | ✅ | ❌ |
| KCUpper | ❌ (OHLC) | ❌ | ❌ |
| KCLower | ❌ (OHLC) | ❌ | ❌ |
| MAXFOR | ❌ (OHLC) | ❌ | ❌ |
| MINFOR | ❌ (OHLC) | ❌ | ❌ |

---

## Диапазоны параметров для оптимизации

### Централизованная система параметров (`ParameterPresets`)

Все диапазоны параметров определены централизованно в `indicators/parameters.rs`.

### Типы параметров и их диапазоны

| Тип | Диапазон по умолчанию | Шаг |
|-----|----------------------|-----|
| **Period** | 5.0 - 200.0 | 1.0 |
| **Multiplier** | 0.5 - 5.0 | 0.1 |
| **ATR Multiplier** | 1.0 - 10.0 | 0.5 |
| **Deviation** (BB) | 0.5 - 4.0 | 0.5 |
| **Coefficient** | 0.1 - 1.0 | 0.05 |

### Пороговые значения осцилляторов

| Индикатор | Диапазон | Шаг |
|-----------|----------|-----|
| **RSI** | 20 - 80 | 10 |
| **Stochastic** | 10 - 90 | 10 |
| **Williams %R** | -90 - -10 | 10 |
| **CCI** | -200 - 200 | 40 |
| **MACD** | -5 - 5 | 1 |
| **Momentum** | -100 - 100 | 20 |
| **Generic overbought** | 60 - 95 | 10 |
| **Generic oversold** | 5 - 40 | 10 |

### Шаги оптимизации по типам

В функции `get_optimization_range()` определены шаги для разных типов параметров:

```rust
match param_type {
    ParameterType::Period => step = 10.0,
    ParameterType::Multiplier (coeff_atr, atr_*) => step = 0.2,
    ParameterType::Multiplier (другие) => step = 0.2,
    _ => step = range.step (из пресетов)
}
```

### Формат имен параметров

Параметры стратегии именуются по шаблону:
```
{alias}_{param_name}
```

Примеры:
- `sma_period` - период для SMA
- `supertrend_coeff_atr` - множитель ATR для SuperTrend
- `stop_StopLossPct_percentage` - процент для стоп-лосса

---

## Глобальные параметры

Система поддерживает глобальные параметры, которые применяются к нескольким индикаторам:

| Глобальное имя | Применяется к |
|----------------|---------------|
| `period` | Все параметры с "period" или "length" в имени |
| `coeff_atr` | Параметры с "coeff_atr" или "atr_coeff" |
| `pct` | Параметры с "pct" или "percentage" |

Глобальные параметры определяются в `IndicatorInfoCollector::infer_global_param_name()`:

```rust
fn infer_global_param_name(param_name: &str) -> Option<String> {
    let name_lower = param_name.to_lowercase();
    
    if name_lower.contains("period") || name_lower.contains("length") {
        Some("period".to_string())
    } else if name_lower.contains("coeff_atr") || name_lower.contains("atr_coeff") {
        Some("coeff_atr".to_string())
    } else if name_lower.contains("pct") || name_lower.contains("percentage") {
        Some("pct".to_string())
    } else {
        None
    }
}
```

---

## Приоритеты стоп-обработчиков

Приоритеты определяют порядок проверки стопов при наличии нескольких активных сигналов:

| Обработчик | Тип | Приоритет по умолчанию | Описание |
|------------|-----|------------------------|----------|
| StopLossPct | stop_loss | 100 | Фиксированный стоп, высший приоритет |
| ATRTrailStop | stop_loss | 100 | ATR трейлинг стоп |
| HILOTrailingStop | stop_loss | 100 | HILO трейлинг стоп |
| PercentTrailingStop | stop_loss | 100 | Процентный трейлинг |
| TakeProfitPct | take_profit | 90 | Тейк-профит, ниже стоп-лоссов |

При совпадении нескольких стоп-сигналов в одном баре, выбирается сигнал с **меньшим** значением `priority`.

**Примечание**: Приоритет можно настроить при создании `StopHandlerConfig`.

---

## Workflow генерации стратегий

### 1. Инициализация
```rust
let config = StrategyDiscoveryConfig {
    max_optimization_params: 10,
    timeframe_count: 3,
    base_timeframe: TimeFrame::Minutes(60),
    max_timeframe_minutes: 1440,
    allow_indicator_on_indicator: false,
    max_indicator_depth: 1,
};
let engine = StrategyDiscoveryEngine::new(config);
```

### 2. Сбор индикаторов
```rust
let registry = IndicatorRegistry::new();
let indicators = IndicatorInfoCollector::collect_from_registry(&registry);
```

### 3. Генерация комбинаций индикаторов
```rust
let combinations = IndicatorCombinationGenerator::generate_with_indicator_inputs(
    &indicators,
    config.max_optimization_params,
    true,  // include_stops
    config.max_indicator_depth,
);
```

### 4. Генерация условий для каждой комбинации
```rust
let conditions = ConditionCombinationGenerator::generate_all_conditions_with_constants(
    &indicators,
    &[PriceField::Close],
    &[ConditionOperator::GreaterThan, ConditionOperator::LessThan, 
      ConditionOperator::CrossesAbove, ConditionOperator::CrossesBelow],
    true,  // allow_indicator_indicator
    Some(&timeframes),
);
```

### 5. Генерация стоп-обработчиков
```rust
let stop_configs = vec![
    // Процентный стоп-лосс
    StopHandlerConfig {
        handler_name: "StopLossPct".to_string(),
        stop_type: "stop_loss".to_string(),
        parameter_values: vec![2.0, 3.0, 5.0],
        parameter_name: "percentage".to_string(),
        global_param_name: Some("pct".to_string()),
        priority: 100,
    },
    // ATR трейлинг стоп (требует ATR индикатор)
    StopHandlerConfig {
        handler_name: "ATRTrailStop".to_string(),
        stop_type: "stop_loss".to_string(),
        parameter_values: vec![3.0, 5.0, 7.0], // coeff_atr
        parameter_name: "coeff_atr".to_string(),
        global_param_name: Some("coeff_atr".to_string()),
        priority: 100,
    },
    // HILO трейлинг стоп (требует MINFOR/MAXFOR)
    StopHandlerConfig {
        handler_name: "HILOTrailingStop".to_string(),
        stop_type: "stop_loss".to_string(),
        parameter_values: vec![14.0, 20.0, 50.0], // period
        parameter_name: "period".to_string(),
        global_param_name: Some("period".to_string()),
        priority: 100,
    },
    // Процентный трейлинг
    StopHandlerConfig {
        handler_name: "PercentTrailingStop".to_string(),
        stop_type: "stop_loss".to_string(),
        parameter_values: vec![1.0, 2.0, 3.0],
        parameter_name: "percentage".to_string(),
        global_param_name: Some("pct".to_string()),
        priority: 100,
    },
    // Тейк-профит
    StopHandlerConfig {
        handler_name: "TakeProfitPct".to_string(),
        stop_type: "take_profit".to_string(),
        parameter_values: vec![3.0, 5.0, 10.0],
        parameter_name: "percentage".to_string(),
        global_param_name: Some("pct".to_string()),
        priority: 90,
    },
];
let stop_combinations = StopHandlerCombinationGenerator::generate_combinations_from_configs(&stop_configs);
```

### 6. Сборка кандидата
```rust
let candidate = StrategyCandidate {
    indicators: combination.base_indicators,
    nested_indicators: combination.nested_indicators,
    conditions: entry_conditions,
    exit_conditions: exit_conditions,
    stop_handlers,
    take_handlers,
    timeframes,
    config: config.clone(),
};
```

### 7. Валидация
```rust
if candidate.is_valid() {
    // 8. Конвертация в StrategyDefinition
    let definition = candidate.to_strategy_definition(base_timeframe)?;
    
    // 9. Построение стратегии
    let strategy = StrategyBuilder::new(definition)
        .with_parameters(optimized_params)
        .build()?;
}
```

---

## Версия документа

**Версия:** 1.2  
**Дата:** 2025-11-25  
**Автор:** Strategy Discovery System Documentation

### История изменений
- **v1.2**: Добавлена полная информация о всех доступных стоп-обработчиках (ATRTrailStop, HILOTrailingStop, PercentTrailingStop), их зависимостях от индикаторов, примеры использования.
- **v1.1**: Добавлены диапазоны параметров оптимизации, workflow генерации.
- **v1.0**: Начальная версия документации.

