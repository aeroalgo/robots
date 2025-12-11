use super::TimeFrameAggregationError;
use crate::data_model::types::TimeFrame;
use chrono::{DateTime, Utc};

pub struct TimeFrameConverter;

impl TimeFrameConverter {
    pub fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        match tf {
            TimeFrame::Minutes(m) => Some(*m),
            TimeFrame::Hours(h) => Some(h * 60),
            TimeFrame::Days(d) => Some(d * 24 * 60),
            TimeFrame::Weeks(w) => Some(w * 7 * 24 * 60),
            TimeFrame::Months(m) => Some(m * 30 * 24 * 60),
            TimeFrame::Custom(_) => None,
        }
    }

    pub fn minutes_to_timeframe(minutes: u32) -> Option<TimeFrame> {
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

    pub fn calculate_ratio(
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

    pub fn align_to_timeframe(
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
}

