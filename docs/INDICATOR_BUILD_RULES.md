# Документация IndicatorBuildRules

Система правил построения стратегий для индикаторов. Каждый индикатор определяет свои правила через метод `build_rules()`.

---

## Структура IndicatorBuildRules

```rust
pub struct IndicatorBuildRules {
    pub allowed_conditions: &'static [&'static str],
    pub price_compare: PriceCompareConfig,
    pub threshold_type: ThresholdType,
    pub indicator_compare: IndicatorCompareConfig,
    pub nesting: NestingConfig,
    pub phase_1_allowed: bool,
    pub supports_percent_condition: bool,
    pub can_compare_with_input_source: bool,
    pub can_compare_with_nested_result: bool,
    pub nested_compare_conditions: &'static [&'static str],  // NEW
}
```

---

## 1. allowed_conditions

**Тип:** `&'static [&'static str]`

Список имён условий, которые можно применять к индикатору. Имена берутся из `ConditionFactory`.

### Доступные условия:

| Условие | Описание | Пример |
|---------|----------|--------|
| `"Above"` | Значение выше | `SMA > Close` |
| `"Below"` | Значение ниже | `SMA < Close` |
| `"CrossesAbove"` | Пересечение вверх | `SMA пересекает Close вверх` |
| `"CrossesBelow"` | Пересечение вниз | `SMA пересекает Close вниз` |
| `"RisingTrend"` | Восходящий тренд | `RSI растёт N баров` |
| `"FallingTrend"` | Нисходящий тренд | `RSI падает N баров` |
| `"GreaterPercent"` | Выше на процент | `SMA > Close * 1.02` |
| `"LowerPercent"` | Ниже на процент | `SMA < Close * 0.98` |

### Примеры:

```rust
// Осциллятор - базовые условия + тренд
allowed_conditions: &["Above", "Below", "CrossesAbove", "CrossesBelow", "RisingTrend", "FallingTrend"]

// Volatility - только сравнение
allowed_conditions: &["Above", "Below"]

// Трендовый - все условия включая процентные
allowed_conditions: &["Above", "Below", "CrossesAbove", "CrossesBelow", "RisingTrend", "FallingTrend", "GreaterPercent", "LowerPercent"]
```

---

## 2. price_compare (PriceCompareConfig)

**Тип:** `PriceCompareConfig`

Определяет, можно ли сравнивать индикатор с ценой и какие поля цены допустимы.

### Поля:

```rust
pub struct PriceCompareConfig {
    pub enabled: bool,                        // Включено ли сравнение
    pub price_fields: &'static [&'static str], // Допустимые поля: "Close", "High", "Low", "Open"
}
```

### Константы:

| Константа | enabled | price_fields | Использование |
|-----------|---------|--------------|---------------|
| `DISABLED` | `false` | `[]` | Осцилляторы (RSI, Stochastic) |
| `STANDARD` | `true` | `["Close", "High", "Low"]` | Трендовые (SMA, EMA) |
| `CLOSE_ONLY` | `true` | `["Close"]` | Volatility (ATR) |

### Примеры:

```rust
// RSI не сравнивается с ценой (значения 0-100)
price_compare: PriceCompareConfig::DISABLED

// SMA сравнивается с Close, High, Low
price_compare: PriceCompareConfig::STANDARD
// Генерирует: SMA > Close, SMA CrossesAbove High

// ATR сравнивается только с Close
price_compare: PriceCompareConfig::CLOSE_ONLY
// Генерирует: ATR > Close * 2%
```

---

## 3. threshold_type (ThresholdType)

**Тип:** `ThresholdType`

Определяет, можно ли сравнивать индикатор с порогом (абсолютное значение или процент от цены).

### Варианты:

| Вариант | Описание | Пример |
|---------|----------|--------|
| `None` | Не сравнивается с порогом | SMA (сравнивается с ценой) |
| `Absolute` | Абсолютное значение | `RSI > 70`, `RSI < 30` |
| `PercentOfPrice { base_price_fields }` | Процент от цены | `ATR > Close * 2%` |

