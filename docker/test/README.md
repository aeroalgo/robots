# 🧪 Тестовая инфраструктура

Этот каталог содержит полностью изолированную тестовую инфраструктуру для интеграционных тестов.

## 🎯 Концепция: Test-First Deployment

```
┌─────────────────┐
│  Запуск тестов  │
└────────┬────────┘
         │
         ├─── ❌ Ошибка → Остановка деплоя
         │
         └─── ✅ Успех → Запуск production
```

## 🏗️ Архитектура

### Тестовое окружение (изолированное)
- **ClickHouse** (23.8-alpine) - порт 9001 (production: 9002)
- **MongoDB** (7.0) - порт 27018 (production: 27017)
- **Redis** (7-alpine) - порт 6380 (production: 6379)
- **test-runner** - контейнер с Rust 1.90 для запуска тестов

### Отличия от production:
✅ Разные порты (нет конфликтов)
✅ Отдельные volumes (изоляция данных)
✅ Отдельная сеть (test-network)
✅ Тестовые credentials
✅ Автоматическая очистка после тестов

## 🚀 Использование

### Быстрый старт (Test → Deploy)

```bash
# Из корня проекта
./docker/test/run-tests-then-deploy.sh
```

Этот скрипт выполняет:
1. ✅ Проверка зависимостей
2. ✅ Очистка старых тестовых контейнеров
3. ✅ Запуск тестового окружения
4. ✅ Применение миграций к тестовым БД
5. ✅ Запуск интеграционных тестов
6. ✅ Если тесты прошли → запуск production
7. ❌ Если тесты провалились → остановка (production НЕ запускается!)

### Только тесты (без деплоя)

```bash
cd docker/test
docker compose -f docker-compose.test.yml up --abort-on-container-exit
```

### Ручной режим

```bash
cd docker/test

# 1. Запуск тестовых БД
docker compose up -d clickhouse-test mongodb-test redis-test

# 2. Ожидание готовности
sleep 15

# 3. Запуск тестов
docker compose run --rm test-runner

# 4. Очистка
docker compose down -v
```

## 📊 Структура файлов

```
docker/test/
├── docker-compose.test.yml      # Конфигурация тестовой инфраструктуры
├── Dockerfile.test              # Образ для запуска тестов (Rust 1.90)
├── run-tests-then-deploy.sh     # Главный скрипт: Test → Deploy
├── .env.test                    # Переменные окружения для тестов
└── README.md                    # Эта документация
```

## 🔧 Конфигурация

### Переменные окружения (.env.test)

Файл создается автоматически, но можно переопределить:

```bash
# Тестовые БД
CLICKHOUSE_HOST=clickhouse-test
CLICKHOUSE_PORT=9000
MONGODB_HOST=mongodb-test
MONGODB_PORT=27017
REDIS_HOST=redis-test
REDIS_PORT=6379

# Флаги доступности (для тестов)
CLICKHOUSE_AVAILABLE=true
MONGODB_AVAILABLE=true
```

### Кастомизация портов

Если нужны другие порты, отредактируйте `docker-compose.test.yml`:

```yaml
clickhouse-test:
  ports:
    - "ВАШИ_ПОРТ:9000"  # Измените здесь
```

## 🧪 Запускаемые тесты

### ClickHouse Integration Tests
- Подключение/отключение
- OHLCV данные (свечи)
- Тиковые данные
- Информация о символах
- Индикаторы
- Торговые сигналы
- Сделки
- Стратегии
- Результаты бэктестов
- Batch операции
- Аналитика

### MongoDB Integration Tests
- Подключение/отключение
- Конфигурации (CRUD)
- Метаданные (CRUD)
- Пользовательские настройки (CRUD)
- Системные конфигурации (CRUD)
- Агрегационные пайплайны
- Поиск и фильтрация
- Bulk операции
- Performance тесты

**Всего**: 24 интеграционных теста

## 📈 Ожидаемый вывод

### При успешных тестах:

