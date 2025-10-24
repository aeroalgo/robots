# Docker Infrastructure для торговых роботов

Этот каталог содержит Docker Compose конфигурацию для полной инфраструктуры торговых роботов.

## 🚀 Быстрый старт

### Запуск инфраструктуры

```bash
# Запуск всех сервисов
./start-infrastructure.sh

# Или с очисткой старых образов
./start-infrastructure.sh --clean
```

### Остановка инфраструктуры

```bash
# Остановка сервисов
./stop-infrastructure.sh

# Остановка с очисткой volumes
./stop-infrastructure.sh --clean

# Полная очистка
./stop-infrastructure.sh --purge
```

### Ручной запуск

```bash
# Запуск в фоновом режиме
docker compose up -d

# Запуск с пересборкой
docker compose up --build -d

# Просмотр логов
docker compose logs -f

# Остановка
docker compose down
```

## 📊 Сервисы

### Основные базы данных

| Сервис | Порт | Описание | Веб-интерфейс |
|--------|------|----------|---------------|
| **ClickHouse** | 8123, 9000 | Основное хранилище исторических данных | http://localhost:8123 |
| **Redis** | 6379 | Кэширование горячих данных | - |
| **MongoDB** | 27017 | Метаданные и конфигурации | - |
| **PostgreSQL** | 5432 | Пользовательские данные | - |

### Message Queue

| Сервис | Порт | Описание |
|--------|------|----------|
| **Zookeeper** | 2181 | Координация Kafka |
| **Kafka** | 9092 | Event streaming |

### Мониторинг

| Сервис | Порт | Описание | Веб-интерфейс |
|--------|------|----------|---------------|
| **Prometheus** | 9090 | Сбор метрик | http://localhost:9090 |
| **Grafana** | 3000 | Визуализация метрик | http://localhost:3000 |
| **Jaeger** | 16686 | Трейсинг | http://localhost:16686 |
| **Node Exporter** | 9100 | Системные метрики | http://localhost:9100 |

### Логирование

| Сервис | Порт | Описание | Веб-интерфейс |
|--------|------|----------|---------------|
| **Elasticsearch** | 9200 | Поиск и анализ логов | http://localhost:9200 |
| **Kibana** | 5601 | Визуализация логов | http://localhost:5601 |

### Хранилище

| Сервис | Порт | Описание | Веб-интерфейс |
|--------|------|----------|---------------|
| **MinIO** | 9000, 9001 | S3-совместимое хранилище | http://localhost:9001 |

## 🔧 Конфигурация

### Переменные окружения

Основные переменные окружения определены в файле `env.local`:

```bash
# ClickHouse
CLICKHOUSE_HOST=clickhouse
CLICKHOUSE_PORT=8123
CLICKHOUSE_USER=trading_user
CLICKHOUSE_PASSWORD=trading_password_2024
CLICKHOUSE_DATABASE=trading

# Redis
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=redis_password_2024

# MongoDB
MONGO_HOST=mongodb
MONGO_PORT=27017
MONGO_USER=admin
MONGO_PASSWORD=password
MONGO_DATABASE=trading_meta

# PostgreSQL
PSQL_HOST=postgres
PSQL_PORT=5432
PSQL_USER=postgres
PSQL_PASS=postgres
PSQL_DB=trading_users

# Kafka
KAFKA_HOST=kafka
KAFKA_PORT=9092
```

### Файлы конфигурации

- `clickhouse/config.xml` - конфигурация ClickHouse
- `clickhouse/users.xml` - пользователи ClickHouse
- `clickhouse/init.sql` - инициализация базы данных
- `prometheus/prometheus.yml` - конфигурация Prometheus
- `grafana/provisioning/` - конфигурация Grafana

## 📝 Использование

### Подключение к ClickHouse

```bash
# Через HTTP
curl http://localhost:8123/ping

# Через clickhouse-client
docker exec -it docker-clickhouse-1 clickhouse-client
```

### Подключение к Redis

```bash
# Через redis-cli
docker exec -it docker-redis-1 redis-cli -a redis_password_2024
```

### Подключение к MongoDB

```bash
# Через mongo shell
docker exec -it docker-mongodb-1 mongosh -u admin -p password
```

### Подключение к PostgreSQL

```bash
# Через psql
docker exec -it docker-postgres-1 psql -U postgres -d trading_users
```

## 🔍 Мониторинг

### Grafana

- URL: http://localhost:3000
- Логин: admin
- Пароль: admin

### Prometheus

- URL: http://localhost:9090
- Метрики доступны по адресу: http://localhost:9090/metrics

### Jaeger

- URL: http://localhost:16686
- Трейсинг доступен для всех сервисов

## 🛠️ Разработка

### Логи

```bash
# Просмотр логов всех сервисов
docker compose logs -f

# Просмотр логов конкретного сервиса
docker compose logs -f clickhouse
docker compose logs -f redis
docker compose logs -f mongodb
```

### Перезапуск сервиса

```bash
# Перезапуск конкретного сервиса
docker compose restart clickhouse

# Перезапуск с пересборкой
docker compose up --build -d clickhouse
```

### Отладка

```bash
# Подключение к контейнеру
docker exec -it docker-clickhouse-1 bash

# Проверка статуса сервисов
docker compose ps

# Проверка использования ресурсов
docker stats
```

## 🔒 Безопасность

### Пароли по умолчанию

- **ClickHouse**: trading_user / trading_password_2024
- **Redis**: redis_password_2024
- **MongoDB**: admin / password
- **PostgreSQL**: postgres / postgres
- **Grafana**: admin / admin
- **MinIO**: minioadmin / minioadmin123

⚠️ **ВАЖНО**: Измените все пароли в production среде!

### Сетевая безопасность

Все сервисы доступны только локально. Для production используйте:
- Reverse proxy (nginx, traefik)
- VPN или приватную сеть
- Firewall правила

## 📚 Дополнительная информация

### Требования

- Docker 20.10+
- Docker Compose 2.0+
- Минимум 8GB RAM
- Минимум 20GB свободного места

### Производительность

- ClickHouse оптимизирован для аналитических запросов
- Redis настроен с ограничением памяти 2GB
- Все сервисы имеют health checks
- Автоматический restart при сбоях

### Резервное копирование

```bash
# Создание backup
docker compose exec clickhouse clickhouse-backup create

# Восстановление backup
docker compose exec clickhouse clickhouse-backup restore
```

## 🆘 Устранение проблем

### Частые проблемы

1. **Порт уже используется**
   ```bash
   # Проверка занятых портов
   netstat -tulpn | grep :8123
   
   # Остановка конфликтующих сервисов
   sudo systemctl stop clickhouse-server
   ```

2. **Недостаточно памяти**
   ```bash
   # Проверка использования памяти
   free -h
   
   # Увеличение лимитов Docker
   # В docker-compose.yml добавьте:
   deploy:
     resources:
       limits:
         memory: 4G
   ```

3. **Проблемы с правами доступа**
   ```bash
   # Исправление прав доступа
   sudo chown -R 1000:1000 ./clickhouse
   sudo chown -R 1000:1000 ./prometheus
   ```

### Логи ошибок

```bash
# Просмотр логов ошибок
docker compose logs --tail=100 | grep ERROR

# Проверка статуса сервисов
docker compose ps
```

## 📞 Поддержка

При возникновении проблем:

1. Проверьте логи сервисов
2. Убедитесь, что все порты свободны
3. Проверьте доступность ресурсов (память, диск)
4. Перезапустите сервисы
5. При необходимости выполните полную очистку и пересборку