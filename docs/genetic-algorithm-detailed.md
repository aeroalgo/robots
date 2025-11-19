# Генетический алгоритм оптимизации торговых стратегий - Подробная документация

## Содержание

1. [Общий обзор](#общий-обзор)
2. [Структура данных](#структура-данных)
3. [Инициализация популяции](#инициализация-популяции)
4. [Процесс эволюции поколения](#процесс-эволюции-поколения)
5. [Селекция родителей](#селекция-родителей)
6. [Кроссовер структуры стратегии](#кроссовер-структуры-стратегии)
7. [Кроссовер параметров](#кроссовер-параметров)
8. [Мутация структуры стратегии](#мутация-структуры-стратегии)
9. [Мутация параметров](#мутация-параметров)
10. [Создание новых особей](#создание-новых-особей)
11. [Элитизм и замена слабых особей](#элитизм-и-замена-слабых-особей)
12. [Оценка фитнеса](#оценка-фитнеса)

---

## Общий обзор

Генетический алгоритм оптимизации торговых стратегий представляет собой эволюционный подход к поиску оптимальных стратегий. Алгоритм работает с популяцией стратегий, каждая из которых состоит из двух компонентов:

1. **Структура стратегии (StrategyCandidate)** - определяет какие индикаторы, условия входа/выхода и стоп-обработчики используются
2. **Параметры стратегии (StrategyParameterMap)** - числовые значения параметров индикаторов, условий и стоп-обработчиков

Алгоритм эволюционирует популяцию через следующие операции:
- **Селекция** - выбор лучших стратегий для размножения
- **Кроссовер** - комбинирование структур и параметров двух родительских стратегий
- **Мутация** - случайные изменения в структуре и параметрах
- **Элитизм** - сохранение лучших стратегий без изменений

---

## Структура данных

### StrategyCandidate

Структура стратегии содержит следующие компоненты:

```rust
pub struct StrategyCandidate {
    pub indicators: Vec<IndicatorInfo>,           // Базовые индикаторы
    pub nested_indicators: Vec<NestedIndicator>,  // Вложенные индикаторы
    pub conditions: Vec<ConditionInfo>,           // Условия входа
    pub exit_conditions: Vec<ConditionInfo>,      // Условия выхода
    pub stop_handlers: Vec<StopHandlerInfo>,      // Стоп-обработчики (stop-loss)
    pub take_handlers: Vec<StopHandlerInfo>,       // Тейк-обработчики (take-profit)
    pub timeframes: Vec<TimeFrame>,               // Используемые таймфреймы
    pub config: StrategyDiscoveryConfig,          // Конфигурация
}
```

**Компоненты структуры:**

- **indicators** - список базовых индикаторов, строящихся по цене (например, SMA, RSI, MACD)
- **nested_indicators** - список вложенных индикаторов, строящихся по другим индикаторам
- **conditions** - условия входа в позицию (например, RSI > 70, цена пересекает SMA)
- **exit_conditions** - условия выхода из позиции
- **stop_handlers** - обработчики стоп-лосс (stop-loss)
- **take_handlers** - обработчики тейк-профит (take-profit). Отдельный параметр от stop_handlers
- **timeframes** - список таймфреймов, на которых работает стратегия

### StrategyParameterMap

Параметры стратегии хранятся в виде HashMap, где ключ - это имя параметра, а значение - его значение:

```rust
pub type StrategyParameterMap = HashMap<String, StrategyParamValue>;

pub enum StrategyParamValue {
    Number(f64),      // Числовое значение с плавающей точкой
    Integer(i64),     // Целочисленное значение
    Text(String),     // Текстовое значение
    Flag(bool),       // Булево значение
    List(Vec<StrategyParamValue>), // Список значений
}
```

**Формат ключей параметров:**

- Для индикаторов: `"{indicator_name}_{param_name}"` (например, `"SMA_period"`)
- Для вложенных индикаторов: `"nested_{indicator_name}_{param_name}"` (например, `"nested_RSI_period"`)
- Для условий: `"condition_{condition_id}_{param_name}"` (например, `"condition_1_threshold"`)
- Для стоп-обработчиков: `"stop_{handler_name}_{param_name}"` (например, `"stop_StopLoss_value"`)

### GeneticIndividual

Индивидуум в популяции содержит:

```rust
pub struct GeneticIndividual {
    pub strategy: EvaluatedStrategy,  // Оцененная стратегия
    pub generation: usize,            // Номер поколения
    pub island_id: Option<usize>,     // ID острова (для island model)
}
```

### EvaluatedStrategy

Оцененная стратегия содержит:

```rust
pub struct EvaluatedStrategy {
    pub candidate: Option<StrategyCandidate>,           // Структура стратегии
    pub parameters: StrategyParameterMap,               // Параметры
    pub fitness: Option<f64>,                          // Значение фитнеса
    pub backtest_report: Option<BacktestReport>,       // Отчет бэктеста
}
```

---

## Инициализация популяции

Процесс создания начальной популяции выполняется классом `InitialPopulationGenerator`.

### Шаг 1: Генерация кандидатов стратегий

Если `use_existing_strategies = false`, система генерирует новые кандидаты стратегий:

1. Создается `StrategyDiscoveryEngine` с конфигурацией
2. Собираются доступные индикаторы из реестра
3. Определяются доступные поля цены (Open, High, Low, Close)
4. Определяются доступные операторы условий (GreaterThan, LessThan, CrossesAbove, CrossesBelow)
5. Генерируется заданное количество случайных стратегий через `generate_strategies_random()`

**Количество генерируемых кандидатов:**
```
strategies_to_generate = ceil(population_size / decimation_coefficient)
```

Например, при `population_size = 100` и `decimation_coefficient = 2.0` будет сгенерировано 50 уникальных структур стратегий.

### Шаг 2: Генерация параметров для каждого кандидата

Для каждого кандидата стратегии генерируется несколько наборов случайных параметров:

**Количество наборов параметров на кандидат:**
```
target_size = floor(population_size * decimation_coefficient)
```

Например, при `population_size = 100` и `decimation_coefficient = 2.0` для каждого кандидата будет создано 200 наборов параметров.

**Процесс генерации параметров:**

1. **Параметры базовых индикаторов:**
   - Для каждого индикатора перебираются все его параметры
   - Если параметр помечен как `optimizable`, генерируется случайное значение:
     - Для всех типов параметров (кроме `Boolean`) всегда вызывается `get_optimization_range()` для получения диапазона оптимизации
     - Если диапазон найден, генерируется случайное значение в этом диапазоне с учетом шага
     - Если диапазон не найден, параметр пропускается
     - Для `Integer`: значение из диапазона преобразуется в целое число
     - Для `Float`: значение из диапазона используется как число с плавающей точкой
     - Для `Boolean`: случайное булево значение (без использования диапазона)

2. **Параметры вложенных индикаторов:**
   - Аналогично базовым индикаторам, но с префиксом `"nested_"` в ключе

3. **Параметры условий входа и выхода:**
   - Для каждого условия извлекается имя связанного индикатора (если условие использует индикатор)
   - Для параметра `"threshold"`: если есть `constant_value`, используется оно, иначе вызывается `get_optimization_range()` с типом `Threshold`
   - Для параметров `"percentage"` или `"percent"`: вызывается `get_optimization_range()` с типом `Multiplier`
   - Для остальных параметров: вызывается `get_optimization_range()` с типом `Threshold`
   - Если диапазон найден, генерируется случайное значение в этом диапазоне с учетом шага
   - Если индикатор не найден или диапазон не определен, параметр пропускается

4. **Параметры стоп-обработчиков:**
   - Для всех параметров стоп-обработчиков всегда вызывается `get_stop_optimization_range()` с именем обработчика (`handler_name`) и именем параметра
   - Если диапазон найден, генерируется случайное значение в этом диапазоне с учетом шага
   - Если диапазон не найден, параметр пропускается

### Шаг 3: Оценка и фильтрация

Для каждого набора параметров:

1. Выполняется бэктест стратегии через `evaluator.evaluate_strategy()`
2. Если `filter_initial_population = true`, проверяется соответствие пороговым значениям:
   - Минимальный Sharpe Ratio
   - Максимальная просадка
   - Минимальный win rate
   - Минимальный profit factor
   - Минимальная общая прибыль
   - Минимальное количество сделок
   - Минимальный CAGR
   - Максимальная просадка
3. Вычисляется фитнес через `FitnessFunction::calculate_fitness()` (подробное описание см. раздел [Оценка фитнеса](#оценка-фитнеса))
   
   **Краткое описание вычисления фитнеса:**
   - Фитнес = взвешенная сумма нормализованных метрик:
     - Sharpe Ratio (вес 30%) - нормализуется делением на 3.0 (Sharpe ≥ 3.0 считается отличным)
     - Profit Factor (вес 25%) - нормализуется делением на 5.0 (PF ≥ 5.0 считается отличным)
     - Win Rate (вес 15%) - используется напрямую (уже в диапазоне [0, 1])
     - CAGR (вес 20%) - нормализуется делением на 100.0 (CAGR ≥ 100% считается отличным)
     - Drawdown Penalty (вес 5%) - штраф, вычитается из фитнеса
     - Trades Bonus (вес 5%) - бонус за количество сделок
   - Все метрики нормализуются к диапазону [0, 1]
   - Фитнес всегда ≥ 0.0
4. Создается `GeneticIndividual` с generation = 0

### Шаг 4: Сортировка и отбор

1. Все индивидуумы сортируются по убыванию фитнеса
2. Отбираются первые `population_size` индивидуумов
3. Формируется начальная популяция

---

## Процесс эволюции поколения

Основной процесс эволюции выполняется методом `evolve_generation()` класса `GeneticAlgorithmV3`.

### Общая схема процесса

```
1. Селекция элитных особей
2. Пока не создано достаточно новых особей:
   a. Выбор двух родителей
   b. Кроссовер структуры стратегии
   c. Кроссовер параметров
   d. Мутация структуры стратегии (для обоих потомков)
   e. Мутация параметров (для обоих потомков)
   f. Создание и оценка потомков
3. Замена слабых особей новыми
4. Применение элитизма
5. Увеличение номера поколения
```

### Детальное описание шагов

#### Шаг 1: Селекция элитных особей

```rust
let elites = self.select_elites(population);
```

Метод `select_elites()`:
1. Создает копию списка индивидуумов
2. Сортирует по убыванию фитнеса
3. Возвращает первые `elitism_count` индивидуумов

Элитные особи будут сохранены в следующем поколении без изменений.

#### Шаг 2: Создание новых особей

Цикл продолжается, пока не создано `population_size - elites.len()` новых особей:

```rust
while new_individuals.len() < population.individuals.len() - elites.len()
```

**Для каждой пары родителей:**

1. **Выбор родителей:**
   ```rust
   let parents = self.population_manager.select_parents(population, 2);
   ```
   Используется пропорциональная селекция (roulette wheel selection) на основе фитнеса.

2. **Кроссовер структуры:**
   ```rust
   let (mut child1_candidate, mut child2_candidate) = 
       self.crossover_structure(&cand1, &cand2);
   ```

3. **Кроссовер параметров:**
   ```rust
   let (child1_params, child2_params) = 
       self.population_manager.crossover(parents[0], parents[1]);
   ```

4. **Мутация структуры:**
   ```rust
   Self::mutate_structure(&mut child1_candidate, ...);
   Self::mutate_structure(&mut child2_candidate, ...);
   ```

5. **Мутация параметров:**
   ```rust
   self.population_manager.mutate(&mut child1_params, &child1_candidate);
   self.population_manager.mutate(&mut child2_params, &child2_candidate);
   ```

6. **Создание потомков:**
   ```rust
   let child1 = self.create_individual(child1_candidate, child1_params, ...).await?;
   ```

#### Шаг 3: Замена слабых особей

```rust
self.population_manager.replace_weakest(population, new_individuals);
```

1. Популяция сортируется по возрастанию фитнеса (слабейшие первые)
2. Слабейшие особи заменяются новыми потомками

#### Шаг 4: Применение элитизма

```rust
self.population_manager.apply_elitism(population, elites);
```

1. Популяция сортируется по убыванию фитнеса
2. Элитные особи размещаются в начале списка

#### Шаг 5: Обновление поколения

```rust
population.generation += 1;
```

---

## Селекция родителей

Метод `select_parents()` в `PopulationManager` использует пропорциональную селекцию (roulette wheel selection).

### Алгоритм селекции

1. **Вычисление общей суммы фитнеса:**
   ```rust
   let total_fitness: f64 = population
       .individuals
       .iter()
       .filter_map(|ind| ind.strategy.fitness)
       .sum();
   ```

2. **Если сумма фитнеса равна 0:**
   - Выбираются случайные индивидуумы (равномерное распределение)

3. **Иначе (пропорциональная селекция):**
   - Генерируется случайное число от 0 до `total_fitness`
   - Проходим по популяции, накапливая фитнес
   - Когда накопленный фитнес превышает случайное число, выбираем этого индивидуума

**Пример:**
- Индивидуум 1: фитнес = 10.0
- Индивидуум 2: фитнес = 5.0
- Индивидуум 3: фитнес = 15.0
- Общая сумма = 30.0

Вероятность выбора:
- Индивидуум 1: 10/30 = 33.3%
- Индивидуум 2: 5/30 = 16.7%
- Индивидуум 3: 15/30 = 50.0%

**Особенности:**
- Стратегии с большим фитнесом имеют большую вероятность быть выбранными
- Но даже слабые стратегии могут быть выбраны (хотя с меньшей вероятностью)
- Это обеспечивает баланс между эксплуатацией лучших решений и исследованием пространства поиска

---

## Кроссовер структуры стратегии

Метод `crossover_structure()` комбинирует структуры двух родительских стратегий.

### Алгоритм кроссовера

1. **Проверка вероятности кроссовера:**
   ```rust
   if rng.gen::<f64>() < self.config.crossover_rate
   ```
   Если случайное число больше `crossover_rate`, структуры не меняются.

2. **Сохранение исходных условий:**
   ```rust
   let cond1 = child1.conditions.clone();
   let exit1 = child1.exit_conditions.clone();
   let cond2 = child2.conditions.clone();
   let exit2 = child2.exit_conditions.clone();
   ```
   Сохраняются копии условий до обмена индикаторов, чтобы использовать их при фильтрации.

3. **Кроссовер индикаторов с условиями (50% вероятность):**
   
   Если `rng.gen::<f64>() < 0.5`:
   - Обмен индикаторами и вложенными индикаторами:
     ```rust
     std::mem::swap(&mut child1.indicators, &mut child2.indicators);
     std::mem::swap(&mut child1.nested_indicators, &mut child2.nested_indicators);
     ```
   - Фильтрация условий по доступным индикаторам:
     ```rust
     child1.conditions = Self::merge_conditions_with_indicators(
         &cond1, &cond2, &child1_aliases
     );
     ```
     Условия от обоих родителей объединяются, но остаются только те, которые используют индикаторы, присутствующие в стратегии после обмена.

4. **Кроссовер условий независимо (если индикаторы не обменивались):**
   
   Если кроссовер индикаторов не произошел:
   - Условия входа обмениваются с вероятностью 50%:
     ```rust
     if rng.gen::<f64>() < 0.5 {
         child1.conditions = Self::filter_conditions_by_indicators(&cond1, &child1_aliases);
         child1.conditions.extend(
             Self::filter_conditions_by_indicators(&cond2, &child1_aliases)
         );
     }
     ```
   - Условия выхода обмениваются аналогично с вероятностью 50%
   - При обмене проверяется наличие соответствующих индикаторов

5. **Кроссовер стоп-обработчиков (50% вероятность):**
   ```rust
   if rng.gen::<f64>() < 0.5 {
       std::mem::swap(&mut child1.stop_handlers, &mut child2.stop_handlers);
       std::mem::swap(&mut child1.take_handlers, &mut child2.take_handlers);
   }
   ```

6. **Кроссовер таймфреймов с индикаторами (50% вероятность):**
   
   Если `rng.gen::<f64>() < 0.5`:
   - Извлекаются таймфреймы и связанные с ними индикаторы:
     ```rust
     let (tf1, ind1_for_tf) = Self::extract_timeframes_with_indicators(&child1);
     let (tf2, ind2_for_tf) = Self::extract_timeframes_with_indicators(&child2);
     ```
   - Обмен таймфреймами:
     ```rust
     std::mem::swap(&mut child1.timeframes, &mut child2.timeframes);
     ```
   - Перенос индикаторов для новых таймфреймов:
     ```rust
     for (tf, indicators) in ind2_for_tf {
         if child1.timeframes.contains(&tf) {
             for ind in indicators {
                 if !Self::has_indicator_alias(&child1, &ind.alias) {
                     child1.indicators.push(ind);
                 }
             }
         }
     }
     ```
     Для каждого таймфрейма из второго родителя, который находится в диапазоне между минимальным и максимальным таймфреймом первого потомка, переносятся все индикаторы, используемые в условиях с этим таймфреймом.
     
     **Важно**: Индикаторы переносятся не только при точном совпадении таймфрейма, но и если таймфрейм индикатора находится в диапазоне между минимальным и максимальным таймфреймом потомка.
     
     **Пример**: Если у потомка есть таймфреймы 60 и 240, то индикатор с таймфреймом 120 будет перенесен, так как 120 находится между 60 и 240.

### Новая логика кроссовера структуры

Кроссовер структуры стратегии теперь учитывает зависимости между компонентами:

#### 1. Кроссовер индикаторов с условиями (50% вероятность)

Если происходит кроссовер индикаторов:
- Индикаторы и вложенные индикаторы обмениваются между потомками
- Условия входа и выхода фильтруются: остаются только те, которые используют индикаторы, присутствующие в стратегии после обмена
- Условия от второго родителя добавляются к условиям первого родителя, если они используют доступные индикаторы

#### 2. Кроссовер условий независимо (50% вероятность)

Если кроссовер индикаторов не произошел:
- Условия входа и выхода обмениваются независимо с вероятностью 50%
- При обмене условий проверяется, что все используемые индикаторы присутствуют в стратегии
- Условия, использующие отсутствующие индикаторы, отфильтровываются

#### 3. Кроссовер стоп-обработчиков (50% вероятность)

- Стоп-обработчики обмениваются независимо с вероятностью 50%

#### 4. Кроссовер таймфреймов с индикаторами (50% вероятность)

Если происходит кроссовер таймфреймов:
- Таймфреймы обмениваются между потомками
- Вычисляется диапазон таймфреймов потомка (минимальный и максимальный)
- Для каждого таймфрейма из второго родителя, который находится в диапазоне между минимальным и максимальным таймфреймом первого потомка, переносятся все индикаторы, используемые в условиях с этим таймфреймом
- Аналогично для второго потомка
- **Важно**: Индикаторы переносятся не только при точном совпадении таймфрейма, но и если таймфрейм индикатора находится в диапазоне между минимальным и максимальным таймфреймом потомка

### Пример кроссовера

**Родитель 1:**
- indicators: [SMA, RSI]
- conditions: [RSI > 70, SMA > Close]
- exit_conditions: [RSI < 30]
- timeframes: [M15]

**Родитель 2:**
- indicators: [MACD, BollingerBands]
- conditions: [MACD crosses above, Price > UpperBand]
- exit_conditions: [Price > UpperBand]
- timeframes: [H1, H4]

**Результат кроссовера (пример с обменом индикаторов):**
- **Child 1:**
  - indicators: [MACD, BollingerBands] (обменяно)
  - conditions: [MACD crosses above, Price > UpperBand] (отфильтровано: только условия, использующие MACD и BollingerBands)
  - exit_conditions: [Price > UpperBand] (отфильтровано)
  - timeframes: [M15] (не обменяно)

- **Child 2:**
  - indicators: [SMA, RSI] (обменяно)
  - conditions: [RSI > 70, SMA > Close] (отфильтровано: только условия, использующие SMA и RSI)
  - exit_conditions: [RSI < 30] (отфильтровано)
  - timeframes: [H1, H4] (не обменяно)

**Результат кроссовера (пример с обменом таймфреймов):**
- **Child 1:**
  - indicators: [SMA, RSI, MACD] (добавлен MACD, т.к. используется в условиях с H1, который находится в диапазоне [H1, H4])
  - conditions: [RSI > 70, SMA > Close]
  - exit_conditions: [RSI < 30]
  - timeframes: [H1, H4] (обменяно)

- **Child 2:**
  - indicators: [MACD, BollingerBands]
  - conditions: [MACD crosses above, Price > UpperBand]
  - exit_conditions: [Price > UpperBand]
  - timeframes: [M15] (обменяно)

**Пример с диапазоном таймфреймов:**
- **Родитель 1:** timeframes: [M60, M240], индикатор с M120
- **Родитель 2:** timeframes: [M15, M30]
- **После кроссовера Child 1:** timeframes: [M15, M30]
  - Индикатор с M120 НЕ переносится, т.к. M120 > M30 (выходит за диапазон)

- **Родитель 1:** timeframes: [M15, M240], индикатор с M120
- **Родитель 2:** timeframes: [M60]
- **После кроссовера Child 1:** timeframes: [M60]
  - Индикатор с M120 НЕ переносится, т.к. M120 > M60 (выходит за диапазон)

- **Родитель 1:** timeframes: [M60, M240], индикатор с M120
- **Родитель 2:** timeframes: [M15, M240]
- **После кроссовера Child 1:** timeframes: [M15, M240]
  - Индикатор с M120 переносится, т.к. M120 находится в диапазоне [M15, M240]

**Особенности новой логики:**
- Индикаторы переносятся вместе с условиями, которые их используют
- Условия переносятся только если соответствующие индикаторы присутствуют в стратегии
- При кроссовере таймфреймов автоматически переносятся индикаторы, используемые в условиях с таймфреймами, которые находятся в диапазоне между минимальным и максимальным таймфреймом потомка
- Мутации таймфреймов могут происходить только в большую сторону от базового таймфрейма
- Это обеспечивает целостность стратегий и предотвращает создание невалидных комбинаций

---

## Кроссовер параметров

Метод `crossover()` в `PopulationManager` комбинирует параметры двух родительских стратегий.

### Алгоритм кроссовера параметров

1. **Проверка вероятности кроссовера:**
   ```rust
   if rng.gen::<f64>() > self.config.crossover_rate {
       return None;
   }
   ```
   Если случайное число больше `crossover_rate`, возвращаются параметры родителей без изменений.

2. **Сбор всех ключей параметров:**
   ```rust
   let all_keys: Vec<String> = params1
       .keys()
       .chain(params2.keys())
       .cloned()
       .collect::<std::collections::HashSet<_>>()
       .into_iter()
       .collect();
   ```
   Собираются все уникальные ключи из обоих наборов параметров.

3. **Обмен значениями параметров:**
   
   Для каждого ключа с вероятностью 50% происходит обмен значений между потомками:
   
   ```rust
   for key in all_keys {
       let val1 = params1.get(&key);
       let val2 = params2.get(&key);
       
       if rng.gen::<f64>() < 0.5 {
           // Child1 получает значение от Parent1, Child2 от Parent2
           if let Some(v1) = val1 {
               child1.insert(key.clone(), v1.clone());
           }
           if let Some(v2) = val2 {
               child2.insert(key.clone(), v2.clone());
           }
       } else {
           // Child1 получает значение от Parent2, Child2 от Parent1
           if let Some(v2) = val2 {
               child1.insert(key.clone(), v2.clone());
           }
           if let Some(v1) = val1 {
               child2.insert(key.clone(), v1.clone());
           }
       }
   }
   ```

### Пример кроссовера параметров

**Родитель 1:**
- `"SMA_period"`: 20
- `"RSI_period"`: 14
- `"condition_1_threshold"`: 70.0

**Родитель 2:**
- `"SMA_period"`: 50
- `"MACD_fast"`: 12
- `"condition_1_threshold"`: 80.0

**Результат кроссовера (пример):**
- **Child 1:**
  - `"SMA_period"`: 50 (от Parent2)
  - `"RSI_period"`: 14 (от Parent1)
  - `"MACD_fast"`: 12 (от Parent2)
  - `"condition_1_threshold"`: 70.0 (от Parent1)

- **Child 2:**
  - `"SMA_period"`: 20 (от Parent1)
  - `"RSI_period"`: 14 (от Parent1, но может быть не включен если нет в Parent2)
  - `"MACD_fast"`: 12 (от Parent2)
  - `"condition_1_threshold"`: 80.0 (от Parent2)

**Особенности:**
- Каждый параметр обменивается независимо
- Если параметр есть только у одного родителя, он может быть унаследован одним из потомков
- Это позволяет комбинировать лучшие параметры от разных родителей

---

## Мутация структуры стратегии

Метод `mutate_structure()` вносит случайные изменения в структуру стратегии.

### Алгоритм мутации структуры

Мутация применяется независимо к каждому компоненту структуры с вероятностью `mutation_rate`.

#### 1. Мутация индикаторов

```rust
if rng.gen::<f64>() < config.mutation_rate {
    if rng.gen::<f64>() < 0.3 && !candidate.indicators.is_empty() {
        // Удаление индикатора (30% вероятность)
        let idx = rng.gen_range(0..candidate.indicators.len());
        candidate.indicators.remove(idx);
    } else if !available_indicators.is_empty() {
        // Добавление нового индикатора (70% вероятность)
        let new_indicator = available_indicators[rng.gen_range(0..available_indicators.len())].clone();
        candidate.indicators.push(new_indicator);
    }
}
```

**Действия:**
- С вероятностью 30%: удаление случайного индикатора (если список не пуст)
- С вероятностью 70%: добавление случайного индикатора из доступных

#### 2. Мутация условий входа

```rust
if rng.gen::<f64>() < config.mutation_rate {
    if rng.gen::<f64>() < 0.3 && !candidate.conditions.is_empty() {
        // Удаление условия (30% вероятность)
        let idx = rng.gen_range(0..candidate.conditions.len());
        candidate.conditions.remove(idx);
    } else {
        // Добавление нового условия (70% вероятность)
        let mut engine = StrategyDiscoveryEngine::new(candidate.config.clone());
        let mut iter = engine.generate_strategies_random(...);
        if let Some(new_candidate) = iter.next() {
            if !new_candidate.conditions.is_empty() {
                let new_condition = new_candidate.conditions[rng.gen_range(0..new_candidate.conditions.len())].clone();
                candidate.conditions.push(new_condition);
            }
        }
    }
}
```

**Действия:**
- С вероятностью 30%: удаление случайного условия входа
- С вероятностью 70%: генерация новой случайной стратегии и добавление одного из её условий входа

#### 3. Мутация условий выхода

Аналогично условиям входа, но для `exit_conditions`:

```rust
if rng.gen::<f64>() < config.mutation_rate {
    let has_exit_conditions = !candidate.exit_conditions.is_empty();
    let has_stop_handlers = !candidate.stop_handlers.is_empty();
    let can_remove_exit = has_exit_conditions && (candidate.exit_conditions.len() > 1 || has_stop_handlers);
    
    if rng.gen::<f64>() < 0.3 && can_remove_exit {
        candidate.exit_conditions.remove(idx);
    } else {
        // Добавление нового условия выхода
        ...
    }
}
```

**Важное ограничение:** Удаление условия выхода разрешено только если:
- В стратегии больше одного условия выхода, ИЛИ
- В стратегии есть хотя бы один stop handler, ИЛИ
- В стратегии есть хотя бы один take handler

Это гарантирует, что в стратегии всегда будет хотя бы один способ выхода (exit condition, stop handler или take handler).

#### 4. Мутация стоп-обработчиков

```rust
if rng.gen::<f64>() < config.mutation_rate {
    let has_exit_conditions = !candidate.exit_conditions.is_empty();
    let has_stop_handlers = !candidate.stop_handlers.is_empty();
    let has_take_handlers = !candidate.take_handlers.is_empty();
    let can_remove_stop = has_stop_handlers && (candidate.stop_handlers.len() > 1 || has_exit_conditions || has_take_handlers);
    
    if rng.gen::<f64>() < 0.3 && can_remove_stop {
        candidate.stop_handlers.remove(idx);
    } else if !stop_handler_configs.is_empty() {
        // Добавление нового стоп-обработчика
        ...
    }
}
```

**Важное ограничение:** Удаление стоп-обработчика разрешено только если:
- В стратегии больше одного стоп-обработчика, ИЛИ
- В стратегии есть хотя бы одно условие выхода, ИЛИ
- В стратегии есть хотя бы один take handler

Это гарантирует, что в стратегии всегда будет хотя бы один способ выхода (exit condition, stop handler или take handler).

#### 5. Мутация тейк-обработчиков

```rust
if rng.gen::<f64>() < config.mutation_rate {
    let has_exit_conditions = !candidate.exit_conditions.is_empty();
    let has_stop_handlers = !candidate.stop_handlers.is_empty();
    let has_take_handlers = !candidate.take_handlers.is_empty();
    let can_remove_take = has_take_handlers && (candidate.take_handlers.len() > 1 || has_exit_conditions || has_stop_handlers);
    
    if rng.gen::<f64>() < 0.3 && can_remove_take {
        candidate.take_handlers.remove(idx);
    } else if !stop_handler_configs.is_empty() {
        // Добавление нового тейк-обработчика
        ...
    }
}
```

**Важное ограничение:** 
- **Take profit может быть только при наличии stop handlers или exit conditions** - нельзя иметь стратегию только с take handlers без stop или exit
- Удаление тейк-обработчика разрешено только если:
  - В стратегии больше одного тейк-обработчика, ИЛИ
  - В стратегии есть хотя бы одно условие выхода, ИЛИ
  - В стратегии есть хотя бы один stop handler
- Добавление тейк-обработчика разрешено только если:
  - В стратегии уже есть хотя бы одно условие выхода, ИЛИ
  - В стратегии уже есть хотя бы один stop handler

Это гарантирует, что take profit никогда не будет единственным правилом выхода из позиции - всегда должен быть хотя бы один stop handler или exit condition.

#### 6. Мутация таймфреймов

```rust
if rng.gen::<f64>() < config.mutation_rate * 0.5 {
    let base_tf = &candidate.config.base_timeframe;
    let base_duration = base_tf.duration();
    
    let all_timeframes = vec![
        TimeFrame::from_identifier("1"),
        TimeFrame::from_identifier("5"),
        TimeFrame::from_identifier("15"),
        TimeFrame::from_identifier("30"),
        TimeFrame::from_identifier("60"),
        TimeFrame::from_identifier("240"),
        TimeFrame::from_identifier("D"),
    ];
    
    let available_timeframes: Vec<TimeFrame> = if let Some(base_dur) = base_duration {
        all_timeframes
            .into_iter()
            .filter(|tf| {
                if let Some(tf_dur) = tf.duration() {
                    tf_dur >= base_dur
                } else {
                    false
                }
            })
            .collect()
    } else {
        all_timeframes
    };
    
    if !candidate.timeframes.is_empty() && rng.gen::<f64>() < 0.5 {
        // Удаление таймфрейма (50% вероятность)
        let idx = rng.gen_range(0..candidate.timeframes.len());
        candidate.timeframes.remove(idx);
    } else if !available_timeframes.is_empty() {
        // Добавление нового таймфрейма (50% вероятность)
        let new_tf = available_timeframes[rng.gen_range(0..available_timeframes.len())].clone();
        if !candidate.timeframes.contains(&new_tf) {
            candidate.timeframes.push(new_tf);
        }
    }
}
```

**Особенности:**
- Вероятность мутации таймфреймов в 2 раза меньше (`mutation_rate * 0.5`)
- **Важное ограничение**: Мутации таймфреймов могут происходить только в большую сторону от базового таймфрейма
  - Если базовый таймфрейм M15, то можно добавить только M30, H1, H4, D и т.д.
  - Нельзя добавить M1, M5 (меньше базового)
- Фильтрация доступных таймфреймов происходит по сравнению длительности (`duration()`)
- Проверяется, чтобы не добавлять дубликаты таймфреймов

**Пример:**
- Базовый таймфрейм: M15 (15 минут)
- Доступные для мутации: M30, H1 (60 минут), H4 (240 минут), D (1 день)
- Недоступные: M1, M5 (меньше базового)

### Пример мутации структуры

**До мутации:**
- indicators: [SMA, RSI]
- conditions: [RSI > 70]
- exit_conditions: [RSI < 30]
- stop_handlers: [StopLoss]
- timeframes: [TimeFrame::M15]

**После мутации (пример):**
- indicators: [SMA, RSI, MACD] (добавлен MACD)
- conditions: [RSI > 70] (без изменений)
- exit_conditions: [] (удалено условие выхода, но остался StopLoss)
- stop_handlers: [StopLoss] (без изменений)
- timeframes: [TimeFrame::M15, TimeFrame::H1] (добавлен H1)

**Важно:** Если бы в стратегии был только один exit_condition и не было stop_handlers, то удаление exit_condition было бы запрещено. Удаление разрешено только если после удаления останется хотя бы один способ выхода.

**Особенности:**
- Мутация каждого компонента независима
- Вероятность удаления (30%) меньше вероятности добавления (70%)
- Это способствует росту сложности стратегий, но также позволяет упрощать их

---

## Мутация параметров

Метод `mutate()` в `PopulationManager` вносит случайные изменения в значения параметров.

### Алгоритм мутации параметров

1. **Итерация по всем параметрам:**
   ```rust
   let keys: Vec<String> = parameters.keys().cloned().collect();
   for key in keys {
       if rng.gen::<f64>() < self.config.mutation_rate {
           if let Some(param_value) = parameters.get_mut(&key) {
               let range = Self::get_parameter_range(&key, candidate);
               if let Some(opt_range) = range {
                   Self::mutate_parameter_with_range(
                       param_value,
                       &opt_range,
                       mutation_config.param_mutation_min_percent,
                       mutation_config.param_mutation_max_percent,
                   );
               } else {
                   Self::mutate_parameter_fallback(param_value);
               }
           }
       }
   }
   ```

2. **Получение диапазона параметра:**
   
   Для каждого параметра определяется его диапазон оптимизации:
   - Для индикаторов: через `get_optimization_range()` с именем индикатора и параметра
   - Для вложенных индикаторов: аналогично, но с префиксом `"nested_"`
   - Для условий: извлекается имя индикатора из условия, затем вызывается `get_optimization_range()`
   - Для стоп-обработчиков: через `get_stop_optimization_range()` с именем обработчика и параметра

3. **Мутация значения параметра с использованием диапазона:**
   
   Если диапазон найден, используется настраиваемый процент мутации:
   
   ```rust
   let range_size = (range.end - range.start) as f64;
   let mutation_percent = rng.gen_range(min_percent..=max_percent);
   let mutation_amount = range_size * mutation_percent;
   let mutation = rng.gen_range(-mutation_amount..=mutation_amount);
   
   match value {
       StrategyParamValue::Number(n) => {
           *n = (*n + mutation).max(range.start as f64).min(range.end as f64);
       }
       StrategyParamValue::Integer(i) => {
           *i = ((*i as f64 + mutation) as i64).max(range.start as i64).min(range.end as i64);
       }
       StrategyParamValue::Flag(b) => {
           *b = !*b;
       }
       _ => {}
   }
   ```
   
   **Настраиваемые параметры мутации в `GeneticAlgorithmConfig`:**
   - `param_mutation_min_percent` (по умолчанию 0.03 = 3%) - минимальный процент мутации от размера диапазона параметра
   - `param_mutation_max_percent` (по умолчанию 0.05 = 5%) - максимальный процент мутации от размера диапазона параметра
   
   Эти параметры можно настроить при создании конфигурации генетического алгоритма:
   ```rust
   let config = GeneticAlgorithmConfig {
       // ... другие параметры ...
       param_mutation_min_percent: 0.02,  // 2% минимальная мутация
       param_mutation_max_percent: 0.08,  // 8% максимальная мутация
       ..Default::default()
   };
   ```
   
   Мутация происходит в диапазоне от `min_percent` до `max_percent` от размера диапазона параметра.
   
   **Пример:** Если параметр имеет диапазон от 10 до 200 (размер = 190), то:
   - Минимальная мутация: 190 * 0.03 = 5.7
   - Максимальная мутация: 190 * 0.05 = 9.5
   - Случайная мутация будет в диапазоне от -9.5 до +9.5

4. **Fallback мутация:**
   
   Если диапазон не найден, используется старый метод мутации:
   - Для чисел: изменение от -10% до +10% от текущего значения
   - Для целых чисел: изменение от -2 до +2
   - Для булевых значений: инверсия

### Пример мутации параметров

**До мутации:**
- `"SMA_period"`: 20 (Integer, диапазон: 10-200, размер: 190)
- `"RSI_period"`: 14 (Integer, диапазон: 5-50, размер: 45)
- `"condition_1_threshold"`: 70.0 (Number, диапазон: 0-100, размер: 100)
- `"use_filter"`: true (Flag)

**После мутации (пример с param_mutation_min_percent=0.03, param_mutation_max_percent=0.05):**

- `"SMA_period"`: 28 (20 + 8, мутация 4.2% от диапазона 190 = 8)
- `"RSI_period"`: 14 (не мутировал, вероятность не сработала)
- `"condition_1_threshold"`: 74.2 (70.0 + 4.2, мутация 4.2% от диапазона 100 = 4.2)
- `"use_filter"`: false (инвертировано)

**Расчет мутации для SMA_period:**
- Диапазон: 10-200, размер = 190
- Процент мутации: случайное от 3% до 5%, например 4.2%
- Размер мутации: 190 * 0.042 = 7.98 ≈ 8
- Новое значение: 20 + 8 = 28 (ограничено диапазоном 10-200)

**Особенности:**
- Каждый параметр мутирует независимо с вероятностью `mutation_rate`
- Для параметров с известным диапазоном: мутация в диапазоне от `param_mutation_min_percent` до `param_mutation_max_percent` от размера диапазона
- Значение после мутации ограничивается границами диапазона параметра
- Для параметров без диапазона используется fallback метод (старая логика)
- Для булевых значений всегда инверсия
- Процент мутации настраивается через `param_mutation_min_percent` и `param_mutation_max_percent` в конфигурации

---

## Создание новых особей

Метод `create_individual()` создает новую особь из структуры стратегии и параметров.

### Процесс создания особи

1. **Оценка стратегии:**
   ```rust
   let report = self
       .evaluator
       .evaluate_strategy(&candidate, parameters.clone())
       .await?;
   ```
   
   Выполняется бэктест стратегии с заданными параметрами. Процесс включает:
   - Построение индикаторов с заданными параметрами
   - Применение условий входа и выхода
   - Симуляция торговли на исторических данных
   - Расчет метрик производительности

2. **Вычисление фитнеса:**
   ```rust
   let fitness = Some(
       FitnessFunction::calculate_fitness(
           &report,
           &self.config.fitness_weights,
       ),
   );
   ```
   
   На основе отчета бэктеста вычисляется значение фитнеса (см. раздел "Оценка фитнеса").

3. **Создание EvaluatedStrategy:**
   ```rust
   let evaluated = EvaluatedStrategy {
       candidate: Some(candidate.clone()),
       parameters,
       fitness,
       backtest_report: Some(report),
   };
   ```

4. **Создание GeneticIndividual:**
   ```rust
   Ok(GeneticIndividual {
       strategy: evaluated,
       generation: generation,
       island_id: island_id,
   })
   ```

**Особенности:**
- Каждая новая особь обязательно проходит бэктест
- Это самая затратная операция по времени
- Фитнес вычисляется сразу после бэктеста
- Отчет бэктеста сохраняется для последующего анализа

---

## Элитизм и замена слабых особей

### Элитизм

Метод `apply_elitism()` сохраняет лучшие особи из предыдущего поколения.

**Алгоритм:**
1. Популяция сортируется по убыванию фитнеса
2. Первые `elitism_count` позиций заменяются элитными особями

```rust
pub fn apply_elitism(&self, population: &mut Population, elites: Vec<GeneticIndividual>) {
    population.individuals.sort_by(|a, b| {
        let fitness_a = a.strategy.fitness.unwrap_or(0.0);
        let fitness_b = b.strategy.fitness.unwrap_or(0.0);
        fitness_b.partial_cmp(&fitness_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    for (i, elite) in elites.into_iter().enumerate() {
        if i < population.individuals.len() {
            population.individuals[i] = elite;
        }
    }
}
```

**Преимущества элитизма:**
- Гарантирует, что лучшие решения не будут потеряны
- Ускоряет сходимость алгоритма
- Предотвращает деградацию популяции

**Недостатки:**
- Может привести к преждевременной сходимости
- Уменьшает разнообразие популяции

### Замена слабых особей

Метод `replace_weakest()` заменяет слабейшие особи новыми потомками.

**Алгоритм:**
1. Популяция сортируется по возрастанию фитнеса (слабейшие первые)
2. Слабейшие особи заменяются новыми потомками

```rust
pub fn replace_weakest(
    &self,
    population: &mut Population,
    new_individuals: Vec<GeneticIndividual>,
) {
    population.individuals.sort_by(|a, b| {
        let fitness_a = a.strategy.fitness.unwrap_or(0.0);
        let fitness_b = b.strategy.fitness.unwrap_or(0.0);
        fitness_a.partial_cmp(&fitness_b).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    for (i, new_ind) in new_individuals.into_iter().enumerate() {
        if i < population.individuals.len() {
            population.individuals[i] = new_ind;
        }
    }
}
```

**Особенности:**
- Новые потомки заменяют только слабейших особей
- Элитные особи не затрагиваются (они будут применены позже)
- Размер популяции сохраняется

---

## Оценка фитнеса

Функция фитнеса вычисляется методом `calculate_fitness()` в `FitnessFunction`.

### Компоненты фитнеса

Фитнес вычисляется как взвешенная сумма нормализованных метрик:

```rust
fitness = (
    sharpe_score * sharpe_ratio_weight +
    profit_factor_score * profit_factor_weight +
    win_rate_score * win_rate_weight +
    cagr_score * cagr_weight -
    drawdown_penalty * drawdown_penalty +
    trades_bonus * trades_count_bonus
) / total_weight
```

### Нормализация метрик

#### 1. Sharpe Ratio

```rust
fn normalize_sharpe_ratio(sharpe: Option<f64>) -> f64 {
    match sharpe {
        Some(s) if s >= 0.0 => (s / 3.0).min(1.0),  // Нормализация к [0, 1]
        Some(s) => 0.0,                              // Отрицательный Sharpe = 0
        None => 0.0,
    }
}
```

**Почему деление на 3.0?**

Значение 3.0 выбрано как порог "отличного" Sharpe Ratio в трейдинге:
- **Sharpe Ratio < 1.0** - плохая стратегия (высокий риск относительно доходности)
- **Sharpe Ratio 1.0-2.0** - приемлемая стратегия
- **Sharpe Ratio 2.0-3.0** - хорошая стратегия
- **Sharpe Ratio ≥ 3.0** - отличная стратегия (очень низкий риск относительно доходности)

Это связано с тем, что для нормального распределения более 99% значений находятся в пределах ±3 стандартных отклонений от среднего. Sharpe Ratio выше 3 указывает на очень низкую вероятность убытков и высокую эффективность стратегии.

**Нормализация:**
- Sharpe Ratio делится на 3.0 и ограничивается 1.0
- Отрицательные значения дают 0.0
- Sharpe Ratio = 3.0 или выше дает максимальный балл (1.0)
- Это означает, что стратегии с Sharpe Ratio ≥ 3.0 получают максимальную оценку по этой метрике

#### 2. Profit Factor

```rust
fn normalize_profit_factor(pf: Option<f64>) -> f64 {
    match pf {
        Some(p) if p > 0.0 => (p / 5.0).min(1.0),  // Нормализация к [0, 1]
        Some(_) => 0.0,
        None => 0.0,
    }
}
```

**Почему деление на 5.0?**

Значение 5.0 выбрано как порог "отличного" Profit Factor в трейдинге:
- **Profit Factor < 1.0** - убыточная стратегия
- **Profit Factor 1.0-1.5** - слабая стратегия
- **Profit Factor 1.5-2.0** - приемлемая стратегия
- **Profit Factor 2.0-3.0** - хорошая стратегия
- **Profit Factor ≥ 5.0** - отличная стратегия (прибыль в 5 раз превышает убытки)

**Нормализация:**
- Profit Factor делится на 5.0 и ограничивается 1.0
- Profit Factor = 5.0 или выше дает максимальный балл (1.0)
- Это означает, что стратегии с Profit Factor ≥ 5.0 получают максимальную оценку по этой метрике

#### 3. Win Rate

```rust
let win_rate_score = metrics.winning_percentage;
```

- Win Rate уже в диапазоне [0, 1] (процент от 0% до 100%)
- Используется напрямую без нормализации

#### 4. CAGR (Compound Annual Growth Rate)

```rust
fn normalize_cagr(cagr: Option<f64>) -> f64 {
    match cagr {
        Some(c) if c >= 0.0 => (c / 100.0).min(1.0),  // Нормализация к [0, 1]
        Some(_) => 0.0,
        None => 0.0,
    }
}
```

**Почему деление на 100.0?**

Значение 100% выбрано как порог "отличного" CAGR в трейдинге:
- **CAGR < 10%** - низкая доходность
- **CAGR 10-20%** - приемлемая доходность
- **CAGR 20-50%** - хорошая доходность
- **CAGR 50-100%** - очень хорошая доходность
- **CAGR ≥ 100%** - отличная доходность (удвоение капитала в год)

**Нормализация:**
- CAGR делится на 100.0 и ограничивается 1.0
- CAGR = 100% или выше дает максимальный балл (1.0)
- Это означает, что стратегии с CAGR ≥ 100% получают максимальную оценку по этой метрике

#### 5. Drawdown Penalty (штраф за просадку)

```rust
fn calculate_drawdown_penalty(dd_pct: Option<f64>) -> f64 {
    match dd_pct {
        Some(dd) if dd > 0.0 => (dd / 50.0).min(1.0),  // Нормализация к [0, 1]
        Some(_) => 0.0,
        None => 0.0,
    }
}
```

- Просадка в процентах делится на 50.0 и ограничивается 1.0
- Просадка 50% или выше дает максимальный штраф (1.0)
- Штраф вычитается из фитнеса

#### 6. Trades Bonus (бонус за количество сделок)

```rust
fn calculate_trades_bonus(trades_count: usize) -> f64 {
    if trades_count >= 100 {
        1.0
    } else if trades_count >= 50 {
        0.75
    } else if trades_count >= 30 {
        0.5
    } else {
        (trades_count as f64 / 30.0).min(0.5)
    }
}
```

- ≥ 100 сделок: бонус 1.0
- ≥ 50 сделок: бонус 0.75
- ≥ 30 сделок: бонус 0.5
- < 30 сделок: бонус пропорционален количеству (max 0.5)

### Веса по умолчанию

```rust
pub struct FitnessWeights {
    pub sharpe_ratio_weight: f64,      // 0.3 (30%)
    pub profit_factor_weight: f64,     // 0.25 (25%)
    pub win_rate_weight: f64,          // 0.15 (15%)
    pub cagr_weight: f64,              // 0.2 (20%)
    pub drawdown_penalty: f64,         // 0.05 (5%)
    pub trades_count_bonus: f64,       // 0.05 (5%)
}
```

### Пример расчета фитнеса

**Метрики стратегии:**
- Sharpe Ratio: 2.0
- Profit Factor: 3.0
- Win Rate: 0.55 (55%)
- CAGR: 25.0%
- Drawdown: 15.0%
- Trades Count: 80

**Нормализованные значения:**
- sharpe_score = 2.0 / 3.0 = 0.667
- profit_factor_score = 3.0 / 5.0 = 0.6
- win_rate_score = 0.55
- cagr_score = 25.0 / 100.0 = 0.25
- drawdown_penalty = 15.0 / 50.0 = 0.3
- trades_bonus = 0.75

**Расчет фитнеса:**
```
fitness = (
    0.667 * 0.3 +
    0.6 * 0.25 +
    0.55 * 0.15 +
    0.25 * 0.2 -
    0.3 * 0.05 +
    0.75 * 0.05
) / 1.0

fitness = (
    0.2001 +
    0.15 +
    0.0825 +
    0.05 -
    0.015 +
    0.0375
) / 1.0

fitness = 0.5051
```

**Особенности:**
- Фитнес всегда ≥ 0.0 (из-за `fitness.max(0.0)`)
- Высокий Sharpe Ratio и Profit Factor дают наибольший вклад
- Большая просадка уменьшает фитнес
- Большое количество сделок увеличивает фитнес

---

## Заключение

Генетический алгоритм оптимизации торговых стратегий представляет собой сложную систему, которая эволюционирует популяцию стратегий через селекцию, кроссовер и мутацию. Ключевые особенности:

1. **Двухуровневая оптимизация**: Оптимизируются как структура стратегии, так и её параметры
2. **Пропорциональная селекция**: Лучшие стратегии имеют больше шансов быть выбранными, но слабые также могут участвовать
3. **Независимый кроссовер**: Компоненты структуры и параметры обмениваются независимо
4. **Адаптивная мутация**: Вероятность добавления выше вероятности удаления, что способствует росту сложности
5. **Элитизм**: Лучшие стратегии сохраняются без изменений
6. **Многокритериальная оценка**: Фитнес учитывает множество метрик производительности

Алгоритм продолжает эволюционировать популяцию до достижения максимального количества поколений или других критериев остановки.

