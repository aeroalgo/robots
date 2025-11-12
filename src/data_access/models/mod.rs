//! Модели данных для торгового робота

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Свеча (OHLCV данные)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
}

/// Торговая сделка
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: f32,
    pub quantity: f32,
    pub side: TradeSide,
    pub order_id: Option<String>,
}

/// Направление сделки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// Торговый ордер
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f32,
    pub price: Option<f32>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Направление ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Тип ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// Статус ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

/// Пользователь
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Баланс аккаунта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: f32,
    pub locked: f32,
    pub total: f32,
}

/// Текущая цена (тикер)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub price: f32,
    pub bid: f32,
    pub ask: f32,
    pub volume: f32,
    pub timestamp: DateTime<Utc>,
}

/// Запрос на размещение ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f32,
    pub price: Option<f32>,
    pub stop_price: Option<f32>,
    pub time_in_force: Option<TimeInForce>,
}

/// Ответ на размещение ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: OrderStatus,
    pub filled_quantity: f32,
    pub average_price: Option<f32>,
}

/// Время действия ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Canceled
    IOC, // Immediate or Cancel
    FOK, // Fill or Kill
}

/// Торговая стратегия
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parameters: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Результат бэктеста
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub strategy_id: String,
    pub symbol: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_return: f32,
    pub sharpe_ratio: f32,
    pub max_drawdown: f32,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f32,
    pub created_at: DateTime<Utc>,
}

/// Технический индикатор
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    pub name: String,
    pub value: f32,
    pub timestamp: DateTime<Utc>,
    pub parameters: serde_json::Value,
}

/// Торговый сигнал
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub signal_type: SignalType,
    pub confidence: f32,
    pub price: f32,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Тип торгового сигнала
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
}

/// Портфель
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub total_value: f32,
    pub cash: f32,
    pub positions: Vec<Position>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Позиция в портфеле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f32,
    pub average_price: f32,
    pub current_price: f32,
    pub unrealized_pnl: f32,
    pub realized_pnl: f32,
}

/// Системное событие
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub id: String,
    pub event_type: EventType,
    pub message: String,
    pub severity: EventSeverity,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Тип системного события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    StrategyStarted,
    StrategyStopped,
    OrderPlaced,
    OrderFilled,
    OrderCanceled,
    Error,
    Warning,
    Info,
}

/// Серьезность события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
    Critical,
}
