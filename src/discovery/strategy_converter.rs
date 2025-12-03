use chrono::Utc;
use rand::Rng;
use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;
use crate::discovery::config::StrategyDiscoveryConfig;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator, StopHandlerInfo};
use crate::optimization::condition_id::ConditionId;
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperandSpec,
    ConditionOperator, DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec,
    PositionDirection, RuleLogic, StopHandlerSpec, StrategyCategory, StrategyDefinition,
    StrategyMetadata, StrategyParamValue, StrategyParameterMap, StrategyParameterSpec,
    StrategyRuleSpec, StrategySignalType, TakeHandlerSpec,
};

pub struct StrategyConverter;

impl StrategyConverter {
    pub fn candidate_to_definition(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        let metadata = Self::create_metadata(candidate);
        let parameters = Self::extract_parameters(candidate);
        let defaults = Self::extract_defaults(candidate);
        let base_tf = base_timeframe.clone();

        let indicator_bindings = Self::create_indicator_bindings(candidate, base_tf.clone())?;
        let condition_bindings =
            Self::create_condition_bindings(candidate, &indicator_bindings, base_tf.clone())?;
        let (mut stop_handlers, mut take_handlers) =
            Self::create_stop_and_take_handlers(candidate, base_tf.clone())?;

        let exit_condition_bindings =
            Self::create_condition_bindings_for_exit(candidate, &indicator_bindings, base_tf)?;

        let entry_rules = Self::create_entry_rules(candidate, &condition_bindings)?;
        let exit_rules = Self::create_exit_rules(candidate, &exit_condition_bindings)?;

        // Устанавливаем target_entry_ids для stop и take handlers на ID entry rules
        let entry_rule_ids: Vec<String> = entry_rules.iter().map(|r| r.id.clone()).collect();
        for stop_handler in &mut stop_handlers {
            if stop_handler.target_entry_ids.is_empty() {
                stop_handler.target_entry_ids = entry_rule_ids.clone();
            }
        }
        for take_handler in &mut take_handlers {
            if take_handler.target_entry_ids.is_empty() {
                take_handler.target_entry_ids = entry_rule_ids.clone();
            }
        }

        let mut all_condition_bindings = condition_bindings;
        all_condition_bindings.extend(exit_condition_bindings);

        Ok(StrategyDefinition::new(
            metadata,
            parameters,
            indicator_bindings,
            vec![], // formulas
            all_condition_bindings,
            entry_rules,
            exit_rules,
            stop_handlers,
            take_handlers,
            defaults,
            BTreeMap::new(), // optimizer_hints
        ))
    }

