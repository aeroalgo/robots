pub mod converter;
pub mod expander;
pub mod validator;

pub use converter::TimeFrameConverter;
pub use expander::TimeFrameExpander;
pub use validator::TimeFrameValidator;

use crate::data_model::quote::Quote;
use crate::data_model::quote_frame::{QuoteFrame, QuoteFrameError};
use crate::data_model::types::{Price, Symbol, TimeFrame, Volume};
use chrono::{DateTime, Duration, Utc};

struct AggregationState {
    current_bar_start: Option<DateTime<Utc>>,
    current_bar_open: Option<Price>,
    current_bar_high: Option<Price>,
    current_bar_low: Option<Price>,
    current_bar_close: Option<Price>,
    current_bar_volume: Volume,
    current_bar_indices: Vec<usize>,
}

impl AggregationState {
    fn new(capacity: usize) -> Self {
        Self {
            current_bar_start: None,
            current_bar_open: None,
            current_bar_high: None,
            current_bar_low: None,
            current_bar_close: None,
            current_bar_volume: 0.0,
            current_bar_indices: Vec::with_capacity(capacity),
        }
    }

    fn should_start_new_bar(&self, bar_start: DateTime<Utc>) -> bool {
        self.current_bar_start.is_none() || self.current_bar_start.unwrap() != bar_start
    }

    fn start_new_bar(&mut self, bar_start: DateTime<Utc>, quote: &Quote, idx: usize) {
        self.current_bar_start = Some(bar_start);
        self.current_bar_open = Some(quote.open());
        self.current_bar_high = Some(quote.high());
        self.current_bar_low = Some(quote.low());
        self.current_bar_close = Some(quote.close());
        self.current_bar_volume = quote.volume();
        self.current_bar_indices = vec![idx];
    }

    fn update_current_bar(&mut self, quote: &Quote, idx: usize) {
        self.current_bar_high = Some(self.current_bar_high.unwrap().max(quote.high()));
        self.current_bar_low = Some(self.current_bar_low.unwrap().min(quote.low()));
        self.current_bar_close = Some(quote.close());
        self.current_bar_volume += quote.volume();
        self.current_bar_indices.push(idx);
    }

    fn finalize_current_bar(
        &mut self,
        aggregated_quotes: &mut Vec<Quote>,
        source_indices: &mut Vec<Vec<usize>>,
        symbol: &Symbol,
        target_timeframe: &TimeFrame,
    ) {
        if let Some(bar_start) = self.current_bar_start {
            let aggregated_quote = Quote::from_parts(
                symbol.clone(),
                target_timeframe.clone(),
                bar_start,
                self.current_bar_open.unwrap(),
                self.current_bar_high.unwrap(),
                self.current_bar_low.unwrap(),
                self.current_bar_close.unwrap(),
                self.current_bar_volume,
            );

            let aggregated_idx = aggregated_quotes.len();
            aggregated_quotes.push(aggregated_quote);
            source_indices.push(self.current_bar_indices.clone());
        }
    }
}

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
    pub source_indices: Vec<Vec<usize>>,
}

