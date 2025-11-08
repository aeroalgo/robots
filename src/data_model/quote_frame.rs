use std::ops::RangeBounds;

use chrono::{DateTime, Utc};
use thiserror::Error;

use super::quote::Quote;
use super::types::{Symbol, TimeFrame};
use super::vector::ValueVector;

#[derive(Debug, Error)]
pub enum QuoteFrameError {
    #[error("symbol mismatch: expected {expected}, got {actual}")]
    SymbolMismatch { expected: String, actual: String },
    #[error("timeframe mismatch: expected {expected}, got {actual}")]
    TimeFrameMismatch { expected: String, actual: String },
    #[error("timestamp is not strictly increasing: last {last:?}, new {new:?}")]
    NonMonotonicTimestamp { last: DateTime<Utc>, new: DateTime<Utc> },
}

pub struct QuoteFrame {
    symbol: Symbol,
    timeframe: TimeFrame,
    quotes: Vec<Quote>,
    max_len: Option<usize>,
}

impl QuoteFrame {
    pub fn new(symbol: Symbol, timeframe: TimeFrame) -> Self {
        Self::with_capacity(symbol, timeframe, 0)
    }

    pub fn with_capacity(symbol: Symbol, timeframe: TimeFrame, capacity: usize) -> Self {
        Self {
            symbol,
            timeframe,
            quotes: Vec::with_capacity(capacity),
            max_len: None,
        }
    }

    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn timeframe(&self) -> &TimeFrame {
        &self.timeframe
    }

    pub fn len(&self) -> usize {
        self.quotes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quotes.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.quotes.capacity()
    }

    pub fn set_max_len(&mut self, max_len: Option<usize>) {
        self.max_len = max_len;
        self.enforce_max_len();
    }

    pub fn quotes(&self) -> &[Quote] {
        &self.quotes
    }

    pub fn latest(&self) -> Option<&Quote> {
        self.quotes.last()
    }

    pub fn get(&self, index: usize) -> Option<&Quote> {
        self.quotes.get(index)
    }

    pub fn push(&mut self, quote: Quote) -> Result<(), QuoteFrameError> {
        self.validate_symbol(&quote)?;
        self.validate_timeframe(&quote)?;
        if let Some(last) = self.quotes.last() {
            if quote.timestamp() < last.timestamp() {
                return Err(QuoteFrameError::NonMonotonicTimestamp {
                    last: last.timestamp(),
                    new: quote.timestamp(),
                });
            }
            if quote.timestamp() == last.timestamp() {
                let len = self.quotes.len();
                self.quotes[len - 1] = quote;
                return Ok(());
            }
        }
        self.quotes.push(quote);
        self.enforce_max_len();
        Ok(())
    }

    pub fn extend<I>(&mut self, iter: I) -> Result<(), QuoteFrameError>
    where
        I: IntoIterator<Item = Quote>,
    {
        for quote in iter {
            self.push(quote)?;
        }
        Ok(())
    }

    pub fn window<R>(&self, range: R) -> &[Quote]
    where
        R: RangeBounds<usize>,
    {
        use std::ops::Bound;
        let len = self.quotes.len();
        let start = match range.start_bound() {
            Bound::Included(&idx) => idx,
            Bound::Excluded(&idx) => idx + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&idx) => idx + 1,
            Bound::Excluded(&idx) => idx,
            Bound::Unbounded => len,
        };
        &self.quotes[start.min(len)..end.min(len)]
    }

    pub fn closes(&self) -> ValueVector {
        ValueVector::from_quotes(self.quotes(), |quote| quote.close())
    }

    pub fn opens(&self) -> ValueVector {
        ValueVector::from_quotes(self.quotes(), |quote| quote.open())
    }

    pub fn highs(&self) -> ValueVector {
        ValueVector::from_quotes(self.quotes(), |quote| quote.high())
    }

    pub fn lows(&self) -> ValueVector {
        ValueVector::from_quotes(self.quotes(), |quote| quote.low())
    }

    pub fn volumes(&self) -> ValueVector {
        ValueVector::from_quotes(self.quotes(), |quote| quote.volume())
    }

    fn enforce_max_len(&mut self) {
        if let Some(limit) = self.max_len {
            if self.quotes.len() > limit {
                let excess = self.quotes.len() - limit;
                self.quotes.drain(0..excess);
            }
        }
    }

    fn validate_symbol(&self, quote: &Quote) -> Result<(), QuoteFrameError> {
        if quote.symbol() == self.symbol() {
            Ok(())
        } else {
            Err(QuoteFrameError::SymbolMismatch {
                expected: self.symbol().to_string(),
                actual: quote.symbol().to_string(),
            })
        }
    }

    fn validate_timeframe(&self, quote: &Quote) -> Result<(), QuoteFrameError> {
        if quote.timeframe() == self.timeframe() {
            Ok(())
        } else {
            Err(QuoteFrameError::TimeFrameMismatch {
                expected: self.timeframe().identifier(),
                actual: quote.timeframe().identifier(),
            })
        }
    }
}
