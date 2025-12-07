//! Область видимости сервисов
//!
//! Позволяет создавать scoped сервисы, которые живут
//! в рамках определенной области (например, одного запроса или бэктеста).

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Область видимости сервисов
/// 
/// Используется для создания scoped сервисов, которые
/// существуют только в рамках этой области.
pub struct ServiceScope {
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl ServiceScope {
    /// Создать новую область видимости
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    /// Зарегистрировать сервис в области видимости
    pub fn register<T: 'static + Send + Sync>(&mut self, service: Arc<T>) {
        self.services.insert(TypeId::of::<T>(), service);
    }
    
    /// Получить сервис из области видимости
    pub fn get<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|service| service.clone().downcast::<T>().ok())
    }
}

impl Default for ServiceScope {
    fn default() -> Self {
        Self::new()
    }
}

