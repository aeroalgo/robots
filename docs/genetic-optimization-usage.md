# Запуск генетической оптимизации

## Быстрый старт

### 1. Запуск примера

```bash
cargo run --example genetic_optimization
```

### 2. Базовое использование в коде

```rust
use robots::optimization::*;
use robots::data_model::types::{Symbol, TimeFrame};
use std::collections::HashMap;

// 1. Подготовка данных
let symbol = Symbol::from_descriptor("AFLT.MM");
let base_timeframe = TimeFrame::from_identifier("60");
let mut frames = HashMap::new();
// ... загрузка данных в frames

// 2. Настройка конфигурации
let config = GeneticAlgorithmConfig {
    population_size: 50,
    max_generations: 10,
    crossover_rate: 0.7,
    mutation_rate: 0.1,
    elitism_count: 5,
    islands_count: 2,
    migration_interval: 5,
    migration_rate: 0.05,
    fitness_thresholds: FitnessThresholds::default(),
    fitness_weights: FitnessWeights::default(),
    use_existing_strategies: false,
    decimation_coefficient: 2.0,
    filter_initial_population: true,
    restart_on_finish: false,
    restart_on_stagnation: true,
    fresh_blood_rate: 0.1,
    detect_duplicates: true,
};

// 3. Генерация начальной популяции
let generator = InitialPopulationGenerator::new(
    config.clone(),
    frames.clone(),
    base_timeframe.clone(),
);
let initial_population = generator.generate(None).await?;

// 4. Создание островов
let mut initial_populations = vec![initial_population.clone()];
for i in 1..config.islands_count {
    let mut pop = initial_population.clone();
    pop.island_id = Some(i);
    initial_populations.push(pop);
}
let mut island_manager = IslandManager::new(config.clone(), initial_populations);

// 5. Создание генетического алгоритма
let genetic_algorithm = GeneticAlgorithm::new(
    config.clone(),
    frames.clone(),
    base_timeframe.clone(),
);

// 6. Запуск эволюции
for generation in 0..config.max_generations {
    let islands = island_manager.get_all_islands_mut();
    for island in islands.iter_mut() {
        genetic_algorithm.evolve_generation(island).await?;
    }
    
    // Миграция между островами
    if generation > 0 && (generation + 1) % config.migration_interval == 0 {
        let islands = island_manager.get_all_islands_mut();
        let migration_system = MigrationSystem::new(config.clone());
        migration_system.migrate_between_islands(islands)?;
    }
}

// 7. Получение лучших результатов
let islands = island_manager.get_all_islands();
for island in islands {
    let best = island.individuals.iter()
        .max_by(|a, b| {
            a.strategy.fitness.unwrap_or(0.0)
                .partial_cmp(&b.strategy.fitness.unwrap_or(0.0))
                .unwrap()
        });
    // ... обработка лучшей стратегии
}
```

## Параметры конфигурации

### Основные параметры

- `population_size` - размер популяции (рекомендуется 30-100)
- `max_generations` - максимальное количество поколений
- `crossover_rate` - вероятность скрещивания (0.0-1.0, рекомендуется 0.7)
- `mutation_rate` - вероятность мутации (0.0-1.0, рекомендуется 0.1)
- `elitism_count` - количество элитных особей, сохраняемых в каждом поколении

### Параметры островов

- `islands_count` - количество островов (1-10, рекомендуется 1-5)
- `migration_interval` - интервал миграции в поколениях (рекомендуется 10)
- `migration_rate` - процент мигрирующих особей (0.01-0.05)

### Пороговые значения fitness

- `min_sharpe_ratio` - минимальный Sharpe ratio
- `max_drawdown_pct` - максимальная просадка в процентах
- `min_win_rate` - минимальный процент прибыльных сделок
- `min_profit_factor` - минимальный profit factor
- `min_total_profit` - минимальная общая прибыль
- `min_trades_count` - минимальное количество сделок
- `min_cagr` - минимальный CAGR

### Веса метрик для fitness

- `sharpe_ratio_weight` - вес Sharpe ratio (по умолчанию 0.3)
- `profit_factor_weight` - вес profit factor (по умолчанию 0.25)
- `win_rate_weight` - вес win rate (по умолчанию 0.15)
- `cagr_weight` - вес CAGR (по умолчанию 0.2)
- `drawdown_penalty` - штраф за просадку (по умолчанию 0.05)
- `trades_count_bonus` - бонус за количество сделок (по умолчанию 0.05)

## Примеры конфигураций

### Быстрая оптимизация (для тестирования)

```rust
GeneticAlgorithmConfig {
    population_size: 20,
    max_generations: 5,
    islands_count: 1,
    // ... остальные параметры
}
```

### Полная оптимизация

```rust
GeneticAlgorithmConfig {
    population_size: 100,
    max_generations: 50,
    islands_count: 5,
    migration_interval: 10,
    // ... остальные параметры
}
```

### Консервативная оптимизация (строгие пороги)

```rust
GeneticAlgorithmConfig {
    fitness_thresholds: FitnessThresholds {
        min_sharpe_ratio: Some(2.0),
        max_drawdown_pct: Some(15.0),
        min_win_rate: Some(0.55),
        min_profit_factor: Some(2.0),
        // ... остальные пороги
    },
    // ... остальные параметры
}
```

## Мониторинг процесса

Во время выполнения эволюции можно отслеживать:

- Лучший fitness в каждом поколении
- Метрики лучшей стратегии (profit, Sharpe ratio, win rate)
- Количество сделок
- Процесс миграции между островами
- Инъекцию "свежей крови"

## Результаты

После завершения эволюции доступны:

- Лучшие стратегии по каждому острову
- Полные метрики backtest для каждой стратегии
- Параметры оптимизированных стратегий
- История fitness по поколениям