impl AggregatedQuoteFrame {
    pub fn get_source_indices(&self, aggregated_index: usize) -> Option<&Vec<usize>> {
        self.source_indices.get(aggregated_index)
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

    pub fn expand(
        &self,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, TimeFrameAggregationError> {
        TimeFrameExpander::expand(self, source_frame)
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

pub struct TimeFrameAggregator;

impl TimeFrameAggregator {
    pub fn aggregate(
        source_frame: &QuoteFrame,
        target_timeframe: TimeFrame,
    ) -> Result<AggregatedQuoteFrame, TimeFrameAggregationError> {
        if source_frame.is_empty() {
            return Ok(AggregatedQuoteFrame {
                frame: QuoteFrame::new(source_frame.symbol().clone(), target_timeframe.clone()),
                metadata: TimeFrameMetadata {
                    source_timeframe: source_frame.timeframe().clone(),
                    target_timeframe,
                    aggregation_ratio: 1.0,
                    created_at: Utc::now(),
                },
                source_indices: Vec::new(),
            });
        }

        let source_tf = source_frame.timeframe();
        let symbol = source_frame.symbol();

        TimeFrameValidator::validate_aggregation(source_tf, &target_timeframe)?;

        let ratio = TimeFrameConverter::calculate_ratio(source_tf, &target_timeframe)?;
        let estimated_bars = (source_frame.len() as f64 / ratio as f64).ceil() as usize;
        let mut aggregated_quotes = Vec::with_capacity(estimated_bars);
        let mut source_indices = Vec::with_capacity(estimated_bars);
        let target_timeframe_for_quotes = target_timeframe.clone();

        let mut aggregation_state = AggregationState::new(ratio as usize);

        for (idx, quote) in source_frame.iter().enumerate() {
            let quote_time = quote.timestamp();
            let bar_start = TimeFrameConverter::align_to_timeframe(quote_time, &target_timeframe)?;

            if aggregation_state.should_start_new_bar(bar_start) {
                aggregation_state.finalize_current_bar(
                    &mut aggregated_quotes,
                    &mut source_indices,
                    symbol,
                    &target_timeframe_for_quotes,
                );
                aggregation_state.start_new_bar(bar_start, quote, idx);
            } else {
                aggregation_state.update_current_bar(quote, idx);
            }
        }

        aggregation_state.finalize_current_bar(
            &mut aggregated_quotes,
            &mut source_indices,
            symbol,
            &target_timeframe_for_quotes,
        );

        let target_timeframe_clone = target_timeframe.clone();
        let mut target_frame = QuoteFrame::new(symbol.clone(), target_timeframe);
        for quote in aggregated_quotes {
            target_frame.push(quote)?;
        }

        let metadata = TimeFrameMetadata {
            source_timeframe: source_tf.clone(),
            target_timeframe: target_timeframe_clone,
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
        TimeFrameConverter::generate_derived_timeframes(base_timeframe, multipliers)
    }

    pub fn build_compressed_frame_from_source(
        source_frame: &QuoteFrame,
        target_timeframe: &TimeFrame,
        up_to_index: usize,
    ) -> Result<QuoteFrame, TimeFrameAggregationError> {
        if source_frame.is_empty() {
            return Ok(QuoteFrame::new(
                source_frame.symbol().clone(),
                target_timeframe.clone(),
            ));
        }

        TimeFrameValidator::validate_compression_params(
            source_frame,
            target_timeframe,
            up_to_index,
        )?;

        let source_tf = source_frame.timeframe();
        let target_minutes = TimeFrameConverter::timeframe_to_minutes(target_timeframe)
            .ok_or_else(|| {
                TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", target_timeframe))
            })?;
        let source_minutes =
            TimeFrameConverter::timeframe_to_minutes(source_tf).ok_or_else(|| {
                TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", source_tf))
            })?;

        let count_old_candles_in_one_new = (target_minutes / source_minutes) as usize;

        if count_old_candles_in_one_new == 1 {
            let mut result_frame =
                QuoteFrame::new(source_frame.symbol().clone(), target_timeframe.clone());
            for i in 0..=up_to_index.min(source_frame.len().saturating_sub(1)) {
                if let Some(quote) = source_frame.get(i) {
                    result_frame.push(quote.clone())?;
                }
            }
            return Ok(result_frame);
        }

        let symbol_clone = source_frame.symbol().clone();
        let target_timeframe_for_quotes = target_timeframe.clone();
        let mut result_frame = QuoteFrame::new(symbol_clone, target_timeframe.clone());
        let candle_minute_len = Duration::minutes(target_minutes as i64);

        let mut i = 0;
        while i <= up_to_index.min(source_frame.len().saturating_sub(1)) {
            if let Some(first_quote) = source_frame.get(i) {
                let (aggregated_quote, next_index) = Self::collect_candles_for_bar(
                    source_frame,
                    first_quote,
                    i,
                    up_to_index,
                    count_old_candles_in_one_new,
                    candle_minute_len,
                    target_timeframe,
                    &target_timeframe_for_quotes,
                )?;

                if let Some(quote) = aggregated_quote {
                    result_frame.push(quote)?;
                }

                i = next_index;
            } else {
                i += 1;
            }
        }

        Ok(result_frame)
    }

    pub fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        TimeFrameConverter::timeframe_to_minutes(tf)
    }

    pub fn is_valid_aggregation(source: &TimeFrame, target: &TimeFrame) -> bool {
        TimeFrameValidator::is_valid_aggregation(source, target)
    }

    pub fn align_to_timeframe(
        timestamp: DateTime<Utc>,
        timeframe: &TimeFrame,
    ) -> Result<DateTime<Utc>, TimeFrameAggregationError> {
        TimeFrameConverter::align_to_timeframe(timestamp, timeframe)
    }

    fn collect_candles_for_bar(
        source_frame: &QuoteFrame,
        first_quote: &Quote,
        start_index: usize,
        up_to_index: usize,
        count_old_candles_in_one_new: usize,
        candle_minute_len: Duration,
        target_timeframe: &TimeFrame,
        target_timeframe_for_quotes: &TimeFrame,
    ) -> Result<(Option<Quote>, usize), TimeFrameAggregationError> {
        let mut new_candle_start = first_quote.timestamp();
        let aligned_start =
            TimeFrameConverter::align_to_timeframe(new_candle_start, target_timeframe)?;
        new_candle_start = aligned_start;

        let mut new_candle_open = first_quote.open();
        let mut new_candle_high = first_quote.high();
        let mut new_candle_low = first_quote.low();
        let mut new_candle_close = first_quote.close();
        let mut new_candle_volume = first_quote.volume();

        let mut i = start_index + 1;
        let end_candle_time = new_candle_start + candle_minute_len;
        let mut collected_count = 1;

        for i2 in 0..(count_old_candles_in_one_new - 1) {
            if i > up_to_index.min(source_frame.len().saturating_sub(1)) {
                break;
            }

            if let Some(current_quote) = source_frame.get(i) {
                let current_time = current_quote.timestamp();

                if current_time >= end_candle_time {
                    i -= 1;
                    break;
                }

                if new_candle_start.date_naive() != current_time.date_naive() {
                    i -= 1;
                    break;
                }

                if current_quote.high() > new_candle_high {
                    new_candle_high = current_quote.high();
                }
                if current_quote.low() < new_candle_low {
                    new_candle_low = current_quote.low();
                }

                new_candle_close = current_quote.close();
                new_candle_volume += current_quote.volume();
                collected_count += 1;

                if i2 + 1 < count_old_candles_in_one_new - 1 {
                    i += 1;
                }
            } else {
                break;
            }
        }

        if collected_count >= count_old_candles_in_one_new {
            let aggregated_quote = Quote::from_parts(
                source_frame.symbol().clone(),
                target_timeframe_for_quotes.clone(),
                new_candle_start,
                new_candle_open,
                new_candle_high,
                new_candle_low,
                new_candle_close,
                new_candle_volume,
            );
            Ok((Some(aggregated_quote), i + 1))
        } else {
            Ok((None, i + 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::quote::Quote;
    use crate::data_model::types::{Symbol, TimeFrame};
    use chrono::{DateTime, Utc};

    fn create_test_quote_frame(symbol: Symbol, timeframe: TimeFrame, count: usize) -> QuoteFrame {
        let mut frame = QuoteFrame::new(symbol, timeframe);
        let base_time = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        for i in 0..count {
            let timestamp = base_time + chrono::Duration::minutes(i as i64 * 5);
            let base_price = 100.0 + (i as f32 * 0.5);
            let quote = Quote::from_parts(
                frame.symbol().clone(),
                frame.timeframe().clone(),
                timestamp,
                base_price,
                base_price + 1.0,
                base_price - 1.0,
                base_price + 0.5,
                1000.0,
            );
            frame.push(quote).unwrap();
        }
        frame
    }

    #[test]
    fn test_aggregate_5min_to_15min() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol, source_tf, 12);

        let result = TimeFrameAggregator::aggregate(&source_frame, target_tf.clone()).unwrap();

        assert!(!result.frame().is_empty());
        assert_eq!(result.frame().timeframe(), &target_tf);
        assert!(result.frame().len() <= source_frame.len());
        assert!(result.frame().len() >= source_frame.len() / 3);
    }

    #[test]
    fn test_aggregate_invalid_aggregation() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(15);
        let target_tf = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol, source_tf, 10);

        let result = TimeFrameAggregator::aggregate(&source_frame, target_tf);
        assert!(result.is_err());
        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("Invalid aggregation"));
            }
            Ok(_) => panic!("Expected error"),
        }
    }

    #[test]
    fn test_generate_derived_timeframes() {
        let base_tf = TimeFrame::Minutes(5);
        let multipliers = vec![2, 3, 4];

        let derived = TimeFrameAggregator::generate_derived_timeframes(&base_tf, &multipliers);

        assert_eq!(derived.len(), 3);
        assert_eq!(derived[0], TimeFrame::Minutes(10));
        assert_eq!(derived[1], TimeFrame::Minutes(15));
        assert_eq!(derived[2], TimeFrame::Minutes(20));
    }

    #[test]
    fn test_timeframe_to_minutes() {
        assert_eq!(
            TimeFrameAggregator::timeframe_to_minutes(&TimeFrame::Minutes(5)),
            Some(5)
        );
        assert_eq!(
            TimeFrameAggregator::timeframe_to_minutes(&TimeFrame::Hours(1)),
            Some(60)
        );
        assert_eq!(
            TimeFrameAggregator::timeframe_to_minutes(&TimeFrame::Days(1)),
            Some(1440)
        );
        assert_eq!(
            TimeFrameAggregator::timeframe_to_minutes(&TimeFrame::Custom("test".to_string())),
            None
        );
    }

    #[test]
    fn test_is_valid_aggregation() {
        let source = TimeFrame::Minutes(5);
        let target1 = TimeFrame::Minutes(15);
        let target2 = TimeFrame::Minutes(7);

        assert!(TimeFrameAggregator::is_valid_aggregation(&source, &target1));
        assert!(!TimeFrameAggregator::is_valid_aggregation(
            &source, &target2
        ));
    }

    #[test]
    fn test_aggregated_quote_frame_get_source_indices() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol, source_tf, 12);

