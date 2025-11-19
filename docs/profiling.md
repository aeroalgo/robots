# Профилирование бэктеста

Этот документ описывает различные методы профилирования бэктеста для измерения времени выполнения функций и выявления узких мест в производительности.

## Инструменты профилирования

### 1. perf + FlameGraph

**perf** - системный инструмент Linux для профилирования производительности.  
**FlameGraph** - инструмент для визуализации профилей в виде "пламенного графика".

#### Установка

**perf:**
```bash
# Ubuntu/Debian
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-$(uname -r)

# Fedora/RHEL
sudo dnf install perf
```

**FlameGraph:**
```bash
git clone https://github.com/brendangregg/FlameGraph.git
```

#### Использование

**Автоматический скрипт:**
```bash
chmod +x scripts/profile-perf.sh
./scripts/profile-perf.sh
```

**Ручной запуск:**

1. Соберите проект в release режиме:
```bash
cargo build --release
```

2. Запустите perf для записи данных:
```bash
mkdir -p profiling
perf record -g --call-graph dwarf -o profiling/perf.data ./target/release/robots
```

3. Просмотрите интерактивный отчет:
```bash
perf report -i profiling/perf.data
```

4. Создайте flamegraph:
```bash
./scripts/profile-flamegraph.sh
```

Или вручную:
```bash
perf script -i profiling/perf.data | ./FlameGraph/stackcollapse-perf.pl | ./FlameGraph/flamegraph.pl > profiling/flamegraph.svg
```

5. Откройте flamegraph в браузере:
```bash
xdg-open profiling/flamegraph.svg
```

#### Интерпретация результатов

- **perf report**: Показывает функции, отсортированные по времени выполнения. Чем выше процент, тем больше времени занимает функция.
- **FlameGraph**: Ширина прямоугольника соответствует времени выполнения. Вертикальная высота показывает стек вызовов. Самые широкие прямоугольники указывают на узкие места.

### 2. cargo-flamegraph

**cargo-flamegraph** - удобный инструмент для создания flamegraph'ов из Rust проектов.

#### Установка

```bash
cargo install flamegraph
```

#### Использование

**Автоматический скрипт:**
```bash
chmod +x scripts/profile-cargo-flamegraph.sh
./scripts/profile-cargo-flamegraph.sh
```

**Ручной запуск:**
```bash
cargo flamegraph
```

Создаст файл `flamegraph.svg` в текущей директории.

#### Опции

```bash
# Профилирование конкретного бинарного файла
cargo flamegraph --bin robots

# Указать выходной файл
cargo flamegraph --output profiling/flamegraph.svg

# Профилирование тестов
cargo flamegraph --test test_name

# Изменить частоту выборки (по умолчанию 997 Hz)
cargo flamegraph --freq 1000
```

### 3. pprof-rs (программное профилирование)

**pprof-rs** - библиотека для программного профилирования, интегрируемая в код.

#### Активация

Включите feature `profiling`:
```bash
cargo build --release --features profiling
```

#### Использование в коде

Пример интеграции в `main.rs`:

```rust
#[cfg(feature = "profiling")]
use pprof::ProfilerGuard;

#[tokio::main]
async fn main() {
    #[cfg(feature = "profiling")]
    let _guard = ProfilerGuard::new(100).unwrap();

    // Ваш код бэктеста
    run().await.unwrap();

    #[cfg(feature = "profiling")]
    if let Ok(report) = _guard.report().build() {
        let file = std::fs::File::create("profiling/flamegraph.svg")
            .expect("Failed to create flamegraph.svg");
        report.flamegraph(file).expect("Failed to write flamegraph");
        println!("Flamegraph сохранен в profiling/flamegraph.svg");
    }
}
```

#### Параметры

- `ProfilerGuard::new(100)` - частота выборки в Hz (100 выборок в секунду)
- Более высокая частота дает больше деталей, но увеличивает overhead

## Сравнение инструментов

| Инструмент | Преимущества | Недостатки | Рекомендация |
|-----------|--------------|------------|--------------|
| **perf** | Системный уровень, минимальный overhead, точные измерения | Требует sudo, специфичен для Linux | Для детального анализа |
| **cargo-flamegraph** | Простота использования, автоматическая сборка | Может иметь небольшой overhead | Для быстрого анализа |
| **pprof-rs** | Интеграция в код, контроль над профилированием | Overhead выше, требует изменений в коде | Для точечного профилирования |

## Рекомендуемый workflow

1. **Быстрый анализ**: Используйте `cargo-flamegraph`
   ```bash
   ./scripts/profile-cargo-flamegraph.sh
   ```

2. **Детальный анализ**: Используйте `perf` + `FlameGraph`
   ```bash
   ./scripts/profile-perf.sh
   ./scripts/profile-flamegraph.sh
   ```

3. **Точечное профилирование**: Интегрируйте `pprof-rs` в код для профилирования конкретных участков

## Интерпретация результатов оптимизации

После оптимизации (предварительный расчет условий):

**До оптимизации:**
- Большая часть времени в `evaluate_conditions()` и `condition.check()`
- Много времени в циклах обработки условий на каждой свечке

**После оптимизации:**
- Время в `populate_conditions()` (один раз)
- Минимальное время в `evaluate_conditions()` (только чтение из кэша)

## Дополнительные опции perf

```bash
# Профилирование с указанием событий
perf record -e cpu-cycles,cache-misses ./target/release/robots

# Профилирование с ограничением по времени
perf record -g --call-graph dwarf --duration 60 ./target/release/robots

# Статистика без записи
perf stat ./target/release/robots

# Профилирование с учетом всех потоков
perf record -g --call-graph dwarf --all-cpus ./target/release/robots
```

## Troubleshooting

**perf: Permission denied**
```bash
# Временно (до перезагрузки)
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid

# Постоянно
sudo sysctl -w kernel.perf_event_paranoid=-1
```

**cargo-flamegraph: No symbols**
```bash
# Убедитесь, что проект собран в release режиме
cargo build --release
```

**Низкое качество flamegraph**
- Увеличьте частоту выборки
- Увеличьте длительность профилирования
- Используйте `--call-graph dwarf` для лучшего разрешения стека

## Ресурсы

- [perf wiki](https://perf.wiki.kernel.org/index.php/Main_Page)
- [FlameGraph GitHub](https://github.com/brendangregg/FlameGraph)
- [cargo-flamegraph docs](https://github.com/flamegraph-rs/flamegraph)
- [pprof-rs GitHub](https://github.com/tikv/pprof-rs)

