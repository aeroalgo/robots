use crate::condition::ConditionParameterPresets;
use crate::data_model::types::TimeFrame;
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator};
use crate::discovery::StrategyCandidate;
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::ConditionOperator;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::optimization::builders::IndicatorBuilder;
use crate::optimization::builders::OperatorSelectorFactory;
use crate::optimization::candidate_builder_config::{
    CandidateBuilderConfig, ConditionProbabilities, ParameterConstraint,
};

pub struct ConditionBuilder<'a> {
    config: &'a CandidateBuilderConfig,
    rng: &'a mut rand::rngs::ThreadRng,
}

impl<'a> ConditionBuilder<'a> {
    pub fn new(config: &'a CandidateBuilderConfig, rng: &'a mut rand::rngs::ThreadRng) -> Self {
        Self { config, rng }
    }

    pub fn build_condition(
        &mut self,
        indicators: &[IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        probabilities: &ConditionProbabilities,
        is_entry: bool,
    ) -> Option<ConditionInfo> {
        let all_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .chain(nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        if all_indicators.is_empty() {
            return None;
        }

        let Some(primary_indicator) = all_indicators.choose(&mut *self.rng) else {
            return None;
        };

        let (operator, condition_type) =
            OperatorSelectorFactory::select_operator_and_condition_type(
                primary_indicator,
                nested_indicators,
                &all_indicators,
                probabilities,
                &mut *self.rng,
            );

        let (condition_id, condition_name, constant_value, price_field, optimization_params) = self
            .generate_condition_details(
                primary_indicator,
                &all_indicators,
                nested_indicators,
                indicators,
                &condition_type,
                &operator,
                probabilities,
                is_entry,
            )?;

        let (primary_alias, secondary_alias) = if condition_type == "indicator_indicator" {
            if let Some(separator_pos) = condition_id.find("::") {
                let prefix_len = if condition_id.starts_with("entry_") {
                    6
                } else if condition_id.starts_with("exit_") {
                    5
                } else {
                    0
                };
                let primary = &condition_id[prefix_len..separator_pos];
                let after_separator = &condition_id[separator_pos + 2..];
                if let Some(last_underscore) = after_separator.rfind('_') {
                    let secondary = &after_separator[..last_underscore];
                    (primary.to_string(), Some(secondary.to_string()))
                } else {
                    (primary_indicator.alias.clone(), None)
                }
            } else {
                (primary_indicator.alias.clone(), None)
            }
        } else {
            (primary_indicator.alias.clone(), None)
        };

        Some(ConditionInfo {
            id: condition_id,
            name: condition_name,
            operator,
            condition_type: condition_type.to_string(),
            optimization_params,
            constant_value,
            primary_indicator_alias: primary_alias,
            secondary_indicator_alias: secondary_alias,
            primary_timeframe: None,
            secondary_timeframe: None,
            price_field,
        })
    }

    fn generate_condition_details(
        &mut self,
        primary_indicator: &IndicatorInfo,
        all_indicators: &[&IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        base_indicators: &[IndicatorInfo],
        condition_type: &str,
        operator: &ConditionOperator,
        probabilities: &ConditionProbabilities,
        is_entry: bool,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        match condition_type {
            "indicator_constant" => {
                self.generate_indicator_constant_condition(primary_indicator, operator, is_entry)
            }
            "trend_condition" => {
                self.generate_trend_condition(primary_indicator, operator, is_entry)
            }
            "indicator_indicator" => self.generate_indicator_indicator_condition(
                primary_indicator,
                all_indicators,
                nested_indicators,
                base_indicators,
                operator,
                probabilities,
                is_entry,
            ),
            "indicator_price" | _ => self.generate_indicator_price_condition(
                primary_indicator,
                operator,
                probabilities,
                is_entry,
            ),
        }
    }

    fn generate_indicator_constant_condition(
        &mut self,
        primary_indicator: &IndicatorInfo,
        operator: &ConditionOperator,
        is_entry: bool,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let constant_value = if primary_indicator.indicator_type == "volatility" {
            let rules = &self.config.rules.indicator_parameter_rules;
            let mut percentage_range = (0.2, 10.0, 0.1);

            for rule in rules {
                if rule.indicator_type == "volatility" {
                    if !rule.indicator_names.is_empty()
                        && !rule.indicator_names.contains(&primary_indicator.name)
                    {
                        continue;
                    }
                    if let Some(constraint) = &rule.price_field_constraint {
                        if let ParameterConstraint::PercentageFromPrice {
                            min_percent,
                            max_percent,
                            step,
                        } = &constraint.parameter_constraint
                        {
                            percentage_range = (*min_percent, *max_percent, *step);
                            break;
                        }
                    }
                }
            }

            let steps = ((percentage_range.1 - percentage_range.0) / percentage_range.2) as usize;
            let step_index = self.rng.gen_range(0..=steps);
            percentage_range.0 + (step_index as f64 * percentage_range.2)
        } else if primary_indicator.name == "RSI" {
            if *operator == ConditionOperator::Above {
                self.rng.gen_range(70.0..=90.0)
            } else {
                self.rng.gen_range(10.0..=30.0)
            }
        } else if primary_indicator.name == "Stochastic" {
            if *operator == ConditionOperator::Above {
                self.rng.gen_range(80.0..=95.0)
            } else {
                self.rng.gen_range(5.0..=20.0)
            }
        } else {
            self.rng.gen_range(0.0..=100.0)
        };

        let id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            self.rng.gen::<u32>()
        );
        let name = if primary_indicator.indicator_type == "volatility" {
            format!(
                "{} {:?} Close * {:.2}%",
                primary_indicator.name, operator, constant_value
            )
        } else {
            format!(
                "{} {:?} {:.1}",
                primary_indicator.name, operator, constant_value
            )
        };

        Some((id, name, Some(constant_value), None, Vec::new()))
    }

    fn generate_trend_condition(
        &mut self,
        primary_indicator: &IndicatorInfo,
        operator: &ConditionOperator,
        is_entry: bool,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let trend_range = ConditionParameterPresets::trend_period();
        let period = self.rng.gen_range(trend_range.min..=trend_range.max);
        let trend_name = match operator {
            ConditionOperator::RisingTrend => "RisingTrend",
            ConditionOperator::FallingTrend => "FallingTrend",
            _ => "RisingTrend",
        };
        let id = format!(
            "{}_{}::{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            trend_name.to_lowercase(),
            self.rng.gen::<u32>()
        );
        let name = format!(
            "{} {} (period: {:.0})",
            primary_indicator.name, trend_name, period
        );

        Some((id, name, None, None, Vec::new()))
    }

    fn generate_indicator_indicator_condition(
        &mut self,
        primary_indicator: &IndicatorInfo,
        all_indicators: &[&IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        base_indicators: &[IndicatorInfo],
        operator: &ConditionOperator,
        probabilities: &ConditionProbabilities,
        is_entry: bool,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let available_secondary: Vec<&IndicatorInfo> = all_indicators
            .iter()
            .filter(|ind| ind.alias != primary_indicator.alias)
            .filter(|ind| {
                Self::can_compare_indicators(
                    primary_indicator,
                    *ind,
                    nested_indicators,
                    base_indicators,
                )
            })
            .copied()
            .collect();

        if let Some(secondary) = available_secondary.choose(&mut *self.rng) {
            let id = format!(
                "{}_{}::{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                secondary.alias,
                self.rng.gen::<u32>()
            );
            let name = format!(
                "{} {:?} {}",
                primary_indicator.name, operator, secondary.name
            );
            Some((id, name, None, None, Vec::new()))
        } else {
            self.generate_indicator_price_condition(
                primary_indicator,
                operator,
                probabilities,
                is_entry,
            )
        }
    }

    fn generate_indicator_price_condition(
        &mut self,
        primary_indicator: &IndicatorInfo,
        operator: &ConditionOperator,
        probabilities: &ConditionProbabilities,
        is_entry: bool,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let price_field_str = self
            .config
            .condition_config
            .price_fields
            .choose(&mut *self.rng)
            .cloned()
            .unwrap_or_else(|| "Close".to_string());

        let supports_percentage = matches!(
            operator,
            ConditionOperator::Above | ConditionOperator::Below
        );
        let (opt_params, percent_val) =
            if supports_percentage && self.should_add(probabilities.use_percent_condition) {
                let percent = self.rng.gen_range(0.1..=5.0);
                (
                    vec![crate::discovery::ConditionParamInfo {
                        name: "percentage".to_string(),
                        optimizable: true,
                        global_param_name: None,
                    }],
                    Some(percent),
                )
            } else {
                (Vec::new(), None)
            };

        let id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            self.rng.gen::<u32>()
        );
        let name = if let Some(percent) = percent_val {
            format!(
                "{} {:?} {} на {:.2}%",
                primary_indicator.name, operator, "target", percent
            )
        } else {
            format!("{} {:?} {}", primary_indicator.name, operator, "target")
        };

        Some((id, name, percent_val, Some(price_field_str), opt_params))
    }

    pub fn build_condition_simple(
        &mut self,
        indicator: &IndicatorInfo,
        is_entry: bool,
    ) -> Option<ConditionInfo> {
        self.build_condition_simple_with_timeframe(indicator, is_entry, None)
    }

    pub fn build_condition_simple_with_timeframe(
        &mut self,
        indicator: &IndicatorInfo,
        is_entry: bool,
        timeframe: Option<TimeFrame>,
    ) -> Option<ConditionInfo> {
        let operator = if indicator.indicator_type == "volatility" {
            if self.rng.gen_bool(0.5) {
                ConditionOperator::GreaterPercent
            } else {
                ConditionOperator::LowerPercent
            }
        } else if self.rng.gen_bool(0.5) {
            ConditionOperator::Above
        } else {
            ConditionOperator::Below
        };

        let condition_id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            indicator.alias,
            self.rng.gen::<u32>()
        );

        let (condition_type, condition_name, constant_value, price_field, optimization_params) =
            if indicator.indicator_type == "oscillator" {
                let const_val = if indicator.name == "RSI" {
                    if operator == ConditionOperator::Above {
                        self.rng.gen_range(70.0..=90.0)
                    } else {
                        self.rng.gen_range(10.0..=30.0)
                    }
                } else if indicator.name == "Stochastic" {
                    if operator == ConditionOperator::Above {
                        self.rng.gen_range(80.0..=95.0)
                    } else {
                        self.rng.gen_range(5.0..=20.0)
                    }
                } else {
                    self.rng.gen_range(0.0..=100.0)
                };
                (
                    "indicator_constant".to_string(),
                    format!("{} {:?} {:.1}", indicator.name, operator, const_val),
                    Some(const_val),
                    None,
                    Vec::new(),
                )
            } else if indicator.indicator_type == "volatility" {
                let rules = &self.config.rules.indicator_parameter_rules;
                let mut percentage_range = (0.2, 10.0, 0.1);

                for rule in rules {
                    if rule.indicator_type == "volatility" {
                        if !rule.indicator_names.is_empty()
                            && !rule.indicator_names.contains(&indicator.name)
                        {
                            continue;
                        }
                        if let Some(constraint) = &rule.price_field_constraint {
                            if let ParameterConstraint::PercentageFromPrice {
                                min_percent,
                                max_percent,
                                step,
                            } = &constraint.parameter_constraint
                            {
                                percentage_range = (*min_percent, *max_percent, *step);
                                break;
                            }
                        }
                    }
                }

                let steps =
                    ((percentage_range.1 - percentage_range.0) / percentage_range.2) as usize;
                let step_index = self.rng.gen_range(0..=steps);
                let const_val = percentage_range.0 + (step_index as f64 * percentage_range.2);

                (
                    "indicator_constant".to_string(),
                    format!(
                        "{} {:?} Close * {:.2}%",
                        indicator.name, operator, const_val
                    ),
                    Some(const_val),
                    None,
                    vec![crate::discovery::ConditionParamInfo {
                        name: "percentage".to_string(),
                        optimizable: true,
                        global_param_name: None,
                    }],
                )
            } else {
                let price_field = if self.config.condition_config.price_fields.len() == 1 {
                    self.config.condition_config.price_fields[0].clone()
                } else {
                    self.config
                        .condition_config
                        .price_fields
                        .choose(&mut *self.rng)
                        .cloned()
                        .unwrap_or_else(|| "Close".to_string())
                };

                let probabilities = &self.config.probabilities.conditions;
                let (optimization_params, constant_value) =
                    if self.should_add(probabilities.use_percent_condition) {
                        let percent_value = self.rng.gen_range(0.1..=5.0);
                        (
                            vec![crate::discovery::ConditionParamInfo {
                                name: "percentage".to_string(),
                                optimizable: true,
                                global_param_name: None,
                            }],
                            Some(percent_value),
                        )
                    } else {
                        (Vec::new(), None)
                    };

                (
                    "indicator_price".to_string(),
                    if let Some(percent) = constant_value {
                        format!(
                            "{} {:?} {} на {:.2}%",
                            indicator.name, operator, "target", percent
                        )
                    } else {
                        format!("{} {:?} {}", indicator.name, operator, "target")
                    },
                    constant_value,
                    Some(price_field),
                    optimization_params,
                )
            };

        let (primary_alias, secondary_alias) = if condition_type == "indicator_indicator" {
            if let Some(separator_pos) = condition_id.find("::") {
                let prefix_len = if condition_id.starts_with("entry_") {
                    6
                } else if condition_id.starts_with("exit_") {
                    5
                } else {
                    0
                };
                let primary = &condition_id[prefix_len..separator_pos];
                let after_separator = &condition_id[separator_pos + 2..];
                if let Some(last_underscore) = after_separator.rfind('_') {
                    let secondary = &after_separator[..last_underscore];
                    (primary.to_string(), Some(secondary.to_string()))
                } else {
                    (indicator.alias.clone(), None)
                }
            } else {
                (indicator.alias.clone(), None)
            }
        } else {
            (indicator.alias.clone(), None)
        };

        Some(ConditionInfo {
            id: condition_id,
            name: condition_name,
            operator,
            condition_type,
            optimization_params,
            constant_value,
            primary_indicator_alias: primary_alias,
            secondary_indicator_alias: secondary_alias,
            primary_timeframe: timeframe,
            secondary_timeframe: None,
            price_field,
        })
    }

    pub fn weighted_condition_type_choice(
        &mut self,
        probabilities: &ConditionProbabilities,
    ) -> &'static str {
        let w_trend = probabilities.use_trend_condition;
        let w_ind_ind = probabilities.use_indicator_indicator_condition;
        let w_ind_price = 1.0 - w_trend - w_ind_ind;

        let roll = self.rng.gen::<f64>();
        if roll < w_trend {
            "trend_condition"
        } else if roll < w_trend + w_ind_ind {
            "indicator_indicator"
        } else if roll < w_trend + w_ind_ind + w_ind_price {
            "indicator_price"
        } else {
            "indicator_price"
        }
    }

