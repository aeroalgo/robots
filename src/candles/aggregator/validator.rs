use super::{TimeFrameAggregationError, TimeFrameConverter};
use crate::data_model::types::TimeFrame;

pub struct TimeFrameValidator;

impl TimeFrameValidator {
    pub fn is_valid_aggregation(source: &TimeFrame, target: &TimeFrame) -> bool {
        let source_minutes = TimeFrameConverter::timeframe_to_minutes(source);
        let target_minutes = TimeFrameConverter::timeframe_to_minutes(target);

        match (source_minutes, target_minutes) {
            (Some(s), Some(t)) => t > s && t % s == 0,
            _ => false,
        }
    }

    pub fn validate_aggregation(
        source: &TimeFrame,
        target: &TimeFrame,
    ) -> Result<(), TimeFrameAggregationError> {
        if !Self::is_valid_aggregation(source, target) {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                format!("{:?}", source),
                format!("{:?}", target),
            ));
        }
        Ok(())
    }

    pub fn validate_compression_params(
        source_frame: &crate::data_model::quote_frame::QuoteFrame,
        target_timeframe: &TimeFrame,
        up_to_index: usize,
    ) -> Result<(), TimeFrameAggregationError> {
        if up_to_index >= source_frame.len() {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                format!("up_to_index {} is out of bounds", up_to_index),
                format!("Source frame length is {}", source_frame.len()),
            ));
        }

        let source_tf = source_frame.timeframe();
        let target_minutes = TimeFrameConverter::timeframe_to_minutes(target_timeframe)
            .ok_or_else(|| {
                TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", target_timeframe))
            })?;
        let source_minutes =
            TimeFrameConverter::timeframe_to_minutes(source_tf).ok_or_else(|| {
                TimeFrameAggregationError::UnsupportedTimeFrame(format!("{:?}", source_tf))
            })?;

        if source_minutes == 0 {
            return Err(TimeFrameAggregationError::InvalidTimeFrame);
        }

        if target_minutes < source_minutes {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                format!(
                    "Target timeframe {} is smaller than source {}",
                    target_minutes, source_minutes
                ),
                "Cannot compress to smaller timeframe".to_string(),
            ));
        }

        if target_minutes % source_minutes != 0 {
            return Err(TimeFrameAggregationError::InvalidAggregation(
                format!(
                    "Target timeframe {} is not a multiple of source {}",
                    target_minutes, source_minutes
                ),
                "Target must be a multiple of source".to_string(),
            ));
        }

        Ok(())
    }
}

