//! Robots - Торговый робот на Rust
//!
//! Этот крейт предоставляет функциональность для создания торговых роботов,
//! включая работу с данными, индикаторами, условиями и стратегиями.

pub mod app;
pub mod condition;
pub mod core;
pub mod data_access;
pub mod data_model;
pub mod indicators;

// Re-export основных модулей для удобства использования
pub use data_access::*;
pub use data_model::*;