    pub fn weighted_choice_for_oscillator_based(
        &mut self,
        probabilities: &ConditionProbabilities,
    ) -> &'static str {
        let w_trend = probabilities.use_trend_condition;
        let w_ind_ind = probabilities.use_indicator_indicator_condition;

        let roll = self.rng.gen::<f64>();
        if roll < w_trend {
            "trend_condition"
        } else if roll < w_trend + w_ind_ind {
            "indicator_indicator"
        } else {
            "indicator_indicator"
        }
    }

    pub fn can_compare_indicators(
        primary: &IndicatorInfo,
        secondary: &IndicatorInfo,
        nested_indicators: &[NestedIndicator],
        all_indicators: &[IndicatorInfo],
    ) -> bool {
        if primary.indicator_type == "oscillator" && secondary.indicator_type == "oscillator" {
            return false;
        }

        let is_built_on_oscillator = |indicator: &IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    input_indicator.indicator_type == "oscillator"
                } else {
                    false
                }
            } else {
                false
            }
        };

        let is_built_on_non_oscillator = |indicator: &IndicatorInfo| -> bool {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    input_indicator.indicator_type != "oscillator"
                } else {
                    false
                }
            } else {
                false
            }
        };

        let primary_built_on_oscillator = is_built_on_oscillator(primary);
        let secondary_built_on_oscillator = is_built_on_oscillator(secondary);
        let primary_built_on_non_oscillator = is_built_on_non_oscillator(primary);
        let secondary_built_on_non_oscillator = is_built_on_non_oscillator(secondary);

        let get_source_oscillator_alias = |indicator: &IndicatorInfo| -> Option<String> {
            if let Some(nested) = nested_indicators
                .iter()
                .find(|n| n.indicator.alias == indicator.alias)
            {
                if let Some(input_indicator) = all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                {
                    if input_indicator.indicator_type == "oscillator" {
                        return Some(input_indicator.alias.clone());
                    }
                }
            }
            None
        };

        if primary.indicator_type == "oscillator" && primary_built_on_non_oscillator {
            return false;
        }
        if secondary.indicator_type == "oscillator" && secondary_built_on_non_oscillator {
            return false;
        }

        if primary_built_on_oscillator {
            if let Some(source_oscillator_alias) = get_source_oscillator_alias(primary) {
                return secondary.alias == source_oscillator_alias;
            }
        }
        if secondary_built_on_oscillator {
            if let Some(source_oscillator_alias) = get_source_oscillator_alias(secondary) {
                return primary.alias == source_oscillator_alias;
            }
        }

        if primary.indicator_type == "oscillator" {
            return secondary_built_on_oscillator;
        }

        if secondary.indicator_type == "oscillator" {
            return primary_built_on_oscillator;
        }

        true
    }

    pub fn is_duplicate_condition(
        new_condition: &ConditionInfo,
        existing_conditions: &[ConditionInfo],
    ) -> bool {
        for existing in existing_conditions {
            if new_condition.operator == existing.operator
                && new_condition.condition_type == existing.condition_type
            {
                if new_condition.primary_indicator_alias == existing.primary_indicator_alias {
                    return true;
                }
            }
        }
        false
    }


    pub fn is_comparison_operator(operator: &ConditionOperator) -> bool {
        matches!(
            operator,
            ConditionOperator::CrossesAbove
                | ConditionOperator::CrossesBelow
                | ConditionOperator::Above
                | ConditionOperator::Below
                | ConditionOperator::GreaterPercent
                | ConditionOperator::LowerPercent
        )
    }

    pub fn extract_operands(condition: &ConditionInfo) -> Option<ConditionOperands> {
        if condition.condition_type == "trend_condition" {
            return None;
        }

        let primary_alias = &condition.primary_indicator_alias;

        if condition.condition_type == "indicator_indicator" {
            if let Some(secondary_alias) = &condition.secondary_indicator_alias {
                return Some(ConditionOperands::IndicatorIndicator {
                    primary_alias: primary_alias.to_string(),
                    secondary_alias: secondary_alias.clone(),
                });
            }
        } else if condition.condition_type == "indicator_price" {
            let price_field = condition
                .price_field
                .clone()
                .unwrap_or_else(|| "Close".to_string());
            return Some(ConditionOperands::IndicatorPrice {
                indicator_alias: primary_alias.clone(),
                price_field,
            });
        } else if condition.condition_type == "indicator_constant" {
            return Some(ConditionOperands::IndicatorConstant {
                indicator_alias: primary_alias.clone(),
            });
        }

        None
    }

    pub fn has_conflicting_comparison_operator(
        new_condition: &ConditionInfo,
        existing_conditions: &[ConditionInfo],
    ) -> bool {
        if new_condition.condition_type == "trend_condition" {
            return false;
        }

        if !Self::is_comparison_operator(&new_condition.operator) {
            return false;
        }

        let new_operands = match Self::extract_operands(new_condition) {
            Some(ops) => ops,
            None => return false,
        };

        for existing in existing_conditions {
            if existing.condition_type == "trend_condition" {
                continue;
            }

            if !Self::is_comparison_operator(&existing.operator) {
                continue;
            }

            let existing_operands = match Self::extract_operands(existing) {
                Some(ops) => ops,
                None => continue,
            };

            if new_operands.same_operands(&existing_operands) {
                return true;
            }
        }

        false
    }

    pub fn create_for_candidate(
        &mut self,
        indicator: &IndicatorInfo,
        candidate: &StrategyCandidate,
        is_entry: bool,
        probabilities: &ConditionProbabilities,
    ) -> Option<ConditionInfo> {
        self.build_condition(
            &candidate.indicators,
            &candidate.nested_indicators,
            probabilities,
            is_entry,
        )
    }

    pub fn create_for_candidate_indicator(
        indicator: &IndicatorInfo,
        candidate: &StrategyCandidate,
        is_entry: bool,
        probabilities: &ConditionProbabilities,
    ) -> Option<ConditionInfo> {
        let mut rng = rand::thread_rng();

        let all_indicators: Vec<&IndicatorInfo> = candidate
            .indicators
            .iter()
            .chain(candidate.nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        let (operator, condition_type) =
            OperatorSelectorFactory::select_operator_and_condition_type(
                indicator,
                &candidate.nested_indicators,
                &all_indicators,
                probabilities,
                &mut rng,
            );

        let (condition_id, condition_name, constant_value, price_field, optimization_params) =
            Self::generate_condition_details_static(
                indicator,
                &all_indicators,
                &candidate.nested_indicators,
                &candidate.indicators,
                condition_type,
                &operator,
                probabilities,
                is_entry,
                &mut rng,
            )?;

        let (primary_alias, secondary_alias) = if condition_type == "indicator_indicator" {
            if let Some(separator_pos) = condition_id.find("::") {
                let prefix_len = if condition_id.starts_with("entry_") {
                    6
                } else if condition_id.starts_with("exit_") {
                    5
                } else {
                    0
                };
                let primary = &condition_id[prefix_len..separator_pos];
                let after_separator = &condition_id[separator_pos + 2..];
                if let Some(last_underscore) = after_separator.rfind('_') {
                    let secondary = &after_separator[..last_underscore];
                    (primary.to_string(), Some(secondary.to_string()))
                } else {
                    (indicator.alias.clone(), None)
                }
            } else {
                (indicator.alias.clone(), None)
            }
        } else {
            (indicator.alias.clone(), None)
        };

        Some(ConditionInfo {
            id: condition_id,
            name: condition_name,
            operator,
            condition_type: condition_type.to_string(),
            optimization_params,
            constant_value,
            primary_indicator_alias: primary_alias,
            secondary_indicator_alias: secondary_alias,
            primary_timeframe: None,
            secondary_timeframe: None,
            price_field,
        })
    }

    fn generate_condition_details_static(
        primary_indicator: &IndicatorInfo,
        all_indicators: &[&IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        base_indicators: &[IndicatorInfo],
        condition_type: &str,
        operator: &ConditionOperator,
        probabilities: &ConditionProbabilities,
        is_entry: bool,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        match condition_type {
            "indicator_constant" => {
                Self::generate_indicator_constant_static(primary_indicator, operator, is_entry, rng)
            }
            "trend_condition" => {
                Self::generate_trend_condition_static(primary_indicator, operator, is_entry, rng)
            }
            "indicator_indicator" => Self::generate_indicator_indicator_static(
                primary_indicator,
                all_indicators,
                nested_indicators,
                base_indicators,
                operator,
                probabilities,
                is_entry,
                rng,
            ),
            "indicator_price" | _ => Self::generate_indicator_price_static(
                primary_indicator,
                operator,
                probabilities,
                is_entry,
                rng,
            ),
        }
    }

    fn generate_indicator_constant_static(
        primary_indicator: &IndicatorInfo,
        operator: &ConditionOperator,
        is_entry: bool,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let constant_value = if primary_indicator.indicator_type == "volatility" {
            rng.gen_range(0.2..=10.0)
        } else if primary_indicator.name == "RSI" {
            if *operator == ConditionOperator::Above {
                rng.gen_range(70.0..=90.0)
            } else {
                rng.gen_range(10.0..=30.0)
            }
        } else if primary_indicator.name == "Stochastic" {
            if *operator == ConditionOperator::Above {
                rng.gen_range(80.0..=95.0)
            } else {
                rng.gen_range(5.0..=20.0)
            }
        } else {
            rng.gen_range(0.0..=100.0)
        };

        let id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            rng.gen::<u32>()
        );
        let name = if primary_indicator.indicator_type == "volatility" {
            format!(
                "{} {:?} Close * {:.2}%",
                primary_indicator.name, operator, constant_value
            )
        } else {
            format!(
                "{} {:?} {:.1}",
                primary_indicator.name, operator, constant_value
            )
        };

        let opt_params = if primary_indicator.indicator_type == "volatility" {
            vec![crate::discovery::ConditionParamInfo {
                name: "percentage".to_string(),
                optimizable: true,
                global_param_name: None,
            }]
        } else {
            vec![crate::discovery::ConditionParamInfo {
                name: "threshold".to_string(),
                optimizable: true,
                global_param_name: None,
            }]
        };

        Some((id, name, Some(constant_value), None, opt_params))
    }

    fn generate_trend_condition_static(
        primary_indicator: &IndicatorInfo,
        operator: &ConditionOperator,
        is_entry: bool,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let trend_range = ConditionParameterPresets::trend_period();
        let period = rng.gen_range(trend_range.min..=trend_range.max);
        let trend_name = match operator {
            ConditionOperator::RisingTrend => "RisingTrend",
            ConditionOperator::FallingTrend => "FallingTrend",
            _ => "RisingTrend",
        };
        let id = format!(
            "{}_{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            trend_name.to_lowercase(),
            rng.gen::<u32>()
        );
        let name = format!(
            "{} {} (period: {:.0})",
            primary_indicator.name, trend_name, period
        );
        let opt_params = vec![crate::discovery::ConditionParamInfo {
            name: "period".to_string(),
            optimizable: true,
            global_param_name: None,
        }];

        Some((id, name, None, None, opt_params))
    }

    fn generate_indicator_indicator_static(
        primary_indicator: &IndicatorInfo,
        all_indicators: &[&IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        base_indicators: &[IndicatorInfo],
        operator: &ConditionOperator,
        probabilities: &ConditionProbabilities,
        is_entry: bool,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let available_secondary: Vec<&IndicatorInfo> = all_indicators
            .iter()
            .filter(|ind| ind.alias != primary_indicator.alias)
            .filter(|ind| {
                Self::can_compare_indicators(
                    primary_indicator,
                    *ind,
                    nested_indicators,
                    base_indicators,
                )
            })
            .copied()
            .collect();

        if let Some(secondary) = available_secondary.choose(rng) {
            let id = format!(
                "{}_{}_{}_{}",
                if is_entry { "entry" } else { "exit" },
                primary_indicator.alias,
                secondary.alias,
                rng.gen::<u32>()
            );
            let name = format!(
                "{} {:?} {}",
                primary_indicator.name, operator, secondary.name
            );
            let supports_percentage = matches!(
                operator,
                ConditionOperator::Above | ConditionOperator::Below
            );
            let (opt_params, percent_val) =
                if supports_percentage && rng.gen::<f64>() < probabilities.use_percent_condition {
                    let percent = rng.gen_range(0.1..=5.0);
                    (
                        vec![crate::discovery::ConditionParamInfo {
                            name: "percentage".to_string(),
                            optimizable: true,
                            global_param_name: None,
                        }],
                        Some(percent),
                    )
                } else {
                    (Vec::new(), None)
                };
            let final_name = if let Some(percent) = percent_val {
                format!("{} на {:.2}%", name, percent)
            } else {
                name
            };
            Some((id, final_name, percent_val, None, opt_params))
        } else {
            None
        }
    }

    fn generate_indicator_price_static(
        primary_indicator: &IndicatorInfo,
        operator: &ConditionOperator,
        probabilities: &ConditionProbabilities,
        is_entry: bool,
        rng: &mut rand::rngs::ThreadRng,
    ) -> Option<(
        String,
        String,
        Option<f64>,
        Option<String>,
        Vec<crate::discovery::ConditionParamInfo>,
    )> {
        let price_field_str = "Close".to_string();

        let supports_percentage = matches!(
            operator,
            ConditionOperator::Above | ConditionOperator::Below
        );
        let (opt_params, percent_val) =
            if supports_percentage && rng.gen::<f64>() < probabilities.use_percent_condition {
                let percent = rng.gen_range(0.1..=5.0);
                (
                    vec![crate::discovery::ConditionParamInfo {
                        name: "percentage".to_string(),
                        optimizable: true,
                        global_param_name: None,
                    }],
                    Some(percent),
                )
            } else {
                (Vec::new(), None)
            };

        let id = format!(
            "{}_{}_{}",
            if is_entry { "entry" } else { "exit" },
            primary_indicator.alias,
            rng.gen::<u32>()
        );
        let name = if let Some(percent) = percent_val {
            format!(
                "{} {:?} {} на {:.2}%",
                primary_indicator.name, operator, "target", percent
            )
        } else {
            format!("{} {:?} {}", primary_indicator.name, operator, "target")
        };

        Some((id, name, percent_val, Some(price_field_str), opt_params))
    }

    fn should_add(&mut self, probability: f64) -> bool {
        self.rng.gen::<f64>() < probability
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionOperands {
    IndicatorPrice {
        indicator_alias: String,
        price_field: String,
    },
    IndicatorIndicator {
        primary_alias: String,
        secondary_alias: String,
    },
    IndicatorConstant {
        indicator_alias: String,
    },
}

impl ConditionOperands {
    pub fn same_operands(&self, other: &ConditionOperands) -> bool {
        match (self, other) {
            (
                ConditionOperands::IndicatorPrice {
                    indicator_alias: a1,
                    price_field: p1,
                },
                ConditionOperands::IndicatorPrice {
                    indicator_alias: a2,
                    price_field: p2,
                },
            ) => a1 == a2 && p1 == p2,
            (
                ConditionOperands::IndicatorIndicator {
                    primary_alias: p1,
                    secondary_alias: s1,
                },
                ConditionOperands::IndicatorIndicator {
                    primary_alias: p2,
                    secondary_alias: s2,
                },
            ) => (p1 == p2 && s1 == s2) || (p1 == s2 && s1 == p2),
            (
                ConditionOperands::IndicatorConstant {
                    indicator_alias: a1,
                },
                ConditionOperands::IndicatorConstant {
                    indicator_alias: a2,
                },
            ) => a1 == a2,
            _ => false,
        }
    }
}
