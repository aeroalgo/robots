use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};

use crate::candles::bar_types::BarType;
use crate::data_model::types::{Symbol, TimeFrame};
use crate::strategy::types::{ConditionEvaluation, StrategySignal, StrategySignalType};

#[derive(Debug, Clone)]
pub struct TimeFrameSignalState {
    pub timeframe: TimeFrame,
    pub bar_type: BarType,
    pub signal_active: bool,
    pub signal_start_time: Option<DateTime<Utc>>,
    pub signal_end_time: Option<DateTime<Utc>>,
    pub last_evaluation: Option<DateTime<Utc>>,
}

impl TimeFrameSignalState {
    pub fn new(timeframe: TimeFrame, bar_type: BarType) -> Self {
        Self {
            timeframe,
            bar_type,
            signal_active: false,
            signal_start_time: None,
            signal_end_time: None,
            last_evaluation: None,
        }
    }

    pub fn update_signal(&mut self, is_active: bool, current_time: DateTime<Utc>) {
        self.last_evaluation = Some(current_time);

        if is_active && !self.signal_active {
            self.signal_active = true;
            self.signal_start_time = Some(current_time);
            self.signal_end_time = None;
        } else if !is_active && self.signal_active {
            self.signal_active = false;
            self.signal_end_time = Some(current_time);
        }
    }

