//! Robots - Торговый робот на Rust
//!
//! Этот крейт предоставляет функциональность для создания торговых роботов,
//! включая работу с данными, индикаторами, условиями и стратегиями.

pub mod candles;
pub mod condition;
pub mod data_access;
pub mod data_model;
pub mod debug;
pub mod di;
pub mod discovery;
pub mod indicators;
pub mod metrics;
pub mod optimization;
pub mod position;
pub mod risk;
pub mod strategy;

// Re-export основных модулей для удобства использования
pub use data_access::*;
pub use data_model::*;
