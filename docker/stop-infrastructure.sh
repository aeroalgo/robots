#!/bin/bash

# ============================================================================
# Скрипт для остановки инфраструктуры торговых роботов
# ============================================================================

set -e

echo "🛑 Остановка инфраструктуры торговых роботов"
echo "=" * 50

# Определение команды Docker Compose
if docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

# Переход в директорию docker
cd "$(dirname "$0")"

# Остановка контейнеров
echo "🛑 Остановка контейнеров..."
$DOCKER_COMPOSE down

# Очистка volumes (опционально)
if [ "$1" = "--clean" ]; then
    echo "🧹 Очистка volumes..."
    $DOCKER_COMPOSE down -v
fi

# Удаление образов (опционально)
if [ "$1" = "--remove-images" ]; then
    echo "🗑️ Удаление образов..."
    $DOCKER_COMPOSE down --rmi all
fi

# Полная очистка (опционально)
if [ "$1" = "--purge" ]; then
    echo "🧹 Полная очистка..."
    $DOCKER_COMPOSE down -v --rmi all
    docker system prune -f
fi

echo "✅ Инфраструктура остановлена"
echo ""
echo "💡 Доступные опции:"
echo "   --clean          Остановить и удалить volumes"
echo "   --remove-images  Остановить и удалить образы"
echo "   --purge          Полная очистка (volumes + образы + система)"