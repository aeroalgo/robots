//! Вспомогательные функции для настройки DI контейнера
//!
//! Предоставляет готовые конфигурации для типичных сценариев использования.

use std::sync::Arc;

use crate::position::PositionManager;
use crate::risk::RiskManager;
use crate::strategy::base::Strategy;
use crate::strategy::executor::BacktestConfig;
use crate::strategy::types::StrategyId;

use super::container::ServiceContainer;

/// Настроить DI контейнер для бэктеста
/// 
/// Регистрирует стандартные сервисы, необходимые для выполнения бэктеста:
/// - PositionManager (Singleton)
/// - RiskManager (Singleton)
/// 
/// # Пример
/// 
/// ```rust
/// use crate::di::setup::setup_backtest_container;
/// use crate::backtest::BacktestEngine;
/// 
/// let container = setup_backtest_container(
///     "strategy-1",
///     BacktestConfig::default(),
/// );
/// 
/// let executor = BacktestEngine::new_with_provider(
///     strategy,
///     frames,
///     Some(Arc::new(container)),
/// )?;
/// ```
pub fn setup_backtest_container(
    strategy_id: impl Into<StrategyId>,
    config: BacktestConfig,
) -> ServiceContainer {
    let container = ServiceContainer::new();
    
    // Регистрируем PositionManager как Singleton
    let strategy_id_clone = strategy_id.into();
    let config_clone = config.clone();
    container.register_singleton::<PositionManager, _>(move || {
        PositionManager::new(strategy_id_clone.clone())
            .with_capital(
                config_clone.initial_capital,
                config_clone.use_full_capital,
                config_clone.reinvest_profits,
            )
    });
    
    // Регистрируем RiskManager как Singleton
    container.register_singleton::<RiskManager, _>(|| {
        RiskManager::new()
    });
    
    container
}

/// Настроить DI контейнер с кастомными сервисами
/// 
/// Позволяет зарегистрировать кастомные реализации сервисов.
/// 
/// # Пример
/// 
/// ```rust
/// use crate::di::setup::setup_custom_container;
/// use crate::position::PositionManager;
/// 
/// let container = setup_custom_container();
/// 
/// // Регистрируем кастомный PositionManager
/// let custom_pm = Arc::new(PositionManager::new("custom-strategy"));
/// container.register_instance::<PositionManager>(custom_pm);
/// ```
pub fn setup_custom_container() -> ServiceContainer {
    ServiceContainer::new()
}




