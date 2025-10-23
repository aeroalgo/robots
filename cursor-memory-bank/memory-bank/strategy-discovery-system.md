# Система автоматического поиска стратегий

## Обзор системы

Система автоматического поиска стратегий представляет собой революционный подход к созданию торговых стратегий, который автоматически генерирует, тестирует и оптимизирует стратегии без участия человека. Это значительно превосходит возможности OsEngine, который только оптимизирует параметры существующих стратегий.

## Архитектура системы

### 1. Strategy Discovery Layer

#### 1.1. Strategy Generator
**Цель**: Автоматическая генерация всех возможных комбинаций индикаторов и условий

**Компоненты**:
- **Indicator Combinator**: Генерация всех возможных комбинаций индикаторов
- **Condition Builder**: Создание условий на основе индикаторов
- **Parameter Generator**: Генерация диапазонов параметров для индикаторов
- **Logic Generator**: Создание логических операторов (AND, OR, NOT)

**Алгоритм**:
```rust
// Пример генерации стратегий
fn generate_strategies(indicators: Vec<Indicator>, conditions: Vec<Condition>) -> Vec<Strategy> {
    let mut strategies = Vec::new();
    
    // Генерация всех возможных комбинаций
    for indicator_combo in generate_combinations(indicators) {
        for condition_combo in generate_combinations(conditions) {
            for logic in generate_logic_operators() {
                let strategy = Strategy::new(indicator_combo, condition_combo, logic);
                strategies.push(strategy);
            }
        }
    }
    
    strategies
}
```

#### 1.2. Strategy Builder
**Цель**: Создание стратегий из сгенерированных комбинаций

**Компоненты**:
- **Entry Condition Builder**: Создание условий входа в позицию
- **Exit Condition Builder**: Создание условий выхода из позиции
- **Risk Management Builder**: Добавление стоп-лоссов и тейк-профитов
- **Position Sizing Builder**: Определение размера позиции

#### 1.3. Strategy Validator
**Цель**: Валидация созданных стратегий на корректность и логику

**Проверки**:
- **Логическая корректность**: Проверка на противоречия в условиях
- **Параметрическая валидность**: Проверка корректности параметров
- **Производительность**: Базовая проверка на прибыльность
- **Стабильность**: Проверка на стабильность результатов

#### 1.4. Strategy Filter
**Цель**: Фильтрация стратегий по базовым критериям

**Критерии фильтрации**:
- **Минимальная прибыльность**: > 0%
- **Минимальное количество сделок**: > 10
- **Максимальная просадка**: < 50%
- **Коэффициент Шарпа**: > 0.5

### 2. Genetic Algorithm Layer

#### 2.1. Genetic Optimizer
**Цель**: Эволюционная оптимизация стратегий

**Компоненты**:
- **Population Manager**: Управление популяцией стратегий
- **Selection Operator**: Селекция лучших стратегий
- **Crossover Operator**: Скрещивание стратегий
- **Mutation Operator**: Мутация стратегий
- **Fitness Evaluator**: Оценка пригодности стратегий

#### 2.2. Population Management
**Цель**: Управление популяцией стратегий

**Функции**:
- **Инициализация популяции**: Создание начальной популяции
- **Размер популяции**: Динамическое управление размером
- **Диверсификация**: Поддержание разнообразия в популяции
- **Элитизм**: Сохранение лучших стратегий

#### 2.3. Fitness Function
**Цель**: Оценка пригодности стратегий

**Метрики**:
- **Sharpe Ratio**: Коэффициент Шарпа
- **Maximum Drawdown**: Максимальная просадка
- **Win Rate**: Процент прибыльных сделок
- **Profit Factor**: Фактор прибыли
- **Calmar Ratio**: Коэффициент Кальмара

**Формула**:
```rust
fn calculate_fitness(strategy: &Strategy) -> f64 {
    let sharpe = strategy.sharpe_ratio;
    let drawdown = strategy.max_drawdown;
    let win_rate = strategy.win_rate;
    let profit_factor = strategy.profit_factor;
    
    // Взвешенная сумма метрик
    sharpe * 0.4 + (1.0 - drawdown) * 0.3 + win_rate * 0.2 + profit_factor * 0.1
}
```

