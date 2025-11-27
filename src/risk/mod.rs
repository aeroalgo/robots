pub mod stops;
pub mod takes;
pub mod registry;

pub use stops::{
    AuxiliaryIndicatorSpec, StopEvaluationContext, StopValidationContext,
    StopValidationResult, StopOutcome, StopHandler, StopHandlerFactory, StopHandlerError,
    get_default_indicator_params, fill_missing_indicator_params, normalize_indicator_params,
    collect_required_auxiliary_indicators, compute_auxiliary_indicators,
    get_auxiliary_specs_from_handler_spec, collect_auxiliary_specs_from_stop_handlers,
};
pub use takes::{TakeEvaluationContext, TakeOutcome, TakeHandler, TakeHandlerFactory, TakeHandlerError};
pub use registry::*;
