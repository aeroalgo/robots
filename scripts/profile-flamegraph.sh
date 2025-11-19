#!/bin/bash

# Скрипт для создания flamegraph из данных perf
# Использование: ./scripts/profile-flamegraph.sh

set -e

OUTPUT_DIR="./profiling"
PERF_DATA="$OUTPUT_DIR/perf.data"
FLAMEGRAPH_SVG="$OUTPUT_DIR/flamegraph.svg"

# Проверяем наличие perf.data
if [ ! -f "$PERF_DATA" ]; then
    echo "⚠️  Файл $PERF_DATA не найден"
    echo "   Сначала запустите: ./scripts/profile-perf.sh"
    exit 1
fi

# Находим путь к flamegraph
if command -v flamegraph &> /dev/null; then
    FLAMEGRAPH_CMD=$(command -v flamegraph)
elif [ -f "$HOME/.cargo/bin/flamegraph" ]; then
    FLAMEGRAPH_CMD="$HOME/.cargo/bin/flamegraph"
else
    echo "❌ Утилита flamegraph не найдена"
    echo "   Установите её: cargo install flamegraph"
    exit 1
fi

echo "🔥 Создание flamegraph из perf данных..."

# Проверяем права на perf
USE_SUDO=0
if [ -r /proc/sys/kernel/perf_event_paranoid ]; then
    PARANOID=$(cat /proc/sys/kernel/perf_event_paranoid)
    if [ "$PARANOID" -gt 1 ]; then
        USE_SUDO=1
    fi
fi

# Создаем flamegraph напрямую из perf.data
echo "   Генерация flamegraph из perf.data..."
if [ "$USE_SUDO" -eq 1 ]; then
    sudo -E env "PATH=$PATH" "$FLAMEGRAPH_CMD" --perfdata "$PERF_DATA" --output "$FLAMEGRAPH_SVG" --title "Backtest Profiling" || {
        echo "⚠️  Требуются права sudo для доступа к perf.data"
        sudo -E env "PATH=$PATH" "$FLAMEGRAPH_CMD" --perfdata "$PERF_DATA" --output "$FLAMEGRAPH_SVG" --title "Backtest Profiling"
    }
else
    "$FLAMEGRAPH_CMD" --perfdata "$PERF_DATA" --output "$FLAMEGRAPH_SVG" --title "Backtest Profiling" || {
        echo "⚠️  Ошибка при создании flamegraph, попытка с sudo..."
        sudo -E env "PATH=$PATH" "$FLAMEGRAPH_CMD" --perfdata "$PERF_DATA" --output "$FLAMEGRAPH_SVG" --title "Backtest Profiling"
    }
fi

if [ ! -f "$FLAMEGRAPH_SVG" ]; then
    echo "❌ Ошибка: flamegraph не создан"
    exit 1
fi

echo "✅ Flamegraph создан!"
echo ""
echo "📊 Откройте файл в браузере:"
echo "   $FLAMEGRAPH_SVG"
echo ""
echo "   или выполните:"
echo "   xdg-open $FLAMEGRAPH_SVG"