### Поля PercentOfPrice:

```rust
ThresholdType::PercentOfPrice {
    base_price_fields: &["Close"],  // Какие поля цены использовать как базу для процента
}
```

**Важно:** `PercentOfPrice` — это НЕ прямое сравнение с ценой! Это только определяет базу для расчёта процентного порога.

- `ATR > Close` — ❌ НЕТ (это было бы `price_compare`)
- `ATR > 2% от Close` — ✅ ДА (это `PercentOfPrice`)

### Вспомогательные методы:

```rust
// Процент только от Close
threshold_type: ThresholdType::percent_of_close()

// Процент от нескольких цен
threshold_type: ThresholdType::percent_of(&["Close", "High", "Low"])
```

### Примеры:

```rust
// RSI - абсолютные значения (диапазон из ParameterPresets)
threshold_type: ThresholdType::Absolute
// Генерирует: RSI > 70, RSI < 30, RSI CrossesAbove 50

// ATR - процент от цены (НЕ прямое сравнение!)
threshold_type: ThresholdType::PercentOfPrice {
    base_price_fields: &["Close"],
}
// Генерирует: ATR > Close * 0.02 (2% от цены)
// НЕ генерирует: ATR > Close (прямое сравнение запрещено)

// SMA - не сравнивается с порогом
threshold_type: ThresholdType::None
// Не генерирует условий с числами
```

---

## 4. indicator_compare (IndicatorCompareConfig)

**Тип:** `IndicatorCompareConfig`

Определяет, с какими индикаторами можно сравнивать.

### Поля:

```rust
pub struct IndicatorCompareConfig {
    pub enabled: bool,                                    // Включено ли
    pub allowed_categories: &'static [IndicatorCategory], // Разрешённые категории
    pub denied_categories: &'static [IndicatorCategory],  // Запрещённые категории
    pub specific_indicators: &'static [&'static str],     // Конкретные индикаторы (приоритет)
    pub supports_percent: bool,                           // Поддержка GreaterPercent/LowerPercent
}
```

### Поле supports_percent

Определяет, можно ли использовать процентные условия (`GreaterPercent`, `LowerPercent`) при сравнении с другими индикаторами.

```rust
// SMA может сравниваться с EMA с процентом
indicator_compare: IndicatorCompareConfig::TREND_AND_CHANNEL  // supports_percent: true
// Генерирует: SMA > EMA * 1.02%, SMA GreaterPercent EMA 2%

// Без процентных условий
indicator_compare: IndicatorCompareConfig::only_with_no_percent(&["SMA"])
// Генерирует только: Indicator > SMA, Indicator CrossesAbove SMA
```

### Константы:

| Константа | Описание |
|-----------|----------|
| `DISABLED` | Не сравнивается с другими индикаторами |
| `TREND_AND_CHANNEL` | Сравнивается с Trend и Channel, запрещены Oscillator, поддерживает процент |
| `only_with(&[...])` | Сравнивается только с конкретными индикаторами, поддерживает процент |
| `only_with_no_percent(&[...])` | Сравнивается только с конкретными индикаторами, без процента |

### Примеры:

```rust
// RSI не сравнивается с другими индикаторами
indicator_compare: IndicatorCompareConfig::DISABLED

// SMA сравнивается с другими трендовыми и каналами
indicator_compare: IndicatorCompareConfig::TREND_AND_CHANNEL
// Генерирует: SMA > EMA, SMA CrossesAbove BBUpper

// Сравнивается только с конкретными индикаторами
indicator_compare: IndicatorCompareConfig::only_with(&["SMA", "EMA"])
// Генерирует ТОЛЬКО: Indicator > SMA, Indicator CrossesAbove EMA
```

### Логика проверки:

1. Если `specific_indicators` не пуст → проверяем по имени
2. Если категория в `denied_categories` → запрещено
3. Если `allowed_categories` пуст → разрешено всё
4. Иначе → проверяем, есть ли категория в `allowed_categories`

