use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;

use super::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, PriceField,
    StopHandlerSpec, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyParameterSpec, StrategyRuleSpec, StrategySignalType,
    TakeHandlerSpec, UserFormulaMetadata,
};

pub fn default_strategy_definitions() -> Vec<StrategyDefinition> {
    vec![
        sinewma_rising_trend_definition(),
        amma_rising_trend_definition(),
        vtrand_falling_trend_definition(),
    ]
}

fn sinewma_rising_trend_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let sinewma_alias = "sinewma".to_string();

    let indicator_bindings = vec![IndicatorBindingSpec {
        alias: sinewma_alias.clone(),
        timeframe: timeframe.clone(),
        source: IndicatorSourceSpec::Registry {
            name: "SINEWMA".to_string(),
            parameters: HashMap::from([("period".to_string(), 170.0)]),
        },
        tags: vec!["base".to_string()],
    }];

    let sinewma_rising_input = ConditionInputSpec::Single {
        source: DataSeriesSource::indicator(sinewma_alias.clone()),
    };

    let condition_bindings = vec![ConditionBindingSpec {
        id: "entry_sinewma::risingtrend_2442449988".to_string(),
        name: "SINEWMA RisingTrend (period: 4)".to_string(),
        timeframe: timeframe.clone(),
        declarative: ConditionDeclarativeSpec::from_input(
            ConditionOperator::RisingTrend,
            &sinewma_rising_input,
        ),
        parameters: HashMap::from([("period".to_string(), 3.0)]),
        input: sinewma_rising_input,
        weight: 1.0,
        tags: vec!["trend_condition".to_string()],
        user_formula: None,
    }];

    let entry_rules = vec![StrategyRuleSpec {
        id: "entry_rule_1".to_string(),
        name: "Entry Rule".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["entry_sinewma::risingtrend_2442449988".to_string()],
        signal: StrategySignalType::Entry,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["auto-generated".to_string()],
        position_group: None,
        target_entry_ids: Vec::new(),
    }];

    let exit_rules = vec![];

    let mut hilo_trail_params = StrategyParameterMap::new();
    hilo_trail_params.insert("period".to_string(), StrategyParamValue::Number(80.0));

    let stop_handlers = vec![StopHandlerSpec {
        id: "stop_2525818392".to_string(),
        name: "HILOTrailingStop".to_string(),
        handler_name: "HILOTrailingStop".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: hilo_trail_params,
        direction: PositionDirection::Long,
        priority: 100,
        tags: vec!["stop_loss".to_string()],
        target_entry_ids: vec!["entry_rule_1".to_string()],
    }];

    let take_handlers = vec![];

    let mut parameters = Vec::new();
    parameters.push(StrategyParameterSpec::new_numeric(
        "sinewma_period".to_string(),
        Some("period parameter for SINEWMA".to_string()),
        StrategyParamValue::Integer(105),
        Some(10.0),
        Some(200.0),
        Some(10.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "entry_sinewma::risingtrend_2442449988_period".to_string(),
        Some("period parameter for entry condition SINEWMA RisingTrend (period: 4)".to_string()),
        StrategyParamValue::Number(4.0),
        Some(2.0),
        Some(4.0),
        Some(1.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "stop_2525818392_period".to_string(),
        Some("period parameter for stop handler HILOTrailingStop".to_string()),
        StrategyParamValue::Number(80.0),
        Some(10.0),
        Some(150.0),
        Some(10.0),
        true,
        true,
    ));

    let mut defaults = StrategyParameterMap::new();
    // defaults.insert(
    //     "sinewma_period".to_string(),
    //     StrategyParamValue::Integer(105),
    // );
    // defaults.insert(
    //     "entry_sinewma::risingtrend_2442449988_period".to_string(),
    //     StrategyParamValue::Number(4.0),
    // );
    // defaults.insert(
    //     "stop_2525818392_period".to_string(),
    //     StrategyParamValue::Number(80.0),
    // );

    StrategyDefinition::new(
        StrategyMetadata {
            id: "auto_strategy_1764959359".to_string(),
            name: "Auto Strategy: SINEWMA".to_string(),
            description: Some(
                "Автоматически сгенерированная стратегия. Индикаторы: SINEWMA. Условия: SINEWMA RisingTrend (period: 4).".to_string(),
            ),
            version: Some("1.0.0".to_string()),
            author: Some("Strategy Discovery Engine".to_string()),
            categories: vec![super::types::StrategyCategory::Custom("Auto Generated".to_string())],
            tags: vec!["auto-generated".to_string(), "discovery".to_string()],
            created_at: None,
            updated_at: None,
        },
        parameters,
        indicator_bindings,
        Vec::new(),
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        take_handlers,
        defaults,
        BTreeMap::new(),
    )
}

fn amma_rising_trend_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let amma_alias = "amma".to_string();

    let indicator_bindings = vec![IndicatorBindingSpec {
        alias: amma_alias.clone(),
        timeframe: timeframe.clone(),
        source: IndicatorSourceSpec::Registry {
            name: "AMMA".to_string(),
            parameters: HashMap::from([("period".to_string(), 40.0)]),
        },
        tags: vec!["base".to_string()],
    }];

    let amma_rising_input = ConditionInputSpec::Single {
        source: DataSeriesSource::indicator(amma_alias.clone()),
    };

    let condition_bindings = vec![ConditionBindingSpec {
        id: "entry_amma::risingtrend_3325280133".to_string(),
        name: "AMMA RisingTrend (period: 2)".to_string(),
        timeframe: timeframe.clone(),
        declarative: ConditionDeclarativeSpec::from_input(
            ConditionOperator::RisingTrend,
            &amma_rising_input,
        ),
        parameters: HashMap::from([("period".to_string(), 3.0)]),
        input: amma_rising_input,
        weight: 1.0,
        tags: vec!["trend_condition".to_string()],
        user_formula: None,
    }];

    let entry_rules = vec![StrategyRuleSpec {
        id: "entry_rule_1".to_string(),
        name: "Entry Rule".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["entry_amma::risingtrend_3325280133".to_string()],
        signal: StrategySignalType::Entry,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["auto-generated".to_string()],
        position_group: None,
        target_entry_ids: Vec::new(),
    }];

    let exit_rules = vec![];

    let mut atr_trail_params = StrategyParameterMap::new();
    atr_trail_params.insert("period".to_string(), StrategyParamValue::Number(90.0));
    atr_trail_params.insert("coeff_atr".to_string(), StrategyParamValue::Number(7.0));

    let stop_handlers = vec![StopHandlerSpec {
        id: "stop_2503316804".to_string(),
        name: "ATRTrailStop".to_string(),
        handler_name: "ATRTrailStop".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: atr_trail_params,
        direction: PositionDirection::Long,
        priority: 100,
        tags: vec!["stop_loss".to_string()],
        target_entry_ids: vec!["entry_rule_1".to_string()],
    }];

    let take_handlers = vec![];

    let mut parameters = Vec::new();
    parameters.push(StrategyParameterSpec::new_numeric(
        "amma_period".to_string(),
        Some("period parameter for AMMA".to_string()),
        StrategyParamValue::Integer(105),
        Some(10.0),
        Some(200.0),
        Some(10.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "entry_amma::risingtrend_3325280133_period".to_string(),
        Some("period parameter for entry condition AMMA RisingTrend (period: 2)".to_string()),
        StrategyParamValue::Number(3.0),
        Some(2.0),
        Some(4.0),
        Some(1.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "stop_2503316804_coeff_atr".to_string(),
        Some("coeff_atr parameter for stop handler ATRTrailStop".to_string()),
        StrategyParamValue::Number(5.0),
        Some(2.0),
        Some(8.0),
        Some(0.5),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "stop_2503316804_period".to_string(),
        Some("period parameter for stop handler ATRTrailStop".to_string()),
        StrategyParamValue::Number(14.0),
        Some(10.0),
        Some(150.0),
        Some(10.0),
        true,
        true,
    ));

    let mut defaults = StrategyParameterMap::new();

    StrategyDefinition::new(
        StrategyMetadata {
            id: "auto_strategy_1764968715".to_string(),
            name: "Auto Strategy: AMMA".to_string(),
            description: Some(
                "Автоматически сгенерированная стратегия. Индикаторы: AMMA. Условия: AMMA RisingTrend (period: 2).".to_string(),
            ),
            version: Some("1.0.0".to_string()),
            author: Some("Strategy Discovery Engine".to_string()),
            categories: vec![super::types::StrategyCategory::Custom("Auto Generated".to_string())],
            tags: vec!["auto-generated".to_string(), "discovery".to_string()],
            created_at: None,
            updated_at: None,
        },
        parameters,
        indicator_bindings,
        Vec::new(),
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        take_handlers,
        defaults,
        BTreeMap::new(),
    )
}

fn vtrand_falling_trend_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let vtrand_alias = "vtrand".to_string();

    let indicator_bindings = vec![IndicatorBindingSpec {
        alias: vtrand_alias.clone(),
        timeframe: timeframe.clone(),
        source: IndicatorSourceSpec::Registry {
            name: "VTRAND".to_string(),
            parameters: HashMap::from([("period".to_string(), 50.0)]),
        },
        tags: vec!["base".to_string()],
    }];

    let vtrand_falling_input = ConditionInputSpec::Single {
        source: DataSeriesSource::indicator_with_timeframe(vtrand_alias.clone(), timeframe.clone()),
    };

    let condition_bindings = vec![ConditionBindingSpec {
        id: "entry_vtrand::fallingtrend_608425930".to_string(),
        name: "VTRAND FallingTrend (period: 3)".to_string(),
        timeframe: timeframe.clone(),
        declarative: ConditionDeclarativeSpec::from_input(
            ConditionOperator::FallingTrend,
            &vtrand_falling_input,
        ),
        parameters: HashMap::from([("period".to_string(), 3.0)]),
        input: vtrand_falling_input,
        weight: 1.0,
        tags: vec!["trend_condition".to_string()],
        user_formula: None,
    }];

    let entry_rules = vec![StrategyRuleSpec {
        id: "entry_rule_1".to_string(),
        name: "Entry Rule".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["entry_vtrand::fallingtrend_608425930".to_string()],
        signal: StrategySignalType::Entry,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["auto-generated".to_string()],
        position_group: None,
        target_entry_ids: Vec::new(),
    }];

    let exit_rules = vec![];

    let mut atr_trail_params = StrategyParameterMap::new();
    atr_trail_params.insert("period".to_string(), StrategyParamValue::Number(60.0));
    atr_trail_params.insert("coeff_atr".to_string(), StrategyParamValue::Number(6.0));

    let stop_handlers = vec![StopHandlerSpec {
        id: "stop_734731176".to_string(),
        name: "ATRTrailStop".to_string(),
        handler_name: "ATRTrailStop".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: atr_trail_params,
        direction: PositionDirection::Long,
        priority: 100,
        tags: vec!["stop_loss".to_string()],
        target_entry_ids: vec!["entry_rule_1".to_string()],
    }];

    let mut take_profit_params = StrategyParameterMap::new();
    take_profit_params.insert("percentage".to_string(), StrategyParamValue::Number(30.0));

    let take_handlers = vec![TakeHandlerSpec {
        id: "take_2516110301".to_string(),
        name: "TakeProfitPct".to_string(),
        handler_name: "TakeProfitPct".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: take_profit_params,
        direction: PositionDirection::Long,
        priority: 100,
        tags: vec!["take_profit".to_string()],
        target_entry_ids: vec!["entry_rule_1".to_string()],
    }];

    let mut parameters = Vec::new();
    parameters.push(StrategyParameterSpec::new_numeric(
        "vtrand_period".to_string(),
        Some("period parameter for VTRAND".to_string()),
        StrategyParamValue::Integer(105),
        Some(10.0),
        Some(200.0),
        Some(10.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "entry_vtrand::fallingtrend_608425930_period".to_string(),
        Some("period parameter for entry condition VTRAND FallingTrend (period: 3)".to_string()),
        StrategyParamValue::Number(3.0),
        Some(2.0),
        Some(4.0),
        Some(1.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "stop_734731176_period".to_string(),
        Some("period parameter for stop handler ATRTrailStop".to_string()),
        StrategyParamValue::Number(14.0),
        Some(10.0),
        Some(150.0),
        Some(10.0),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "stop_734731176_coeff_atr".to_string(),
        Some("coeff_atr parameter for stop handler ATRTrailStop".to_string()),
        StrategyParamValue::Number(5.0),
        Some(2.0),
        Some(8.0),
        Some(0.5),
        true,
        true,
    ));
    parameters.push(StrategyParameterSpec::new_numeric(
        "take_2516110301_percentage".to_string(),
        Some("percentage parameter for take handler TakeProfitPct".to_string()),
        StrategyParamValue::Number(9.0),
        Some(4.0),
        Some(20.0),
        Some(1.0),
        true,
        true,
    ));

    let mut defaults = StrategyParameterMap::new();

    StrategyDefinition::new(
        StrategyMetadata {
            id: "auto_strategy_1764972624".to_string(),
            name: "Auto Strategy: VTRAND".to_string(),
            description: Some(
                "Автоматически сгенерированная стратегия. Индикаторы: VTRAND. Условия: VTRAND FallingTrend (period: 3).".to_string(),
            ),
            version: Some("1.0.0".to_string()),
            author: Some("Strategy Discovery Engine".to_string()),
            categories: vec![super::types::StrategyCategory::Custom("Auto Generated".to_string())],
            tags: vec!["auto-generated".to_string(), "discovery".to_string()],
            created_at: None,
            updated_at: None,
        },
        parameters,
        indicator_bindings,
        Vec::new(),
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        take_handlers,
        defaults,
        BTreeMap::new(),
    )
}
