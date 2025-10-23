# Краткое описание проекта

## Название проекта
Система бэктеста торговых стратегий на Rust

## Цель проекта
Создание высокопроизводительной, масштабируемой системы для бэктестирования торговых стратегий с поддержкой live trading и бумажной торговли.

## Основные компоненты

### 1. Инфраструктурный слой
- Хранилище данных (ClickHouse + Redis + Arrow/Parquet)
- Генератор баров/тиков
- Event Bus System
- Dependency Injection & Configuration

### 2. Слой модели данных
- QuoteFrame для работы с котировками
- Vector для векторных операций
- Meta для метаданных

### 3. Слой индикаторов
- Базовая система индикаторов
- Registry и фабрика
- Execution Engine с DAG
- Библиотека технических и ML индикаторов

### 4. Слой условий
- Атомарные и составные условия
- Smart Condition Builder
- Условия с нечеткой логикой

### 5. Слой позиций и ордеров
- Управление ордерами
- Управление позициями
- Smart Position Management

### 6. Слой управления рисками
- Stop Loss Management
- Take Profit Management
- Portfolio Risk Management

### 7. Слой метрик (ОБНОВЛЕНО - Детальная система метрик)
- **Базовые метрики производительности**: Total Profit, Profit In Pips, Yearly AVG Profit, Yearly AVG % Return, CAGR
- **Метрики риска и доходности**: Sharpe Ratio, Profit Factor, Return/DD ratio, Winning Percentage
- **Метрики просадки**: Draw Down, % Draw Down, Max Consec Wins, Max Consec Losses
- **Статистические метрики**: R Expectancy, R Expectancy Score, STR Quality Number, SQN Score
- **Продвинутые метрики**: Z-Score, Z-Probability, Expectancy, Deviation, Exposure
- **Метрики симметрии и стабильности**: Symmetry, Trades Symmetry, NSymmetry, Stability
- **Метрики застоя**: Stagnation In Days, Stagnation In %, Gross Profit, Gross Loss
- **Дополнительные метрики**: Average Win, Average Loss, Payout ratio, AHPR, Daily/Monthly AVG Profit

### 8. Слой оптимизации
- Поисковые алгоритмы
- Multi-objective optimization
- Smart Optimization

### 9. Walk-Forward & Validation
- Методы разделения данных
- Статистическая валидация
- Smart Validation

### 10. Live Trading Gateway
- Интеграция с брокерами
- Real-time processing
- Smart Execution

### 11. UI & API Layer
- REST API
- Web UI с Strategy Builder
- Dashboard

## Технологический стек
- **Язык программирования**: Rust
- **Хранилище данных**: ClickHouse (исторические данные) + Redis (кэш) + Arrow/Parquet (вычисления)
- **Архитектура**: Модульная, многослойная
- **Производительность**: SIMD, GPU acceleration, параллельные алгоритмы
- **Кэширование**: Многоуровневая система (Redis L1 + Arrow L2 + ClickHouse L3)

## Уникальные особенности
- AI-powered strategy generation
- Social trading features
- Advanced analytics с ML
- Real-time optimization
- Adaptive indicators
- Context-aware conditions

## Целевая аудитория
- Начинающие трейдеры
- Профессиональные трейдеры
- Институциональные инвесторы
- Исследователи в области количественных финансов
