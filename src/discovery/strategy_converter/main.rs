use chrono::Utc;
use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;
use crate::discovery::config::StrategyDiscoveryConfig;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo};
use crate::optimization::condition_id::ConditionId;

use super::condition_builder::ConditionBuilder;
use super::parameter_extractor::ParameterExtractor;
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, RuleLogic,
    StopHandlerSpec, StrategyCategory, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyParameterSpec, StrategyRuleSpec, StrategySignalType,
    TakeHandlerSpec,
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
        ParameterExtractor::extract_all(candidate)
    }

    fn extract_defaults(_candidate: &StrategyCandidate) -> StrategyParameterMap {
        HashMap::new()
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
        _indicator: &IndicatorInfo,
    ) -> Result<HashMap<String, f32>, StrategyConversionError> {
        Ok(HashMap::new())
    }

    fn create_condition_bindings(
        candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
        base_timeframe: TimeFrame,
    ) -> Result<Vec<ConditionBindingSpec>, StrategyConversionError> {
        ConditionBuilder::create_bindings(
            &candidate.conditions,
            candidate,
            indicator_bindings,
            base_timeframe,
            "entry",
        )
    }

    fn create_stop_and_take_handlers(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<(Vec<StopHandlerSpec>, Vec<TakeHandlerSpec>), StrategyConversionError> {
        let mut stop_handlers = Vec::new();
        let mut take_handlers = Vec::new();

        for stop_handler in &candidate.stop_handlers {
            // Нормализуем имя хендлера (убираем индикатор из имени для получения дефолтных параметров)
            let (normalized_name_for_defaults, _) =
                crate::risk::extract_indicator_from_handler_name(&stop_handler.handler_name);
            // Получаем дефолтные параметры для стоп-хендлера (используем нормализованное имя)
            let mut parameters = Self::get_default_stop_params(&normalized_name_for_defaults);

            // Обрабатываем индикатор: извлекаем из handler_name и добавляем в parameters
            let normalized_handler_name = crate::risk::process_stop_handler_indicator(
                &stop_handler.handler_name,
                &mut parameters,
            );

            stop_handlers.push(StopHandlerSpec {
                id: stop_handler.id.clone(),
                name: stop_handler.name.clone(),
                handler_name: normalized_handler_name,
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
            let parameters = StrategyParameterMap::new();

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
        ConditionBuilder::create_bindings(
            &candidate.exit_conditions,
            candidate,
            indicator_bindings,
            base_timeframe,
            "exit",
        )
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

    fn get_default_stop_params(handler_name: &str) -> HashMap<String, StrategyParamValue> {
        ParameterExtractor::get_default_stop_params(handler_name)
    }

    fn get_default_take_params(handler_name: &str) -> HashMap<String, StrategyParamValue> {
        ParameterExtractor::get_default_take_params(handler_name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::parameters::ConditionParameterPresets;
    use crate::discovery::types::{ConditionInfo, ConditionParamInfo};
    use crate::strategy::types::ConditionOperator;

    #[test]
    fn test_condition_parameter_ranges_match_presets() {
        let trend_range = ConditionParameterPresets::trend_period();
        assert_eq!(trend_range.min, 2.0, "trend_period min должен быть 2.0");
        assert_eq!(trend_range.max, 4.0, "trend_period max должен быть 4.0");
        assert_eq!(trend_range.step, 1.0, "trend_period step должен быть 1.0");

        let percentage_range = ConditionParameterPresets::percentage();
        assert_eq!(percentage_range.min, 0.5, "percentage min должен быть 0.5");
        assert_eq!(
            percentage_range.max, 10.0,
            "percentage max должен быть 10.0"
        );
        assert_eq!(
            percentage_range.step, 0.5,
            "percentage step должен быть 0.5"
        );
    }

    #[test]
    fn test_extract_parameters_uses_correct_ranges() {
        let candidate = StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![ConditionInfo {
                id: "test_condition_1".to_string(),
                name: "Test RisingTrend".to_string(),
                operator: ConditionOperator::RisingTrend,
                primary_indicator_alias: "test_sma".to_string(),
                secondary_indicator_alias: None,
                condition_type: "trend_condition".to_string(),
                primary_timeframe: Some(TimeFrame::Minutes(60)),
                secondary_timeframe: None,
                price_field: None,
                constant_value: Some(3.0),
                optimization_params: vec![ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }],
            }],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let params = StrategyConverter::extract_parameters(&candidate);

        let period_param = params
            .iter()
            .find(|p| p.name.contains("period"))
            .expect("Должен быть параметр period");

        assert_eq!(
            period_param.min,
            Some(2.0),
            "min для period должен быть 2.0"
        );
        assert_eq!(
            period_param.max,
            Some(4.0),
            "max для period должен быть 4.0 (не 10.0!)"
        );
        assert_eq!(
            period_param.step,
            Some(1.0),
            "step для period должен быть 1.0"
        );
    }

    #[test]
    fn test_extract_defaults_returns_empty() {
        let candidate = StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![ConditionInfo {
                id: "test_condition_1".to_string(),
                name: "Test RisingTrend".to_string(),
                operator: ConditionOperator::RisingTrend,
                primary_indicator_alias: "test_sma".to_string(),
                secondary_indicator_alias: None,
                condition_type: "trend_condition".to_string(),
                primary_timeframe: Some(TimeFrame::Minutes(60)),
                secondary_timeframe: None,
                price_field: None,
                constant_value: Some(3.0),
                optimization_params: vec![ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }],
            }],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let defaults = StrategyConverter::extract_defaults(&candidate);

        assert!(
            defaults.is_empty(),
            "extract_defaults должен возвращать пустой HashMap, так как все параметры должны передаваться явно"
        );
    }
}
