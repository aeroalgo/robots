pub mod auxiliary;
pub mod context;
pub mod errors;
pub mod factory;
pub mod manager;
pub mod parameter_extractor;
pub mod parameters;
pub mod registry;
pub mod state;
pub mod stops;
pub mod takes;
pub mod traits;
pub mod utils;

pub use auxiliary::{
    collect_auxiliary_specs_from_stop_handlers, collect_required_auxiliary_indicators,
    compute_auxiliary_indicators, get_auxiliary_specs_from_handler_spec, AuxiliaryIndicatorSpec,
};
pub use context::{StopEvaluationContext, StopValidationContext, TakeEvaluationContext};
pub use errors::{StopHandlerError, TakeHandlerError};
pub use factory::{StopHandlerFactory, TakeHandlerFactory};
pub use manager::{RiskManager, StopHandlerEntry};
pub use parameter_extractor::{
    convert_params_from_f64, convert_params_to_f64, extract_bool,
    extract_indicator_params_with_aliases, extract_number, extract_number_required,
    extract_percentage, extract_string, extract_string_required, fill_missing_indicator_params,
    get_default_indicator_params, has_parameter, normalize_indicator_params,
    ParameterExtractionError, ParameterResult,
};
pub use parameters::StopParameterPresets;
pub use registry::{
    get_global_registry, get_stop_optimization_range, get_take_optimization_range,
    StopHandlerRegistry,
};
pub use state::{PositionRiskState, RiskStateBook, StopHistoryRecord};
pub use stops::{
    ATRTrailIndicatorStopHandler, ATRTrailStopHandler, HILOTrailingStopHandler,
    PercentTrailIndicatorStopHandler, PercentTrailingStopHandler, StopLossPctHandler,
};
pub use takes::TakeProfitPctHandler;
pub use traits::{StopHandler, StopOutcome, StopValidationResult, TakeHandler, TakeOutcome};
pub use utils::{
    extract_indicator_from_handler_name, process_stop_handler_indicator,
    stop_handler_requires_indicator,
};
