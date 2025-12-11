use super::converter::TimeFrameConverter;
use super::{AggregatedQuoteFrame, TimeFrameAggregationError};
use crate::data_model::quote::Quote;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::{Symbol, TimeFrame};
use chrono::{DateTime, Duration, Utc};

pub struct TimeFrameExpander;

impl TimeFrameExpander {
    pub fn expand(
        aggregated_frame: &AggregatedQuoteFrame,
        source_frame: &QuoteFrame,
    ) -> Result<QuoteFrame, TimeFrameAggregationError> {
        if aggregated_frame.frame.is_empty() {
            return Ok(QuoteFrame::new(
                aggregated_frame.frame.symbol().clone(),
                aggregated_frame.metadata.source_timeframe.clone(),
            ));
        }

        if source_frame.is_empty() {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                "Source frame is empty".to_string(),
                "Cannot expand without source data".to_string(),
            ));
        }

        let source_tf = &aggregated_frame.metadata.source_timeframe;
        let symbol = aggregated_frame.frame.symbol();
        let source_tf_clone = source_tf.clone();

        let ratio_f64 = aggregated_frame.metadata.aggregation_ratio;
        if ratio_f64 < 0.0 || ratio_f64 > usize::MAX as f64 {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                format!("Aggregation ratio {} is out of valid range", ratio_f64),
                format!("Must be between 0 and {}", usize::MAX),
            ));
        }
        let ratio = ratio_f64 as usize;

        let estimated_size = aggregated_frame
            .frame
            .len()
            .checked_mul(ratio)
            .ok_or_else(|| {
                TimeFrameAggregationError::InvalidAggregation(
                    "Estimated size overflow".to_string(),
                    format!(
                        "Cannot multiply {} * {}",
                        aggregated_frame.frame.len(),
                        ratio
                    ),
                )
            })?;
        let mut expanded_quotes = Vec::with_capacity(estimated_size);

        for (agg_idx, aggregated_quote) in aggregated_frame.frame.iter().enumerate() {
            if let Some(source_indices) = aggregated_frame.source_indices.get(agg_idx) {
                Self::expand_with_source_indices(
                    aggregated_quote,
                    source_indices,
                    source_frame,
                    symbol,
                    &source_tf_clone,
                    &mut expanded_quotes,
                )?;
            } else {
                Self::expand_without_source_indices(
                    aggregated_quote,
                    ratio,
                    source_tf,
                    symbol,
                    &source_tf_clone,
                    &mut expanded_quotes,
                )?;
            }
        }

        let mut expanded_frame = QuoteFrame::new(symbol.clone(), source_tf.clone());
        for quote in expanded_quotes {
            expanded_frame.push(quote)?;
        }

        Ok(expanded_frame)
    }

    fn expand_with_source_indices(
        aggregated_quote: &Quote,
        source_indices: &[usize],
        source_frame: &QuoteFrame,
        symbol: &Symbol,
        source_tf: &TimeFrame,
        expanded_quotes: &mut Vec<Quote>,
    ) -> Result<(), TimeFrameAggregationError> {
        if source_indices.is_empty() {
            return Ok(());
        }

        let source_quotes: Vec<&Quote> = source_indices
            .iter()
            .filter_map(|idx| source_frame.get(*idx))
            .collect();

        if source_quotes.is_empty() {
            return Ok(());
        }

        let first_source = source_quotes[0];
        let open_price = first_source.open();
        let close_price = aggregated_quote.close();
        let high_price = aggregated_quote.high();
        let low_price = aggregated_quote.low();
        let total_volume = aggregated_quote.volume();
        let volume_per_bar = if source_quotes.len() > 0 {
            total_volume / source_quotes.len() as f32
        } else {
            0.0
        };

        let aggregated_timestamp = aggregated_quote.timestamp();
        let source_tf_minutes = TimeFrameConverter::timeframe_to_minutes(source_tf).ok_or(
            TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", source_tf)),
        )?;

        for (i, _source_quote) in source_quotes.iter().enumerate() {
            let is_first = i == 0;
            let is_last = i == source_quotes.len() - 1;

            let quote_open = if is_first { open_price } else { close_price };
            let quote_close = close_price;
            let quote_high = high_price;
            let quote_low = low_price;

            let quote_volume = if source_quotes.len() == 1 {
                total_volume
            } else if is_last {
                total_volume - (volume_per_bar * (source_quotes.len() - 1) as f32)
            } else {
                volume_per_bar
            };

            let expanded_timestamp = aggregated_timestamp
                + chrono::Duration::minutes(i as i64 * source_tf_minutes as i64);

            let expanded_quote = Quote::from_parts(
                symbol.clone(),
                source_tf.clone(),
                expanded_timestamp,
                quote_open,
                quote_high,
                quote_low,
                quote_close,
                quote_volume,
            );

            expanded_quotes.push(expanded_quote);
        }

        Ok(())
    }

    fn expand_without_source_indices(
        aggregated_quote: &Quote,
        ratio: usize,
        source_tf: &TimeFrame,
        symbol: &Symbol,
        source_tf_clone: &TimeFrame,
        expanded_quotes: &mut Vec<Quote>,
    ) -> Result<(), TimeFrameAggregationError> {
        let source_tf_minutes =
            TimeFrameConverter::timeframe_to_minutes(source_tf).ok_or_else(|| {
                TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", source_tf))
            })?;

        let aggregated_timestamp = aggregated_quote.timestamp();
        let mut current_timestamp = aggregated_timestamp;

        for i in 0..ratio.min(usize::MAX) {
            let quote_open = if i == 0 {
                aggregated_quote.open()
            } else {
                aggregated_quote.close()
            };

            let quote_close = if i == ratio - 1 {
                aggregated_quote.close()
            } else {
                aggregated_quote.open()
            };

            let quote_high = aggregated_quote.high();
            let quote_low = aggregated_quote.low();
            let quote_volume = aggregated_quote.volume() / ratio as f32;

            let expanded_quote = Quote::from_parts(
                symbol.clone(),
                source_tf_clone.clone(),
                current_timestamp,
                quote_open,
                quote_high,
                quote_low,
                quote_close,
                quote_volume,
            );

            expanded_quotes.push(expanded_quote);

            if i < ratio - 1 {
                current_timestamp = current_timestamp + Duration::minutes(source_tf_minutes as i64);
            }
        }

        Ok(())
    }
}