#### 2.4. Multi-objective Optimization
**Цель**: Оптимизация по нескольким критериям одновременно

**Методы**:
- **Pareto Front**: Построение парето-фронта
- **NSGA-II**: Non-dominated Sorting Genetic Algorithm
- **MOEA/D**: Multi-objective Evolutionary Algorithm based on Decomposition
- **SPEA2**: Strength Pareto Evolutionary Algorithm 2

### 3. Advanced Testing Layer

#### 3.1. In-Sample / Out-of-Sample Testing
**Цель**: Разделение данных на обучающую и тестовую выборки

**Методы разделения**:
- **Fixed Split**: Фиксированное разделение (70% IS, 30% OOS)
- **Rolling Window**: Скользящее окно
- **Expanding Window**: Расширяющееся окно
- **Purged K-Fold**: Очищенная K-кратная кросс-валидация

**Реализация**:
```rust
fn split_data(data: &DataFrame, split_ratio: f64) -> (DataFrame, DataFrame) {
    let split_index = (data.len() as f64 * split_ratio) as usize;
    let in_sample = data.slice(0..split_index);
    let out_of_sample = data.slice(split_index..);
    (in_sample, out_of_sample)
}
```

#### 3.2. Monte Carlo Testing
**Цель**: Симуляции случайных сценариев для оценки стабильности

**Типы тестов**:
- **Trade Order Randomization**: Случайное изменение порядка сделок
- **Trade Skipping**: Случайный пропуск сделок
- **Parameter Randomization**: Случайное изменение параметров
- **Data Randomization**: Случайное изменение исторических данных

**Реализация**:
```rust
fn monte_carlo_test(strategy: &Strategy, iterations: usize) -> MonteCarloResults {
    let mut results = Vec::new();
    
    for _ in 0..iterations {
        // Случайное изменение порядка сделок
        let randomized_trades = randomize_trade_order(&strategy.trades);
        
        // Пересчет метрик
        let metrics = calculate_metrics(&randomized_trades);
        results.push(metrics);
    }
    
    MonteCarloResults::new(results)
}
```

#### 3.3. OOS/IS Ratios Analysis
**Цель**: Анализ соотношения метрик OOS к IS для выявления переобучения

**Метрики для анализа**:
- **Return Ratio**: OOS Return / IS Return
- **Sharpe Ratio**: OOS Sharpe / IS Sharpe
- **Drawdown Ratio**: OOS Drawdown / IS Drawdown
- **Win Rate Ratio**: OOS Win Rate / IS Win Rate

**Интерпретация**:
- **Ratio > 0.8**: Хорошая стабильность
- **Ratio 0.5-0.8**: Умеренная стабильность
- **Ratio < 0.5**: Возможное переобучение

#### 3.4. Multi-Market Testing
**Цель**: Тестирование стратегий на множественных рынках

**Рынки для тестирования**:
- **Криптовалютные**: BTC, ETH, ADA, DOT
- **Фондовые**: AAPL, MSFT, GOOGL, TSLA
- **Форекс**: EUR/USD, GBP/USD, USD/JPY
- **Товарные**: Gold, Silver, Oil, Gas

**Критерии успеха**:
- **Consistency**: Стабильность результатов на разных рынках
- **Diversification**: Разнообразие доходности
- **Risk-Adjusted Returns**: Доходность с учетом риска

#### 3.5. Walk-Forward Analysis
**Цель**: Скользящее окно для валидации стратегий

**Параметры**:
- **Training Period**: Период обучения (например, 2 года)
- **Testing Period**: Период тестирования (например, 6 месяцев)
- **Step Size**: Шаг перемещения окна (например, 3 месяца)