```
🔧 Применение миграций ClickHouse...
✅ Миграции применены

🧪 Запуск интеграционных тестов...
===================================

running 12 tests
test test_clickhouse_connection ... ok
test test_ohlcv_insert_and_query ... ok
test test_trades_operations ... ok
...

test result: ok. 12 passed; 0 failed; 0 ignored

✅ Все тесты пройдены успешно! ✨

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
▶ Запуск production окружения
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Деплой завершен успешно!
```

### При провале тестов:

```
test test_ohlcv_insert_and_query ... FAILED

failures:
    test_ohlcv_insert_and_query

test result: FAILED. 10 passed; 1 failed; 1 ignored

❌ Тесты не прошли (код выхода: 101)
❌ Деплой отменён!
```

## 🔍 Отладка

### Просмотр логов тестовых БД

```bash
# ClickHouse
docker logs clickhouse-test

# MongoDB
docker logs mongodb-test

# Все логи
docker compose -f docker/test/docker-compose.test.yml logs
```

### Подключение к тестовым БД

```bash
# ClickHouse
docker exec -it clickhouse-test clickhouse-client --database trading_test

# MongoDB
docker exec -it mongodb-test mongosh trading_test

# Redis
docker exec -it redis-test redis-cli -a test_password
```

### Запуск тестов с детальным выводом

```bash
docker compose -f docker/test/docker-compose.test.yml run --rm \
  test-runner \
  cargo test --tests -- --test-threads=1 --ignored --nocapture
```

### Запуск конкретного теста

```bash
docker compose -f docker/test/docker-compose.test.yml run --rm \
  test-runner \
  cargo test test_ohlcv_insert_and_query -- --ignored --nocapture
```

## 🧹 Очистка

### Быстрая очистка

```bash
docker compose -f docker/test/docker-compose.test.yml down
```

### Полная очистка (включая volumes)

```bash
docker compose -f docker/test/docker-compose.test.yml down -v
```

### Очистка build кэша

```bash
docker compose -f docker/test/docker-compose.test.yml down -v --rmi all
```

## ⚙️ CI/CD интеграция

### GitHub Actions пример:

```yaml
name: Test and Deploy

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-and-deploy:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Run integration tests
        run: |
          cd docker/test
          docker compose up -d clickhouse-test mongodb-test redis-test
          sleep 15
          docker compose run --rm test-runner || exit 1
          docker compose down -v
      
      - name: Deploy to production
        if: success()
        run: |
          cd docker
          ./start-infrastructure.sh
```

### GitLab CI пример:

```yaml
stages:
  - test
  - deploy

integration-tests:
  stage: test
  script:
    - cd docker/test
    - docker compose up -d
    - docker compose run --rm test-runner
  after_script:
    - docker compose down -v

deploy-production:
  stage: deploy
  needs: [integration-tests]
  only:
    - main
  script:
    - cd docker
    - ./start-infrastructure.sh
```

## 📋 Чек-лист перед деплоем

- [ ] Все тесты пройдены локально
- [ ] Миграции актуальны
- [ ] Переменные окружения настроены
- [ ] Docker daemon запущен
- [ ] Достаточно дискового пространства
- [ ] Production сервисы не запущены (или будут перезапущены)

## 🎓 Best Practices

1. **Всегда запускайте тесты перед деплоем**
   ```bash
   ./docker/test/run-tests-then-deploy.sh
   ```

2. **Не пропускайте тесты в CI/CD**
   - Тесты должны блокировать деплой при ошибках

3. **Используйте отдельные тестовые БД**
   - Никогда не тестируйте на production данных

4. **Очищайте тестовые данные**
   - После каждого запуска: `down -v`

5. **Мониторьте время выполнения**
   - Если тесты долгие → оптимизируйте или распараллельте

## 🔗 Связанные файлы

- `../../tests/clickhouse_integration_tests.rs` - ClickHouse тесты
- `../../tests/mongodb_integration_tests.rs` - MongoDB тесты
- `../docker-compose.yml` - Production инфраструктура
- `../start-infrastructure.sh` - Production запуск
- `../stop-infrastructure.sh` - Production остановка

---

**Автор**: AI Assistant  
**Дата**: 2024-11-01  
**Версия**: 1.0.0  
**Статус**: ✅ Production Ready




















