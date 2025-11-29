# Анализ архитектурных проблем проекта

## Обзор

Данный документ описывает архитектурные проблемы, найденные в проекте торгового робота, их влияние на поддерживаемость кода и предложения по решению.

---

## 1. God Objects (Божественные объекты)

### Проблема
Несколько файлов содержат слишком много логики и ответственностей:

| Файл | Строк | Проблема |
|------|-------|----------|
| `candidate_builder.rs` | 2075 | 40+ методов, генерация всех компонентов стратегии |
| `genetic.rs` | 1713 | Эволюция + кроссовер + мутация + оценка |
| `executor.rs` | 1492 | Бэктест + управление позициями + индикаторы |
| `initial_population.rs` | 1274 | Генерация начальной популяции всех типов |
| `strategy_converter.rs` | 1019 | Конвертация всех типов элементов |

### Нарушение SOLID
- **SRP (Single Responsibility)**: Каждый класс должен иметь одну причину для изменения

### Решение
Декомпозировать на специализированные компоненты:

```rust
// Было: CandidateBuilder с 40+ методами
// Станет:
pub struct IndicatorBuilder { ... }
pub struct ConditionBuilder { ... }
pub struct StopHandlerBuilder { ... }
pub struct TimeframeBuilder { ... }

pub struct CandidateBuilder {
    indicator_builder: IndicatorBuilder,
    condition_builder: ConditionBuilder,
    stop_builder: StopHandlerBuilder,
    timeframe_builder: TimeframeBuilder,
}
```

---

## 2. Дублирование логики генерации условий

### Проблема
Логика генерации условий дублируется в нескольких местах:

- `candidate_builder.rs` → `build_condition()`, `build_condition_simple()`
- `genetic.rs` → `create_condition_for_indicator()`
- `initial_population.rs` → похожая логика выбора операторов