**Реализация**:
```rust
fn walk_forward_analysis(strategy: &Strategy, data: &DataFrame) -> WalkForwardResults {
    let mut results = Vec::new();
    let training_period = 24 * 30; // 2 года в днях
    let testing_period = 6 * 30;   // 6 месяцев в днях
    let step_size = 3 * 30;        // 3 месяца в днях
    
    let mut start = 0;
    while start + training_period + testing_period < data.len() {
        let training_data = data.slice(start..start + training_period);
        let testing_data = data.slice(start + training_period..start + training_period + testing_period);
        
        // Оптимизация на обучающих данных
        let optimized_strategy = optimize_strategy(strategy, &training_data);
        
        // Тестирование на тестовых данных
        let test_results = backtest_strategy(&optimized_strategy, &testing_data);
        results.push(test_results);
        
        start += step_size;
    }
    
    WalkForwardResults::new(results)
}
```

## Детальная конфигурация генетического алгоритма

### 2.5. Genetic Options (Параметры генетического алгоритма)

#### 2.5.1. Max # of Generations
**Описание**: Количество поколений для эволюции популяции
**Рекомендуемые значения**: 5-100
**Логика**: Не рекомендуется использовать слишком много поколений, лучше перезапустить эволюцию с нуля

**Реализация**:
```rust
#[derive(Debug, Clone)]
pub struct GeneticConfig {
    pub max_generations: u32,        // 5-100
    pub population_size: u32,        // 10-100+
    pub crossover_probability: f64,  // 0.0-1.0
    pub mutation_probability: f64,   // 0.0-1.0
}

impl Default for GeneticConfig {
    fn default() -> Self {
        Self {
            max_generations: 50,
            population_size: 50,
            crossover_probability: 0.8,
            mutation_probability: 0.1,
        }
    }
}
```

#### 2.5.2. Population Size
**Описание**: Размер популяции на одном острове
**Рекомендуемые значения**: 10-100 или больше
**Важно**: Общая популяция = (количество островов) × (размер популяции)

**Реализация**:
```rust
pub struct Population {
    pub strategies: Vec<Strategy>,
    pub size: u32,
    pub island_id: u32,
}

impl Population {
    pub fn new(size: u32, island_id: u32) -> Self {
        Self {
            strategies: Vec::with_capacity(size as usize),
            size,
            island_id,
        }
    }
    
    pub fn total_population_size(&self, num_islands: u32) -> u32 {
        num_islands * self.size
    }
}
```

#### 2.5.3. Crossover and Mutation Probability
**Описание**: Вероятность генетических операций
**Рекомендации**: Экспериментировать с значениями, увеличение мутации генерирует более разнообразные стратегии

**Реализация**:
```rust
pub struct GeneticOperators {
    pub crossover_probability: f64,
    pub mutation_probability: f64,
}

impl GeneticOperators {
    pub fn should_crossover(&self) -> bool {
        rand::random::<f64>() < self.crossover_probability
    }
    
    pub fn should_mutate(&self) -> bool {
        rand::random::<f64>() < self.mutation_probability
    }
}
```

### 2.6. Island Options (Параметры островов)

#### 2.6.1. Islands
**Описание**: Количество отдельных островов
**Рекомендуемые значения**: 1-10
**Логика**: Острова позволяют запускать генетическую эволюцию отдельно в изолированных островах с периодической миграцией особей

**Реализация**:
```rust
pub struct IslandManager {
    pub islands: Vec<Island>,
    pub migration_interval: u32,    // каждые X поколений
    pub migration_rate: f64,        // процент мигрирующих стратегий
}

pub struct Island {
    pub id: u32,
    pub population: Population,
    pub generation: u32,
    pub best_fitness: f64,
}

impl IslandManager {
    pub fn new(num_islands: u32, population_size: u32) -> Self {
        let mut islands = Vec::new();
        for i in 0..num_islands {
            islands.push(Island {
                id: i,
                population: Population::new(population_size, i),
                generation: 0,
                best_fitness: 0.0,
            });
        }
        
        Self {
            islands,
            migration_interval: 10,  // каждые 10 поколений
            migration_rate: 0.1,     // 10% стратегий
        }
    }
}
```

#### 2.6.2. Migrate every Xth generation
**Описание**: Как часто мигрировать особи между островами
**Рекомендуемые значения**: каждые 10 поколений
**Логика**: Миграция может "разблокировать" остров, застрявший в локальном минимуме