---

## 5. nesting (NestingConfig)

**Тип:** `NestingConfig`

Определяет правила вложенности индикаторов: `SMA(RSI)`, `EMA(SMA)`.

### Поля:

```rust
pub struct NestingConfig {
    // === Этот индикатор как ИСТОЧНИК (input) ===
    pub can_be_input: bool,                               // Может быть входом для других
    pub input_for_categories: &'static [IndicatorCategory], // Для каких категорий
    pub input_for_indicators: &'static [&'static str],      // Для каких конкретно (приоритет)

    // === Этот индикатор как ОБЁРТКА (wrapper) ===
    pub accepts_input: bool,                              // Может принимать другие как вход
    pub accepts_from_categories: &'static [IndicatorCategory], // От каких категорий
    pub accepts_from_indicators: &'static [&'static str],      // От каких конкретно (приоритет)
}
```

### Константы:

#### `NestingConfig::OSCILLATOR`
```rust
can_be_input: true,
input_for_categories: &[Trend],        // RSI может быть входом для SMA → SMA(RSI)
input_for_indicators: &[],
accepts_input: false,                   // RSI НЕ принимает вход → RSI(SMA) ❌
accepts_from_categories: &[],
accepts_from_indicators: &[],
```

#### `NestingConfig::TREND`
```rust
can_be_input: true,
input_for_categories: &[Trend, Oscillator], // SMA может быть входом для EMA, RSI
input_for_indicators: &[],
accepts_input: true,                          // SMA принимает вход → SMA(RSI) ✅
accepts_from_categories: &[Trend, Oscillator],
accepts_from_indicators: &[],
```

#### `NestingConfig::DISABLED`
```rust
can_be_input: false,     // Не может быть входом
accepts_input: false,    // Не принимает вход
```

#### `NestingConfig::VOLATILITY`
```rust
can_be_input: true,
input_for_categories: &[Trend],        // ATR может быть входом для SMA → SMA(ATR)
input_for_indicators: &[],
accepts_input: false,                   // ATR НЕ принимает вход → ATR(SMA) ❌
accepts_from_categories: &[],
accepts_from_indicators: &[],
```

### Примеры вложенности:

```
SMA(RSI) - допустимо:
  RSI.can_be_input = true
  RSI.input_for_categories содержит Trend
  SMA.accepts_input = true
  SMA.accepts_from_categories содержит Oscillator

RSI(SMA) - НЕ допустимо:
  RSI.accepts_input = false ← стоп

ATR(SMA) - НЕ допустимо:
  ATR.nesting = DISABLED
  ATR.accepts_input = false ← стоп
```

### Проверка через методы:

```rust
// Может ли RSI быть входом для SMA?
rsi_rules.nesting.can_be_input_for("SMA", IndicatorCategory::Trend)

// Может ли SMA принять RSI как вход?
sma_rules.nesting.can_accept_from("RSI", IndicatorCategory::Oscillator)
```

---

## 6. phase_1_allowed

**Тип:** `bool`

Определяет, можно ли использовать индикатор в первой фазе построения стратегии.

### Значения:

| Значение | Описание | Индикаторы |
|----------|----------|------------|
| `true` | Можно в Phase 1 | SMA, EMA, RSI, Stochastic, BB, KC |
| `false` | Только Phase 2+ | ATR, Volume, WATR |

### Пример:

```rust
// RSI можно использовать в первой фазе
phase_1_allowed: true

// ATR только во второй+ фазе (вспомогательный)
phase_1_allowed: false
```

---

## 7. supports_percent_condition

**Тип:** `bool`

Определяет, поддерживает ли индикатор условия `GreaterPercent` и `LowerPercent`.

### Значения:

| Значение | Описание | Пример |
|----------|----------|--------|
| `true` | Поддерживает | `SMA > Close * 1.02%` |
| `false` | Не поддерживает | RSI (значения 0-100, процент бессмысленен) |

