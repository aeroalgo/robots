use std::collections::HashMap;

use crate::strategy::types::StopSignalKind;

use super::auxiliary::AuxiliaryIndicatorSpec;
use super::context::{StopEvaluationContext, StopValidationContext, TakeEvaluationContext};

pub struct StopValidationResult {
    pub stop_level: f64,
    pub is_valid: bool,
    pub reason: Option<String>,
}

pub struct StopOutcome {
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub metadata: HashMap<String, String>,
}

pub trait StopHandler: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome>;

    fn validate_before_entry(
        &self,
        _ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![]
    }
}

pub struct TakeOutcome {
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub metadata: HashMap<String, String>,
}

pub trait TakeHandler: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &TakeEvaluationContext<'_>) -> Option<TakeOutcome>;
}