**Реализация**:
```rust
impl IslandManager {
    pub fn should_migrate(&self, generation: u32) -> bool {
        generation % self.migration_interval == 0 && generation > 0
    }
    
    pub fn migrate_strategies(&mut self) {
        if self.islands.len() < 2 {
            return;
        }
        
        for i in 0..self.islands.len() {
            let source_island = &mut self.islands[i];
            let num_to_migrate = (source_island.population.size as f64 * self.migration_rate) as u32;
            
            // Выбираем лучшие стратегии для миграции
            let mut strategies_to_migrate = source_island.population.select_best(num_to_migrate);
            
            // Мигрируем на следующий остров
            let target_island_id = (i + 1) % self.islands.len();
            self.islands[target_island_id].population.add_strategies(&mut strategies_to_migrate);
        }
    }
}
```

#### 2.6.3. Population migration rate
**Описание**: Сколько стратегий в популяции будет мигрировать
**Рекомендации**: 1-5 стратегий в зависимости от размера популяции
- Для размера популяции = 10: 10-20%
- Для размера популяции = 100: 1-5%

**Реализация**:
```rust
impl Population {
    pub fn calculate_migration_rate(&self) -> f64 {
        match self.size {
            1..=20 => 0.15,   // 15% для малых популяций
            21..=50 => 0.10,  // 10% для средних популяций
            _ => 0.05,        // 5% для больших популяций
        }
    }
    
    pub fn select_best(&self, count: u32) -> Vec<Strategy> {
        let mut strategies = self.strategies.clone();
        strategies.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());
        strategies.into_iter().take(count as usize).collect()
    }
}
```

### 2.7. Initial Population Generation (Генерация начальной популяции)

#### 2.7.1. Use strategies from Initial population databank
**Описание**: Использование существующих стратегий как начальная популяция
**Логика**: Если недостаточно стратегий, остальные генерируются случайно

**Реализация**:
```rust
pub struct InitialPopulationGenerator {
    pub use_existing_strategies: bool,
    pub existing_strategies: Vec<Strategy>,
    pub decimation_coefficient: u32,
    pub filter_config: PopulationFilter,
}

impl InitialPopulationGenerator {
    pub fn generate_initial_population(&self, target_size: u32) -> Vec<Strategy> {
        let mut population = Vec::new();
        
        if self.use_existing_strategies {
            // Добавляем существующие стратегии
            population.extend(self.existing_strategies.clone());
        }
        
        // Генерируем недостающие стратегии
        let remaining_size = target_size.saturating_sub(population.len() as u32);
        if remaining_size > 0 {
            let strategies_to_generate = remaining_size * self.decimation_coefficient;
            let mut generated = self.generate_random_strategies(strategies_to_generate);
            
            // Применяем фильтр
            generated = self.filter_config.apply(generated);
            
            // Выбираем лучшие
            generated.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());
            population.extend(generated.into_iter().take(remaining_size as usize));
        }
        
        population
    }
}
```

#### 2.7.2. Generated decimation coefficient
**Описание**: Коэффициент децимации для улучшения качества начальной популяции
**Логика**: Генерируется X раз больше стратегий, из которых выбираются лучшие

**Реализация**:
```rust
impl InitialPopulationGenerator {
    pub fn generate_with_decimation(&self, target_size: u32) -> Vec<Strategy> {
        let strategies_to_generate = target_size * self.decimation_coefficient;
        let mut generated = self.generate_random_strategies(strategies_to_generate);
        
        // Применяем фильтр
        generated = self.filter_config.apply(generated);
        
        // Сортируем по пригодности и выбираем лучшие
        generated.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());
        generated.into_iter().take(target_size as usize).collect()
    }
}
```

#### 2.7.3. Filter generated initial population
**Описание**: Фильтр для установки базовых минимумов для стратегий в начальном поколении
**Рекомендации**: Не быть слишком строгим, рекомендуется фильтровать только по количеству сделок

