# Рефакторинг BacktestExecutor

## Текущее состояние

`BacktestExecutor` в `src/strategy/executor.rs` (1492 строки) отвечает за множество задач:

1. **Управление историческим фидом** (`HistoricalFeed`)
2. **Агрегация таймфреймов** (через `TimeFrameAggregator`)
3. **Расчёт индикаторов** (`populate_indicators`, `populate_auxiliary_indicators`)
4. **Оценка условий** (`populate_conditions`)
5. **Управление позициями** (`PositionManager`)
6. **Управление рисками** (`RiskManager`)
7. **Сбор метрик** (`BacktestAnalytics`)

### Нарушение SRP
Класс имеет слишком много ответственностей, что затрудняет поддержку и тестирование.

## План рефакторинга

### Фаза 1: Выделение FeedManager

Текущая структура `HistoricalFeed` (строки 1007-1098) уже выделена как отдельная структура внутри файла. 

**Действие**: Вынести в `src/strategy/feed_manager.rs`

```rust
pub struct FeedManager {
    frames: HashMap<TimeFrame, Arc<QuoteFrame>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: Option<TimeFrame>,
    higher_timeframe_timestamps: HashMap<TimeFrame, Vec<i64>>,
    cached_aligned_timestamps: HashMap<TimeFrame, i64>,
}

impl FeedManager {
    pub fn new() -> Self;
    pub fn with_frames(frames: HashMap<TimeFrame, Arc<QuoteFrame>>) -> Self;
    pub fn reset(&mut self);
    pub fn step(&mut self, context: &mut StrategyContext) -> bool;
    pub fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32>;
    pub fn is_higher_timeframe(higher: &TimeFrame, lower: &TimeFrame) -> bool;
    // ... остальные методы
}
```

### Фаза 2: Выделение IndicatorEngine

Методы `populate_indicators`, `populate_auxiliary_indicators`, `populate_custom_data` должны быть вынесены в отдельный компонент.

**Действие**: Создать `src/strategy/indicator_engine.rs`

```rust
pub struct IndicatorEngine {
    runtime: IndicatorRuntimeEngine,
}

impl IndicatorEngine {
    pub fn new() -> Self;
    
    pub fn populate_indicators(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), StrategyExecutionError>;
    
    pub fn populate_auxiliary_indicators(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), StrategyExecutionError>;
    
    pub fn populate_custom_data(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), StrategyExecutionError>;
}
```

### Фаза 3: Выделение ConditionEvaluator

Метод `populate_conditions` должен быть вынесен в отдельный компонент.

**Действие**: Создать `src/strategy/condition_evaluator.rs`

```rust
pub struct ConditionEvaluator;

impl ConditionEvaluator {
    pub fn new() -> Self;
    
    pub fn populate_conditions(
        &self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), StrategyExecutionError>;
}
```

### Фаза 4: Рефакторинг BacktestExecutor

После выделения компонентов, `BacktestExecutor` станет координатором:

```rust
pub struct BacktestExecutor {
    feed_manager: FeedManager,
    indicator_engine: IndicatorEngine,
    condition_evaluator: ConditionEvaluator,
    position_manager: PositionManager,  // уже существует
    risk_manager: RiskManager,          // уже существует
    analytics: BacktestAnalytics,       // уже существует
    strategy: Box<dyn Strategy>,
    context: StrategyContext,
    config: BacktestConfig,
}

impl BacktestExecutor {
    pub fn run_backtest(&mut self) -> Result<BacktestReport, StrategyExecutionError> {
        // Инициализация
        self.feed_manager.reset();
        self.position_manager.reset();
        self.risk_manager.reset();
        self.analytics.reset();
        
        // Расчёт индикаторов
        self.indicator_engine.populate_indicators(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;
        
        self.indicator_engine.populate_auxiliary_indicators(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;
        
        self.indicator_engine.populate_custom_data(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;
        
        // Оценка условий
        self.condition_evaluator.populate_conditions(
            self.strategy.as_ref(),
            self.feed_manager.frames(),
            &mut self.context,
        )?;
        
        // Основной цикл
        while self.feed_manager.step(&mut self.context) {
            // ... логика торговли
        }
        
        self.build_report()
    }
}
```

## Существующие компоненты (не требуют изменений)

1. **PositionManager** (`src/position/manager.rs`) - управление позициями
2. **RiskManager** (`src/risk/manager.rs`) - управление рисками и стопами
3. **BacktestAnalytics** (`src/metrics/backtest.rs`) - сбор метрик

## Преимущества рефакторинга

1. **Тестируемость**: Каждый компонент можно тестировать изолированно
2. **Переиспользуемость**: Компоненты можно использовать в других контекстах
3. **Читаемость**: Меньшие файлы проще поддерживать
4. **Расширяемость**: Легче добавлять новую функциональность

## Риски

1. Изменение публичного API может сломать существующий код
2. Необходимо сохранить обратную совместимость
3. Тесты должны проходить после каждой фазы

## Рекомендации

1. Выполнять рефакторинг поэтапно (по одной фазе за раз)
2. Запускать тесты после каждого изменения
3. Сохранять старые методы как deprecated до полной миграции
4. Документировать изменения в CHANGELOG

## Статус

- [ ] Фаза 1: FeedManager
- [ ] Фаза 2: IndicatorEngine
- [ ] Фаза 3: ConditionEvaluator
- [ ] Фаза 4: Рефакторинг BacktestExecutor
