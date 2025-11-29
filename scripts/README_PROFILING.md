# Профилирование run_backtest с помощью perf и flamegraph

## Быстрый старт

```bash
./scripts/profile_backtest.sh
```

Скрипт автоматически:
1. Соберет проект в release режиме
2. Запустит perf record для профилирования
3. Отфильтрует данные по стеку вызовов `run_backtest`
4. Создаст flamegraph в `profiling/flamegraph_backtest.svg`

## Требования

- `perf` (обычно установлен в Linux)
- `flamegraph` (установите через `cargo install flamegraph`)

## Альтернативный метод: профилирование только участка кода

Если вы хотите профилировать только конкретный участок кода (строки 95-97 в main.rs), можно использовать более точный метод:

### Вариант 1: Использование perf record с фильтрацией по символам

```bash
# Сначала соберите проект
cargo build --release

# Запустите perf record с фильтрацией по символам функций
perf record -F 997 -g --call-graph dwarf \
    --filter 'robots::strategy::executor::BacktestExecutor::run_backtest' \
    -o profiling/perf.data \
    ./target/release/robots

# Создайте flamegraph
perf script -i profiling/perf.data | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_backtest.svg
```

### Вариант 2: Использование perf record с временными метками

Если вы хотите профилировать только определенный временной интервал:

```bash
# Запустите perf record для всего процесса
perf record -F 997 -g --call-graph dwarf -o profiling/perf.data \
    ./target/release/robots

# Определите временной интервал выполнения run_backtest из логов
# Затем отфильтруйте данные по времени
perf script -i profiling/perf.data --time 10.5,15.2 | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_backtest.svg
```

### Вариант 3: Использование perf record с фильтрацией по PID и времени

```bash
# Запустите программу в фоне и получите PID
./target/release/robots &
PID=$!

# Запустите perf record для конкретного PID
perf record -F 997 -g --call-graph dwarf -p $PID -o profiling/perf.data

# Остановите запись после завершения run_backtest
# (можно использовать Ctrl+C или автоматически по таймеру)

# Создайте flamegraph
perf script -i profiling/perf.data | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_backtest.svg
```

## Просмотр результатов

Откройте созданный flamegraph в браузере:

```bash
firefox profiling/flamegraph_backtest.svg
# или
google-chrome profiling/flamegraph_backtest.svg
```

## Дополнительные опции perf

- `-F 997` - частота сэмплирования (997 Hz)
- `-g` - запись call graph
- `--call-graph dwarf` - использование DWARF для более точного call graph
- `-o` - выходной файл

## Фильтрация результатов

Для более точной фильтрации можно использовать:

```bash
# Фильтрация по конкретным функциям
perf script -i profiling/perf.data | \
    grep -A 50 "run_backtest" | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_filtered.svg

# Фильтрация по модулю
perf script -i profiling/perf.data | \
    grep "strategy::executor" | \
    stackcollapse-perf.pl | \
    flamegraph.pl > profiling/flamegraph_strategy.svg
```

## Устранение проблем

### Ошибка "perf: command not found"
```bash
sudo apt-get install linux-perf
# или
sudo apt-get install linux-tools-generic
```

### Ошибка "Permission denied"
```bash
# Временно разрешите perf для всех пользователей
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
# или запустите с sudo
sudo perf record ...
```

### Ошибка "stackcollapse-perf.pl not found"
```bash
cargo install flamegraph
```



















