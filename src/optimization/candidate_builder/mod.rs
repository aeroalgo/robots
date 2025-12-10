mod helpers;
mod phase_builder;
mod rules_applier;
mod validator;

use crate::data_model::types::TimeFrame;
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerConfig};
use rand::Rng;

use super::builders::ConditionBuilder;
use super::candidate_builder_config::{
    CandidateBuilderConfig, ElementConstraints, ElementProbabilities,
};

pub struct CandidateBuilder {
    config: CandidateBuilderConfig,
    rng: rand::rngs::ThreadRng,
}

impl CandidateBuilder {
    pub fn new(config: CandidateBuilderConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
        }
    }

    pub fn build_candidate(
        &mut self,
        available_indicators: &[IndicatorInfo],
        available_stop_handlers: &[StopHandlerConfig],
        available_timeframes: &[TimeFrame],
    ) -> CandidateElements {
        let constraints = self.config.constraints.clone();
        let probabilities = self.config.probabilities.clone();

        let mut candidate = CandidateElements {
            indicators: Vec::new(),
            nested_indicators: Vec::new(),
            entry_conditions: Vec::new(),
            exit_conditions: Vec::new(),
            stop_handlers: Vec::new(),
            take_handlers: Vec::new(),
            timeframes: Vec::new(),
        };

        phase_builder::build_phase_1(
            &mut candidate,
            available_indicators,
            available_stop_handlers,
            available_timeframes,
            &constraints,
            &probabilities,
            &self.config,
            &mut self.rng,
        );

        rules_applier::apply_rules(
            &mut candidate,
            available_stop_handlers,
            &self.config.rules,
            &self.config,
            &mut self.rng,
        );

        let mut phase = 2;
        while helpers::should_add(probabilities.phases.continue_building, &mut self.rng) {
            let all_limits_reached = phase_builder::build_additional_phase(
                &mut candidate,
                available_indicators,
                available_stop_handlers,
                available_timeframes,
                &constraints,
                &probabilities,
                phase,
                &self.config,
                &mut self.rng,
            );
            if all_limits_reached {
                break;
            }
            phase += 1;
        }

        validator::ensure_all_indicators_used(
            &candidate.indicators,
            &candidate.nested_indicators,
            &mut candidate.entry_conditions,
            &candidate.exit_conditions,
            &constraints,
            &self.config,
            &mut self.rng,
        );

        validator::ensure_minimum_requirements(
            &mut candidate,
            &constraints,
            available_stop_handlers,
            available_indicators,
            &self.config,
            &mut self.rng,
        );

        candidate
    }

    pub fn is_comparison_operator(operator: &crate::strategy::types::ConditionOperator) -> bool {
        ConditionBuilder::is_comparison_operator(operator)
    }

    pub fn extract_operands(condition: &ConditionInfo) -> Option<ConditionOperands> {
        ConditionBuilder::extract_operands(condition)
    }

    pub fn has_conflicting_comparison_operator(
        new_condition: &ConditionInfo,
        existing_conditions: &[ConditionInfo],
    ) -> bool {
        ConditionBuilder::has_conflicting_comparison_operator(new_condition, existing_conditions)
    }
}

pub use super::builders::condition_builder::ConditionOperands;

pub struct CandidateElements {
    pub indicators: Vec<IndicatorInfo>,
    pub nested_indicators: Vec<NestedIndicator>,
    pub entry_conditions: Vec<ConditionInfo>,
    pub exit_conditions: Vec<ConditionInfo>,
    pub stop_handlers: Vec<crate::discovery::types::StopHandlerInfo>,
    pub take_handlers: Vec<crate::discovery::types::StopHandlerInfo>,
    pub timeframes: Vec<TimeFrame>,
}
