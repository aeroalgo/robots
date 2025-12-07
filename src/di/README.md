# Dependency Injection (DI) Container

## Обзор

DI контейнер предоставляет централизованное управление зависимостями для улучшения тестируемости и гибкости системы.

## Основные компоненты

### ServiceContainer

Центральный контейнер для регистрации и разрешения зависимостей.

```rust
use crate::di::ServiceContainer;

let container = ServiceContainer::new();

// Регистрация Singleton
container.register_singleton::<MyService, _>(|| MyService::new());

// Регистрация готового экземпляра
let instance = Arc::new(MyService::new());
container.register_instance::<MyService>(instance);

// Разрешение зависимости
if let Some(service) = container.resolve::<MyService>() {
    // Использование сервиса
}
```

### ServiceProvider Trait

Trait для абстракции DI контейнера, позволяющий легко подменять реализации в тестах.

### ServiceScope

Область видимости для scoped сервисов (один экземпляр в рамках области).

## Использование в BacktestExecutor

### Legacy режим (без DI)

```rust
let executor = BacktestEngine::new(strategy, frames)?;
```

### С DI контейнером

```rust
use crate::di::{setup_backtest_container, ServiceContainer};
use std::sync::Arc;

// Настройка контейнера
let container = setup_backtest_container(
    "strategy-1",
    BacktestConfig::default(),
);

// Создание executor с DI
let executor = BacktestExecutor::new_with_provider(
    strategy,
    frames,
    Some(Arc::new(container)),
)?;
```

## Использование в BacktestEngine

Аналогично `BacktestExecutor`:

```rust
let engine = BacktestEngine::new_with_provider(
    strategy,
    frames,
    Some(Arc::new(container)),
)?;
```

## Преимущества

1. **Тестируемость**: Легко подменять зависимости mock-объектами
2. **Гибкость**: Централизованное управление зависимостями
3. **SOLID**: Соблюдение принципа Dependency Inversion
4. **Обратная совместимость**: Legacy режим сохранен

## Время жизни сервисов

- **Singleton**: Один экземпляр на весь контейнер
- **Transient**: Новый экземпляр при каждом запросе
- **Scoped**: Один экземпляр в рамках области видимости

## Примеры

### Регистрация кастомного сервиса

```rust
let container = ServiceContainer::new();

// Регистрация с фабрикой
container.register_singleton::<MyService, _>(|| {
    MyService::with_config("custom-config")
});

// Регистрация готового экземпляра
let custom_service = Arc::new(MyService::new());
container.register_instance::<MyService>(custom_service);
```

### Использование в тестах

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::di::ServiceContainer;
    use std::sync::Arc;

    #[test]
    fn test_with_mock() {
        let container = ServiceContainer::new();
        
        // Регистрируем mock-объект
        let mock_service = Arc::new(MockService::new());
        container.register_instance::<MyService>(mock_service);
        
        // Создаем executor с mock-зависимостями
        let executor = BacktestEngine::new_with_provider(
            strategy,
            frames,
            Some(Arc::new(container)),
        )?;
        
        // Тестируем...
    }
}
```




