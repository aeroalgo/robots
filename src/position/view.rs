use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::data_model::types::{Symbol, TimeFrame};
use crate::strategy::types::PositionDirection;

#[derive(Clone, Debug, Default)]
pub struct PositionInsights {
    pub mae: Option<f64>,
    pub mfe: Option<f64>,
    pub bars_held: Option<u32>,
    pub custom: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ActivePosition {
    pub id: String,
    pub symbol: Symbol,
    pub timeframe: TimeFrame,
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub quantity: f64,
    pub opened_at: Option<DateTime<Utc>>,
    pub last_price: Option<f64>,
    pub metadata: HashMap<String, String>,
    pub insights: PositionInsights,
}

impl ActivePosition {
    pub fn new(
        id: impl Into<String>,
        symbol: Symbol,
        timeframe: TimeFrame,
        direction: PositionDirection,
        entry_price: f64,
        quantity: f64,
    ) -> Self {
        Self {
            id: id.into(),
            symbol,
            timeframe,
            direction,
            entry_price,
            quantity,
            opened_at: None,
            last_price: None,
            metadata: HashMap::new(),
            insights: PositionInsights::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PositionBook {
    entries: Vec<ActivePosition>,
    index: HashMap<String, usize>,
}

impl PositionBook {
    pub fn new(entries: Vec<ActivePosition>) -> Self {
        let mut index = HashMap::with_capacity(entries.len());
        for (idx, position) in entries.iter().enumerate() {
            index.insert(position.id.clone(), idx);
        }
        Self { entries, index }
    }

    pub fn entries(&self) -> &[ActivePosition] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn first(&self) -> Option<&ActivePosition> {
        self.entries.first()
    }

    pub fn get(&self, id: &str) -> Option<&ActivePosition> {
        self.index.get(id).and_then(|idx| self.entries.get(*idx))
    }
}

impl From<Vec<ActivePosition>> for PositionBook {
    fn from(entries: Vec<ActivePosition>) -> Self {
        PositionBook::new(entries)
    }
}
