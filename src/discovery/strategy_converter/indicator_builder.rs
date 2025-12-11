use std::collections::{HashMap, HashSet};

use crate::data_model::types::TimeFrame;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{IndicatorInfo, NestedIndicator};
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::{IndicatorBindingSpec, IndicatorSourceSpec};

use super::main::StrategyConversionError;

pub struct IndicatorBuilder;

impl IndicatorBuilder {
    pub fn create_indicator_bindings(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<IndicatorBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();
        let mut binding_keys = HashSet::new();

        let mut required_indicator_timeframes =
            Self::collect_required_timeframes(candidate, &base_timeframe);

        Self::add_missing_indicators_to_timeframes(
            candidate,
            &mut required_indicator_timeframes,
            &base_timeframe,
        );

        Self::create_base_indicator_bindings(
            candidate,
            &required_indicator_timeframes,
            &mut bindings,
            &mut binding_keys,
        )?;

        let base_bindings = bindings.clone();
        Self::create_nested_indicator_bindings(
            candidate,
            &required_indicator_timeframes,
            &base_timeframe,
            &base_bindings,
            &mut bindings,
            &mut binding_keys,
        )?;

        Ok(bindings)
    }

    fn collect_required_timeframes(
        candidate: &StrategyCandidate,
        base_timeframe: &TimeFrame,
    ) -> HashMap<String, HashSet<TimeFrame>> {
        let mut all_conditions: Vec<&dyn crate::optimization::condition_id::ConditionInfoTrait> =
            Vec::new();
        for condition in &candidate.conditions {
            all_conditions.push(condition);
        }
        for condition in &candidate.exit_conditions {
            all_conditions.push(condition);
        }

        ConditionId::collect_required_timeframes(&all_conditions, base_timeframe)
    }

    fn add_missing_indicators_to_timeframes(
        candidate: &StrategyCandidate,
        required_indicator_timeframes: &mut HashMap<String, HashSet<TimeFrame>>,
        base_timeframe: &TimeFrame,
    ) {
        for indicator in &candidate.indicators {
            if !required_indicator_timeframes.contains_key(&indicator.alias) {
                required_indicator_timeframes
                    .entry(indicator.alias.clone())
                    .or_insert_with(HashSet::new)
                    .insert(base_timeframe.clone());
            }
        }
        for nested in &candidate.nested_indicators {
            if !required_indicator_timeframes.contains_key(&nested.indicator.alias) {
                required_indicator_timeframes
                    .entry(nested.indicator.alias.clone())
                    .or_insert_with(HashSet::new)
                    .insert(base_timeframe.clone());
            }
        }
    }

    fn create_base_indicator_bindings(
        candidate: &StrategyCandidate,
        required_indicator_timeframes: &HashMap<String, HashSet<TimeFrame>>,
        bindings: &mut Vec<IndicatorBindingSpec>,
        binding_keys: &mut HashSet<String>,
    ) -> Result<(), StrategyConversionError> {
        for indicator in &candidate.indicators {
            let params = Self::extract_indicator_params(indicator)?;

            if let Some(timeframes) = required_indicator_timeframes.get(&indicator.alias) {
                for timeframe in timeframes {
                    let key = format!("{}:{:?}", indicator.alias, timeframe);
                    if !binding_keys.contains(&key) {
                        binding_keys.insert(key.clone());
                        bindings.push(IndicatorBindingSpec {
                            alias: indicator.alias.clone(),
                            timeframe: timeframe.clone(),
                            source: IndicatorSourceSpec::Registry {
                                name: indicator.name.clone(),
                                parameters: params.clone(),
                            },
                            tags: vec!["base".to_string()],
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn create_nested_indicator_bindings(
        candidate: &StrategyCandidate,
        required_indicator_timeframes: &HashMap<String, HashSet<TimeFrame>>,
        base_timeframe: &TimeFrame,
        existing_bindings: &[IndicatorBindingSpec],
        bindings: &mut Vec<IndicatorBindingSpec>,
        binding_keys: &mut HashSet<String>,
    ) -> Result<(), StrategyConversionError> {
        for nested in &candidate.nested_indicators {
            let params = Self::extract_indicator_params(&nested.indicator)?;

            let timeframes_to_use = Self::determine_nested_timeframes(
                nested,
                required_indicator_timeframes,
                base_timeframe,
                existing_bindings,
            );

            for timeframe in timeframes_to_use {
                let key = format!("{}:{:?}", nested.indicator.alias, timeframe);
                if !binding_keys.contains(&key) {
                    binding_keys.insert(key.clone());
                    bindings.push(IndicatorBindingSpec {
                        alias: nested.indicator.alias.clone(),
                        timeframe: timeframe.clone(),
                        source: IndicatorSourceSpec::Registry {
                            name: nested.indicator.name.clone(),
                            parameters: params.clone(),
                        },
                        tags: vec!["nested".to_string(), format!("depth_{}", nested.depth)],
                    });
                }
            }
        }
        Ok(())
    }

    fn determine_nested_timeframes(
        nested: &NestedIndicator,
        required_indicator_timeframes: &HashMap<String, HashSet<TimeFrame>>,
        base_timeframe: &TimeFrame,
        existing_bindings: &[IndicatorBindingSpec],
    ) -> HashSet<TimeFrame> {
        if let Some(explicit_timeframes) =
            required_indicator_timeframes.get(&nested.indicator.alias)
        {
            explicit_timeframes.clone()
        } else {
            let input_timeframes: HashSet<TimeFrame> = existing_bindings
                .iter()
                .filter(|binding| binding.alias == nested.input_indicator_alias)
                .map(|binding| binding.timeframe.clone())
                .collect();

            if !input_timeframes.is_empty() {
                input_timeframes
            } else {
                let mut result = HashSet::new();
                result.insert(base_timeframe.clone());
                result
            }
        }
    }

    fn extract_indicator_params(
        _indicator: &IndicatorInfo,
    ) -> Result<HashMap<String, f32>, StrategyConversionError> {
        Ok(HashMap::new())
    }
}