### Пример:

```rust
// SMA поддерживает процентные условия
supports_percent_condition: true
// Генерирует: SMA GreaterPercent Close 2%

// RSI не поддерживает (0-100, процент не имеет смысла)
supports_percent_condition: false
```

---

## 8. can_compare_with_input_source

**Тип:** `bool`

**Для nested индикаторов:** Может ли индикатор-обёртка сравниваться со своим источником.

### Пример SMA(RSI):

```
SMA(RSI) vs RSI?
SMA.can_compare_with_input_source = true → ДА ✅
```

### Пример RSI(SMA) (если бы RSI принимал вход):

```
RSI(SMA) vs SMA?
RSI.can_compare_with_input_source = false → НЕТ ❌
```

### Правила по категориям:

| Категория | can_compare_with_input_source | Причина |
|-----------|-------------------------------|---------|
| Trend (SMA, EMA) | `true` | SMA(RSI) vs RSI - имеет смысл |
| Oscillator (RSI) | `false` | Осциллятор сравнивается только с константой |
| Channel | `false` | Не участвует в nested |
| Volatility (ATR) | `false` | Не участвует в nested |

---

## 9. can_compare_with_nested_result

**Тип:** `bool`

**Для источника:** Может ли индикатор-источник сравниваться с индикатором, построенным по нему.

### Пример RSI как источник:

```
RSI vs SMA(RSI)?
RSI.can_compare_with_nested_result = true → ДА ✅
```

### Пример ATR как источник (если бы ATR был источником):

```
ATR vs SMA(ATR)?
ATR.can_compare_with_nested_result = false → НЕТ ❌
```

### Правила по категориям:

| Категория | can_compare_with_nested_result | Причина |
|-----------|--------------------------------|---------|
| Trend (SMA, EMA) | `true` | SMA vs EMA(SMA) - имеет смысл |
| Oscillator (RSI) | `true` | RSI vs SMA(RSI) - сглаживание |
| Channel | `false` | Не участвует в nested |
| Volatility (ATR) | `true` | ATR vs SMA(ATR) - сглаживание волатильности |

---

## 10. nested_compare_conditions

**Тип:** `&'static [&'static str]`

Дополнительные условия, доступные только при сравнении с nested индикаторами. Если пусто - используются `allowed_conditions`.

### Пример ATR:

ATR с ценой использует только `Above`/`Below`, но с SMA(ATR) может использовать расширенный набор:

```rust
allowed_conditions: &["Above", "Below"],  // Для сравнения с ценой
nested_compare_conditions: &[
    "Above",
    "Below", 
    "CrossesAbove",
    "CrossesBelow",
    "GreaterPercent",
    "LowerPercent",
],  // Для сравнения с SMA(ATR)
```

### Генерируемые условия:

```
ATR vs Close * 2%     → Above, Below (из allowed_conditions)
ATR vs SMA(ATR)       → CrossesAbove, GreaterPercent (из nested_compare_conditions)
```

### Пример SMA:

SMA использует одинаковые условия везде:

```rust
allowed_conditions: &["Above", "Below", "CrossesAbove", "CrossesBelow", ...],
nested_compare_conditions: &[],  // Пусто = используются allowed_conditions
```

### Метод проверки:

```rust
let rules = indicator.build_rules();

// Условие для обычного сравнения
rules.is_condition_allowed("CrossesAbove")

// Условие для сравнения с nested
rules.is_nested_condition_allowed("CrossesAbove")
```

---

## Готовые константы IndicatorBuildRules

### OSCILLATOR (RSI, Stochastic)

