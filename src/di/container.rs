//! Контейнер зависимостей
//!
//! Централизованное хранилище и разрешение зависимостей.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use super::service_provider::ServiceProviderError;

/// Время жизни сервиса
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLifetime {
    /// Singleton - один экземпляр на весь контейнер
    Singleton,
    /// Transient - новый экземпляр при каждом запросе
    Transient,
    /// Scoped - один экземпляр в рамках области видимости
    Scoped,
}

/// Фабрика для создания сервисов
type ServiceFactory = Box<dyn Fn() -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// Контейнер зависимостей
/// 
/// Позволяет регистрировать и разрешать зависимости.
/// Поддерживает три типа времени жизни: Singleton, Transient, Scoped.
pub struct ServiceContainer {
    /// Зарегистрированные сервисы
    registrations: Arc<RwLock<HashMap<TypeId, (ServiceLifetime, ServiceFactory)>>>,
    /// Singleton экземпляры
    singletons: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceContainer {
    /// Создать новый контейнер
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            singletons: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Зарегистрировать сервис как Singleton
    /// 
    /// Сервис будет создан один раз и переиспользоваться.
    pub fn register_singleton<T: 'static + Send + Sync, F>(&self, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let factory: ServiceFactory = Box::new(move || {
            Arc::new(factory()) as Arc<dyn Any + Send + Sync>
        });
        
        let mut registrations = self.registrations.write().unwrap();
        registrations.insert(type_id, (ServiceLifetime::Singleton, factory));
    }
    
    /// Зарегистрировать готовый экземпляр как Singleton
    pub fn register_instance<T: 'static + Send + Sync>(&self, instance: Arc<T>) {
        let type_id = TypeId::of::<T>();
        let instance_any = instance.clone() as Arc<dyn Any + Send + Sync>;
        
        // Сохраняем экземпляр в singletons
        {
            let mut singletons = self.singletons.write().unwrap();
            singletons.insert(type_id, instance_any);
        }
        
        // Регистрируем фабрику, которая возвращает этот экземпляр
        let singletons_clone = self.singletons.clone();
        let factory: ServiceFactory = Box::new(move || {
            let singletons = singletons_clone.read().unwrap();
            singletons.get(&type_id).unwrap().clone()
        });
        
        let mut registrations = self.registrations.write().unwrap();
        registrations.insert(type_id, (ServiceLifetime::Singleton, factory));
    }
    
    /// Зарегистрировать сервис как Transient
    /// 
    /// Новый экземпляр будет создаваться при каждом запросе.
    pub fn register_transient<T: 'static + Send + Sync, F>(&self, factory: F)
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let factory: ServiceFactory = Box::new(move || {
            Arc::new(factory()) as Arc<dyn Any + Send + Sync>
        });
        
        let mut registrations = self.registrations.write().unwrap();
        registrations.insert(type_id, (ServiceLifetime::Transient, factory));
    }
    
    /// Разрешить зависимость по типу
    /// 
    /// Возвращает `Some(Arc<T>)` если сервис зарегистрирован,
    /// `None` в противном случае.
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        
        // Проверяем регистрацию
        let registrations = self.registrations.read().unwrap();
        let (lifetime, factory) = registrations.get(&type_id)?;
        
        match lifetime {
            ServiceLifetime::Singleton => {
                // Проверяем, есть ли уже созданный экземпляр
                {
                    let singletons = self.singletons.read().unwrap();
                    if let Some(instance) = singletons.get(&type_id) {
                        // Используем downcast для приведения типа
                        // T имеет bounds Send + Sync, что гарантирует безопасность
                        return instance.clone().downcast::<T>().ok();
                    }
                }
                
                // Создаем новый экземпляр и сохраняем
                let instance = factory();
                let typed_instance = instance.clone().downcast::<T>().ok()?;
                
                {
                    let mut singletons = self.singletons.write().unwrap();
                    singletons.insert(type_id, instance);
                }
                Some(typed_instance)
            }
            ServiceLifetime::Transient => {
                // Создаем новый экземпляр каждый раз
                let instance = factory();
                instance.downcast::<T>().ok()
            }
            ServiceLifetime::Scoped => {
                // Scoped сервисы должны разрешаться через ServiceScope
                // Здесь возвращаем None, так как scope не предоставлен
                None
            }
        }
    }
    
    /// Разрешить зависимость или вернуть ошибку
    pub fn resolve_required<T: 'static + Send + Sync>(&self) -> Result<Arc<T>, ServiceProviderError> {
        self.resolve()
            .ok_or_else(|| ServiceProviderError::ServiceNotFound {
                type_name: std::any::type_name::<T>(),
            })
    }
    
    /// Проверить, зарегистрирован ли сервис
    pub fn is_registered<T: 'static>(&self) -> bool {
        let registrations = self.registrations.read().unwrap();
        registrations.contains_key(&TypeId::of::<T>())
    }
    
    /// Очистить все регистрации (в основном для тестов)
    pub fn clear(&self) {
        let mut registrations = self.registrations.write().unwrap();
        registrations.clear();
        
        let mut singletons = self.singletons.write().unwrap();
        singletons.clear();
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}
