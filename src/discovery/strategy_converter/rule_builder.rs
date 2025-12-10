use crate::discovery::engine::StrategyCandidate;
use crate::strategy::types::{
    ConditionBindingSpec, PositionDirection, RuleLogic, StrategyRuleSpec, StrategySignalType,
};

use super::main::StrategyConversionError;

pub struct RuleBuilder;

impl RuleBuilder {
    pub fn create_entry_rules(
        candidate: &StrategyCandidate,
        condition_bindings: &[ConditionBindingSpec],
    ) -> Result<Vec<StrategyRuleSpec>, StrategyConversionError> {
        if condition_bindings.is_empty() {
            return Ok(vec![]);
        }

        let condition_ids: Vec<String> = condition_bindings.iter().map(|c| c.id.clone()).collect();

        Ok(vec![StrategyRuleSpec {
            id: "entry_rule_1".to_string(),
            name: "Entry Rule".to_string(),
            logic: RuleLogic::All,
            conditions: condition_ids,
            signal: StrategySignalType::Entry,
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["auto-generated".to_string()],
            position_group: None,
            target_entry_ids: vec![],
        }])
    }

    pub fn create_exit_rules(
        candidate: &StrategyCandidate,
        exit_condition_bindings: &[ConditionBindingSpec],
    ) -> Result<Vec<StrategyRuleSpec>, StrategyConversionError> {
        let mut exit_rules = Vec::new();

        if !exit_condition_bindings.is_empty() {
            let condition_ids: Vec<String> = exit_condition_bindings
                .iter()
                .map(|c| c.id.clone())
                .collect();
            exit_rules.push(StrategyRuleSpec {
                id: "exit_rule_1".to_string(),
                name: "Exit Rule".to_string(),
                logic: RuleLogic::All,
                conditions: condition_ids,
                signal: StrategySignalType::Exit,
                direction: PositionDirection::Long,
                quantity: None,
                tags: vec!["auto-generated".to_string(), "exit-conditions".to_string()],
                position_group: None,
                target_entry_ids: vec![],
            });
        }

        Ok(exit_rules)
    }
}
