# Рефакторинг CandidateBuilder - Декомпозиция God Object

## Дата: 2025-11-29

## Проблема

Файл `candidate_builder.rs` содержал 2075 строк кода и 40+ методов, что нарушало принцип Single Responsibility (SRP) из SOLID. Это затрудняло:
- Понимание кода
- Тестирование отдельных компонентов
- Поддержку и расширение функциональности

## Решение

Декомпозиция на специализированные компоненты:

```
src/optimization/
├── builders/
│   ├── mod.rs                    # Модуль билдеров
│   ├── indicator_builder.rs      # Работа с индикаторами
│   ├── condition_builder.rs      # Работа с условиями
│   ├── stop_handler_builder.rs   # Работа со стоп-обработчиками
│   └── timeframe_builder.rs      # Работа с таймфреймами
├── candidate_builder.rs          # Оркестрация (уменьшен до ~600 строк)
└── ...
```

## Новые компоненты

### 1. IndicatorBuilder (`indicator_builder.rs`)

Отвечает за:
- `select_single_indicator()` - выбор индикатора с учётом вероятностей и исключений
- `try_add_nested_indicator()` - добавление вложенных индикаторов
- `is_oscillator_used_in_nested()` - проверка использования осциллятора во вложенных

### 2. ConditionBuilder (`condition_builder.rs`)

Отвечает за:
- `build_condition()` - генерация условий
- `build_condition_simple()` / `build_condition_simple_with_timeframe()` - упрощённая генерация
- `weighted_condition_type_choice()` - выбор типа условия
- `weighted_choice_for_oscillator_based()` - выбор для осцилляторов
- `can_compare_indicators()` - правила сравнения индикаторов
- `is_duplicate_condition()` - проверка дубликатов
- `is_comparison_operator()` - проверка оператора сравнения
- `extract_operands()` - извлечение операндов
- `has_conflicting_comparison_operator()` - проверка конфликтов
- `extract_indicator_alias_from_condition_id()` - парсинг ID условия

Также содержит перенесённый enum `ConditionOperands`.

### 3. StopHandlerBuilder (`stop_handler_builder.rs`)

Отвечает за:
- `select_stop_handler()` - выбор stop loss обработчика
- `select_take_handler()` - выбор take profit обработчика
- `make_handler_params()` - создание параметров обработчика

### 4. TimeframeBuilder (`timeframe_builder.rs`)

Отвечает за:
- `add_higher_timeframes_with_probability()` - добавление higher timeframes
- `ensure_higher_timeframes_used()` - гарантия использования всех timeframes

### 5. CandidateBuilder (обновлённый)

Оркестрация процесса генерации кандидата:
- `build_candidate()` - основной метод
- `build_phase_1()` - первая фаза сборки
- `build_additional_phase()` - дополнительные фазы
- `apply_rules()` - применение правил зависимостей
- `matches_selector()`, `evaluate_condition()`, `apply_action()` - работа с правилами
- `add_required_element()`, `remove_excluded_element()` - управление элементами
- `ensure_all_indicators_used()` - гарантия использования индикаторов
- `ensure_minimum_requirements()` - минимальные требования

Также предоставляет публичные методы-обёртки для обратной совместимости:
- `is_comparison_operator()` -> `ConditionBuilder::is_comparison_operator()`
- `has_conflicting_comparison_operator()` -> `ConditionBuilder::has_conflicting_comparison_operator()`
- `extract_operands()` -> конвертация из `ConditionBuilder::extract_operands()`

## Обратная совместимость

Все публичные API сохранены:
- `CandidateBuilder::new()`
- `CandidateBuilder::build_candidate()`
- `CandidateBuilder::is_comparison_operator()` (статический)
- `CandidateBuilder::has_conflicting_comparison_operator()` (статический)
- `CandidateBuilder::extract_operands()` (статический)
- `CandidateElements` struct
- `ConditionOperands` enum

## Результат

| Файл | Строк |
|------|-------|
| `candidate_builder.rs` (было 2075) | 774 |
| `builders/condition_builder.rs` | 860 |
| `builders/indicator_builder.rs` | 167 |
| `builders/stop_handler_builder.rs` | 84 |
| `builders/timeframe_builder.rs` | 135 |
| `builders/mod.rs` | 9 |
| **Итого** | 2029 |

| Метрика | До | После |
|---------|-----|-------|
| Строк в candidate_builder.rs | 2075 | 774 |
| Методов в CandidateBuilder | 40+ | ~15 |
| Количество файлов | 1 | 6 |
| Принцип SRP | Нарушен | Соблюдён |

## Изменённые файлы

1. `src/optimization/mod.rs` - добавлен `pub mod builders;`
2. `src/optimization/candidate_builder.rs` - рефакторинг
3. `src/optimization/builders/mod.rs` - новый
4. `src/optimization/builders/indicator_builder.rs` - новый
5. `src/optimization/builders/condition_builder.rs` - новый
6. `src/optimization/builders/stop_handler_builder.rs` - новый
7. `src/optimization/builders/timeframe_builder.rs` - новый

## Тестирование

Компиляция проекта прошла успешно. Логика работы не изменилась, только структура кода.