    fn create_metadata(candidate: &StrategyCandidate) -> StrategyMetadata {
        let indicator_names: Vec<String> = candidate
            .indicators
            .iter()
            .map(|ind| ind.name.clone())
            .collect();
        let nested_names: Vec<String> = candidate
            .nested_indicators
            .iter()
            .map(|nested| nested.indicator.name.clone())
            .collect();
        let all_names = [indicator_names, nested_names].concat();
        let name = format!("Auto Strategy: {}", all_names.join(" + "));

        let condition_names: Vec<String> = candidate
            .conditions
            .iter()
            .map(|cond| cond.name.clone())
            .collect();
        let description = Some(format!(
            "Автоматически сгенерированная стратегия. Индикаторы: {}. Условия: {}.",
            all_names.join(", "),
            condition_names.join(", ")
        ));

        StrategyMetadata {
            id: format!("auto_strategy_{}", Utc::now().timestamp()),
            name,
            description,
            version: Some("1.0.0".to_string()),
            author: Some("Strategy Discovery Engine".to_string()),
            categories: vec![StrategyCategory::Custom("Auto Generated".to_string())],
            tags: vec!["auto-generated".to_string(), "discovery".to_string()],
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    fn extract_parameters(candidate: &StrategyCandidate) -> Vec<StrategyParameterSpec> {
        use crate::indicators::parameters::ParameterPresets;

        let mut params = Vec::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&indicator.alias, &param.name);
                    let range = ParameterPresets::get_range_for_parameter(
                        &indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    let (default_val, min_val, max_val, step_val) = if let Some(r) = range {
                        let default = ((r.start + r.end) / 2.0) as f64;
                        (
                            default,
                            Some(r.start as f64),
                            Some(r.end as f64),
                            Some(r.step as f64),
                        )
                    } else {
                        (50.0, Some(10.0), Some(200.0), Some(10.0))
                    };
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for {}",
                            param.name, indicator.name
                        )),
                        default_value: Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            default_val,
                        ),
                        min: min_val,
                        max: max_val,
                        step: step_val,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&nested.indicator.alias, &param.name);
                    let range = ParameterPresets::get_range_for_parameter(
                        &nested.indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    let (default_val, min_val, max_val, step_val) = if let Some(r) = range {
                        let default = ((r.start + r.end) / 2.0) as f64;
                        (
                            default,
                            Some(r.start as f64),
                            Some(r.end as f64),
                            Some(r.step as f64),
                        )
                    } else {
                        (50.0, Some(10.0), Some(200.0), Some(10.0))
                    };
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for nested {}",
                            param.name, nested.indicator.name
                        )),
                        default_value: Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            default_val,
                        ),
                        min: min_val,
                        max: max_val,
                        step: step_val,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for condition in &candidate.conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&condition.id, &param.name);
                    let (default_val, min_val, max_val, step_val) = if param.name == "period" {
                        (3.0, Some(2.0), Some(10.0), Some(1.0))
                    } else if param.name == "percentage" {
                        (2.0, Some(0.5), Some(10.0), Some(0.5))
                    } else {
                        (1.0, None, None, None)
                    };
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for entry condition {}",
                            param.name, condition.name
                        )),
                        default_value: StrategyParamValue::Number(default_val),
                        min: min_val,
                        max: max_val,
                        step: step_val,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for condition in &candidate.exit_conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&format!("exit_{}", condition.id), &param.name);
                    let (default_val, min_val, max_val, step_val) = if param.name == "period" {
                        (3.0, Some(2.0), Some(10.0), Some(1.0))
                    } else if param.name == "percentage" {
                        (2.0, Some(0.5), Some(10.0), Some(0.5))
                    } else {
                        (1.0, None, None, None)
                    };
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for exit condition {}",
                            param.name, condition.name
                        )),
                        default_value: StrategyParamValue::Number(default_val),
                        min: min_val,
                        max: max_val,
                        step: step_val,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for stop_handler in &candidate.stop_handlers {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&stop_handler.id, &param.name);
                    let range_opt = crate::risk::get_stop_optimization_range(
                        &stop_handler.handler_name,
                        &param.name,
                    );
                    let (default_val, min_val, max_val, step_val) = if let Some(range) = range_opt {
                        (
                            ((range.start + range.end) / 2.0) as f64,
                            Some(range.start as f64),
                            Some(range.end as f64),
                            Some(range.step as f64),
                        )
                    } else {
                        (50.0, Some(10.0), Some(150.0), Some(10.0))
                    };
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for stop handler {}",
                            param.name, stop_handler.name
                        )),
                        default_value: StrategyParamValue::Number(default_val),
                        min: min_val,
                        max: max_val,
                        step: step_val,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        for take_handler in &candidate.take_handlers {
            for param in &take_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&take_handler.id, &param.name);
                    let range_opt = crate::risk::get_stop_optimization_range(
                        &take_handler.handler_name,
                        &param.name,
                    );
                    let (default_val, min_val, max_val, step_val) = if let Some(range) = range_opt {
                        (
                            ((range.start + range.end) / 2.0) as f64,
                            Some(range.start as f64),
                            Some(range.end as f64),
                            Some(range.step as f64),
                        )
                    } else {
                        (10.0, Some(5.0), Some(30.0), Some(1.0))
                    };
                    params.push(StrategyParameterSpec {
                        name: param_name,
                        description: Some(format!(
                            "{} parameter for take handler {}",
                            param.name, take_handler.name
                        )),
                        default_value: StrategyParamValue::Number(default_val),
                        min: min_val,
                        max: max_val,
                        step: step_val,
                        discrete_values: None,
                        optimize: true,
                    });
                }
            }
        }

        params
    }

    fn extract_defaults(candidate: &StrategyCandidate) -> StrategyParameterMap {
        use crate::indicators::parameters::ParameterPresets;

        let mut defaults = HashMap::new();

        for indicator in &candidate.indicators {
            for param in &indicator.parameters {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&indicator.alias, &param.name);
                    let range = ParameterPresets::get_range_for_parameter(
                        &indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    let default_val = if let Some(r) = range {
                        ((r.start + r.end) / 2.0) as f64
                    } else {
                        50.0
                    };
                    defaults.insert(
                        param_name,
                        Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            default_val,
                        ),
                    );
                }
            }
        }

        for nested in &candidate.nested_indicators {
            for param in &nested.indicator.parameters {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&nested.indicator.alias, &param.name);
                    let range = ParameterPresets::get_range_for_parameter(
                        &nested.indicator.name,
                        &param.name,
                        &param.param_type,
                    );
                    let default_val = if let Some(r) = range {
                        ((r.start + r.end) / 2.0) as f64
                    } else {
                        50.0
                    };
                    defaults.insert(
                        param_name,
                        Self::param_value_to_strategy_param_from_enum(
                            &param.param_type,
                            default_val,
                        ),
                    );
                }
            }
        }

        for condition in &candidate.conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&condition.id, &param.name);
                    let default_val = if param.name == "period" {
                        3.0
                    } else if param.name == "percentage" {
                        2.0
                    } else {
                        1.0
                    };
                    defaults.insert(param_name, StrategyParamValue::Number(default_val));
                }
            }
        }

        for condition in &candidate.exit_conditions {
            for param in &condition.optimization_params {
                if param.optimizable {
                    let param_name =
                        Self::make_parameter_name(&format!("exit_{}", condition.id), &param.name);
                    let default_val = if param.name == "period" {
                        3.0
                    } else if param.name == "percentage" {
                        2.0
                    } else {
                        1.0
                    };
                    defaults.insert(param_name, StrategyParamValue::Number(default_val));
                }
            }
        }

        for stop_handler in &candidate.stop_handlers {
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&stop_handler.id, &param.name);
                    let range_opt = crate::risk::get_stop_optimization_range(
                        &stop_handler.handler_name,
                        &param.name,
                    );
                    let default_val = if let Some(range) = range_opt {
                        ((range.start + range.end) / 2.0) as f64
                    } else {
                        50.0
                    };
                    defaults.insert(param_name, StrategyParamValue::Number(default_val));
                }
            }
        }

        for take_handler in &candidate.take_handlers {
            for param in &take_handler.optimization_params {
                if param.optimizable {
                    let param_name = Self::make_parameter_name(&take_handler.id, &param.name);
                    let range_opt = crate::risk::get_stop_optimization_range(
                        &take_handler.handler_name,
                        &param.name,
                    );
                    let default_val = if let Some(range) = range_opt {
                        ((range.start + range.end) / 2.0) as f64
                    } else {
                        10.0
                    };
                    defaults.insert(param_name, StrategyParamValue::Number(default_val));
                }
            }
        }

        defaults
    }

    fn create_indicator_bindings(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<Vec<IndicatorBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();
        let mut binding_keys = std::collections::HashSet::new();

        // Собираем какие индикаторы на каких TF реально используются в условиях
        let mut all_conditions: Vec<&dyn crate::optimization::condition_id::ConditionInfoTrait> =
            Vec::new();
        for condition in &candidate.conditions {
            all_conditions.push(condition);
        }
        for condition in &candidate.exit_conditions {
            all_conditions.push(condition);
        }

        let mut required_indicator_timeframes =
            ConditionId::collect_required_timeframes(&all_conditions, &base_timeframe);

        // Индикаторы БЕЗ условий (не используются) - добавляем на базовый TF
        // чтобы они хотя бы где-то были (защита от пустых bindings)
        for indicator in &candidate.indicators {
            if !required_indicator_timeframes.contains_key(&indicator.alias) {
                required_indicator_timeframes
                    .entry(indicator.alias.clone())
                    .or_default()
                    .insert(base_timeframe.clone());
            }
        }
        for nested in &candidate.nested_indicators {
            if !required_indicator_timeframes.contains_key(&nested.indicator.alias) {
                required_indicator_timeframes
                    .entry(nested.indicator.alias.clone())
                    .or_default()
                    .insert(base_timeframe.clone());
            }
        }

        // Создаём bindings только для используемых комбинаций indicator+TF
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

        for nested in &candidate.nested_indicators {
            let params = Self::extract_indicator_params(&nested.indicator)?;

            // Для nested индикаторов определяем timeframe:
            // 1. Если явно указан в условиях - используем его
            // 2. Иначе - используем timeframe input индикатора
            let mut timeframes_to_use = std::collections::HashSet::new();

            if let Some(explicit_timeframes) =
                required_indicator_timeframes.get(&nested.indicator.alias)
            {
                // Если в условиях явно указан timeframe для nested индикатора - используем его
                timeframes_to_use = explicit_timeframes.clone();
            } else {
                // Ищем input индикатор в уже созданных bindings
                // Важно: nested индикатор должен использовать те же timeframes, что и его input индикатор
                // Например, если sma_on_ema построен по ema, то sma_on_ema должен быть на тех же timeframes, что и ema
                let input_timeframes: std::collections::HashSet<TimeFrame> = bindings
                    .iter()
                    .filter(|binding| binding.alias == nested.input_indicator_alias)
                    .map(|binding| binding.timeframe.clone())
                    .collect();

                if !input_timeframes.is_empty() {
                    // Используем все timeframes input индикатора
                    // Это правильно, потому что если ema есть на 60, 120, 240, то sma_on_ema тоже должен быть на 60, 120, 240
                    timeframes_to_use = input_timeframes;
                } else {
                    // Если input индикатор не найден в bindings, используем base_timeframe
                    timeframes_to_use.insert(base_timeframe.clone());
                }
            }

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

        Ok(bindings)
    }

    fn extract_indicator_params(
        indicator: &IndicatorInfo,
    ) -> Result<HashMap<String, f32>, StrategyConversionError> {
        use crate::indicators::parameters::ParameterPresets;
        let mut rng = rand::thread_rng();
        let mut params = HashMap::new();

        for param in &indicator.parameters {
            let range = ParameterPresets::get_range_for_parameter(
                &indicator.name,
                &param.name,
                &param.param_type,
            )
            .ok_or_else(|| StrategyConversionError::MissingParameterRange {
                indicator_name: indicator.name.clone(),
                parameter_name: param.name.clone(),
                parameter_type: format!("{:?}", param.param_type),
            })?;

            let steps = ((range.end - range.start) / range.step) as usize;
            let step_index = rng.gen_range(0..=steps);
            let value = range.start + (step_index as f32 * range.step);
            params.insert(param.name.clone(), value);
        }
        Ok(params)
    }

    fn create_condition_bindings(
        candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
        base_timeframe: TimeFrame,
    ) -> Result<Vec<ConditionBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();

        for condition in &candidate.conditions {
            let input = Self::create_condition_input(condition, candidate, indicator_bindings)?;
            let declarative = ConditionDeclarativeSpec {
                operator: condition.operator.clone(),
                operands: vec![],
                description: Some(condition.name.clone()),
            };

            let mut parameters = HashMap::new();
            for param in &condition.optimization_params {
                if param.optimizable {
                    let value = condition.constant_value.unwrap_or(0.0) as f32;
                    parameters.insert(param.name.clone(), value);
                }
            }

            bindings.push(ConditionBindingSpec {
                id: condition.id.clone(),
                name: condition.name.clone(),
                timeframe: base_timeframe.clone(),
                declarative,
                parameters,
                input,
                weight: 1.0,
                tags: vec![condition.condition_type.clone()],
                user_formula: None,
            });
        }

        Ok(bindings)
    }

    fn create_condition_input(
        condition: &ConditionInfo,
        candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        // Создаем HashMap для быстрого поиска таймфрейма по alias
        let mut alias_to_timeframes: std::collections::HashMap<
            String,
            std::collections::HashSet<TimeFrame>,
        > = std::collections::HashMap::new();
        for binding in indicator_bindings {
            alias_to_timeframes
                .entry(binding.alias.clone())
                .or_default()
                .insert(binding.timeframe.clone());
        }
        match condition.condition_type.as_str() {
            "indicator_price" => {
                let indicator_alias = &condition.primary_indicator_alias;
                let price_field = if let Some(ref pf_str) = condition.price_field {
                    Self::parse_price_field_from_string(pf_str)
                        .unwrap_or_else(|| crate::strategy::types::PriceField::Close)
                } else {
                    Self::extract_price_field_from_condition_id(&condition.id)
                        .unwrap_or_else(|| crate::strategy::types::PriceField::Close)
                };

                // Определяем таймфрейм: сначала из condition.primary_timeframe,
                // затем из indicator_bindings по alias
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(indicator_alias.clone(), tf.clone())
                } else if let Some(timeframes) = alias_to_timeframes.get(indicator_alias.as_str()) {
                    // Если для alias есть несколько таймфреймов, берем первый
                    // (обычно должен быть один)
                    if let Some(tf) = timeframes.iter().next() {
                        DataSeriesSource::indicator_with_timeframe(
                            indicator_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        DataSeriesSource::indicator(indicator_alias)
                    }
                } else {
                    DataSeriesSource::indicator(indicator_alias)
                };

                let secondary_source = if let Some(ref tf) = condition.secondary_timeframe {
                    DataSeriesSource::price_with_timeframe(price_field, tf.clone())
                } else {
                    DataSeriesSource::price(price_field)
                };

                // Проверяем, есть ли процент в optimization_params для создания DualWithPercent
                let percent_param = condition
                    .optimization_params
                    .iter()
                    .find(|p| p.name == "percent" || p.name == "percentage");

                if let Some(_percent_param) = percent_param {
                    // Используем constant_value из ConditionInfo для процента, если оно есть
                    // Иначе используем значение по умолчанию 1.0%
                    let percent_value = condition.constant_value.unwrap_or(1.0) as f32;
                    Ok(ConditionInputSpec::DualWithPercent {
                        primary: primary_source,
                        secondary: secondary_source,
                        percent: percent_value,
                    })
                } else {
                    Ok(ConditionInputSpec::Dual {
                        primary: primary_source,
                        secondary: secondary_source,
                    })
                }
            }
            "indicator_indicator" => {
                let primary_alias = &condition.primary_indicator_alias;
                let secondary_alias =
                    condition
                        .secondary_indicator_alias
                        .as_ref()
                        .ok_or_else(|| {
                            StrategyConversionError::InvalidConditionFormat {
                        condition_id: condition.id.clone(),
                        reason:
                            "Missing secondary_indicator_alias for indicator_indicator condition"
                                .to_string(),
                    }
                        })?;

                // Определяем таймфреймы: сначала из ConditionInfo, затем из indicator_bindings
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(primary_alias.clone(), tf.clone())
                } else if let Some(timeframes) = alias_to_timeframes.get(primary_alias) {
                    if let Some(tf) = timeframes.iter().next() {
                        DataSeriesSource::indicator_with_timeframe(
                            primary_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        DataSeriesSource::indicator(primary_alias.clone())
                    }
                } else {
                    DataSeriesSource::indicator(primary_alias.clone())
                };

                let secondary_source = if let Some(ref tf) = condition.secondary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(secondary_alias.clone(), tf.clone())
                } else if let Some(timeframes) = alias_to_timeframes.get(secondary_alias) {
                    if let Some(tf) = timeframes.iter().next() {
                        DataSeriesSource::indicator_with_timeframe(
                            secondary_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        DataSeriesSource::indicator(secondary_alias.clone())
                    }
                } else {
                    DataSeriesSource::indicator(secondary_alias.clone())
                };

                // Проверяем, есть ли процент в optimization_params для создания DualWithPercent
                let percent_param = condition
                    .optimization_params
                    .iter()
                    .find(|p| p.name == "percent" || p.name == "percentage");

                if let Some(_percent_param) = percent_param {
                    // Используем constant_value из ConditionInfo для процента, если оно есть
                    // Иначе используем значение по умолчанию 1.0%
                    let percent_value = condition.constant_value.unwrap_or(1.0) as f32;
                    Ok(ConditionInputSpec::DualWithPercent {
                        primary: primary_source,
                        secondary: secondary_source,
                        percent: percent_value,
                    })
                } else {
                    Ok(ConditionInputSpec::Dual {
                        primary: primary_source,
                        secondary: secondary_source,
                    })
                }
            }
            "trend_condition" => {
                let indicator_alias = &condition.primary_indicator_alias;

                // Определяем таймфрейм: сначала из ConditionInfo, затем из indicator_bindings
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(indicator_alias.clone(), tf.clone())
                } else if let Some(timeframes) = alias_to_timeframes.get(indicator_alias.as_str()) {
                    if let Some(tf) = timeframes.iter().next() {
                        DataSeriesSource::indicator_with_timeframe(
                            indicator_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        DataSeriesSource::indicator(indicator_alias)
                    }
                } else {
                    DataSeriesSource::indicator(indicator_alias)
                };

                // Для трендовых условий используется Single input
                Ok(ConditionInputSpec::Single {
                    source: primary_source,
                })
            }
            "indicator_constant" => {
                let indicator_alias = &condition.primary_indicator_alias;
                let constant_value = condition.constant_value.unwrap_or(0.0) as f32;

                // Определяем таймфрейм: сначала из ConditionInfo, затем из indicator_bindings
                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    DataSeriesSource::indicator_with_timeframe(indicator_alias.clone(), tf.clone())
                } else if let Some(timeframes) = alias_to_timeframes.get(indicator_alias.as_str()) {
                    if let Some(tf) = timeframes.iter().next() {
                        DataSeriesSource::indicator_with_timeframe(
                            indicator_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        DataSeriesSource::indicator(indicator_alias)
                    }
                } else {
                    DataSeriesSource::indicator(indicator_alias)
                };

                Ok(ConditionInputSpec::Dual {
                    primary: primary_source,
                    secondary: DataSeriesSource::custom(format!("constant_{}", constant_value)),
                })
            }
            _ => Err(StrategyConversionError::UnsupportedConditionType {
                condition_type: condition.condition_type.clone(),
            }),
        }
    }

    fn create_stop_and_take_handlers(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<(Vec<StopHandlerSpec>, Vec<TakeHandlerSpec>), StrategyConversionError> {
        let mut stop_handlers = Vec::new();
        let mut take_handlers = Vec::new();

        for stop_handler in &candidate.stop_handlers {
            let mut parameters = StrategyParameterMap::new();
            for param in &stop_handler.optimization_params {
                if param.optimizable {
                    let range_opt = crate::risk::get_stop_optimization_range(
                        &stop_handler.handler_name,
                        &param.name,
                    );
                    let default_val = if let Some(range) = range_opt {
                        ((range.start + range.end) / 2.0) as f64
                    } else {
                        50.0
                    };
                    parameters.insert(param.name.clone(), StrategyParamValue::Number(default_val));
                }
            }

            stop_handlers.push(StopHandlerSpec {
                id: stop_handler.id.clone(),
                name: stop_handler.name.clone(),
                handler_name: stop_handler.handler_name.clone(),
                timeframe: base_timeframe.clone(),
                price_field: crate::strategy::types::PriceField::Close,
                parameters,
                direction: PositionDirection::Long,
                priority: stop_handler.priority,
                tags: vec!["stop_loss".to_string()],
                target_entry_ids: vec![],
            });
        }

        for take_handler in &candidate.take_handlers {
            let mut parameters = StrategyParameterMap::new();
            for param in &take_handler.optimization_params {
                if param.optimizable {
                    let range_opt = crate::risk::get_stop_optimization_range(
                        &take_handler.handler_name,
                        &param.name,
                    );
                    let default_val = if let Some(range) = range_opt {
                        ((range.start + range.end) / 2.0) as f64
                    } else {
                        10.0
                    };
                    parameters.insert(param.name.clone(), StrategyParamValue::Number(default_val));
                }
            }

            take_handlers.push(TakeHandlerSpec {
                id: take_handler.id.clone(),
                name: take_handler.name.clone(),
                handler_name: take_handler.handler_name.clone(),
                timeframe: base_timeframe.clone(),
                price_field: crate::strategy::types::PriceField::Close,
                parameters,
                direction: PositionDirection::Long,
                priority: take_handler.priority,
                tags: vec!["take_profit".to_string()],
                target_entry_ids: vec![],
            });
        }

        Ok((stop_handlers, take_handlers))
    }

    fn create_entry_rules(
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

    fn create_condition_bindings_for_exit(
        candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
        base_timeframe: TimeFrame,
    ) -> Result<Vec<ConditionBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::new();

        for condition in &candidate.exit_conditions {
            let input = Self::create_condition_input(condition, candidate, indicator_bindings)?;
            let declarative = ConditionDeclarativeSpec {
                operator: condition.operator.clone(),
                operands: vec![],
                description: Some(condition.name.clone()),
            };

            let mut parameters = HashMap::new();
            for param in &condition.optimization_params {
                if param.optimizable {
                    let value = condition.constant_value.unwrap_or(0.0) as f32;
                    parameters.insert(param.name.clone(), value);
                }
            }

            bindings.push(ConditionBindingSpec {
                id: format!("exit_{}", condition.id),
                name: format!("Exit: {}", condition.name),
                timeframe: base_timeframe.clone(),
                declarative,
                parameters,
                input,
                weight: 1.0,
                tags: vec![condition.condition_type.clone(), "exit".to_string()],
                user_formula: None,
            });
        }

        Ok(bindings)
    }

    fn create_exit_rules(
        candidate: &StrategyCandidate,
        exit_condition_bindings: &[ConditionBindingSpec],
    ) -> Result<Vec<StrategyRuleSpec>, StrategyConversionError> {
        let mut exit_rules = Vec::new();

        // Создаем exit rule из exit conditions, если они есть
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

        // Exit rules также могут быть созданы из stop handlers
        // Но stop handlers обрабатываются отдельно через StopHandlerSpec
        // Здесь мы создаем exit rules только из условий

        Ok(exit_rules)
    }

    fn make_parameter_name(prefix: &str, param_name: &str) -> String {
        format!("{}_{}", prefix, param_name)
    }

    fn param_value_to_strategy_param_from_enum(
        param_type: &crate::indicators::types::ParameterType,
        default: f64,
    ) -> StrategyParamValue {
        match param_type {
            crate::indicators::types::ParameterType::Period => {
                StrategyParamValue::Integer(default as i64)
            }
            crate::indicators::types::ParameterType::Multiplier => {
                StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Threshold => {
                StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Coefficient => {
                StrategyParamValue::Number(default)
            }
            crate::indicators::types::ParameterType::Custom => StrategyParamValue::Number(default),
        }
    }

    fn parse_price_field_from_string(
        price_field_str: &str,
    ) -> Option<crate::strategy::types::PriceField> {
        match price_field_str {
            "Open" => Some(crate::strategy::types::PriceField::Open),
            "High" => Some(crate::strategy::types::PriceField::High),
            "Low" => Some(crate::strategy::types::PriceField::Low),
            "Close" => Some(crate::strategy::types::PriceField::Close),
            "Volume" => Some(crate::strategy::types::PriceField::Volume),
            _ => None,
        }
    }

    fn extract_price_field_from_condition_id(
        condition_id: &str,
    ) -> Option<crate::strategy::types::PriceField> {
        if condition_id.starts_with("ind_price_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 4 {
                match parts[3] {
                    "Open" => Some(crate::strategy::types::PriceField::Open),
                    "High" => Some(crate::strategy::types::PriceField::High),
                    "Low" => Some(crate::strategy::types::PriceField::Low),
                    "Close" => Some(crate::strategy::types::PriceField::Close),
                    "Volume" => Some(crate::strategy::types::PriceField::Volume),
                    _ => Some(crate::strategy::types::PriceField::Close),
                }
            } else {
                Some(crate::strategy::types::PriceField::Close)
            }
        } else {
            Some(crate::strategy::types::PriceField::Close)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StrategyConversionError {
    #[error("Invalid condition format: {condition_id} - {reason}")]
    InvalidConditionFormat {
        condition_id: String,
        reason: String,
    },
    #[error("Unsupported condition type: {condition_type}")]
    UnsupportedConditionType { condition_type: String },
    #[error("Missing parameter range for indicator {indicator_name}, parameter {parameter_name} (type: {parameter_type})")]
    MissingParameterRange {
        indicator_name: String,
        parameter_name: String,
        parameter_type: String,
    },
}
