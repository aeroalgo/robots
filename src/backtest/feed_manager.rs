use std::collections::HashMap;
use std::sync::Arc;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::strategy::context::{StrategyContext, TimeframeData};

use super::{constants, helpers, traits::FeedProvider};

pub struct FeedManager {
    frames: HashMap<TimeFrame, Arc<QuoteFrame>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: Option<TimeFrame>,
    higher_timeframe_timestamps: HashMap<TimeFrame, Vec<i64>>,
    cached_aligned_timestamps: HashMap<TimeFrame, i64>,
}

impl FeedManager {
    pub fn with_frames(frames: HashMap<TimeFrame, Arc<QuoteFrame>>) -> Self {
        Self {
            frames,
            indices: HashMap::new(),
            primary_timeframe: None,
            higher_timeframe_timestamps: HashMap::new(),
            cached_aligned_timestamps: HashMap::new(),
        }
    }

    pub fn frames(&self) -> &HashMap<TimeFrame, Arc<QuoteFrame>> {
        &self.frames
    }

    pub fn primary_timeframe(&self) -> Option<&TimeFrame> {
        self.primary_timeframe.as_ref()
    }

    pub fn set_primary_timeframe(&mut self, timeframe: TimeFrame) {
        self.primary_timeframe = Some(timeframe);
    }

    pub fn get_frame(&self, timeframe: &TimeFrame) -> Option<&Arc<QuoteFrame>> {
        self.frames.get(timeframe)
    }

    pub fn initialize_context_ordered(&self, timeframe_order: &[TimeFrame]) -> StrategyContext {
        let mut map = HashMap::with_capacity(self.frames.len());
        for (timeframe, frame) in &self.frames {
            let data = helpers::create_initial_timeframe_data(frame);
            map.insert(timeframe.clone(), data);
        }
        StrategyContext::with_timeframes_ordered(timeframe_order, map)
    }

    pub fn reset(&mut self) {
        self.indices.clear();
        for timeframe in self.frames.keys() {
            self.indices.insert(timeframe.clone(), 0);
        }
        self.cached_aligned_timestamps.clear();
    }

    pub fn step(&mut self, context: &mut StrategyContext) -> bool {
        let Some(primary_tf) = &self.primary_timeframe else {
            return false;
        };

        let primary_frame = match self.frames.get(primary_tf) {
            Some(f) => f,
            None => return false,
        };

        let current_idx = *self
            .indices
            .get(primary_tf)
            .unwrap_or(&constants::INITIAL_INDEX);
        if current_idx >= primary_frame.len() {
            return false;
        }

        if let Ok(data) = context.timeframe_mut(primary_tf) {
            data.set_index(current_idx);
        }

        if let Some(bar) = primary_frame.get(current_idx) {
            let current_timestamp = bar.timestamp_millis();

            for (tf, frame) in &self.frames {
                if tf == primary_tf {
                    continue;
                }

                if !Self::is_higher_timeframe(tf, primary_tf) {
                    continue;
                }

                let aligned = self.cached_aligned_timestamps.get(tf).copied();
                let needs_update = aligned.is_none()
                    || Self::align_timestamp_millis_to_timeframe(current_timestamp, tf) != aligned;

                if needs_update {
                    if let Some(new_aligned) =
                        Self::align_timestamp_millis_to_timeframe(current_timestamp, tf)
                    {
                        self.cached_aligned_timestamps
                            .insert(tf.clone(), new_aligned);
                    }
                }

                let timestamps = self
                    .higher_timeframe_timestamps
                    .entry(tf.clone())
                    .or_insert_with(|| {
                        let mut vec = Vec::with_capacity(frame.len());
                        for i in 0..frame.len() {
                            if let Some(bar) = frame.get(i) {
                                vec.push(bar.timestamp_millis());
                            }
                        }
                        vec
                    });

                let aligned_ts = self.cached_aligned_timestamps.get(tf).copied();
                if let Some(target_ts) = aligned_ts {
                    let higher_idx = timestamps
                        .iter()
                        .rposition(|&ts| ts <= target_ts)
                        .unwrap_or(constants::INITIAL_INDEX);

                    if let Ok(data) = context.timeframe_mut(tf) {
                        data.set_index(higher_idx);
                    }
                }
            }
        }

        self.indices.insert(primary_tf.clone(), current_idx + 1);
        true
    }

    pub fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        match tf {
            TimeFrame::Minutes(m) => Some(*m),
            TimeFrame::Hours(h) => Some(h * constants::MINUTES_PER_HOUR as u32),
            TimeFrame::Days(d) => {
                Some(d * constants::HOURS_PER_DAY as u32 * constants::MINUTES_PER_HOUR as u32)
            }
            TimeFrame::Weeks(w) => Some(
                w * constants::DAYS_PER_WEEK as u32
                    * constants::HOURS_PER_DAY as u32
                    * constants::MINUTES_PER_HOUR as u32,
            ),
            TimeFrame::Months(m) => Some(
                m * constants::DAYS_PER_MONTH as u32
                    * constants::HOURS_PER_DAY as u32
                    * constants::MINUTES_PER_HOUR as u32,
            ),
            TimeFrame::Custom(_) => None,
        }
    }

    pub fn is_higher_timeframe(higher: &TimeFrame, lower: &TimeFrame) -> bool {
        let higher_min =
            Self::timeframe_to_minutes(higher).unwrap_or(constants::INITIAL_INDEX as u32);
        let lower_min =
            Self::timeframe_to_minutes(lower).unwrap_or(constants::INITIAL_INDEX as u32);
        higher_min > lower_min
    }

    pub fn is_multiple_of(base: &TimeFrame, target: &TimeFrame) -> bool {
        let base_min = Self::timeframe_to_minutes(base).unwrap_or(constants::INITIAL_INDEX as u32);
        let target_min =
            Self::timeframe_to_minutes(target).unwrap_or(constants::INITIAL_INDEX as u32);
        if base_min == 0 || target_min == 0 {
            return false;
        }
        target_min > base_min && target_min % base_min == 0
    }

    pub fn align_timestamp_millis_to_timeframe(
        timestamp_millis: i64,
        timeframe: &TimeFrame,
    ) -> Option<i64> {
        let minutes = Self::timeframe_to_minutes(timeframe)?;
        let total_minutes = timestamp_millis / (60 * 1000);
        let aligned_minutes = (total_minutes / minutes as i64) * minutes as i64;
        Some(aligned_minutes * 60 * 1000)
    }
}

impl FeedProvider for FeedManager {
    fn frames(&self) -> &HashMap<TimeFrame, Arc<QuoteFrame>> {
        self.frames()
    }

    fn primary_timeframe(&self) -> Option<&TimeFrame> {
        self.primary_timeframe()
    }

    fn get_frame(&self, timeframe: &TimeFrame) -> Option<&Arc<QuoteFrame>> {
        self.get_frame(timeframe)
    }

    fn step(&mut self, context: &mut StrategyContext) -> bool {
        self.step(context)
    }

    fn reset(&mut self) {
        self.reset()
    }
}
