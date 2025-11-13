# Руководство по работе со Strategy Layer

## 1. Назначение
Strategy Layer объединяет:
- описание стратегий через `StrategyDefinition`;
- подготовку входных данных (`StrategyContext`);
- сборку и исполнение стратегий через `StrategyBuilder` и трейт `Strategy`;
- управление библиотекой стратегий (`StrategyRegistry`).

## 2. Описание стратегии (StrategyDefinition)
```rust
StrategyDefinition {
    metadata: StrategyMetadata::with_id("MY_STRATEGY", "My Strategy"),
    indicator_bindings: vec![/* IndicatorBindingSpec */],
    condition_bindings: vec![/* ConditionBindingSpec */],
    entry_rules: vec![/* StrategyRuleSpec */],
    exit_rules: vec![/* StrategyRuleSpec */],
    timeframe_requirements: vec![/* TimeframeRequirement */],
    parameters: Vec::new(),
    defaults: StrategyParameterMap::new(),
    optimizer_hints: BTreeMap::new(),
}
```

### 2.1 IndicatorBindingSpec
```rust
IndicatorBindingSpec {
    alias: "fast_sma".to_string(),
    timeframe: TimeFrame::minutes(15),
    source: IndicatorSourceSpec::Registry {
        name: "SMA".to_string(),
        parameters: HashMap::from([("period".to_string(), 10.0)]),
    },
    tags: vec!["trend".to_string()],
}
```

### 2.2 ConditionBindingSpec
```rust
ConditionBindingSpec {
    id: "fast_cross_above".to_string(),
    name: "Fast SMA crosses above slow".to_string(),
    timeframe: TimeFrame::minutes(15),
    condition_name: "CROSSESABOVE".to_string(),
    parameters: HashMap::new(),
    input: ConditionInputSpec::Dual {
        primary: DataSeriesSource::Indicator { alias: "fast_sma".into() },
        secondary: DataSeriesSource::Indicator { alias: "slow_sma".into() },
    },
    weight: 1.0,
    tags: vec!["entry".to_string()],
    user_formula: None,
}
```

### 2.3 StrategyRuleSpec
```rust
StrategyRuleSpec {
    id: "enter_long".to_string(),
    name: "Enter long".to_string(),
    logic: RuleLogic::All,
    conditions: vec!["fast_cross_above".to_string()],
    signal: StrategySignalType::Entry,
    direction: PositionDirection::Long,
    quantity: None,
    tags: vec!["core".to_string()],
}
```

## 3. Использование StrategyBuilder
`StrategyBuilder` собирает готовую стратегию из `StrategyDefinition` или пользовательского ввода.

```rust
use robots::strategy::{StrategyBuilder, StrategyParamValue};

let strategy = StrategyBuilder::new(my_definition)
    .with_parameter("risk", StrategyParamValue::Number(1.5))
    .with_parameter("take_profit", StrategyParamValue::Number(3.0))
    .build()?;
```

Если параметры заранее готовы:
```rust
let overrides = StrategyParameterMap::from([
    ("risk".to_string(), StrategyParamValue::Number(2.0)),
]);
let strategy = StrategyBuilder::new(my_definition)
    .with_parameters(overrides)
    .build()?;
```

### 3.1 Пользовательский ввод (StrategyUserInput)
```rust
let user_input = StrategyUserInput {
    name: "My UI Strategy".to_string(),
    description: None,
    indicators: vec![/* UserIndicatorStep */],
    conditions: vec![/* UserConditionStep */],
    actions: vec![/* UserActionStep */],
    parameters: StrategyParameterMap::new(),
    metadata: HashMap::new(),
};

let strategy = StrategyBuilder::from_user_input(user_input)?
    .build()?;
```

## 4. StrategyRegistry
```rust
let registry = StrategyRegistry::with_defaults();
let strategy = registry.build_strategy("SMA_CROSSOVER_LONG", None)?;

let overrides = StrategyParameterMap::from([
    ("fast_period".to_string(), StrategyParamValue::Number(8.0)),
]);
let tuned = registry.build_strategy("SMA_CROSSOVER_LONG", Some(overrides))?;
```

Чтобы добавить свою стратегию:
```rust
let registry = StrategyRegistry::new();
registry.register_definition(my_definition);
let strategy = registry.build_strategy("MY_STRATEGY", None)?;
```

## 5. Подготовка StrategyContext
```rust
let mut context = StrategyContext::new();
let frame = QuoteFrame::new(symbol.clone(), timeframe.clone());
// заполняем frame данными...

let mut tf_data = TimeframeData::with_quote_frame(&frame, frame.len() - 1);
tf_data.insert_indicator("fast_sma", fast_series);
tf_data.insert_indicator("slow_sma", slow_series);
context.insert_timeframe(timeframe.clone(), tf_data);

context.user_settings.insert(
    "max_position".to_string(),
    StrategyParamValue::Integer(3),
);
```

## 6. Исполнение стратегии
```rust
use robots::strategy::base::Strategy;

let decision = Strategy::evaluate(&strategy, &context).await?;

for entry in decision.entries.iter() {
    // оформление ордера на вход
}
for exit in decision.exits.iter() {
    // оформление ордера на выход
}
```

`StrategyDecision` содержит `entries`, `exits`, `custom` и `metadata` для доп. информации.

## 7. Юнит-тесты
В `src/strategy/tests.rs` есть примеры:
- `sma_crossover_entry_generates_signal`
- `sma_crossover_exit_generates_signal`

Они демонстрируют подготовку тестовых данных, сборку стратегии и проверку сигналов через `Strategy::evaluate`.

## 8. Резюме пайплайна
1. Создать `StrategyDefinition` (или получить из UI/DSL).
2. Зарегистрировать стратегию (`StrategyRegistry`) или построить напрямую (`StrategyBuilder`).
3. Подготовить `StrategyContext` (данные, индикаторы, пользовательские параметры).
4. Вызвать `Strategy::evaluate` и обработать `StrategyDecision`.
5. Написать тесты для ключевых сценариев.

Такой подход позволяет подключить автоматический перебор стратегий, оптимизацию и дальнейший анализ результатов на единых структурах.
