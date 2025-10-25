# Arrow/Parquet Infrastructure

## Обзор

Эта директория содержит конфигурации и Dockerfile для Apache Arrow и Parquet сервисов, которые обеспечивают высокопроизводительную обработку данных для торгового робота.

## Сервисы

### 1. Apache Arrow Flight Server (Port 8815)
- **Назначение**: Высокоскоростная передача данных между компонентами
- **Использование**: Передача больших массивов данных между ClickHouse, Redis и приложением
- **Преимущества**: Нулевое копирование данных, векторизованные операции
- **Dockerfile**: `Dockerfile.arrow-flight`

### 2. Apache Parquet Tools (Port 8816)
- **Назначение**: Работа с Parquet файлами
- **Использование**: Чтение/запись исторических данных в колоночном формате
- **Преимущества**: Высокое сжатие, быстрые аналитические запросы
- **Dockerfile**: `Dockerfile.parquet-tools`

### 3. DataFusion (Port 8817)
- **Назначение**: SQL-запросы к Arrow/Parquet данным
- **Использование**: Аналитические запросы без загрузки данных в ClickHouse
- **Преимущества**: Быстрые агрегации, фильтрация данных
- **Dockerfile**: `Dockerfile.datafusion`

## Конфигурация

### flight.conf
- Настройки Arrow Flight Server
- Память, соединения, производительность
- Безопасность и логирование

### datafusion.conf
- Настройки DataFusion
- Память, кэширование, параллелизм
- Директории данных и производительность

## Использование

### Запуск сервисов
```bash
cd docker
docker-compose up arrow-flight parquet-tools datafusion
```

### Проверка статуса
```bash
# Arrow Flight
curl http://localhost:8815/health

# Parquet Tools
curl http://localhost:8816/health

# DataFusion
curl http://localhost:8817/health
```

### Примеры использования

#### Передача данных через Arrow Flight
```rust
use arrow_flight::flight_service_client::FlightServiceClient;
use arrow_flight::FlightDescriptor;

let mut client = FlightServiceClient::new("http://localhost:8815").await?;
let descriptor = FlightDescriptor::new_cmd("SELECT * FROM candles");
let flight_info = client.get_flight_info(descriptor).await?;
```

#### SQL запросы через DataFusion
```bash
curl -X POST http://localhost:8817/query \
  -H "Content-Type: application/json" \
  -d '{"query": "SELECT symbol, AVG(close) FROM candles GROUP BY symbol"}'
```

#### Работа с Parquet
```bash
curl -X POST http://localhost:8816/convert \
  -H "Content-Type: application/json" \
  -d '{"data": [{"symbol": "BTC", "price": 50000}]}'
```

## Интеграция с основным приложением

### Rust зависимости
```toml
[dependencies]
arrow = "50.0"
parquet = "50.0"
datafusion = "37.0"
arrow-flight = "50.0"
```

### Переменные окружения
```bash
ARROW_FLIGHT_HOST=arrow-flight
ARROW_FLIGHT_PORT=8815
PARQUET_TOOLS_HOST=parquet-tools
PARQUET_TOOLS_PORT=8816
DATAFUSION_HOST=datafusion
DATAFUSION_PORT=8817
```

## Мониторинг

### Метрики
- Количество запросов
- Время выполнения
- Использование памяти
- Размер передаваемых данных

### Логи
- Все сервисы логируют в `/var/log/`
- Уровень логирования настраивается в конфигурации
- Логи доступны через Docker logs

## Производительность

### Рекомендации
- Используйте Arrow Flight для передачи больших массивов данных
- Кэшируйте результаты в Redis
- Используйте Parquet для архивного хранения
- Настройте DataFusion для параллельных запросов

### Оптимизация
- Увеличьте batch_size для больших запросов
- Используйте сжатие LZ4 для экономии памяти
- Настройте пулы памяти для каждого сервиса
- Используйте SIMD оптимизации Arrow

## Troubleshooting

### Частые проблемы
1. **Недостаток памяти**: Увеличьте max_memory в конфигурации
2. **Медленные запросы**: Проверьте batch_size и параллелизм
3. **Ошибки соединения**: Проверьте порты и сеть Docker

### Диагностика
```bash
# Проверка логов
docker logs arrow-flight
docker logs datafusion

# Проверка ресурсов
docker stats arrow-flight datafusion parquet-tools

# Проверка сети
docker network ls
docker network inspect robots_default
```

## Сборка образов

### Локальная сборка
```bash
cd docker/arrow

# Arrow Flight
docker build -f Dockerfile.arrow-flight -t arrow-flight:latest .

# Parquet Tools
docker build -f Dockerfile.parquet-tools -t parquet-tools:latest .

# DataFusion
docker build -f Dockerfile.datafusion -t datafusion:latest .
```

### Использование готовых образов
```bash
# Запуск через docker-compose
docker-compose up --build arrow-flight parquet-tools datafusion
```
