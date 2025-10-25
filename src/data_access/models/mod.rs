//! Модели данных для торгового робота

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Свеча (OHLCV данные)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Торговая сделка
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
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
    pub quantity: f64,
    pub price: Option<f64>,
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
    pub free: f64,
    pub locked: f64,
    pub total: f64,
}

/// Текущая цена (тикер)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

/// Запрос на размещение ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub time_in_force: Option<TimeInForce>,
}

/// Ответ на размещение ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub average_price: Option<f64>,
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
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub created_at: DateTime<Utc>,
}

/// Технический индикатор
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indicator {
    pub name: String,
    pub value: f64,
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
    pub confidence: f64,
    pub price: f64,
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
    pub total_value: f64,
    pub cash: f64,
    pub positions: Vec<Position>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Позиция в портфеле
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
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