```rust
IndicatorBuildRules::OSCILLATOR = {
    allowed_conditions: ["Above", "Below", "CrossesAbove", "CrossesBelow", "RisingTrend", "FallingTrend"],
    price_compare: DISABLED,              // RSI не сравнивается с ценой
    threshold_type: Absolute,              // RSI > 70
    indicator_compare: DISABLED,          // RSI не сравнивается с SMA
    nesting: OSCILLATOR,                  // Может быть входом для Trend
    phase_1_allowed: true,
    supports_percent_condition: false,    // Процент не имеет смысла
    can_compare_with_input_source: false, // RSI(X) не сравнивается с X
    can_compare_with_nested_result: true, // RSI сравнивается с SMA(RSI)
    nested_compare_conditions: [],        // Те же что allowed_conditions
}
```

### TREND (SMA, EMA)

```rust
IndicatorBuildRules::TREND = {
    allowed_conditions: ["Above", "Below", "CrossesAbove", "CrossesBelow", "RisingTrend", "FallingTrend", "GreaterPercent", "LowerPercent"],
    price_compare: STANDARD,              // SMA vs Close/High/Low
    threshold_type: None,                  // SMA не сравнивается с числом
    indicator_compare: TREND_AND_CHANNEL, // SMA vs EMA, BBUpper (с supports_percent!)
    nesting: TREND,                       // Принимает и отдаёт вход
    phase_1_allowed: true,
    supports_percent_condition: true,     // SMA > Close * 1.02%
    can_compare_with_input_source: true,  // SMA(RSI) vs RSI ✅
    can_compare_with_nested_result: true, // SMA vs EMA(SMA) ✅
    nested_compare_conditions: [],        // Те же что allowed_conditions
}
```

**Процентные условия с индикаторами:**
- `SMA > EMA * 1.02%` - сгенерируется благодаря `indicator_compare.supports_percent: true`

### CHANNEL (BBUpper, KCLower)

```rust
IndicatorBuildRules::CHANNEL = {
    allowed_conditions: ["Above", "Below", "CrossesAbove", "CrossesBelow", "GreaterPercent", "LowerPercent"],
    price_compare: STANDARD,              // BBUpper vs Close
    threshold_type: None,
    indicator_compare: TREND_AND_CHANNEL, // BBUpper vs SMA (с supports_percent!)
    nesting: DISABLED,                    // Не участвует в nested
    phase_1_allowed: true,
    supports_percent_condition: true,
    can_compare_with_input_source: false,
    can_compare_with_nested_result: false,
    nested_compare_conditions: [],
}
```

### VOLATILITY (ATR, WATR)

```rust
IndicatorBuildRules::VOLATILITY = {
    allowed_conditions: ["Above", "Below"],
    price_compare: DISABLED,                 // ATR НЕ сравнивается с ценой напрямую!
    threshold_type: PercentOfPrice {       // ATR > 2% от Close
        base_price_fields: ["Close"],        // База для процента — Close
    },
    indicator_compare: DISABLED,             // Не сравнивается с другими напрямую
    nesting: VOLATILITY,                     // Может быть входом для Trend
    phase_1_allowed: false,                  // Только Phase 2+
    supports_percent_condition: false,
    can_compare_with_input_source: false,
    can_compare_with_nested_result: true,    // ATR vs SMA(ATR) ✅
    nested_compare_conditions: [             // Расширенные условия для nested
        "Above", "Below",
        "CrossesAbove", "CrossesBelow",
        "GreaterPercent", "LowerPercent",
    ],
}
```

**Что это даёт:**
- ❌ `ATR > Close` — НЕТ (price_compare: DISABLED)
- ✅ `ATR > 2% от Close` — ДА (PercentOfPrice с base_price_fields: ["Close"])
- ✅ `SMA(ATR)` можно построить
- ✅ `ATR vs SMA(ATR)` с условиями CrossesAbove, GreaterPercent и т.д.

### VOLUME

```rust
IndicatorBuildRules::VOLUME = {
    allowed_conditions: ["Above", "Below"],
    price_compare: DISABLED,
    threshold_type: None,
    indicator_compare: DISABLED,
    nesting: DISABLED,
    phase_1_allowed: false,
    supports_percent_condition: false,
    can_compare_with_input_source: false,
    can_compare_with_nested_result: false,
    nested_compare_conditions: [],
}
```

---

