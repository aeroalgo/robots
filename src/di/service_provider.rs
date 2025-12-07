//! Trait для провайдера сервисов
//!
//! Определяет интерфейс для разрешения зависимостей,
//! что позволяет легко подменять реализации в тестах.

use std::any::{Any, TypeId};
use std::sync::Arc;

/// Trait для провайдера сервисов
/// 
/// Позволяет разрешать зависимости по типу, что упрощает
/// тестирование и делает код более гибким.
pub trait ServiceProvider: Send + Sync {
    /// Получить сервис по типу
    /// 
    /// Возвращает `Some(Arc<T>)` если сервис зарегистрирован,
    /// `None` в противном случае.
    fn get_service<T: 'static + Send + Sync>(&self) -> Option<Arc<T>>;
    
    /// Получить сервис по типу или вернуть ошибку
    /// 
    /// Удобный метод для случаев, когда сервис обязателен.
    fn get_required_service<T: 'static + Send + Sync>(&self) -> Result<Arc<T>, ServiceProviderError> {
        self.get_service()
            .ok_or_else(|| ServiceProviderError::ServiceNotFound {
                type_name: std::any::type_name::<T>(),
            })
    }
}

/// Ошибки провайдера сервисов
#[derive(Debug, thiserror::Error)]
pub enum ServiceProviderError {
    #[error("service not found: {type_name}")]
    ServiceNotFound { type_name: &'static str },
    
    #[error("service registration error: {message}")]
    RegistrationError { message: String },
}

/// Реализация ServiceProvider для ServiceContainer
impl ServiceProvider for crate::di::container::ServiceContainer {
    fn get_service<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resolve::<T>()
    }
}

