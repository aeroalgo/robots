use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::types::Symbol;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AssetClass {
    Equity,
    Forex,
    Futures,
    Crypto,
    Commodity,
    Index,
    Bond,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstrumentMeta {
    symbol: Symbol,
    asset_class: AssetClass,
    quote_currency: Option<String>,
    tick_size: f32,
    lot_size: f32,
    timezone: Option<String>,
    additional: HashMap<String, String>,
}

impl InstrumentMeta {
    pub fn builder(symbol: Symbol) -> InstrumentMetaBuilder {
        InstrumentMetaBuilder::new(symbol)
    }

    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn asset_class(&self) -> &AssetClass {
        &self.asset_class
    }

    pub fn quote_currency(&self) -> Option<&str> {
        self.quote_currency.as_deref()
    }

    pub fn tick_size(&self) -> f32 {
        self.tick_size
    }

    pub fn lot_size(&self) -> f32 {
        self.lot_size
    }

    pub fn timezone(&self) -> Option<&str> {
        self.timezone.as_deref()
    }

    pub fn additional(&self) -> &HashMap<String, String> {
        &self.additional
    }
}

pub struct InstrumentMetaBuilder {
    symbol: Symbol,
    asset_class: AssetClass,
    quote_currency: Option<String>,
    tick_size: f32,
    lot_size: f32,
    timezone: Option<String>,
    additional: HashMap<String, String>,
}

impl InstrumentMetaBuilder {
    fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            asset_class: AssetClass::Custom("unknown".to_string()),
            quote_currency: None,
            tick_size: 0.0,
            lot_size: 0.0,
            timezone: None,
            additional: HashMap::new(),
        }
    }

    pub fn asset_class(mut self, asset_class: AssetClass) -> Self {
        self.asset_class = asset_class;
        self
    }

    pub fn quote_currency<S: Into<String>>(mut self, currency: S) -> Self {
        self.quote_currency = Some(currency.into());
        self
    }

    pub fn tick_size(mut self, value: f32) -> Self {
        self.tick_size = value;
        self
    }

    pub fn lot_size(mut self, value: f32) -> Self {
        self.lot_size = value;
        self
    }

    pub fn timezone<S: Into<String>>(mut self, timezone: S) -> Self {
        self.timezone = Some(timezone.into());
        self
    }

    pub fn with_additional<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.additional.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> InstrumentMeta {
        InstrumentMeta {
            symbol: self.symbol,
            asset_class: self.asset_class,
            quote_currency: self.quote_currency,
            tick_size: self.tick_size,
            lot_size: self.lot_size,
            timezone: self.timezone,
            additional: self.additional,
        }
    }
}

#[derive(Default)]
pub struct MetaRegistry {
    entries: HashMap<Arc<str>, InstrumentMeta>,
}

impl MetaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, meta: InstrumentMeta) {
        let key: Arc<str> = Arc::from(meta.symbol().code());
        self.entries.insert(key, meta);
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&InstrumentMeta> {
        self.entries.get(symbol.code())
    }

    pub fn remove(&mut self, symbol: &Symbol) -> Option<InstrumentMeta> {
        self.entries.remove(symbol.code())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Arc<str>, &InstrumentMeta)> {
        self.entries.iter()
    }
}