### Нарушение SOLID
- **DRY (Don't Repeat Yourself)**: Каждое знание должно иметь единственное представление

### Решение
Создать единый `ConditionFactory`:

```rust
pub struct ConditionFactory {
    config: ConditionFactoryConfig,
}

impl ConditionFactory {
    pub fn create_for_oscillator(&self, indicator: &IndicatorInfo) -> ConditionInfo;
    pub fn create_for_trend(&self, indicator: &IndicatorInfo) -> ConditionInfo;
    pub fn create_for_volatility(&self, indicator: &IndicatorInfo) -> ConditionInfo;
    pub fn create_comparison(&self, primary: &IndicatorInfo, secondary: &IndicatorInfo) -> ConditionInfo;
}
```

---

## 3. Жёсткие зависимости между модулями

### Проблема
Модули имеют прямые зависимости друг от друга вместо абстракций:

```rust
// genetic.rs напрямую зависит от конкретных типов
use crate::discovery::IndicatorInfoCollector;
use crate::indicators::registry::IndicatorRegistry;
use crate::optimization::evaluator::StrategyEvaluationRunner;
```

### Нарушение SOLID
- **DIP (Dependency Inversion)**: Модули высокого уровня не должны зависеть от модулей низкого уровня

### Решение
Ввести трейты для абстракции:

```rust
pub trait IndicatorCollector {
    fn collect_indicators(&self) -> Vec<IndicatorInfo>;
}

pub trait StrategyEvaluator {
    fn evaluate(&self, candidate: &StrategyCandidate) -> EvaluationResult;
}

pub struct GeneticAlgorithmV3<C: IndicatorCollector, E: StrategyEvaluator> {
    collector: C,
    evaluator: E,
}
```

---

## 4. Строковая типизация вместо enum

### Проблема
В некоторых местах используются строки вместо типов:

```rust
// discovery/types.rs
pub struct ConditionInfo {
    pub condition_type: String,  // "indicator_price", "indicator_indicator", "indicator_constant"
    pub price_field: Option<String>,  // "Close", "High", "Low"
}

// NestingConfig
pub input_for_indicators: &'static [&'static str],
pub accepts_from_indicators: &'static [&'static str],
```

### Нарушение
- Нет проверки компилятором
- Возможны runtime ошибки от опечаток

### Решение
Заменить на enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionType {
    IndicatorPrice,
    IndicatorIndicator,
    IndicatorConstant,
    TrendCondition,
}

pub struct ConditionInfo {
    pub condition_type: ConditionType,
    pub price_field: Option<PriceField>,
}
```

---

## 5. Отсутствие Strategy Pattern для выбора операторов

### Проблема
Большие цепочки if/else для выбора операторов и типов условий:

```rust
// candidate_builder.rs:1044-1065
let operator = if condition_type == "trend_condition" {
    if self.rng.gen_bool(0.5) {
        ConditionOperator::RisingTrend
    } else {
        ConditionOperator::FallingTrend
    }
} else if primary_indicator.indicator_type == "volatility" {
    // ...
} else if self.should_add(probabilities.use_crosses_operator) {
    // ...
} else {
    // ...
};
```

### Нарушение SOLID
- **OCP (Open/Closed)**: Код открыт для модификации при добавлении нового типа

### Решение
Использовать Strategy Pattern:

```rust
pub trait OperatorSelector {
    fn select(&self, indicator: &IndicatorInfo, rng: &mut impl Rng) -> ConditionOperator;
}

pub struct OscillatorOperatorSelector;
pub struct TrendOperatorSelector;
pub struct VolatilityOperatorSelector;

impl OperatorSelector for OscillatorOperatorSelector {
    fn select(&self, indicator: &IndicatorInfo, rng: &mut impl Rng) -> ConditionOperator {
        if rng.gen_bool(0.5) {
            ConditionOperator::Above
        } else {
            ConditionOperator::Below
        }
    }
}

pub struct OperatorSelectorFactory {
    selectors: HashMap<IndicatorCategory, Box<dyn OperatorSelector>>,
}
```

---

## 6. Смешение уровней абстракции в BacktestExecutor

### Проблема
`BacktestExecutor` отвечает за:
1. Управление историческим фидом
2. Агрегацию таймфреймов
3. Расчёт индикаторов
4. Оценку условий
5. Управление позициями
6. Управление рисками
7. Сбор метрик

### Нарушение SOLID
- **SRP**: Слишком много ответственностей в одном классе

### Решение
Разделить на компоненты:

```rust
pub struct BacktestEngine {
    feed_manager: FeedManager,
    indicator_engine: IndicatorEngine,
    condition_evaluator: ConditionEvaluator,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    metrics_collector: MetricsCollector,
}

impl BacktestEngine {
    pub fn run(&mut self) -> BacktestReport {
        for bar in self.feed_manager.bars() {
            self.indicator_engine.update(bar);
            let signals = self.condition_evaluator.evaluate();
            self.position_manager.process(signals);
            self.risk_manager.check_stops();
            self.metrics_collector.record();
        }
        self.metrics_collector.report()
    }
}
```

---

## 7. Отсутствие Builder Pattern для сложных объектов

### Проблема
Создание `StrategyDefinition` требует передачи множества параметров:

```rust
StrategyDefinition::new(
    metadata,
    parameters,
    indicator_bindings,
    vec![], // formulas
    all_condition_bindings,
    entry_rules,
    exit_rules,
    stop_handlers,
    take_handlers,
    defaults,
    BTreeMap::new(), // optimizer_hints
)
```

### Решение
Использовать Builder:

```rust
let definition = StrategyDefinitionBuilder::new()
    .metadata(metadata)
    .parameters(parameters)
    .indicators(indicator_bindings)
    .conditions(condition_bindings)
    .entry_rules(entry_rules)
    .exit_rules(exit_rules)
    .stops(stop_handlers)
    .takes(take_handlers)
    .build()?;
```

---

## 8. Нарушение LSP в трейте Indicator

### Проблема
Не все индикаторы корректно реализуют все методы трейта:

```rust
pub trait Indicator {
    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;
    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
}
```

Осцилляторы типа RSI требуют OHLC, но также должны реализовать `calculate_simple`.

### Нарушение SOLID
- **LSP (Liskov Substitution)**: Подтипы должны быть заменяемы базовыми типами

### Решение
Разделить трейты:

```rust
pub trait SimpleIndicator {
    fn calculate(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError>;
}

pub trait OHLCIndicator {
    fn calculate(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError>;
}

pub trait UniversalIndicator: SimpleIndicator + OHLCIndicator {}
```

---

## 9. Циклические зависимости между модулями

### Проблема
```
optimization → discovery → strategy → optimization (через types)
```

### Решение
Вынести общие типы в отдельный модуль `core`:

```
src/
├── core/           # Общие типы без зависимостей
│   ├── types.rs
│   └── traits.rs
├── indicators/     # Зависит только от core
├── condition/      # Зависит от core, indicators
├── strategy/       # Зависит от core, indicators, condition
├── discovery/      # Зависит от core, strategy
└── optimization/   # Зависит от core, discovery
```

---

## 10. Magic Numbers и захардкоженные значения

### Проблема
```rust
// genetic.rs
let elitism_count = 5;
let mutation_rate = 0.1;

// candidate_builder.rs
self.rng.gen_range(70.0..=90.0)  // RSI overbought
self.rng.gen_range(10.0..=30.0)  // RSI oversold
```

### Решение
Вынести в конфигурацию:

```rust
#[derive(Clone, Debug)]
pub struct OscillatorThresholds {
    pub rsi_overbought: Range<f64>,
    pub rsi_oversold: Range<f64>,
    pub stochastic_overbought: Range<f64>,
    pub stochastic_oversold: Range<f64>,
}

impl Default for OscillatorThresholds {
    fn default() -> Self {
        Self {
            rsi_overbought: 70.0..90.0,
            rsi_oversold: 10.0..30.0,
            // ...
        }
    }
}
```

---

## Приоритеты исправления

| Приоритет | Проблема | Влияние |
|-----------|----------|---------|
| 🔴 Высокий | God Objects | Сложность поддержки, невозможность тестирования |
| 🔴 Высокий | Дублирование логики | Рассинхронизация при изменениях |
| 🟡 Средний | Жёсткие зависимости | Сложность unit-тестирования |
| 🟡 Средний | Строковая типизация | Runtime ошибки |
| 🟢 Низкий | Отсутствие Builder | Читаемость кода |
| 🟢 Низкий | Magic Numbers | Гибкость конфигурации |

---

## План рефакторинга

### Этап 1: Декомпозиция God Objects
1. Разбить `CandidateBuilder` на специализированные билдеры
2. Разбить `BacktestExecutor` на компоненты
3. Разбить `GeneticAlgorithmV3` на отдельные сервисы

### Этап 2: Устранение дублирования
1. Создать единый `ConditionFactory`
2. Унифицировать логику выбора операторов

### Этап 3: Введение абстракций
1. Создать трейты для основных зависимостей
2. Применить Dependency Injection

### Этап 4: Типизация
1. Заменить строковые типы на enum
2. Ввести конфигурационные структуры для magic numbers
