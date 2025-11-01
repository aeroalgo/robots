# Makefile для тестовой инфраструктуры

.PHONY: help test deploy test-deploy up down clean logs test-only deploy-only status

help: ## Показать помощь
	@echo "╔═══════════════════════════════════════════════════╗"
	@echo "║   🧪 Test Infrastructure Commands                ║"
	@echo "╚═══════════════════════════════════════════════════╝"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""

test-deploy: ## Test → Deploy (рекомендуется)
	@./run-tests-then-deploy.sh

test: up test-only down ## Только тесты (с автоочисткой)

deploy: ## Только deploy (БЕЗ тестов, не рекомендуется!)
	@echo "⚠️  Запуск БЕЗ тестов!"
	@cd .. && ./start-infrastructure.sh

up: ## Запустить тестовое окружение
	@echo "🚀 Запуск тестового окружения..."
	@docker compose up -d clickhouse-test mongodb-test redis-test
	@echo "⏳ Ожидание готовности сервисов..."
	@sleep 15
	@echo "✅ Тестовое окружение готово"

down: ## Остановить тестовое окружение
	@echo "🛑 Остановка тестового окружения..."
	@docker compose down -v
	@echo "✅ Остановлено"

clean: ## Полная очистка (volumes, images)
	@echo "🧹 Полная очистка..."
	@docker compose down -v --rmi all
	@echo "✅ Очистка завершена"

test-only: ## Запустить только тесты (БД должны быть запущены)
	@echo "🧪 Запуск интеграционных тестов..."
	@docker compose run --rm test-runner

logs: ## Показать логи
	@docker compose logs -f

logs-ch: ## Логи ClickHouse
	@docker logs clickhouse-test -f

logs-mongo: ## Логи MongoDB
	@docker logs mongodb-test -f

status: ## Показать статус сервисов
	@echo "📊 Статус тестовых сервисов:"
	@docker compose ps

shell-test: ## Shell в test-runner контейнере
	@docker compose run --rm test-runner bash

shell-ch: ## Shell в ClickHouse
	@docker exec -it clickhouse-test clickhouse-client --database trading_test

shell-mongo: ## Shell в MongoDB
	@docker exec -it mongodb-test mongosh trading_test

shell-redis: ## Shell в Redis
	@docker exec -it redis-test redis-cli -a test_password

build: ## Пересобрать test-runner образ
	@docker compose build test-runner

rebuild: clean build ## Полная пересборка
	@echo "✅ Пересборка завершена"

# Алиасы для удобства
t: test ## Алиас для test
td: test-deploy ## Алиас для test-deploy
s: status ## Алиас для status
l: logs ## Алиас для logs