## Примеры кастомных правил

### Индикатор сравнивается только с SMA и EMA

```rust
fn build_rules(&self) -> IndicatorBuildRules {
    IndicatorBuildRules {
        indicator_compare: IndicatorCompareConfig::only_with(&["SMA", "EMA"]),
        ..IndicatorBuildRules::TREND
    }
}
```

### Индикатор может быть входом только для конкретных

```rust
fn build_rules(&self) -> IndicatorBuildRules {
    IndicatorBuildRules {
        nesting: NestingConfig {
            can_be_input: true,
            input_for_indicators: &["SMA", "EMA", "WMA"],  // Только эти три
            input_for_categories: &[],
            accepts_input: false,
            accepts_from_categories: &[],
            accepts_from_indicators: &[],
        },
        ..IndicatorBuildRules::OSCILLATOR
    }
}
```

### Индикатор принимает вход только от RSI

```rust
fn build_rules(&self) -> IndicatorBuildRules {
    IndicatorBuildRules {
        nesting: NestingConfig {
            can_be_input: true,
            input_for_categories: &[IndicatorCategory::Trend],
            input_for_indicators: &[],
            accepts_input: true,
            accepts_from_indicators: &["RSI"],  // Только RSI
            accepts_from_categories: &[],
        },
        ..IndicatorBuildRules::TREND
    }
}
```

---

## Схема принятия решений

```
Построить условие для индикатора X:

1. С ценой?
   X.price_compare.enabled == true?
   → price_field в X.price_compare.price_fields?
   → condition в X.allowed_conditions?
   → СОЗДАТЬ: X {condition} {price_field}
   
   + Процент с ценой?
   X.supports_percent_condition == true?
   → СОЗДАТЬ: X GreaterPercent {price_field} 2%

2. С порогом?
   X.threshold_type != None?
   → condition в X.allowed_conditions?
   → СОЗДАТЬ: X {condition} {threshold}

3. С другим индикатором Y?
   X.indicator_compare.enabled == true?
   → Y.category разрешена?
   → condition в X.allowed_conditions?
   → СОЗДАТЬ: X {condition} Y
   
   + Процент с индикатором?
   X.indicator_compare.supports_percent == true?
   → СОЗДАТЬ: X GreaterPercent Y 2%

4. Вложенный X(Y)?
   Y.nesting.can_be_input == true?
   Y.nesting.can_be_input_for(X.name, X.category)?
   X.nesting.accepts_input == true?
   X.nesting.can_accept_from(Y.name, Y.category)?
   → СОЗДАТЬ: X(Y)

5. Сравнить X(Y) с Y?
   X.can_compare_with_input_source == true?
   → condition в X.allowed_conditions? (или nested_compare_conditions)
   → СОЗДАТЬ: X(Y) {condition} Y

6. Сравнить Y с X(Y)?
   Y.can_compare_with_nested_result == true?
   → condition в Y.is_nested_condition_allowed()?
   → СОЗДАТЬ: Y {condition} X(Y)
```

---

## Методы для проверки

```rust
let rules = indicator.build_rules();

// Условие разрешено? (для обычного сравнения)
rules.is_condition_allowed("CrossesAbove")

// Условие разрешено для nested сравнения?
rules.is_nested_condition_allowed("CrossesAbove")

// Сравнение с ценой?
rules.can_compare_with_price("Close")

// Сравнение с порогом?
rules.can_compare_with_threshold()

// Сравнение с категорией?
rules.can_compare_with_category(IndicatorCategory::Trend)

// Процентные условия с индикаторами?
rules.indicator_compare.supports_percent

// Может быть входом?
rules.can_be_input_for(IndicatorCategory::Trend)

// Может принять вход?
rules.can_accept_input_from(IndicatorCategory::Oscillator)

// Nested: проверка по имени
rules.nesting.can_be_input_for("SMA", IndicatorCategory::Trend)
rules.nesting.can_accept_from("RSI", IndicatorCategory::Oscillator)
```
