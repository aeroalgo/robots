#!/bin/bash

# Скрипт для профилирования с помощью perf
# Использование: ./scripts/profile-perf.sh [опции]

set -e

BINARY="./target/release/robots"
OUTPUT_DIR="./profiling"
PERF_DATA="$OUTPUT_DIR/perf.data"
PERF_REPORT="$OUTPUT_DIR/perf-report.txt"

# Создаем директорию для результатов
mkdir -p "$OUTPUT_DIR"

echo "🔨 Компиляция проекта в release режиме..."
cargo build --release

if [ ! -f "$BINARY" ]; then
    echo "❌ Ошибка: бинарный файл $BINARY не найден"
    exit 1
fi

echo "📊 Запуск профилирования с perf..."
echo "   Это может занять некоторое время в зависимости от вашего бэктеста"
echo "   Запрос прав sudo для perf..."

# Проверяем права perf
if [ -r /proc/sys/kernel/perf_event_paranoid ]; then
    PARANOID=$(cat /proc/sys/kernel/perf_event_paranoid)
    if [ "$PARANOID" -gt 0 ]; then
        echo "⚠️  Требуются права sudo для perf (perf_event_paranoid = $PARANOID)"
        echo "   Для постоянного решения выполните:"
        echo "   echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid"
        echo ""
    fi
fi

# Запускаем perf для записи данных профилирования
# Пробуем без sudo, если не получится - с sudo
if ! perf record -g --call-graph dwarf -o "$PERF_DATA" "$BINARY" 2>/dev/null; then
    echo "   Пробуем с sudo..."
    sudo perf record -g --call-graph dwarf -o "$PERF_DATA" "$BINARY"
fi

if [ ! -f "$PERF_DATA" ]; then
    echo "❌ Ошибка: файл профилирования не создан"
    exit 1
fi

echo "📝 Генерация текстового отчета..."
perf report -i "$PERF_DATA" > "$PERF_REPORT" 2>&1

echo "✅ Профилирование завершено!"
echo ""
echo "📊 Результаты:"
echo "   - Данные профилирования: $PERF_DATA"
echo "   - Текстовый отчет: $PERF_REPORT"
echo ""
echo "Для просмотра интерактивного отчета выполните:"
echo "   perf report -i $PERF_DATA"
echo ""
echo "Для создания flamegraph выполните:"
echo "   ./scripts/profile-flamegraph.sh"

