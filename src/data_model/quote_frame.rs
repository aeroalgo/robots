use std::ops::RangeBounds;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::data_access::database::clickhouse::OhlcvData;
use crate::indicators::OHLCData;

use super::quote::Quote;
use super::types::{
    timestamp_from_millis, timestamp_to_millis, Symbol, TimeFrame, TimestampMillis,
};
use super::vector::ValueVector;

#[derive(Debug, Error)]
pub enum QuoteFrameError {
    #[error("symbol mismatch: expected {expected}, got {actual}")]
    SymbolMismatch { expected: String, actual: String },
    #[error("timeframe mismatch: expected {expected}, got {actual}")]
    TimeFrameMismatch { expected: String, actual: String },
    #[error("timestamp is not strictly increasing: last {last:?}, new {new:?}")]
    NonMonotonicTimestamp {
        last: DateTime<Utc>,
        new: DateTime<Utc>,
    },
    #[error("quote frame is empty")]
    Empty,
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

    pub fn builder() -> QuoteFrameBuilder {
        QuoteFrameBuilder::default()
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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Quote> + ExactSizeIterator {
        self.quotes.iter()
    }

    pub fn latest(&self) -> Option<&Quote> {
        self.quotes.last()
    }

    pub fn first(&self) -> Option<&Quote> {
        self.quotes.first()
    }

    pub fn get(&self, index: usize) -> Option<&Quote> {
        self.quotes.get(index)
    }

    pub fn find_index_by_timestamp(&self, timestamp: DateTime<Utc>) -> Option<usize> {
        self.quotes
            .binary_search_by_key(&timestamp, |quote| quote.timestamp())
            .ok()
    }

    pub fn find_index_by_millis(&self, millis: TimestampMillis) -> Option<usize> {
        timestamp_from_millis(millis).and_then(|ts| self.find_index_by_timestamp(ts))
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

    pub fn push_ohlcv(&mut self, data: OhlcvData) -> Result<(), QuoteFrameError> {
        self.push(Quote::from(data))
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

    pub fn extend_from_ohlcv<I>(&mut self, iter: I) -> Result<(), QuoteFrameError>
    where
        I: IntoIterator<Item = OhlcvData>,
    {
        for row in iter {
            self.push_ohlcv(row)?;
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

    pub fn slice_by_time(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> &[Quote] {
        let start_idx = self
            .quotes
            .binary_search_by_key(&start, |quote| quote.timestamp())
            .unwrap_or_else(|idx| idx);
        let end_idx = self
            .quotes
            .binary_search_by_key(&end, |quote| quote.timestamp())
            .map(|idx| idx + 1)
            .unwrap_or_else(|idx| idx);
        &self.quotes[start_idx.min(self.len())..end_idx.min(self.len())]
    }

    pub fn slice_by_millis(&self, start: TimestampMillis, end: TimestampMillis) -> &[Quote] {
        match (timestamp_from_millis(start), timestamp_from_millis(end)) {
            (Some(s), Some(e)) => self.slice_by_time(s, e),
            _ => &[],
        }
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

    pub fn timestamps(&self) -> Vec<DateTime<Utc>> {
        self.quotes.iter().map(Quote::timestamp).collect()
    }

    pub fn timestamp_millis(&self) -> Vec<TimestampMillis> {
        self.quotes.iter().map(Quote::timestamp_millis).collect()
    }

    pub fn clear(&mut self) {
        self.quotes.clear();
    }

    pub fn truncate_before(&mut self, timestamp: DateTime<Utc>) {
        let cutoff = self
            .quotes
            .binary_search_by_key(&timestamp, |quote| quote.timestamp())
            .unwrap_or_else(|idx| idx);
        if cutoff > 0 {
            self.quotes.drain(0..cutoff);
        }
    }

    pub fn truncate_before_millis(&mut self, millis: TimestampMillis) {
        if let Some(ts) = timestamp_from_millis(millis) {
            self.truncate_before(ts);
        }
    }

    pub fn truncate_after(&mut self, timestamp: DateTime<Utc>) {
        let idx = self
            .quotes
            .binary_search_by_key(&timestamp, |quote| quote.timestamp())
            .map(|idx| idx + 1)
            .unwrap_or_else(|idx| idx);
        if idx < self.quotes.len() {
            self.quotes.truncate(idx);
        }
    }

    pub fn truncate_after_millis(&mut self, millis: TimestampMillis) {
        if let Some(ts) = timestamp_from_millis(millis) {
            self.truncate_after(ts);
        }
    }

    pub fn into_vec(self) -> Vec<Quote> {
        self.quotes
    }

    pub fn to_ohlcv(&self) -> Vec<OhlcvData> {
        self.quotes.iter().cloned().map(Into::into).collect()
    }

    pub fn to_indicator_ohlc(&self) -> OHLCData {
        let open: Vec<f32> = self.opens().iter().collect();
        let high: Vec<f32> = self.highs().iter().collect();
        let low: Vec<f32> = self.lows().iter().collect();
        let close: Vec<f32> = self.closes().iter().collect();
        let volume: Vec<f32> = self.volumes().iter().collect();
        let timestamp = self.timestamp_millis();
        OHLCData::new(open, high, low, close)
            .with_volume(volume)
            .with_timestamp(timestamp)
    }

    pub fn validate(&self) -> Result<(), QuoteFrameError> {
        if self.quotes.is_empty() {
            Err(QuoteFrameError::Empty)
        } else {
            Ok(())
        }
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

impl QuoteFrame {
    pub fn try_from_ohlcv<I>(
        data: I,
        symbol: Symbol,
        timeframe: TimeFrame,
    ) -> Result<Self, QuoteFrameError>
    where
        I: IntoIterator<Item = OhlcvData>,
    {
        let mut frame = QuoteFrame::with_capacity(symbol, timeframe, 0);
        frame.extend_from_ohlcv(data)?;
        Ok(frame)
    }

    pub fn from_ohlcv_unchecked<I>(data: I, symbol: Symbol, timeframe: TimeFrame) -> Self
    where
        I: IntoIterator<Item = OhlcvData>,
    {
        let mut frame = QuoteFrame::with_capacity(symbol.clone(), timeframe.clone(), 0);
        for row in data {
            if let Some(quote) = Quote::from_timestamp_millis(
                symbol.clone(),
                timeframe.clone(),
                timestamp_to_millis(row.timestamp),
                row.open,
                row.high,
                row.low,
                row.close,
                row.volume,
            ) {
                let _ = frame.push(quote);
            }
        }
        frame
    }
}

impl<'a> IntoIterator for &'a QuoteFrame {
    type Item = &'a Quote;
    type IntoIter = std::slice::Iter<'a, Quote>;

    fn into_iter(self) -> Self::IntoIter {
        self.quotes.iter()
    }
}

impl IntoIterator for QuoteFrame {
    type Item = Quote;
    type IntoIter = std::vec::IntoIter<Quote>;

    fn into_iter(self) -> Self::IntoIter {
        self.quotes.into_iter()
    }
}

#[derive(Default)]
pub struct QuoteFrameBuilder {
    symbol: Option<Symbol>,
    timeframe: Option<TimeFrame>,
    capacity: usize,
    max_len: Option<usize>,
}

impl QuoteFrameBuilder {
    pub fn symbol(mut self, symbol: Symbol) -> Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn timeframe(mut self, timeframe: TimeFrame) -> Self {
        self.timeframe = Some(timeframe);
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn max_len(mut self, max_len: Option<usize>) -> Self {
        self.max_len = max_len;
        self
    }

    pub fn build(self) -> Option<QuoteFrame> {
        let mut frame = QuoteFrame::with_capacity(self.symbol?, self.timeframe?, self.capacity);
        frame.set_max_len(self.max_len);
        Some(frame)
    }
}