**Реализация**:
```rust
#[derive(Debug, Clone)]
pub struct PopulationFilter {
    pub min_trades: u32,           // Минимальное количество сделок
    pub min_profit: f64,           // Минимальная прибыльность
    pub max_drawdown: f64,         // Максимальная просадка
    pub min_sharpe_ratio: f64,     // Минимальный коэффициент Шарпа
}

impl PopulationFilter {
    pub fn apply(&self, strategies: Vec<Strategy>) -> Vec<Strategy> {
        strategies.into_iter()
            .filter(|strategy| {
                strategy.num_trades >= self.min_trades &&
                strategy.total_return >= self.min_profit &&
                strategy.max_drawdown <= self.max_drawdown &&
                strategy.sharpe_ratio >= self.min_sharpe_ratio
            })
            .collect()
    }
    
    pub fn default_initial() -> Self {
        Self {
            min_trades: 5,         // Минимум 5 сделок
            min_profit: 0.0,       // Любая прибыльность
            max_drawdown: 0.8,     // Максимум 80% просадка
            min_sharpe_ratio: 0.0, // Любой коэффициент Шарпа
        }
    }
}
```

### 2.8. Evolution Management (Управление эволюцией)

#### 2.8.1. Start again when finished
**Описание**: Перезапуск процесса построения после завершения
**Логика**: Позволяет запускать процесс автономно, эволюционируя все больше популяций

**Реализация**:
```rust
pub struct EvolutionManager {
    pub auto_restart: bool,
    pub max_restarts: Option<u32>,
    pub restart_count: u32,
}

impl EvolutionManager {
    pub fn should_restart(&mut self) -> bool {
        if !self.auto_restart {
            return false;
        }
        
        if let Some(max) = self.max_restarts {
            if self.restart_count >= max {
                return false;
            }
        }
        
        self.restart_count += 1;
        true
    }
}
```

#### 2.8.2. Restart evolution fitness if stagnating
**Описание**: Перезапуск эволюции при застое в пригодности
**Логика**: Если популяция в целом не улучшается, лучше начать заново

**Реализация**:
```rust
pub struct FitnessStagnationDetector {
    pub stagnation_threshold: u32,  // Количество поколений без улучшения
    pub improvement_threshold: f64, // Минимальное улучшение для сброса счетчика
    pub generations_without_improvement: u32,
    pub best_fitness: f64,
}

impl FitnessStagnationDetector {
    pub fn check_stagnation(&mut self, current_best_fitness: f64) -> bool {
        if current_best_fitness > self.best_fitness + self.improvement_threshold {
            self.best_fitness = current_best_fitness;
            self.generations_without_improvement = 0;
            false
        } else {
            self.generations_without_improvement += 1;
            self.generations_without_improvement >= self.stagnation_threshold
        }
    }
}
```

### 2.9. "Fresh Blood" (Свежая кровь)

#### 2.9.1. Detect same strategies and replace them
**Описание**: Обнаружение одинаковых стратегий и замена их новыми
**Логика**: Помогает сделать стратегии более разнообразными

**Реализация**:
```rust
pub struct StrategyDiversityManager {
    pub similarity_threshold: f64,
    pub replacement_rate: f64,
}

impl StrategyDiversityManager {
    pub fn detect_and_replace_duplicates(&mut self, population: &mut Vec<Strategy>) {
        let mut to_replace = Vec::new();
        
        for i in 0..population.len() {
            for j in (i + 1)..population.len() {
                if self.are_similar(&population[i], &population[j]) {
                    to_replace.push(j);
                }
            }
        }
        
        // Заменяем дубликаты новыми стратегиями
        for &index in &to_replace {
            population[index] = self.generate_new_strategy();
        }
    }
    
    fn are_similar(&self, strategy1: &Strategy, strategy2: &Strategy) -> bool {
        // Сравнение параметров стратегий
        let similarity = self.calculate_similarity(strategy1, strategy2);
        similarity > self.similarity_threshold
    }
}
```

#### 2.9.2. Replace X% of weakest strategies
**Описание**: Замена X% самых слабых стратегий новыми
**Логика**: Заменяет худшие стратегии новыми для увеличения разнообразия

