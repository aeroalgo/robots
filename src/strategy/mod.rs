pub mod base;
pub mod builder;
pub mod context;
pub mod executor;
pub mod multitimeframe_integration;
pub mod multitimeframe_signal_sync;
pub mod presets;
pub mod registry;
pub mod types;

pub use multitimeframe_integration::{
    EnhancedStrategyContext, MultiTimeFrameExecutorExtension, MultiTimeFrameStrategyContext,
    MultiTimeFrameStrategyWrapper,
};
pub use multitimeframe_signal_sync::{
    CombinedSignal, MultiTimeFrameSignalSync, SignalCombiner, TimeFrameBarTypeKey,
    TimeFrameSignalState,
};

#[cfg(test)]
pub mod tests;
