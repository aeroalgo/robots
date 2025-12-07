//! Dependency Injection контейнер
//!
//! Предоставляет централизованное управление зависимостями для улучшения
//! тестируемости и гибкости системы.

mod container;
mod service_provider;
mod service_scope;
mod setup;

pub use container::{ServiceContainer, ServiceLifetime};
pub use service_provider::ServiceProvider;
pub use service_scope::ServiceScope;
pub use setup::{setup_backtest_container, setup_custom_container};

