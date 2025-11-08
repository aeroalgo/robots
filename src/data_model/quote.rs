use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data_access::database::clickhouse::OhlcvData;

use super::types::{Price, QuoteId, Symbol, TimeFrame, Volume};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    id: QuoteId,
    open: Price,
    high: Price,
    low: Price,
    close: Price,
    volume: Volume,
}

impl Quote {
    pub fn new(id: QuoteId, open: Price, high: Price, low: Price, close: Price, volume: Volume) -> Self {
        Self {
            id,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn from_parts(symbol: Symbol, timeframe: TimeFrame, timestamp: DateTime<Utc>, open: Price, high: Price, low: Price, close: Price, volume: Volume) -> Self {
        Self::new(QuoteId::new(symbol, timeframe, timestamp), open, high, low, close, volume)
    }

    pub fn id(&self) -> &QuoteId {
        &self.id
    }

    pub fn symbol(&self) -> &Symbol {
        &self.id.symbol
    }

    pub fn timeframe(&self) -> &TimeFrame {
        &self.id.timeframe
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.id.timestamp
    }

    pub fn open(&self) -> Price {
        self.open
    }

    pub fn high(&self) -> Price {
        self.high
    }

    pub fn low(&self) -> Price {
        self.low
    }

    pub fn close(&self) -> Price {
        self.close
    }

    pub fn volume(&self) -> Volume {
        self.volume
    }

    pub fn to_ohlcv(&self) -> OhlcvData {
        OhlcvData {
            symbol: self.symbol().to_string(),
            timeframe: self.timeframe().identifier(),
            timestamp: self.timestamp(),
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        }
    }
}

impl From<OhlcvData> for Quote {
    fn from(value: OhlcvData) -> Self {
        Self::from_parts(
            Symbol::new(value.symbol),
            TimeFrame::from_identifier(&value.timeframe),
            value.timestamp,
            value.open,
            value.high,
            value.low,
            value.close,
            value.volume,
        )
    }
}

impl From<Quote> for OhlcvData {
    fn from(value: Quote) -> Self {
        value.to_ohlcv()
    }
}
