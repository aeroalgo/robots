# Процесс генерации начальной популяции

## Общий процесс

### Этап 1: Генерация кандидатов стратегий
**Файл:** `src/optimization/initial_population.rs`, функция `generate_candidates()` (строки 414-481)

- Генерируется `population_size * decimation_coefficient` кандидатов стратегий
- Каждый кандидат содержит:
  - Индикаторы с их параметрами
  - Условия входа/выхода
  - Стоп-хендлеры с `optimization_params` (создаются через `make_handler_params`)
  - Тейк-хендлеры

**Создание стоп-хендлеров:**
- В `candidate_builder.rs` функция `make_handler_params()` (строки 33-50) собирает ВСЕ параметры для хендлера из всех конфигов
- Для ATRTrailStop должны быть: `period` и `coeff_atr`
- Параметры добавляются в `StopHandlerInfo.optimization_params`

### Этап 2: Тестирование кандидатов с вариантами параметров
**Файл:** `src/optimization/initial_population.rs`, функция `generate()` (строки 130-374)

- Для каждого кандидата генерируется `param_variants_per_candidate` вариантов параметров
- **КРИТИЧНО:** Для каждого варианта вызывается `generate_random_parameters(candidate)` (строка 280)
- Эта функция должна генерировать НОВЫЕ случайные значения при каждом вызове

**Процесс для каждого варианта:**
```rust
for param_variant in 0..param_variants_count {
    let random_params = self.generate_random_parameters(candidate); // ← НОВЫЕ параметры каждый раз
    let report = self.evaluator.evaluate_strategy(candidate, random_params).await?;
    // ...
}
```

### Этап 3: Отбор лучших особей
- Из всех прошедших фильтр отбираются лучшие `population_size` особей

## Генерация параметров стоп-хендлеров

**Файл:** `src/optimization/initial_population.rs`, функция `generate_random_parameters()` (строки 596-612)

### Алгоритм:

1. Для каждого `stop_handler` в `candidate.stop_handlers`
2. Для каждого `param` в `stop_handler.optimization_params`
3. Если `param.optimizable == true`:
   - Получаем диапазон: `get_stop_optimization_range(handler_name, param.name)`
   - **Если диапазон найден:**
     - Вычисляем количество шагов: `steps = (end - start) / step`
     - Генерируем случайный индекс: `step_index = rng.gen_range(0..=steps)` ← **ДОЛЖНО БЫТЬ РАЗНЫМ**
     - Вычисляем значение: `value = start + (step_index * step)`
     - Добавляем в `params` с ключом: `stop_handler.id + "_" + param.name`

### Проверка диапазонов (`get_stop_optimization_range`)

**Файл:** `src/risk/parameters.rs`, функция `get_range()` (строки 63-84)

- Принимает `handler_name` и `param_name`
- Приводит к верхнему регистру: `handler.to_uppercase()`
- Приводит к нижнему регистру: `param.to_lowercase()`
- Для ATRTrailStop:
  - `"ATRTRAILSTOP"` → вызывает `match_atr_trail_param()`
  - `"period"` → возвращает `trailing_period()` (10.0-150.0, шаг 10.0)
  - `"coeff_atr"` → возвращает `atr_coefficient()` (2.0-8.0, шаг 0.5)

## Возможные проблемы

### 1. Параметры не попадают в `optimization_params`
**Проверка:** Убедиться, что `make_handler_params()` собирает все параметры из всех конфигов

### 2. `get_stop_optimization_range` возвращает `None`
**Причины:**
- Неправильное имя хендлера (регистр, опечатка)
- Неправильное имя параметра
- Параметр не зарегистрирован в `match_atr_trail_param()`

### 3. `rng.gen_range` генерирует одинаковые значения
**Маловероятно**, но возможно если:
- RNG не инициализируется заново
- Диапазон слишком мал (1 шаг = всегда одно значение)

## Отладка

Добавлен отладочный вывод в `generate_random_parameters()`:
- Выводит каждый сгенерированный параметр стопа
- Показывает, если диапазон не найден

**Запуск:** При генерации начальной популяции будут видны сообщения:
```
[DEBUG] Сгенерирован параметр стопа: stop_123_period = 80.00 (handler: ATRTrailStop, param: period, ...)
[DEBUG] Сгенерирован параметр стопа: stop_123_coeff_atr = 5.00 (handler: ATRTrailStop, param: coeff_atr, ...)
```
