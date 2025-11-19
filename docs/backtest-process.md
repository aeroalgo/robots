# Процесс выполнения бэктеста

## Обзор

Документ описывает детальный процесс выполнения бэктеста торговой стратегии в системе. Понимание этого процесса необходимо для оптимизации производительности.

## Архитектура бэктеста

Бэктест выполняется через `BacktestExecutor`, который управляет:
- Историческими данными (`HistoricalFeed`)
- Контекстом стратегии (`StrategyContext`)
- Позициями (`PositionManager`)
- Аналитикой (`BacktestAnalytics`)

## Этапы выполнения бэктеста

### 1. Инициализация (`BacktestExecutor::new()` или `from_definition()`)

1. **Загрузка данных** - создание `QuoteFrame` для каждого требуемого таймфрейма
2. **Генерация недостающих таймфреймов** - агрегация данных для старших таймфреймов
3. **Вычисление warmup bars** - определение минимального количества баров для прогрева индикаторов
4. **Инициализация контекста** - создание `StrategyContext` с данными по каждому таймфрейму

### 2. Предварительный расчет индикаторов (`populate_indicators()`)

**ВАЖНО:** Индикаторы вычисляются **один раз** для всех свечей перед началом основного цикла.

```rust
// Код из src/strategy/executor.rs:409-479
async fn populate_indicators(&mut self) -> Result<(), StrategyExecutionError> {
    // Группировка индикаторов по таймфреймам
    let mut grouped: HashMap<TimeFrame, Vec<IndicatorBindingSpec>> = HashMap::new();
    for binding in self.strategy.indicator_bindings() {
        grouped.entry(binding.timeframe.clone()).or_default().push(binding.clone());
    }
    
    // Для каждого таймфрейма
    for (timeframe, bindings) in grouped {
        let ohlc = frame.to_indicator_ohlc();
        let plan = IndicatorComputationPlan::build(&bindings)?;
        
        // Вычисление всех индикаторов для всего массива данных
        for binding in plan.ordered() {
            let values = engine.compute_registry(&timeframe, name, parameters, &ohlc).await?;
            self.store_indicator_series(&timeframe, &binding.alias, values.clone())?;
        }
    }
}
```

**Оптимизация:** Индикаторы вычисляются сразу для всего исторического периода, что эффективно, так как многие индикаторы (SMA, EMA и т.д.) требуют весь массив данных для расчета.

### 3. Основной цикл бэктеста (`run_backtest()`)

Основной цикл итерируется по каждой свечке:

```rust
// Код из src/strategy/executor.rs:282-406
while self.feed.step(&mut self.context) {
    // Шаг 1: Обновление индексов всех таймфреймов
    // Шаг 2: Проверка warmup периода
    // Шаг 3: Расширение агрегированных таймфреймов
    self.feed.expand_aggregated_timeframes(&mut self.context, ...)?;
    
    // Шаг 4: Оценка стратегии (ГЛАВНАЯ ЧАСТЬ)
    let decision = self.strategy.evaluate(&self.context).await?;
    
    // Шаг 5: Обработка решений о входах/выходах
    // Шаг 6: Проверка stop/take handlers
}
```

### 4. Оценка стратегии на каждой свечке (`strategy.evaluate()`)

**ПРОБЛЕМА ПРОИЗВОДИТЕЛЬНОСТИ:** На каждой свечке выполняется полный проход по всем условиям.

```rust
// Код из src/strategy/builder.rs:501-552
async fn evaluate(&self, context: &StrategyContext) -> Result<StrategyDecision, StrategyError> {
    // Шаг 1: Оценка ВСЕХ условий
    let evaluations = self.evaluate_conditions(context).await?;
    
    // Шаг 2: Оценка stop handlers
    let mut stop_signals = self.evaluate_stop_handlers(context)?;
    
    // Шаг 3: Оценка take handlers
    let mut take_signals = self.evaluate_take_handlers(context)?;
    
    // Шаг 4: Оценка entry rules
    for rule in &self.entry_rules {
        let signal = self.evaluate_rule(rule, &evaluations, ...)?;
        // ...
    }
    
    // Шаг 5: Оценка exit rules
    for rule in &self.exit_rules {
        let signal = self.evaluate_rule(rule, &evaluations, ...)?;
        // ...
    }
}
```

#### 4.1. Оценка условий (`evaluate_conditions()`)

**КРИТИЧЕСКАЯ ПРОБЛЕМА:** На каждой свечке проходимся по **всем** условиям и каждое обрабатывает **весь** массив данных.