    pub fn is_signal_active_at(&self, time: DateTime<Utc>) -> bool {
        if !self.signal_active {
            return false;
        }

        if let Some(start_time) = self.signal_start_time {
            if time < start_time {
                return false;
            }
        }

        if let Some(end_time) = self.signal_end_time {
            if time >= end_time {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimeFrameBarTypeKey {
    pub timeframe: TimeFrame,
    pub bar_type: BarType,
}

pub struct MultiTimeFrameSignalSync {
    symbol: Symbol,
    signal_states: HashMap<TimeFrameBarTypeKey, TimeFrameSignalState>,
    timeframe_hierarchy: Vec<TimeFrameBarTypeKey>,
}

impl MultiTimeFrameSignalSync {
    pub fn new(symbol: Symbol, timeframe_bar_types: Vec<(TimeFrame, BarType)>) -> Self {
        let mut keys: Vec<TimeFrameBarTypeKey> = timeframe_bar_types
            .iter()
            .map(|(tf, bt)| TimeFrameBarTypeKey {
                timeframe: tf.clone(),
                bar_type: bt.clone(),
            })
            .collect();

        keys.sort_by(|a, b| {
            Self::compare_timeframes(&a.timeframe, &b.timeframe)
                .then_with(|| a.bar_type.name().cmp(&b.bar_type.name()))
        });

        let mut signal_states = HashMap::new();
        for key in &keys {
            signal_states.insert(
                key.clone(),
                TimeFrameSignalState::new(key.timeframe.clone(), key.bar_type.clone()),
            );
        }

        Self {
            symbol,
            signal_states,
            timeframe_hierarchy: keys,
        }
    }

    pub fn update_signal(
        &mut self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
        condition_evaluations: &HashMap<String, ConditionEvaluation>,
        current_time: DateTime<Utc>,
    ) {
        let all_satisfied = condition_evaluations
            .values()
            .all(|eval| eval.satisfied);

        let key = TimeFrameBarTypeKey {
            timeframe: timeframe.clone(),
            bar_type: bar_type.clone(),
        };

        if let Some(state) = self.signal_states.get_mut(&key) {
            state.update_signal(all_satisfied, current_time);
        }
    }

    pub fn get_combined_signal(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
        current_time: DateTime<Utc>,
    ) -> Option<bool> {
        let target_key = TimeFrameBarTypeKey {
            timeframe: target_timeframe.clone(),
            bar_type: target_bar_type.clone(),
        };

        let target_state = self.signal_states.get(&target_key)?;
        let target_signal = target_state.is_signal_active_at(current_time);

        let higher_keys: Vec<&TimeFrameBarTypeKey> = self
            .timeframe_hierarchy
            .iter()
            .filter(|key| {
                Self::is_higher_timeframe(&key.timeframe, target_timeframe)
                    || (key.timeframe == *target_timeframe
                        && key.bar_type != *target_bar_type
                        && Self::is_higher_bar_type(&key.bar_type, target_bar_type))
            })
            .collect();

        if higher_keys.is_empty() {
            return Some(target_signal);
        }

        let all_higher_signals_active = higher_keys.iter().all(|key| {
            self.signal_states
                .get(*key)
                .map(|state| state.is_signal_active_at(current_time))
                .unwrap_or(false)
        });

        Some(target_signal && all_higher_signals_active)
    }

    pub fn get_signal_duration(
        &self,
        higher_timeframe: &TimeFrame,
        lower_timeframe: &TimeFrame,
    ) -> Option<Duration> {
        if !Self::is_higher_timeframe(higher_timeframe, lower_timeframe) {
            return None;
        }

        let higher_minutes = Self::timeframe_to_minutes(higher_timeframe)?;
        let lower_minutes = Self::timeframe_to_minutes(lower_timeframe)?;

        if higher_minutes == 0 || lower_minutes == 0 {
            return None;
        }

        let ratio = higher_minutes as f64 / lower_minutes as f64;
        Some(Duration::minutes((ratio * lower_minutes as f64) as i64))
    }

    pub fn extend_signal_to_lower_timeframe(
        &self,
        higher_timeframe: &TimeFrame,
        lower_timeframe: &TimeFrame,
        signal_start: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        let duration = self.get_signal_duration(higher_timeframe, lower_timeframe)?;
        Some(signal_start + duration)
    }

    fn compare_timeframes(a: &TimeFrame, b: &TimeFrame) -> std::cmp::Ordering {
        let a_minutes = Self::timeframe_to_minutes(a).unwrap_or(0);
        let b_minutes = Self::timeframe_to_minutes(b).unwrap_or(0);
        a_minutes.cmp(&b_minutes)
    }

    fn is_higher_timeframe(higher: &TimeFrame, lower: &TimeFrame) -> bool {
        let higher_minutes = Self::timeframe_to_minutes(higher).unwrap_or(0);
        let lower_minutes = Self::timeframe_to_minutes(lower).unwrap_or(0);
        higher_minutes > lower_minutes
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

    pub fn get_timeframe_hierarchy(&self) -> &[TimeFrameBarTypeKey] {
        &self.timeframe_hierarchy
    }

    pub fn get_signal_state(
        &self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
    ) -> Option<&TimeFrameSignalState> {
        let key = TimeFrameBarTypeKey {
            timeframe: timeframe.clone(),
            bar_type: bar_type.clone(),
        };
        self.signal_states.get(&key)
    }

    fn is_higher_bar_type(higher: &BarType, lower: &BarType) -> bool {
        match (higher, lower) {
            (BarType::Time, _) => false,
            (_, BarType::Time) => true,
            (BarType::HeikinAshi, _) => false,
            (_, BarType::HeikinAshi) => true,
            _ => false,
        }
    }
}

pub struct SignalCombiner {
    sync: MultiTimeFrameSignalSync,
}

impl SignalCombiner {
    pub fn new(symbol: Symbol, timeframe_bar_types: Vec<(TimeFrame, BarType)>) -> Self {
        Self {
            sync: MultiTimeFrameSignalSync::new(symbol, timeframe_bar_types),
        }
    }

    pub fn update_signals(
        &mut self,
        timeframe: &TimeFrame,
        bar_type: &BarType,
        condition_evaluations: &HashMap<String, ConditionEvaluation>,
        current_time: DateTime<Utc>,
    ) {
        self.sync
            .update_signal(timeframe, bar_type, condition_evaluations, current_time);
    }

    pub fn combine_signals(
        &self,
        target_timeframe: &TimeFrame,
        target_bar_type: &BarType,
        current_time: DateTime<Utc>,
    ) -> Option<CombinedSignal> {
        let combined = self
            .sync
            .get_combined_signal(target_timeframe, target_bar_type, current_time)?;

        let target_state = self
            .sync
            .get_signal_state(target_timeframe, target_bar_type)?;
        let higher_keys: Vec<&TimeFrameBarTypeKey> = self
            .sync
            .get_timeframe_hierarchy()
            .iter()
            .filter(|key| {
                MultiTimeFrameSignalSync::is_higher_timeframe(&key.timeframe, target_timeframe)
                    || (key.timeframe == *target_timeframe
                        && key.bar_type != *target_bar_type
                        && MultiTimeFrameSignalSync::is_higher_bar_type(&key.bar_type, target_bar_type))
            })
            .collect();

        let higher_signals: Vec<(TimeFrame, BarType, bool)> = higher_keys
            .iter()
            .filter_map(|key| {
                self.sync
                    .get_signal_state(&key.timeframe, &key.bar_type)
                    .map(|state| {
                        (
                            key.timeframe.clone(),
                            key.bar_type.clone(),
                            state.is_signal_active_at(current_time),
                        )
                    })
            })
            .collect();

        Some(CombinedSignal {
            target_timeframe: target_timeframe.clone(),
            target_bar_type: target_bar_type.clone(),
            target_signal: target_state.is_signal_active_at(current_time),
            higher_timeframe_signals: higher_signals,
            combined_signal: combined,
            timestamp: current_time,
        })
    }

    pub fn get_sync(&self) -> &MultiTimeFrameSignalSync {
        &self.sync
    }
}

#[derive(Debug, Clone)]
pub struct CombinedSignal {
    pub target_timeframe: TimeFrame,
    pub target_bar_type: BarType,
    pub target_signal: bool,
    pub higher_timeframe_signals: Vec<(TimeFrame, BarType, bool)>,
    pub combined_signal: bool,
    pub timestamp: DateTime<Utc>,
}

impl CombinedSignal {
    pub fn should_open_position(&self) -> bool {
        self.combined_signal
    }

    pub fn get_signal_strength(&self) -> f32 {
        let mut strength = if self.target_signal { 1.0 } else { 0.0 };
        let higher_count = self.higher_timeframe_signals.len();
        if higher_count > 0 {
            let higher_active_count = self
                .higher_timeframe_signals
                .iter()
                .filter(|(_, _, active)| *active)
                .count();
            strength += (higher_active_count as f32 / higher_count as f32) * 0.5;
        }
        strength.min(1.0)
    }
}

