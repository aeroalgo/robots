use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::view::{ActivePosition, PositionBook, PositionInsights};
use crate::data_model::types::{Symbol, TimeFrame};
use crate::metrics::PortfolioSnapshot;
use crate::strategy::context::StrategyContext;
use crate::strategy::types::{
    PositionDirection, PriceField, StopSignal, StopSignalKind, StrategyDecision, StrategyError,
    StrategyId, StrategySignal,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PositionKey {
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub direction: PositionDirection,
    pub position_group: Option<String>,
    pub entry_rule_id: Option<String>,
}

impl PositionKey {
    pub fn new(
        symbol: Symbol,
        timeframe: TimeFrame,
        direction: PositionDirection,
        position_group: Option<String>,
        entry_rule_id: Option<String>,
    ) -> Self {
        Self {
            symbol,
            timeframe,
            direction,
            position_group,
            entry_rule_id,
        }
    }

    pub fn matches(
        &self,
        symbol: &Symbol,
        timeframe: &TimeFrame,
        direction: &PositionDirection,
    ) -> bool {
        &self.symbol == symbol && &self.timeframe == timeframe && &self.direction == direction
    }
}

impl fmt::Display for PositionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.position_group, &self.entry_rule_id) {
            (Some(group), Some(entry)) => write!(
                f,
                "{}@{}:{:?}[group={} entry={}]",
                self.symbol.descriptor(),
                self.timeframe.identifier(),
                self.direction,
                group,
                entry
            ),
            (Some(group), None) => write!(
                f,
                "{}@{}:{:?}[group={}]",
                self.symbol.descriptor(),
                self.timeframe.identifier(),
                self.direction,
                group
            ),
            (None, Some(entry)) => write!(
                f,
                "{}@{}:{:?}[entry={}]",
                self.symbol.descriptor(),
                self.timeframe.identifier(),
                self.direction,
                entry
            ),
            (None, None) => write!(
                f,
                "{}@{}:{:?}",
                self.symbol.descriptor(),
                self.timeframe.identifier(),
                self.direction
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PositionStatus {
    PendingEntry,
    Open,
    Closing,
    Closed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct PositionState {
    pub id: String,
    pub key: PositionKey,
    pub status: PositionStatus,
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Clone, Debug)]
pub struct OrderTicket {
    pub id: String,
    pub position_id: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub direction: PositionDirection,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub quantity: f64,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum PositionEvent {
    OrderFilled(OrderTicket),
    PositionOpened(PositionState),
    PositionUpdated(PositionState),
    PositionClosed(PositionState),
}

#[derive(Clone, Debug)]
pub struct ClosedTrade {
    pub position_id: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub direction: PositionDirection,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub entry_time: Option<DateTime<Utc>>,
    pub exit_time: Option<DateTime<Utc>>,
    pub pnl: f64,
    pub entry_rule_id: Option<String>,
    pub exit_rule_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutionReport {
    pub orders: Vec<OrderTicket>,
    pub opened_positions: Vec<PositionState>,
    pub updated_positions: Vec<PositionState>,
    pub closed_positions: Vec<PositionState>,
    pub closed_trades: Vec<ClosedTrade>,
}

#[derive(Debug, thiserror::Error)]
pub enum PositionError {
    #[error(transparent)]
    Strategy(#[from] StrategyError),
    #[error("missing symbol for timeframe {0:?}")]
    MissingSymbol(TimeFrame),
    #[error("missing price series for timeframe {0:?} field {1:?}")]
    MissingPriceSeries(TimeFrame, PriceField),
    #[error("unsupported direction {0:?}")]
    UnsupportedDirection(PositionDirection),
    #[error("persistence error: {0}")]
    Persistence(#[source] anyhow::Error),
    #[error("event handler error: {0}")]
    Event(#[source] anyhow::Error),
}

#[async_trait]
pub trait PositionPersistence: Send + Sync {
    async fn persist_position(&self, position: &PositionState) -> anyhow::Result<()>;
    async fn persist_order(&self, order: &OrderTicket) -> anyhow::Result<()>;
    async fn persist_event(&self, event: &PositionEvent) -> anyhow::Result<()>;
}

#[async_trait]
pub trait PositionEventListener: Send + Sync {
    async fn on_event(&self, event: &PositionEvent) -> anyhow::Result<()>;
}

pub struct PositionManager {
    strategy_id: StrategyId,
    positions: HashMap<String, PositionState>,
    open_index: HashMap<PositionKey, String>,
    orders: HashMap<String, OrderTicket>,
    event_history: Vec<PositionEvent>,
    listeners: Vec<Arc<dyn PositionEventListener>>,
    persistence: Option<Arc<dyn PositionPersistence>>,
    sequence: u64,
    portfolio: PortfolioSnapshot,
}

impl PositionManager {
    pub fn new(strategy_id: impl Into<StrategyId>) -> Self {
        Self {
            strategy_id: strategy_id.into(),
            positions: HashMap::new(),
            open_index: HashMap::new(),
            orders: HashMap::new(),
            event_history: Vec::new(),
            listeners: Vec::new(),
            persistence: None,
            sequence: 0,
            portfolio: PortfolioSnapshot::default(),
        }
    }

    pub fn reset(&mut self) {
        self.positions.clear();
        self.open_index.clear();
        self.orders.clear();
        self.event_history.clear();
        self.sequence = 0;
        self.portfolio.reset();
    }

    pub fn set_persistence(&mut self, persistence: Option<Arc<dyn PositionPersistence>>) {
        self.persistence = persistence;
    }

    pub fn register_listener(&mut self, listener: Arc<dyn PositionEventListener>) {
        self.listeners.push(listener);
    }

    pub fn strategy_id(&self) -> &StrategyId {
        &self.strategy_id
    }

    pub fn positions(&self) -> impl Iterator<Item = &PositionState> {
        self.positions.values()
    }

    pub fn portfolio_snapshot(&self) -> &PortfolioSnapshot {
        &self.portfolio
    }

    pub fn event_history(&self) -> &[PositionEvent] {
        &self.event_history
    }

    pub fn open_position_count(&self) -> usize {
        self.open_index.len()
    }

    pub fn process_decision(
        &mut self,
        context: &mut StrategyContext,
        decision: &StrategyDecision,
    ) -> Result<ExecutionReport, PositionError> {
        let mut report = ExecutionReport::default();
        let has_active_positions = !self.open_index.is_empty();

        if has_active_positions {
            let mut stop_ids: HashSet<String> = HashSet::new();
            let mut stop_signals: Vec<&StopSignal> = decision.stop_signals.iter().collect();
            stop_signals.sort_unstable_by_key(|signal| signal.priority);
            for stop in stop_signals {
                stop_ids.insert(stop.signal.rule_id.clone());
                let reason = format!("stop:{:?}", stop.kind);
                self.handle_exit_signal(
                    context,
                    &stop.signal,
                    Some(stop.exit_price),
                    Some(reason),
                    &mut report,
                )?;
            }
            for exit in &decision.exits {
                if stop_ids.contains(&exit.rule_id)
                    && exit.tags.iter().any(|tag| tag.eq_ignore_ascii_case("stop"))
                {
                    continue;
                }
                self.handle_exit_signal(context, exit, None, None, &mut report)?;
            }
        }
        for entry in &decision.entries {
            self.handle_entry_signal(context, entry, &mut report)?;
        }
        let snapshot = self.snapshot_active_positions();
        context.set_active_positions(snapshot);
        Ok(report)
    }

    fn handle_entry_signal(
        &mut self,
        context: &StrategyContext,
        signal: &StrategySignal,
        report: &mut ExecutionReport,
    ) -> Result<(), PositionError> {
        let direction = match signal.direction.clone() {
            PositionDirection::Long => PositionDirection::Long,
            PositionDirection::Short => PositionDirection::Short,
            PositionDirection::Flat => return Ok(()),
            PositionDirection::Both => {
                return Err(PositionError::UnsupportedDirection(PositionDirection::Both))
            }
        };
        let mut quantity = signal.quantity.unwrap_or(1.0);
        if quantity.abs() <= f64::EPSILON {
            quantity = 1.0;
        }
        let info = match Self::resolve_entry_snapshot(context, &signal.timeframe)? {
            Some(snapshot) => snapshot,
            None => return Ok(()),
        };
        let position_group = signal
            .position_group
            .clone()
            .or_else(|| Some(signal.rule_id.clone()));
        let entry_rule_value = signal
            .entry_rule_id
            .clone()
            .unwrap_or_else(|| signal.rule_id.clone());
        let entry_rule_id = Some(entry_rule_value.clone());
        if let Some(opposite_direction) = opposite_direction(&direction) {
            let opposite_ids: Vec<String> = self
                .open_index
                .iter()
                .filter_map(|(key, id)| {
                    if key.matches(&info.symbol, &info.timeframe, &opposite_direction) {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for opposite_id in opposite_ids {
                if let Some(state) = self.positions.get(&opposite_id) {
                    let close_qty = state.quantity;
                    if close_qty.abs() > f64::EPSILON {
                        self.close_position(
                            opposite_id,
                            info.price,
                            close_qty,
                            info.timestamp,
                            Some("reversal".to_string()),
                            Some(signal.rule_id.clone()),
                            report,
                        )?;
                    }
                }
            }
        }
        let key = PositionKey::new(
            info.symbol.clone(),
            info.timeframe.clone(),
            direction.clone(),
            position_group,
            entry_rule_id,
        );
        if self.open_index.contains_key(&key) {
            return Ok(());
        }
        self.open_new_position(key, quantity, info.price, info.timestamp, signal, report)?;
        Ok(())
    }

    fn handle_exit_signal(
        &mut self,
        context: &StrategyContext,
        signal: &StrategySignal,
        price_hint: Option<f64>,
        reason: Option<String>,
        report: &mut ExecutionReport,
    ) -> Result<(), PositionError> {
        let direction = match signal.direction.clone() {
            PositionDirection::Long => PositionDirection::Long,
            PositionDirection::Short => PositionDirection::Short,
            PositionDirection::Flat => return Ok(()),
            PositionDirection::Both => {
                return Err(PositionError::UnsupportedDirection(PositionDirection::Both))
            }
        };
        let info = Self::resolve_market_snapshot(context, &signal.timeframe, price_hint)?;
        let target_groups = if signal.target_entry_ids.is_empty() {
            None
        } else {
            Some(
                signal
                    .target_entry_ids
                    .iter()
                    .cloned()
                    .collect::<std::collections::HashSet<_>>(),
            )
        };
        let targets: Vec<(String, f64)> = self
            .open_index
            .iter()
            .filter_map(|(key, position_id)| {
                if !key.matches(&info.symbol, &info.timeframe, &direction) {
                    return None;
                }
                if let Some(groups) = &target_groups {
                    let matches_target = key
                        .position_group
                        .as_ref()
                        .map_or(false, |group| groups.contains(group))
                        || key
                            .entry_rule_id
                            .as_ref()
                            .map_or(false, |entry_id| groups.contains(entry_id));
                    if !matches_target {
                        return None;
                    }
                }
                let quantity = signal.quantity.unwrap_or_else(|| {
                    self.positions
                        .get(position_id)
                        .map(|state| state.quantity)
                        .unwrap_or(0.0)
                });
                if quantity.abs() > f64::EPSILON {
                    Some((position_id.clone(), quantity))
                } else {
                    None
                }
            })
            .collect();
        if targets.is_empty() {
            return Ok(());
        }
        let reason_label = reason.clone().unwrap_or_else(|| "exit".to_string());
        let reason_with_rule = format!("{} via {}", reason_label, signal.rule_id);
        for (position_id, quantity) in targets {
            self.close_position(
                position_id,
                info.price,
                quantity,
                info.timestamp,
                Some(reason_with_rule.clone()),
                Some(signal.rule_id.clone()),
                report,
            )?;
        }
        Ok(())
    }

    fn open_new_position(
        &mut self,
        key: PositionKey,
        quantity: f64,
        price: f64,
        timestamp: Option<DateTime<Utc>>,
        signal: &StrategySignal,
        report: &mut ExecutionReport,
    ) -> Result<(), PositionError> {
        let position_id = self.next_id("pos");
        let event_time = timestamp.unwrap_or_else(Utc::now);
        let mut metadata = HashMap::new();
        let entry_rule_value = signal
            .entry_rule_id
            .clone()
            .unwrap_or_else(|| signal.rule_id.clone());
        metadata.insert("entry_rule".to_string(), entry_rule_value.clone());
        let state = PositionState {
            id: position_id.clone(),
            key: key.clone(),
            status: PositionStatus::Open,
            quantity,
            average_price: price,
            current_price: price,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            opened_at: event_time,
            updated_at: event_time,
            closed_at: None,
            metadata,
        };
        let order = self.build_order(&position_id, &key, quantity, price);
        self.positions.insert(position_id.clone(), state.clone());
        self.open_index.insert(key, position_id.clone());
        self.orders.insert(order.id.clone(), order.clone());
        if let Some(persistence) = &self.persistence {
            // В бэктесте persistence не используется, пропускаем
            // persistence.persist_order(&order).await?;
            // persistence.persist_position(&state).await?;
        }
        // В бэктесте события не записываются
        // self.record_event(PositionEvent::OrderFilled(order.clone()))?;
        // self.record_event(PositionEvent::PositionOpened(state.clone()))?;
        self.refresh_portfolio_metrics();
        report.orders.push(order);
        report.opened_positions.push(state);
        Ok(())
    }

    fn scale_position(
        &mut self,
        position_id: String,
        quantity: f64,
        price: f64,
        signal: &StrategySignal,
        report: &mut ExecutionReport,
    ) -> Result<(), PositionError> {
        let now = Utc::now();
        if quantity.abs() <= f64::EPSILON {
            return Ok(());
        }
        let state = self.positions.get_mut(&position_id).ok_or_else(|| {
            PositionError::Persistence(anyhow!("position {} missing for scale", position_id))
        })?;
        let new_quantity = state.quantity + quantity;
        if new_quantity <= f64::EPSILON {
            drop(state);
            let exit_rule = signal
                .entry_rule_id
                .clone()
                .or_else(|| Some(signal.rule_id.clone()));
            self.close_position(
                position_id,
                price,
                quantity.abs(),
                None,
                Some("scale_to_flat".to_string()),
                exit_rule,
                report,
            )?;
            return Ok(());
        }
        state.average_price =
            ((state.average_price * state.quantity) + (price * quantity)) / new_quantity;
        state.quantity = new_quantity;
        state.current_price = price;
        state.status = PositionStatus::Open;
        state.updated_at = now;
        state
            .metadata
            .insert("last_entry_rule".to_string(), signal.rule_id.clone());
        let snapshot = state.clone();
        drop(state);
        let order = self.build_order(&position_id, &snapshot.key, quantity, price);
        self.orders.insert(order.id.clone(), order.clone());
        // В бэктесте persistence не используется
        // self.persist_order(&order)?;
        // self.persist_position(&snapshot)?;
        // В бэктесте события не записываются
        // self.record_event(PositionEvent::OrderFilled(order.clone()))?;
        // self.record_event(PositionEvent::PositionUpdated(snapshot.clone()))?;
        self.refresh_portfolio_metrics();
        report.orders.push(order);
        report.updated_positions.push(snapshot);
        Ok(())
    }

    fn close_position(
        &mut self,
        position_id: String,
        price: f64,
        quantity: f64,
        timestamp: Option<DateTime<Utc>>,
        reason: Option<String>,
        exit_rule_id: Option<String>,
        report: &mut ExecutionReport,
    ) -> Result<(), PositionError> {
        if quantity.abs() <= f64::EPSILON {
            return Ok(());
        }
        if !self.open_index.values().any(|id| id == &position_id) {
            return Ok(());
        }
        let event_time = timestamp.unwrap_or_else(Utc::now);
        let state = self.positions.get_mut(&position_id).ok_or_else(|| {
            PositionError::Persistence(anyhow!("position {} missing for close", position_id))
        })?;
        let exit_quantity = quantity.min(state.quantity);
        if exit_quantity.abs() <= f64::EPSILON {
            return Ok(());
        }
        state.current_price = price;
        let direction = state.key.direction.clone();
        let pnl = match direction {
            PositionDirection::Long => (price - state.average_price) * exit_quantity,
            PositionDirection::Short => (state.average_price - price) * exit_quantity,
            PositionDirection::Flat | PositionDirection::Both => 0.0,
        };
        state.quantity -= exit_quantity;
        state.realized_pnl += pnl;
        state.unrealized_pnl = 0.0;
        state.updated_at = event_time;
        if state.quantity <= f64::EPSILON {
            state.status = PositionStatus::Closed;
            state.closed_at = Some(event_time);
        } else {
            state.status = PositionStatus::Open;
        }
        if let Some(reason_value) = reason.clone() {
            state
                .metadata
                .insert("close_reason".to_string(), reason_value);
        }
        if let Some(rule_id) = exit_rule_id.as_ref() {
            state
                .metadata
                .insert("close_rule".to_string(), rule_id.clone());
        }
        let snapshot = state.clone();
        if snapshot.status == PositionStatus::Closed {
            self.open_index.remove(&snapshot.key);
        }
        drop(state);
        self.portfolio.realized_pnl += pnl;
        self.portfolio.update_equity();
        let trade = ClosedTrade {
            position_id: position_id.clone(),
            symbol: snapshot.key.symbol.clone(),
            timeframe: snapshot.key.timeframe.clone(),
            direction,
            quantity: exit_quantity,
            entry_price: snapshot.average_price,
            exit_price: price,
            entry_time: Some(snapshot.opened_at),
            exit_time: Some(event_time),
            pnl,
            entry_rule_id: snapshot.key.entry_rule_id.clone(),
            exit_rule_id: exit_rule_id.clone(),
        };
        let order = self.build_order(&position_id, &snapshot.key, exit_quantity, price);
        self.orders.insert(order.id.clone(), order.clone());
        // В бэктесте persistence не используется
        // self.persist_order(&order)?;
        // self.persist_position(&snapshot)?;
        // В бэктесте события не записываются
        // self.record_event(PositionEvent::OrderFilled(order.clone()))?;
        if snapshot.status == PositionStatus::Closed {
            // В бэктесте события не записываются
            // self.record_event(PositionEvent::PositionClosed(snapshot.clone()))?;
            report.closed_positions.push(snapshot.clone());
        } else {
            // В бэктесте события не записываются
            // self.record_event(PositionEvent::PositionUpdated(snapshot.clone()))?;
            report.updated_positions.push(snapshot.clone());
        }
        self.refresh_portfolio_metrics();
        report.orders.push(order);
        report.closed_trades.push(trade);
        Ok(())
    }

    fn build_order(
        &mut self,
        position_id: &str,
        key: &PositionKey,
        quantity: f64,
        price: f64,
    ) -> OrderTicket {
        let now = Utc::now();
        OrderTicket {
            id: self.next_id("ord"),
            position_id: position_id.to_string(),
            symbol: key.symbol.clone(),
            timeframe: key.timeframe.clone(),
            direction: key.direction.clone(),
            order_type: OrderType::Market,
            status: OrderStatus::Filled,
            quantity,
            price,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    async fn persist_position(&self, position: &PositionState) -> Result<(), PositionError> {
        if let Some(persistence) = &self.persistence {
            persistence
                .persist_position(position)
                .await
                .map_err(PositionError::Persistence)?;
        }
        Ok(())
    }

    async fn persist_order(&self, order: &OrderTicket) -> Result<(), PositionError> {
        if let Some(persistence) = &self.persistence {
            persistence
                .persist_order(order)
                .await
                .map_err(PositionError::Persistence)?;
        }
        Ok(())
    }

    async fn record_event(&mut self, event: PositionEvent) -> Result<(), PositionError> {
        if let Some(persistence) = &self.persistence {
            persistence
                .persist_event(&event)
                .await
                .map_err(PositionError::Persistence)?;
        }
        for listener in &self.listeners {
            listener
                .on_event(&event)
                .await
                .map_err(PositionError::Event)?;
        }
        self.event_history.push(event);
        Ok(())
    }

    fn snapshot_active_positions(&self) -> PositionBook {
        let entries: Vec<ActivePosition> = self
            .open_index
            .values()
            .filter_map(|position_id| self.positions.get(position_id))
            .map(|state| ActivePosition {
                id: state.id.clone(),
                symbol: state.key.symbol.clone(),
                timeframe: state.key.timeframe.clone(),
                direction: state.key.direction.clone(),
                entry_price: state.average_price,
                quantity: state.quantity,
                opened_at: Some(state.opened_at),
                last_price: Some(state.current_price),
                metadata: state.metadata.clone(),
                insights: PositionInsights::default(),
                position_group: state.key.position_group.clone(),
                entry_rule_id: state.key.entry_rule_id.clone(),
            })
            .collect();
        PositionBook::new(entries)
    }

    fn refresh_portfolio_metrics(&mut self) {
        let mut exposure = 0.0;
        let mut unrealized = 0.0;
        for position_id in self.open_index.values() {
            if let Some(state) = self.positions.get_mut(position_id) {
                let pos_exposure = state.quantity.abs() * state.current_price;
                exposure += pos_exposure;
                let pnl = match state.key.direction {
                    PositionDirection::Long => {
                        (state.current_price - state.average_price) * state.quantity
                    }
                    PositionDirection::Short => {
                        (state.average_price - state.current_price) * state.quantity
                    }
                    PositionDirection::Flat | PositionDirection::Both => 0.0,
                };
                state.unrealized_pnl = pnl;
                unrealized += pnl;
            }
        }
        self.portfolio.exposure = exposure;
        self.portfolio.unrealized_pnl = unrealized;
        self.portfolio.update_equity();
    }

    fn resolve_entry_snapshot(
        context: &StrategyContext,
        timeframe: &TimeFrame,
    ) -> Result<Option<MarketSnapshot>, PositionError> {
        let data = context.timeframe(timeframe).map_err(PositionError::from)?;
        let symbol = data
            .symbol()
            .cloned()
            .ok_or_else(|| PositionError::MissingSymbol(timeframe.clone()))?;
        let series = data.price_series_slice(&PriceField::Open).ok_or_else(|| {
            PositionError::MissingPriceSeries(timeframe.clone(), PriceField::Open)
        })?;
        if series.is_empty() {
            return Err(PositionError::MissingPriceSeries(
                timeframe.clone(),
                PriceField::Open,
            ));
        }
        let current_index = data.index().min(series.len().saturating_sub(1));
        let target_index = current_index;
        let price = f64::from(series[target_index]);
        let timestamp = data.timestamp_at(target_index);
        Ok(Some(MarketSnapshot {
            symbol,
            timeframe: timeframe.clone(),
            price,
            index: target_index,
            timestamp,
        }))
    }

    fn resolve_market_snapshot(
        context: &StrategyContext,
        timeframe: &TimeFrame,
        price_hint: Option<f64>,
    ) -> Result<MarketSnapshot, PositionError> {
        let data = context.timeframe(timeframe).map_err(PositionError::from)?;
        let symbol = data
            .symbol()
            .cloned()
            .ok_or_else(|| PositionError::MissingSymbol(timeframe.clone()))?;
        let series = data.price_series_slice(&PriceField::Open).ok_or_else(|| {
            PositionError::MissingPriceSeries(timeframe.clone(), PriceField::Open)
        })?;
        if series.is_empty() {
            return Err(PositionError::MissingPriceSeries(
                timeframe.clone(),
                PriceField::Open,
            ));
        }
        let index = data.index().min(series.len().saturating_sub(1));
        let price = price_hint.unwrap_or(f64::from(series[index]));
        let timestamp = data.timestamp_at(index);
        Ok(MarketSnapshot {
            symbol,
            timeframe: timeframe.clone(),
            price,
            index,
            timestamp,
        })
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.sequence += 1;
        format!("{}-{}-{}", prefix, self.strategy_id, self.sequence)
    }
}

struct MarketSnapshot {
    symbol: Symbol,
    timeframe: TimeFrame,
    price: f64,
    index: usize,
    timestamp: Option<DateTime<Utc>>,
}

fn opposite_direction(direction: &PositionDirection) -> Option<PositionDirection> {
    match direction {
        PositionDirection::Long => Some(PositionDirection::Short),
        PositionDirection::Short => Some(PositionDirection::Long),
        PositionDirection::Flat | PositionDirection::Both => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::types::{SignalStrength, TrendDirection};
    use crate::data_model::quote::Quote;
    use crate::data_model::quote_frame::QuoteFrame;
    use crate::strategy::context::TimeframeData;
    use crate::strategy::types::{StrategyDecision, StrategySignal, StrategySignalType};
    use std::collections::HashMap;

    fn build_context(prices: &[f32], symbol: &Symbol, timeframe: &TimeFrame) -> StrategyContext {
        let mut frame = QuoteFrame::new(symbol.clone(), timeframe.clone());
        for (idx, price) in prices.iter().enumerate() {
            let quote = Quote::from_parts(
                symbol.clone(),
                timeframe.clone(),
                chrono::Utc::now() + chrono::Duration::minutes(idx as i64),
                *price,
                *price,
                *price,
                *price,
                1.0,
            );
            frame.push(quote).unwrap();
        }
        let tf_data = TimeframeData::with_quote_frame(&frame, prices.len().saturating_sub(1));
        let mut context = StrategyContext::new();
        context.insert_timeframe(timeframe.clone(), tf_data);
        context
    }

    fn entry_signal(timeframe: &TimeFrame) -> StrategySignal {
        StrategySignal {
            rule_id: "enter-long".to_string(),
            signal_type: StrategySignalType::Entry,
            direction: PositionDirection::Long,
            timeframe: timeframe.clone(),
            strength: SignalStrength::Strong,
            trend: None,
            quantity: Some(1.0),
            entry_rule_id: Some("enter-long".to_string()),
            tags: Vec::new(),
            position_group: Some("enter-long".to_string()),
            target_entry_ids: Vec::new(),
        }
    }

    fn exit_signal(timeframe: &TimeFrame) -> StrategySignal {
        StrategySignal {
            rule_id: "exit-long".to_string(),
            signal_type: StrategySignalType::Exit,
            direction: PositionDirection::Long,
            timeframe: timeframe.clone(),
            strength: SignalStrength::Strong,
            trend: None,
            quantity: Some(1.0),
            entry_rule_id: None,
            tags: Vec::new(),
            position_group: None,
            target_entry_ids: vec!["enter-long".to_string()],
        }
    }

    #[tokio::test]
    async fn opens_and_closes_position() {
        let symbol = Symbol::from_descriptor("TEST.TEST");
        let timeframe = TimeFrame::minutes(1);
        let mut context = build_context(&[100.0, 101.0], &symbol, &timeframe);
        context
            .timeframe_mut(&timeframe)
            .expect("entry timeframe")
            .set_index(0);
        let mut manager = PositionManager::new("strategy-1");

        let mut decision = StrategyDecision::empty();
        decision.entries.push(entry_signal(&timeframe));
        let report = manager
            .process_decision(&mut context, &decision)
            .await
            .expect("entry processing failed");
        assert_eq!(report.opened_positions.len(), 1);
        assert_eq!(manager.open_position_count(), 1);
        assert_eq!(context.active_positions().len(), 1);

        let mut exit_decision = StrategyDecision::empty();
        exit_decision.exits.push(exit_signal(&timeframe));
        let mut exit_context = build_context(&[105.0, 105.0], &symbol, &timeframe);
        let exit_report = manager
            .process_decision(&mut exit_context, &exit_decision)
            .await
            .expect("exit processing failed");
        assert_eq!(exit_report.closed_positions.len(), 1);
        assert_eq!(manager.open_position_count(), 0);
        assert!(exit_context.active_positions().is_empty());
        let pnl = manager.portfolio_snapshot().realized_pnl;
        assert!((pnl - 4.0).abs() < 1e-6, "expected pnl ≈ 4.0, got {}", pnl);
    }

    #[tokio::test]
    async fn stop_signal_closes_position_first() {
        let symbol = Symbol::from_descriptor("TEST.TEST");
        let timeframe = TimeFrame::minutes(1);
        let mut context = build_context(&[100.0, 100.0], &symbol, &timeframe);
        context
            .timeframe_mut(&timeframe)
            .expect("entry timeframe")
            .set_index(0);
        let mut manager = PositionManager::new("strategy-2");

        let mut decision = StrategyDecision::empty();
        decision.entries.push(entry_signal(&timeframe));
        manager
            .process_decision(&mut context, &decision)
            .await
            .expect("entry failed");

        let mut stop_decision = StrategyDecision::empty();
        let mut stop_exit = exit_signal(&timeframe);
        stop_exit.rule_id = "stop_exit".to_string();
        stop_exit.tags.push("stop".to_string());
        let stop_signal = StopSignal {
            handler_id: "stop_exit".to_string(),
            signal: stop_exit.clone(),
            exit_price: 95.0,
            kind: StopSignalKind::StopLoss,
            priority: 0,
            metadata: HashMap::new(),
        };
        stop_decision.stop_signals.push(stop_signal);
        stop_decision.exits.push(stop_exit);
        stop_decision.exits.push(exit_signal(&timeframe));
        let mut exit_context = build_context(&[110.0, 110.0], &symbol, &timeframe);
        manager
            .process_decision(&mut exit_context, &stop_decision)
            .await
            .expect("stop exit failed");
        assert_eq!(manager.open_position_count(), 0);
        assert!(exit_context.active_positions().is_empty());
        let pnl = manager.portfolio_snapshot().realized_pnl;
        assert!((pnl + 5.0).abs() < 1e-6, "expected pnl ≈ -5.0, got {}", pnl);
        let closed_event = manager
            .event_history()
            .iter()
            .filter_map(|event| match event {
                PositionEvent::PositionClosed(state) => Some(state),
                _ => None,
            })
            .last()
            .expect("no close event");
        let reason = closed_event.metadata.get("close_reason");
        assert!(reason
            .map(|value| value.starts_with("stop:"))
            .unwrap_or(false));
    }
}
