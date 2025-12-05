pub mod auxiliary;
pub mod context;
pub mod errors;
pub mod factory;
pub mod manager;
pub mod parameters;
pub mod registry;
pub mod state;
pub mod stops;
pub mod takes;
pub mod traits;
pub mod utils;

pub use auxiliary::{
    collect_auxiliary_specs_from_stop_handlers, collect_required_auxiliary_indicators,
    compute_auxiliary_indicators, fill_missing_indicator_params,
    get_auxiliary_specs_from_handler_spec, get_default_indicator_params,
    normalize_indicator_params, AuxiliaryIndicatorSpec,
};
pub use context::{StopEvaluationContext, StopValidationContext, TakeEvaluationContext};
pub use errors::{StopHandlerError, TakeHandlerError};
pub use factory::{
    get_stop_optimization_range, get_take_optimization_range, StopHandlerFactory,
    TakeHandlerFactory,
};
pub use manager::{RiskManager, StopHandlerEntry};
pub use parameters::StopParameterPresets;
pub use state::{PositionRiskState, RiskStateBook, StopHistoryRecord};
pub use stops::{
    ATRTrailStopHandler, HILOTrailingStopHandler, IndicatorStopHandler, PercentTrailingStopHandler,
    StopLossPctHandler,
};
pub use takes::TakeProfitPctHandler;
pub use traits::{StopHandler, StopOutcome, StopValidationResult, TakeHandler, TakeOutcome};
