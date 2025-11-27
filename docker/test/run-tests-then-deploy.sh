#!/bin/bash
# Test-First Deployment Script
# Запускает тесты, и только при успехе запускает production

set -e

# Переход в корень проекта
cd "$(dirname "$0")/../.."

# Цвета
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Функции логирования
log_step() {
    echo -e "\n${BOLD}${BLUE}▶ $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Заголовок
echo ""
echo "╔═══════════════════════════════════════════════════╗"
echo "║     🚀 Test-First Deployment Pipeline           ║"
echo "║     Algo Robots Trading System                   ║"
echo "╚═══════════════════════════════════════════════════╝"
echo ""

# ====================================================================
# ШАГ 1: Проверка зависимостей
# ====================================================================

log_step "Шаг 1: Проверка зависимостей"

if ! command -v docker &> /dev/null; then
    log_error "Docker не установлен"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    log_error "Docker Compose не установлен"
    exit 1
fi

log_success "Docker и Docker Compose установлены"

# ====================================================================
# ШАГ 2: Очистка предыдущих тестовых контейнеров
# ====================================================================

log_step "Шаг 2: Очистка старых тестовых контейнеров"

docker compose -f docker/test/docker-compose.test.yml down -v 2>/dev/null || true
log_success "Очистка завершена"

# ====================================================================
# ШАГ 3: Запуск тестового окружения
# ====================================================================

log_step "Шаг 3: Запуск тестового окружения"

log_info "Поднимаем тестовые БД (ClickHouse, MongoDB, Redis)..."
docker compose -f docker/test/docker-compose.test.yml up -d clickhouse-test mongodb-test redis-test

log_info "Ожидание готовности сервисов (15 сек)..."
sleep 15

# Проверка здоровья
log_info "Проверка состояния сервисов..."

all_healthy=true

if docker ps --format '{{.Names}}\t{{.Status}}' | grep clickhouse-test | grep -q "healthy"; then
    log_success "ClickHouse (test) готов - порт 9001"
else
    log_error "ClickHouse (test) не готов"
    all_healthy=false
fi

if docker ps --format '{{.Names}}\t{{.Status}}' | grep mongodb-test | grep -q "healthy"; then
    log_success "MongoDB (test) готов - порт 27018"
else
    log_error "MongoDB (test) не готов"
    all_healthy=false
fi

if docker ps --format '{{.Names}}\t{{.Status}}' | grep redis-test | grep -q "healthy"; then
    log_success "Redis (test) готов - порт 6380"
else
    log_warning "Redis (test) не готов (не критично)"
fi

if [ "$all_healthy" = false ]; then
    log_error "Не все тестовые сервисы готовы. Проверьте логи:"
    echo "  docker compose -f docker/test/docker-compose.test.yml logs"
    exit 1
fi

log_success "Тестовое окружение запущено и готово"

# ====================================================================
# ШАГ 4: Запуск тестов
# ====================================================================

log_step "Шаг 4: Запуск интеграционных тестов"

log_info "Сборка test-runner контейнера..."
docker compose -f docker/test/docker-compose.test.yml build test-runner

echo ""
log_info "Запуск тестов..."
echo ""

# Запуск тестов и захват exit code
set +e
docker compose -f docker/test/docker-compose.test.yml run --rm test-runner
TEST_EXIT_CODE=$?
set -e

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    log_success "Все тесты пройдены успешно! ✨"
else
    log_error "Тесты не прошли (код выхода: $TEST_EXIT_CODE)"
    log_error "Деплой отменён!"
    
    # Очистка тестового окружения
    log_info "Остановка тестового окружения..."
    docker compose -f docker/test/docker-compose.test.yml down -v
    
    exit $TEST_EXIT_CODE
fi

# ====================================================================
# ШАГ 5: Остановка тестового окружения
# ====================================================================

log_step "Шаг 5: Остановка тестового окружения"

docker compose -f docker/test/docker-compose.test.yml down -v
log_success "Тестовое окружение остановлено и очищено"

# ====================================================================
# ШАГ 6: Запуск production окружения
# ====================================================================

log_step "Шаг 6: Запуск production окружения"

log_warning "Тесты пройдены! Запускаем production..."
echo ""

# Переход в docker директорию
cd docker

# Используем существующий скрипт для production
if [ -f "start-infrastructure.sh" ]; then
    log_info "Используем существующий скрипт start-infrastructure.sh"
    ./start-infrastructure.sh
else
    log_info "Запуск через docker compose..."
    docker compose up -d
fi

# Возврат в корень
cd ..

# ====================================================================
# ШАГ 7: Проверка статуса
# ====================================================================

log_step "Шаг 7: Проверка статуса production"

sleep 5

echo ""
log_info "Статус сервисов:"
docker compose -f docker/docker-compose.yml ps

echo ""
log_success "╔═══════════════════════════════════════════════════╗"
log_success "║   ✨ Деплой завершен успешно!                   ║"
log_success "╚═══════════════════════════════════════════════════╝"

echo ""
log_info "Доступные сервисы:"
echo "  📊 ClickHouse:  http://localhost:8123 (native: 9002)"
echo "  📦 MongoDB:     localhost:27017"
echo "  💾 PostgreSQL:  localhost:5432"
echo "  🔴 Redis:       localhost:6379"
echo "  📈 Grafana:     http://localhost:3000"
echo "  🎯 Prometheus:  http://localhost:9090"
echo "  🚀 Application: localhost:8080"

echo ""
log_info "Полезные команды:"
echo "  • Логи приложения:  docker logs -f trading-robot"
echo "  • Логи всех:        docker compose -f docker/docker-compose.yml logs -f"
echo "  • Остановка:        cd docker && ./stop-infrastructure.sh"
echo "  • Перезапуск тестов: $0"

echo ""










































