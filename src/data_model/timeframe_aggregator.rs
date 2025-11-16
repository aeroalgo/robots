use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::data_model::quote::Quote;
use crate::data_model::quote_frame::{QuoteFrame, QuoteFrameError};
use crate::data_model::types::{Symbol, TimeFrame};

#[derive(Debug, Clone)]
pub struct TimeFrameMetadata {
    pub source_timeframe: TimeFrame,
    pub target_timeframe: TimeFrame,
    pub aggregation_ratio: f64,
    pub created_at: DateTime<Utc>,
}

pub struct AggregatedQuoteFrame {
    pub frame: QuoteFrame,
    pub metadata: TimeFrameMetadata,
    pub source_indices: HashMap<usize, Vec<usize>>,
}

pub struct TimeFrameAggregator;

impl TimeFrameAggregator {
    pub fn aggregate(
        source_frame: &QuoteFrame,
        target_timeframe: TimeFrame,
    ) -> Result<AggregatedQuoteFrame, TimeFrameAggregationError> {
        let source_tf = source_frame.timeframe();
        let symbol = source_frame.symbol().clone();

        if !Self::is_valid_aggregation(source_tf, &target_timeframe) {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                format!("{:?}", source_tf),
                format!("{:?}", target_timeframe),
            ));
        }

        let ratio = Self::calculate_ratio(source_tf, &target_timeframe)?;
        let mut aggregated_quotes = Vec::new();
        let mut source_indices = HashMap::new();

        let mut current_bar_start: Option<DateTime<Utc>> = None;
        let mut current_bar_open: Option<Price> = None;
        let mut current_bar_high: Option<Price> = None;
        let mut current_bar_low: Option<Price> = None;
        let mut current_bar_close: Option<Price> = None;
        let mut current_bar_volume: Volume = 0.0;
        let mut current_bar_indices = Vec::new();

        for (idx, quote) in source_frame.iter().enumerate() {
            let quote_time = quote.timestamp();
            let bar_start = Self::align_to_timeframe(quote_time, &target_timeframe)?;

            if current_bar_start.is_none() || current_bar_start.unwrap() != bar_start {
                if current_bar_start.is_some() {
                    let aggregated_quote = Quote::from_parts(
                        symbol.clone(),
                        target_timeframe.clone(),
                        current_bar_start.unwrap(),
                        current_bar_open.unwrap(),
                        current_bar_high.unwrap(),
                        current_bar_low.unwrap(),
                        current_bar_close.unwrap(),
                        current_bar_volume,
                    );

                    let aggregated_idx = aggregated_quotes.len();
                    aggregated_quotes.push(aggregated_quote);
                    source_indices.insert(aggregated_idx, current_bar_indices.clone());
                }

                current_bar_start = Some(bar_start);
                current_bar_open = Some(quote.open());
                current_bar_high = Some(quote.high());
                current_bar_low = Some(quote.low());
                current_bar_close = Some(quote.close());
                current_bar_volume = quote.volume();
                current_bar_indices = vec![idx];
            } else {
                current_bar_high = Some(current_bar_high.unwrap().max(quote.high()));
                current_bar_low = Some(current_bar_low.unwrap().min(quote.low()));
                current_bar_close = Some(quote.close());
                current_bar_volume += quote.volume();
                current_bar_indices.push(idx);
            }
        }

        if current_bar_start.is_some() {
            let aggregated_quote = Quote::from_parts(
                symbol.clone(),
                target_timeframe.clone(),
                current_bar_start.unwrap(),
                current_bar_open.unwrap(),
                current_bar_high.unwrap(),
                current_bar_low.unwrap(),
                current_bar_close.unwrap(),
                current_bar_volume,
            );

            let aggregated_idx = aggregated_quotes.len();
            aggregated_quotes.push(aggregated_quote);
            source_indices.insert(aggregated_idx, current_bar_indices);
        }

        let mut target_frame = QuoteFrame::new(symbol, target_timeframe.clone());
        for quote in aggregated_quotes {
            target_frame.push(quote)?;
        }

        let metadata = TimeFrameMetadata {
            source_timeframe: source_tf.clone(),
            target_timeframe: target_timeframe.clone(),
            aggregation_ratio: ratio,
            created_at: Utc::now(),
        };

        Ok(AggregatedQuoteFrame {
            frame: target_frame,
            metadata,
            source_indices,
        })
    }

    pub fn generate_derived_timeframes(
        base_timeframe: &TimeFrame,
        multipliers: &[u32],
    ) -> Vec<TimeFrame> {
        let base_minutes = Self::timeframe_to_minutes(base_timeframe);
        if base_minutes.is_none() {
            return vec![];
        }

        let base = base_minutes.unwrap();
        multipliers
            .iter()
            .filter_map(|mult| {
                let target_minutes = base * mult;
                Self::minutes_to_timeframe(target_minutes)
            })
            .collect()
    }

    fn is_valid_aggregation(source: &TimeFrame, target: &TimeFrame) -> bool {
        let source_minutes = Self::timeframe_to_minutes(source);
        let target_minutes = Self::timeframe_to_minutes(target);

        match (source_minutes, target_minutes) {
            (Some(s), Some(t)) => t > s && t % s == 0,
            _ => false,
        }
    }

    fn calculate_ratio(
        source: &TimeFrame,
        target: &TimeFrame,
    ) -> Result<f64, TimeFrameAggregationError> {
        let source_minutes = Self::timeframe_to_minutes(source).ok_or_else(|| {
            TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", source))
        })?;
        let target_minutes = Self::timeframe_to_minutes(target).ok_or_else(|| {
            TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", target))
        })?;

        if target_minutes == 0 {
            return Err(TimeFrameAggregationError::InvalidTimeFrame);
        }

        Ok(target_minutes as f64 / source_minutes as f64)
    }

    fn align_to_timeframe(
        timestamp: DateTime<Utc>,
        timeframe: &TimeFrame,
    ) -> Result<DateTime<Utc>, TimeFrameAggregationError> {
        let minutes = Self::timeframe_to_minutes(timeframe).ok_or_else(|| {
            TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", timeframe))
        })?;

        let total_minutes = timestamp.timestamp() / 60;
        let aligned_minutes = (total_minutes / minutes as i64) * minutes as i64;
        let aligned_timestamp = DateTime::from_timestamp(aligned_minutes * 60, 0)
            .ok_or(TimeFrameAggregationError::InvalidTimestamp)?;

        Ok(aligned_timestamp)
    }

    fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        match tf {
            TimeFrame::Minutes(m) => Some(*m),
            TimeFrame::Hours(h) => Some(h * 60),
            TimeFrame::Days(d) => Some(d * 24 * 60),
            TimeFrame::Weeks(w) => Some(w * 7 * 24 * 60),
            TimeFrame::Months(m) => Some(m * 30 * 24 * 60),
            TimeFrame::Custom(_) => None,
        }
    }

    fn minutes_to_timeframe(minutes: u32) -> Option<TimeFrame> {
        if minutes < 60 {
            Some(TimeFrame::Minutes(minutes))
        } else if minutes < 24 * 60 {
            let hours = minutes / 60;
            if minutes % 60 == 0 {
                Some(TimeFrame::Hours(hours))
            } else {
                Some(TimeFrame::Minutes(minutes))
            }
        } else if minutes < 7 * 24 * 60 {
            let days = minutes / (24 * 60);
            if minutes % (24 * 60) == 0 {
                Some(TimeFrame::Days(days))
            } else {
                Some(TimeFrame::Minutes(minutes))
            }
        } else if minutes < 30 * 24 * 60 {
            let weeks = minutes / (7 * 24 * 60);
            if minutes % (7 * 24 * 60) == 0 {
                Some(TimeFrame::Weeks(weeks))
            } else {
                Some(TimeFrame::Minutes(minutes))
            }
        } else {
            let months = minutes / (30 * 24 * 60);
            if minutes % (30 * 24 * 60) == 0 {
                Some(TimeFrame::Months(months))
            } else {
                Some(TimeFrame::Minutes(minutes))
            }
        }
    }
}

impl AggregatedQuoteFrame {
    pub fn get_source_indices(&self, aggregated_index: usize) -> Option<&Vec<usize>> {
        self.source_indices.get(&aggregated_index)
    }

    pub fn get_source_quotes<'a>(
        &self,
        aggregated_index: usize,
        source_frame: &'a QuoteFrame,
    ) -> Option<Vec<&'a Quote>> {
        self.get_source_indices(aggregated_index).map(|indices| {
            indices
                .iter()
                .filter_map(|idx| source_frame.get(*idx))
                .collect()
        })
    }

    pub fn frame(&self) -> &QuoteFrame {
        &self.frame
    }

    pub fn metadata(&self) -> &TimeFrameMetadata {
        &self.metadata
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeFrameAggregationError {
    #[error("Invalid aggregation: source {0} -> target {1}")]
    InvalidAggregation(String, String),
    #[error("Unsupported timeframe: {0}")]
    UnsupportedTimeFrame(String),
    #[error("Invalid timeframe")]
    InvalidTimeFrame,
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("QuoteFrame error: {0}")]
    QuoteFrameError(#[from] QuoteFrameError),
}

type Price = crate::data_model::types::Price;
type Volume = crate::data_model::types::Volume;