```rust
// Код из src/strategy/builder.rs:72-110
async fn evaluate_conditions(&self, context: &StrategyContext) -> Result<HashMap<String, ConditionEvaluation>, StrategyError> {
    let mut result = HashMap::new();
    
    // ЦИКЛ ПО ВСЕМ УСЛОВИЯМ - выполняется на каждой свечке
    for condition in &self.conditions {
        let input = context.prepare_condition_input(condition)?;
        let timeframe_data = context.timeframe(&condition.timeframe)?;
        
        // ВЫЗОВ check() - обрабатывает ВЕСЬ массив данных
        let raw = condition.condition.check(input).await?;
        
        // Из результата берется только значение для текущего индекса
        let idx = self.resolve_index(timeframe_data.index(), raw.signals.len());
        let satisfied = raw.signals.get(idx).copied().unwrap_or(false);
        // ...
    }
}
```

**Пример работы условия:**

```rust
// Код из src/condition/conditions.rs:558-617
async fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData> {
    let (data1, data2) = match input { ... };
    let min_len = std::cmp::min(data1.len(), data2.len());
    
    // ОБРАБОТКА ВСЕГО МАССИВА ДАННЫХ
    for i in 1..min_len {
        let signal = data1[i] < data2[i] && data1[i - 1] >= data2[i - 1];
        signals.push(signal);
        // ... вычисление strengths, directions для ВСЕХ индексов
    }
    
    // Возврат результата для всего массива
    Ok(ConditionResultData { signals, strengths, directions, ... })
}
```

**Проблема:** 
- У нас N свечек в истории
- У нас M условий в стратегии
- На каждой свечке мы проходимся по всем M условиям
- Каждое условие обрабатывает весь массив из N элементов
- **Общая сложность: O(N × M × N) = O(N² × M)**
- Но нам нужно только значение для текущего индекса!

**Ожидаемое поведение:**
- Индикаторы уже вычислены заранее для всех свечек
- На каждой свечке нам нужно только проверить условия для текущего индекса
- Не нужно пересчитывать условие для всех свечек каждый раз

### 5. Подготовка входных данных для условий (`prepare_condition_input()`)

На каждой свечке для каждого условия:

```rust
// Код из src/strategy/context.rs:246-313
pub fn prepare_condition_input(&self, condition: &PreparedCondition) -> Result<ConditionInputData, StrategyError> {
    // Разрешение источников данных (индикаторы, цены)
    let primary_series = self.resolve_series(timeframe, primary)?; // ВЕСЬ массив
    let secondary_series = self.resolve_series(timeframe, secondary)?; // ВЕСЬ массив
    
    // Возврат ВСЕГО массива данных
    Ok(ConditionInputData::dual(primary_series, secondary_series))
}
```

**Проблема:** В условие передается весь массив данных, хотя нужно только значение для текущего индекса (или небольшое окно).

### 6. Оценка правил (`evaluate_rule()`)

После оценки всех условий, правила проверяют комбинации условий:

```rust
// Код из src/strategy/builder.rs:122-219
fn evaluate_rule(&self, rule: &StrategyRuleSpec, evaluations: &HashMap<String, ConditionEvaluation>, ...) -> Result<Option<StrategySignal>, StrategyError> {
    // Проверка всех условий правила
    for condition_id in &rule.conditions {
        let evaluation = evaluations.get(condition_id)?;
        // Использование только значения для текущего индекса
        if evaluation.satisfied { ... }
    }
    
    // Применение логики правила (All, Any, Weighted и т.д.)
    let satisfied = match rule.logic {
        RuleLogic::All => satisfied_count == rule.conditions.len(),
        RuleLogic::Any => satisfied_count > 0,
        // ...
    };
}
```

## Текущие неоптимальности

### 1. Повторная обработка всего массива данных в условиях

**Проблема:**
- На каждой свечке каждое условие обрабатывает весь массив данных
- Результат вычисляется для всех индексов, но используется только текущий

**Пример:**
- История: 10,000 свечек
- Условий: 5
- На свечке #5000:
  - Условие 1: обрабатывает 10,000 элементов → берет только индекс 5000
  - Условие 2: обрабатывает 10,000 элементов → берет только индекс 5000
  - ... и так далее

**Потери производительности:**
- Лишние вычисления для уже обработанных индексов
- Лишние аллокации памяти для результатов

### 2. Отсутствие кэширования результатов условий

**Проблема:**
- Результаты условий вычисляются заново на каждой свечке
- Для статических условий (не зависящих от времени) можно кэшировать

### 3. Последовательная обработка условий

**Проблема:**
- Условия обрабатываются последовательно
- Независимые условия можно обрабатывать параллельно

## Временная сложность

### Текущая реализация

```
T = N × (M × N_cond + P_stop + P_take + R_entry + R_exit)

где:
N = количество свечек
M = количество условий
N_cond = среднее количество элементов, обрабатываемых условием (≈ N)
P_stop = количество stop handlers
P_take = количество take handlers
R_entry = количество entry rules
R_exit = количество exit rules

Приблизительно: T ≈ O(N² × M)
```

### Оптимизированная версия (ожидаемая)

