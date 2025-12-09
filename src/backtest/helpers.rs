use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::strategy::context::{StrategyContext, TimeframeData};
use std::sync::Arc;

use super::constants;

pub fn create_initial_timeframe_data(frame: &Arc<QuoteFrame>) -> TimeframeData {
    TimeframeData::with_quote_frame(frame.as_ref(), constants::INITIAL_INDEX)
}

pub fn ensure_timeframe_in_context(
    context: &mut StrategyContext,
    timeframe: &TimeFrame,
    frame: &Arc<QuoteFrame>,
) {
    if context.timeframe(timeframe).is_err() {
        let data = create_initial_timeframe_data(frame);
        context.insert_timeframe(timeframe.clone(), data);
    }
}
