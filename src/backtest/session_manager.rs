use std::sync::Arc;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::strategy::context::StrategyContext;

#[derive(Clone, Copy, Debug, Default)]
pub struct SessionState {
    pub is_session_start: bool,
    pub is_session_end: bool,
}

pub struct SessionManager {
    cached_duration: Option<chrono::Duration>,
}

impl SessionManager {
    pub fn new(cached_duration: Option<chrono::Duration>) -> Self {
        Self { cached_duration }
    }

    pub fn session_state(
        &self,
        primary_tf: &TimeFrame,
        frame: &Arc<QuoteFrame>,
        context: &StrategyContext,
    ) -> Option<SessionState> {
        let timeframe_data = context.timeframe(primary_tf).ok()?;
        let idx = timeframe_data.index();

        if frame.len() == 0 || idx >= frame.len() {
            return None;
        }

        let duration = self.cached_duration?;
        let current = frame.get(idx)?;
        let mut state = SessionState::default();

        if idx == 0 {
            state.is_session_start = true;
        } else if let Some(prev) = frame.get(idx.saturating_sub(1)) {
            let delta = current.timestamp() - prev.timestamp();
            if delta > duration {
                state.is_session_start = true;
            }
        }

        if idx + 1 >= frame.len() {
            state.is_session_end = true;
        } else if let Some(next) = frame.get(idx + 1) {
            let delta = next.timestamp() - current.timestamp();
            if delta > duration {
                state.is_session_end = true;
            }
        }

        Some(state)
    }

    pub fn update_metadata(&self, context: &mut StrategyContext, state: Option<SessionState>) {
        if let Some(s) = state {
            if s.is_session_start {
                context
                    .metadata
                    .insert("session_start".to_string(), "true".to_string());
            } else {
                context.metadata.remove("session_start");
            }
            if s.is_session_end {
                context
                    .metadata
                    .insert("session_end".to_string(), "true".to_string());
            } else {
                context.metadata.remove("session_end");
            }
        }
    }

    pub fn set_duration(&mut self, duration: Option<chrono::Duration>) {
        self.cached_duration = duration;
    }
}
