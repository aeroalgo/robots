#!/bin/bash

# ============================================================================
# Скрипт для запуска инфраструктуры торговых роботов
# ============================================================================

set -e

echo "🚀 Запуск инфраструктуры торговых роботов"
echo "=" * 50

# Проверка наличия Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker не установлен. Пожалуйста, установите Docker."
    exit 1
fi

# Проверка наличия Docker Compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose не установлен. Пожалуйста, установите Docker Compose."
    exit 1
fi

# Определение команды Docker Compose
if docker compose version &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

# Переход в директорию docker
cd "$(dirname "$0")"

# Проверка наличия файла .env
if [ ! -f ".env" ]; then
    echo "📝 Создание файла .env из env.local..."
    cp env.local .env
fi

# Остановка существующих контейнеров
echo "🛑 Остановка существующих контейнеров..."
$DOCKER_COMPOSE down

# Удаление старых образов (опционально)
if [ "$1" = "--clean" ]; then
    echo "🧹 Очистка старых образов..."
    $DOCKER_COMPOSE down --rmi all
fi

# Сборка и запуск сервисов
echo "🔨 Сборка и запуск сервисов..."
$DOCKER_COMPOSE up --build -d

# Ожидание готовности сервисов
echo "⏳ Ожидание готовности сервисов..."

# Проверка ClickHouse
echo "📊 Проверка ClickHouse..."
timeout=60
while ! curl -s http://localhost:8123/ping > /dev/null; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ ClickHouse не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ ClickHouse готов"

# Проверка Redis
echo "📦 Проверка Redis..."
timeout=60
while ! redis-cli -h localhost -p 6379 -a redis_password_2024 ping > /dev/null 2>&1; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ Redis не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ Redis готов"

# Проверка MongoDB
echo "🍃 Проверка MongoDB..."
timeout=60
while ! nc -z localhost 27017; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ MongoDB не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ MongoDB готов"

# Проверка PostgreSQL
echo "🐘 Проверка PostgreSQL..."
timeout=60
while ! nc -z localhost 5432; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ PostgreSQL не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ PostgreSQL готов"

# Проверка Kafka
echo "📨 Проверка Kafka..."
timeout=60
while ! nc -z localhost 9092; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ Kafka не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ Kafka готов"

# Проверка Prometheus
echo "📈 Проверка Prometheus..."
timeout=60
while ! curl -s http://localhost:9090/-/healthy > /dev/null; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ Prometheus не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ Prometheus готов"

# Проверка Grafana
echo "📊 Проверка Grafana..."
timeout=60
while ! curl -s http://localhost:3000/api/health > /dev/null; do
    sleep 2
    timeout=$((timeout - 2))
    if [ $timeout -le 0 ]; then
        echo "❌ Grafana не запустился в течение 60 секунд"
        exit 1
    fi
done
echo "✅ Grafana готов"

echo ""
echo "🎉 Все сервисы успешно запущены!"
echo "=" * 50
echo "📋 Доступные сервисы:"
echo "  🔗 ClickHouse:    http://localhost:8123"
echo "  🔗 Redis:         localhost:6379"
echo "  🔗 MongoDB:       localhost:27017"
echo "  🔗 PostgreSQL:    localhost:5432"
echo "  🔗 Kafka:         localhost:9092"
echo "  🔗 Prometheus:    http://localhost:9090"
echo "  🔗 Grafana:       http://localhost:3000 (admin/admin)"
echo "  🔗 Jaeger:        http://localhost:16686"
echo "  🔗 Elasticsearch: http://localhost:9200"
echo "  🔗 Kibana:        http://localhost:5601"
echo "  🔗 MinIO:         http://localhost:9001 (minioadmin/minioadmin123)"
echo "  🔗 Node Exporter: http://localhost:9100"
echo "=" * 50

# Показать статус контейнеров
echo "📊 Статус контейнеров:"
$DOCKER_COMPOSE ps

echo ""
echo "💡 Для остановки инфраструктуры выполните:"
echo "   $DOCKER_COMPOSE down"
echo ""
echo "💡 Для просмотра логов выполните:"
echo "   $DOCKER_COMPOSE logs -f [service_name]"
echo ""
echo "💡 Для перезапуска сервиса выполните:"
echo "   $DOCKER_COMPOSE restart [service_name]"