**Реализация**:
```rust
impl StrategyDiversityManager {
    pub fn replace_weakest_strategies(&mut self, population: &mut Vec<Strategy>) {
        let num_to_replace = (population.len() as f64 * self.replacement_rate) as usize;
        
        // Сортируем по пригодности (худшие в начале)
        population.sort_by(|a, b| a.fitness_score.partial_cmp(&b.fitness_score).unwrap());
        
        // Заменяем худшие стратегии
        for i in 0..num_to_replace {
            population[i] = self.generate_new_strategy();
        }
    }
}
```

## Преимущества системы

### По сравнению с OsEngine:
1. **Автоматический поиск**: Генерация всех возможных стратегий
2. **Генетическая оптимизация**: Эволюция стратегий с детальной конфигурацией
3. **Расширенное тестирование**: IS/OOS, Monte Carlo, Multi-Market
4. **Объективная оценка**: Исключение человеческого фактора
5. **Масштабируемость**: Обработка миллионов стратегий

### По сравнению с конкурентами:
1. **Open Source**: Полная прозрачность алгоритмов
2. **Rust производительность**: Высокая скорость вычислений
3. **Модульная архитектура**: Легкая расширяемость
4. **Современные методы**: Использование последних достижений в области ML

## Технические требования

### Производительность:
- **Параллельные вычисления**: Использование всех доступных ядер CPU
- **GPU ускорение**: CUDA/OpenCL для сложных вычислений
- **Распределенные вычисления**: Кластеры для обработки больших объемов
- **Кэширование**: Многоуровневое кэширование результатов

### Масштабируемость:
- **Горизонтальное масштабирование**: Добавление новых узлов
- **Вертикальное масштабирование**: Увеличение мощности узлов
- **Автоматическое масштабирование**: Динамическое управление ресурсами
- **Балансировка нагрузки**: Равномерное распределение задач

### Надежность:
- **Отказоустойчивость**: Продолжение работы при сбоях
- **Восстановление**: Автоматическое восстановление после сбоев
- **Мониторинг**: Отслеживание состояния системы
- **Логирование**: Детальное логирование всех операций

## План реализации

### Этап 1: Базовая система (2-3 месяца)
1. **Strategy Generator**: Генерация простых стратегий
2. **Strategy Validator**: Базовая валидация
3. **Strategy Filter**: Простая фильтрация
4. **Базовое тестирование**: IS/OOS тестирование

### Этап 2: Генетическая оптимизация (2-3 месяца)
1. **Genetic Optimizer**: Базовый генетический алгоритм
2. **Population Management**: Управление популяцией
3. **Fitness Function**: Оценка пригодности
4. **Multi-objective Optimization**: Многоцелевая оптимизация
5. **Island Management**: Управление островами
6. **Migration System**: Система миграции

### Этап 3: Расширенное тестирование (2-3 месяца)
1. **Monte Carlo Testing**: Симуляции случайных сценариев
2. **Multi-Market Testing**: Тестирование на множественных рынках
3. **Walk-Forward Analysis**: Скользящее окно валидации
4. **OOS/IS Ratios**: Анализ переобучения

### Этап 4: Оптимизация и масштабирование (1-2 месяца)
1. **Производительность**: Оптимизация алгоритмов
2. **Распределенные вычисления**: Кластерная обработка
3. **GPU ускорение**: Использование GPU
4. **Мониторинг**: Система мониторинга

## Заключение

Система автоматического поиска стратегий с детальной конфигурацией генетического алгоритма представляет собой революционный подход к созданию торговых стратегий, который значительно превосходит возможности существующих решений. Она обеспечивает:

1. **Полную автоматизацию** процесса создания стратегий
2. **Объективную оценку** без человеческого фактора
3. **Высокую производительность** благодаря Rust
4. **Расширенное тестирование** для валидации стратегий
5. **Масштабируемость** для обработки больших объемов данных
6. **Детальную конфигурацию** генетического алгоритма
7. **Управление островами** для параллельной эволюции
8. **Систему миграции** для обмена лучшими стратегиями

Эта система позволит создать торговые стратегии, которые будут более стабильными, прибыльными и адаптивными к изменяющимся рыночным условиям.