```
T = N_precompute + N × (M + P_stop + P_take + R_entry + R_exit)

где:
N_precompute = одноразовый расчет условий для всех свечек (≈ N × M)

Приблизительно: T ≈ O(N × M)
```

## Пути оптимизации

### 1. Предварительный расчет условий

Как индикаторы, условия тоже можно рассчитать заранее для всех свечек:

```rust
// В populate_indicators() или новом методе populate_conditions()
async fn populate_conditions(&mut self) -> Result<(), StrategyExecutionError> {
    // Группировка условий по таймфреймам
    let mut grouped: HashMap<TimeFrame, Vec<&PreparedCondition>> = HashMap::new();
    
    for condition in &self.strategy.conditions() {
        grouped.entry(condition.timeframe.clone()).or_default().push(condition);
    }
    
    // Для каждого таймфрейма
    for (timeframe, conditions) in grouped {
        let frame = self.feed.frames.get(&timeframe)?;
        let context = self.build_full_context(&timeframe, frame)?;
        
        // Вычисление всех условий для всего массива
        for condition in conditions {
            let input = context.prepare_condition_input(condition)?;
            let result = condition.condition.check(input).await?;
            
            // Сохранение результата
            self.store_condition_result(&timeframe, &condition.id, result)?;
        }
    }
}
```

### 2. Инкрементальная оценка условий

Для условий, которые зависят только от текущего и предыдущего значения, можно использовать инкрементальный подход:

```rust
// Вместо обработки всего массива, обрабатывать только новое значение
async fn check_incremental(&self, prev_value: bool, current_data: &[f32]) -> bool {
    // Обработка только текущего значения на основе предыдущего
}
```

### 3. Кэширование результатов

Кэшировать результаты условий по таймфреймам и индексам.

### 4. Параллельная обработка

Независимые условия обрабатывать параллельно.

## Пример работы на конкретных данных

### Исходные данные:
- История: 10,000 свечек
- Индикаторы: 3 (SMA_10, SMA_30, EMA_50)
- Условия: 5 (cross_above, cross_below, price_above_sma, ...)
- Правила: 2 entry, 2 exit

### Текущий процесс:

1. **Инициализация (1 раз):**
   - Вычисление SMA_10 для 10,000 свечек
   - Вычисление SMA_30 для 10,000 свечек
   - Вычисление EMA_50 для 10,000 свечек
   - Время: ~10ms

2. **Для каждой свечки (10,000 раз):**
   - Свечка #1:
     - Условие 1: обрабатывает 10,000 элементов → берет индекс 0
     - Условие 2: обрабатывает 10,000 элементов → берет индекс 0
     - Условие 3: обрабатывает 10,000 элементов → берет индекс 0
     - Условие 4: обрабатывает 10,000 элементов → берет индекс 0
     - Условие 5: обрабатывает 10,000 элементов → берет индекс 0
     - Время: ~50ms (5 условий × 10ms каждое)
   
   - Свечка #2:
     - Условие 1: обрабатывает 10,000 элементов → берет индекс 1
     - Условие 2: обрабатывает 10,000 элементов → берет индекс 1
     - ... (то же самое)
     - Время: ~50ms
   
   - ... и так далее для всех 10,000 свечек

3. **Общее время:**
   - Инициализация: 10ms
   - Основной цикл: 10,000 × 50ms = 500,000ms = 500 секунд ≈ 8.3 минуты

### Оптимизированный процесс:

1. **Инициализация (1 раз):**
   - Вычисление индикаторов: ~10ms
   - Вычисление условий для всех свечек: ~50ms (1 раз вместо 10,000 раз)
   - Время: ~60ms

2. **Для каждой свечки (10,000 раз):**
   - Свечка #1:
     - Получение результатов условий для индекса 0 (из кэша)
     - Оценка правил
     - Время: ~0.1ms
   
   - Свечка #2:
     - Получение результатов условий для индекса 1 (из кэша)
     - Оценка правил
     - Время: ~0.1ms
   
   - ... и так далее

3. **Общее время:**
   - Инициализация: 60ms
   - Основной цикл: 10,000 × 0.1ms = 1,000ms = 1 секунда
   - **Итого: ~1 секунда вместо 8.3 минуты**

## Выводы

1. **Индикаторы** вычисляются эффективно - один раз для всех свечек
2. **Условия** вычисляются неэффективно - на каждой свечке обрабатывают весь массив данных
3. **Основная оптимизация:** предварительный расчет условий аналогично индикаторам
4. **Ожидаемый выигрыш:** ускорение в 100-1000 раз в зависимости от размера истории и количества условий

## Следующие шаги

1. Реализовать `populate_conditions()` аналогично `populate_indicators()`
2. Модифицировать `evaluate_conditions()` для использования предвычисленных результатов
3. Добавить инкрементальную оценку для условий, где это возможно
4. Добавить параллельную обработку независимых условий

