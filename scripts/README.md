# Скрипты профилирования

Этот каталог содержит скрипты для профилирования производительности бэктеста.

## Скрипты

### profile-perf.sh

Профилирование с помощью `perf` (системный инструмент Linux).

**Использование:**
```bash
./scripts/profile-perf.sh
```

**Требования:**
- Установленный `perf`
- Доступ к `sudo` для настройки прав (см. Troubleshooting в docs/profiling.md)

**Результаты:**
- `profiling/perf.data` - данные профилирования
- `profiling/perf-report.txt` - текстовый отчет

### profile-flamegraph.sh

Создание flamegraph из данных `perf`.

**Использование:**
```bash
./scripts/profile-flamegraph.sh
```

**Требования:**
- Предварительно выполненный `profile-perf.sh`
- Автоматически клонирует FlameGraph репозиторий при первом запуске

**Результаты:**
- `profiling/flamegraph.svg` - визуализация профиля

### profile-cargo-flamegraph.sh

Профилирование с помощью `cargo-flamegraph` (самый простой способ).

**Использование:**
```bash
./scripts/profile-cargo-flamegraph.sh
```

**Требования:**
- Установленный `cargo-flamegraph` (установится автоматически)

**Результаты:**
- `profiling/flamegraph.svg` - визуализация профиля

## Быстрый старт

1. **Самый простой способ:**
```bash
./scripts/profile-cargo-flamegraph.sh
xdg-open profiling/flamegraph.svg
```

2. **Детальный анализ:**
```bash
./scripts/profile-perf.sh
./scripts/profile-flamegraph.sh
xdg-open profiling/flamegraph.svg
```

3. **Программное профилирование (pprof-rs):**
```bash
cargo run --release --features profiling
xdg-open profiling/flamegraph-pprof.svg
```

## Подробная документация

См. [docs/profiling.md](../docs/profiling.md) для детальной информации о профилировании.





