        let aggregated = TimeFrameAggregator::aggregate(&source_frame, target_tf).unwrap();

        for i in 0..aggregated.frame().len() {
            if let Some(indices) = aggregated.get_source_indices(i) {
                assert!(!indices.is_empty());
                assert!(indices.iter().all(|&idx| idx < source_frame.len()));
            }
        }
    }

    #[test]
    fn test_aggregated_quote_frame_get_source_quotes() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol, source_tf, 12);

        let aggregated = TimeFrameAggregator::aggregate(&source_frame, target_tf).unwrap();

        if aggregated.frame().len() > 0 {
            if let Some(quotes) = aggregated.get_source_quotes(0, &source_frame) {
                assert!(!quotes.is_empty());
            }
        }
    }

    #[test]
    fn test_expand_aggregated_frame() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol.clone(), source_tf.clone(), 12);

        let aggregated = TimeFrameAggregator::aggregate(&source_frame, target_tf).unwrap();
        let expanded = aggregated.expand(&source_frame).unwrap();

        assert!(!expanded.is_empty());
        assert_eq!(expanded.timeframe(), &source_tf);
        assert_eq!(expanded.symbol(), &symbol);
    }

    #[test]
    fn test_build_compressed_frame_from_source() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol, source_tf, 12);

        let result =
            TimeFrameAggregator::build_compressed_frame_from_source(&source_frame, &target_tf, 11)
                .unwrap();

        assert!(!result.is_empty());
        assert_eq!(result.timeframe(), &target_tf);
    }

    #[test]
    fn test_build_compressed_frame_same_timeframe() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let timeframe = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol, timeframe.clone(), 10);

        let result =
            TimeFrameAggregator::build_compressed_frame_from_source(&source_frame, &timeframe, 9)
                .unwrap();

        assert_eq!(result.len(), 10);
        assert_eq!(result.timeframe(), &timeframe);
    }

    #[test]
    fn test_align_to_timeframe() {
        let timestamp = DateTime::parse_from_rfc3339("2024-01-01T00:07:30Z")
            .unwrap()
            .with_timezone(&Utc);
        let timeframe = TimeFrame::Minutes(15);

        let aligned = TimeFrameAggregator::align_to_timeframe(timestamp, &timeframe).unwrap();

        let expected = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(aligned, expected);
    }

    #[test]
    fn test_metadata() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol, source_tf.clone(), 12);

        let aggregated = TimeFrameAggregator::aggregate(&source_frame, target_tf.clone()).unwrap();
        let metadata = aggregated.metadata();

        assert_eq!(metadata.source_timeframe, source_tf);
        assert_eq!(metadata.target_timeframe, target_tf);
        assert_eq!(metadata.aggregation_ratio, 3.0);
    }

    #[test]
    fn test_aggregate_empty_frame() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = QuoteFrame::new(symbol, source_tf);

        let result = TimeFrameAggregator::aggregate(&source_frame, target_tf).unwrap();
        assert!(result.frame().is_empty());
        assert_eq!(result.metadata().aggregation_ratio, 1.0);
    }

    #[test]
    fn test_expand_empty_source_frame() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol.clone(), source_tf.clone(), 12);
        let empty_source = QuoteFrame::new(symbol, source_tf);

        let aggregated = TimeFrameAggregator::aggregate(&source_frame, target_tf).unwrap();
        let result = aggregated.expand(&empty_source);
        assert!(result.is_err());
    }

    #[test]
    fn test_expand_empty_aggregated_frame() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol.clone(), source_tf.clone(), 12);
        let empty_frame = QuoteFrame::new(symbol, source_tf.clone());

        let aggregated = TimeFrameAggregator::aggregate(&empty_frame, target_tf).unwrap();
        let result = aggregated.expand(&source_frame);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_build_compressed_frame_empty_source() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = QuoteFrame::new(symbol, source_tf);

        let result =
            TimeFrameAggregator::build_compressed_frame_from_source(&source_frame, &target_tf, 0)
                .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_build_compressed_frame_invalid_index() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(5);
        let target_tf = TimeFrame::Minutes(15);
        let source_frame = create_test_quote_frame(symbol, source_tf, 10);

        let result =
            TimeFrameAggregator::build_compressed_frame_from_source(&source_frame, &target_tf, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_compressed_frame_invalid_aggregation() {
        let symbol = Symbol::new("BTCUSDT".to_string());
        let source_tf = TimeFrame::Minutes(15);
        let target_tf = TimeFrame::Minutes(5);
        let source_frame = create_test_quote_frame(symbol, source_tf, 10);

        let result =
            TimeFrameAggregator::build_compressed_frame_from_source(&source_frame, &target_tf, 9);
        assert!(result.is_err());
    }
}
