# Профилирование только run_backtest с помощью perf

## Быстрый старт

```bash
./scripts/profile_backtest_perf.sh
```

Скрипт автоматически:
1. Соберет проект в release режиме
2. Запустит perf record для профилирования всего процесса
3. Отфильтрует данные по стеку вызовов, содержащему `run_backtest`
4. Создаст flamegraph только с данными из `run_backtest`

## Как это работает

Perf профилирует весь процесс, но затем мы фильтруем результаты, оставляя только те сэмплы, в стеке вызовов которых присутствует функция `run_backtest` или `BacktestExecutor::run_backtest`.

## Альтернативные методы

### Метод 1: Ручная фильтрация через perf script

```bash
# Запустите perf record
perf record -F 997 -g --call-graph dwarf -o profiling/perf.data \
    ./target/release/robots

# Отфильтруйте данные
perf script -i profiling/perf.data | \
    grep -B 2 -A 50 "run_backtest" | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_backtest.svg
```

### Метод 2: Использование perf report с фильтрацией

```bash
# Запустите perf record
perf record -F 997 -g --call-graph dwarf -o profiling/perf.data \
    ./target/release/robots

# Откройте интерактивный отчет с фильтрацией
perf report -i profiling/perf.data --symbol-filter 'run_backtest'
```

### Метод 3: Использование временных меток (если знаете время выполнения)

Если вы знаете примерное время выполнения `run_backtest` (например, с 10.5 до 15.2 секунды):

```bash
perf record -F 997 -g --call-graph dwarf -o profiling/perf.data \
    ./target/release/robots

# Фильтрация по времени
perf script -i profiling/perf.data --time 10.5,15.2 | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_backtest.svg
```

### Метод 4: Использование perf с фильтрацией по символам (экспериментально)

```bash
# Попытка фильтрации по символам функций
perf record -F 997 -g --call-graph dwarf \
    --filter 'robots::strategy::executor::BacktestExecutor::run_backtest' \
    -o profiling/perf.data \
    ./target/release/robots
```

**Примечание:** Фильтрация по символам может не работать в зависимости от версии perf и компилятора.

## Просмотр результатов

```bash
firefox profiling/flamegraph_backtest_perf.svg
# или
google-chrome profiling/flamegraph_backtest_perf.svg
```

## Сравнение с pprof

- **pprof** (встроенный в код): профилирует только код между созданием и удалением `_guard`
- **perf** (внешний): профилирует весь процесс, требует фильтрации результатов

## Устранение проблем

### Нет сэмплов с run_backtest

Если скрипт сообщает "No run_backtest samples found":
1. Убедитесь, что проект собран с отладочными символами (`cargo build --release`)
2. Проверьте, что функция действительно вызывается
3. Попробуйте увеличить частоту сэмплирования: `-F 1997` вместо `-F 997`

### Ошибка "stackcollapse-perf.pl not found"

```bash
cargo install flamegraph
```

### Ошибка "Permission denied" для perf

```bash
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

## Дополнительные опции

- `-F 997` - частота сэмплирования (997 Hz, можно увеличить до 1997 или 3997)
- `-g` - запись call graph
- `--call-graph dwarf` - использование DWARF для более точного call graph
- `--call-graph fp` - альтернативный метод (frame pointer, быстрее но менее точно)













