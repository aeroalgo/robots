use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;

use super::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, PriceField,
    StopHandlerSpec, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyRuleSpec, StrategySignalType, TimeframeRequirement,
    UserFormulaMetadata,
};

pub fn default_strategy_definitions() -> Vec<StrategyDefinition> {
    vec![sma_crossover_definition()]
}

fn sma_crossover_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let fast_alias = "fast_sma".to_string();
    let slow_alias = "slow_sma".to_string();

    let mut indicator_bindings = vec![
        IndicatorBindingSpec {
            alias: fast_alias.clone(),
            timeframe: timeframe.clone(),
            source: IndicatorSourceSpec::Registry {
                name: "SMA".to_string(),
                parameters: HashMap::from([("period".to_string(), 10.0)]),
            },
            tags: vec!["trend".to_string(), "entry".to_string()],
        },
        IndicatorBindingSpec {
            alias: slow_alias.clone(),
            timeframe: timeframe.clone(),
            source: IndicatorSourceSpec::Registry {
                name: "SMA".to_string(),
                parameters: HashMap::from([("period".to_string(), 30.0)]),
            },
            tags: vec!["trend".to_string(), "filter".to_string()],
        },
    ];

    let spread_formula = UserFormulaMetadata {
        id: "SMA_SPREAD".to_string(),
        name: "SMA Spread".to_string(),
        expression: format!("{} - {}", fast_alias, slow_alias),
        description: Some("Разница между быстрым и медленным SMA".to_string()),
        tags: vec!["formula".to_string(), "derived".to_string()],
        inputs: vec![fast_alias.clone(), slow_alias.clone()],
    };

    indicator_bindings.push(spread_formula.as_indicator_binding("sma_spread", timeframe.clone()));

    let formulas = vec![spread_formula];

    let dual_input = ConditionInputSpec::Dual {
        primary: DataSeriesSource::Indicator {
            alias: fast_alias.clone(),
        },
        secondary: DataSeriesSource::Indicator {
            alias: slow_alias.clone(),
        },
    };

    let condition_bindings = vec![
        ConditionBindingSpec {
            id: "fast_cross_above".to_string(),
            name: "Fast SMA crosses above slow".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::CrossesAbove,
                &dual_input,
            ),
            parameters: HashMap::new(),
            input: dual_input.clone(),
            weight: 1.0,
            tags: vec!["entry".to_string()],
            user_formula: None,
        },
        ConditionBindingSpec {
            id: "fast_cross_below".to_string(),
            name: "Fast SMA crosses below slow".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::CrossesBelow,
                &dual_input,
            ),
            parameters: HashMap::new(),
            input: dual_input,
            weight: 1.0,
            tags: vec!["exit".to_string()],
            user_formula: None,
        },
    ];

    let entry_rules = vec![StrategyRuleSpec {
        id: "enter_long".to_string(),
        name: "Enter long".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["fast_cross_above".to_string()],
        signal: StrategySignalType::Entry,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["core".to_string()],
        position_group: Some("enter_long".to_string()),
        target_entry_ids: Vec::new(),
    }];

    let exit_rules = vec![StrategyRuleSpec {
        id: "exit_long".to_string(),
        name: "Exit long".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["fast_cross_below".to_string()],
        signal: StrategySignalType::Exit,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["core".to_string()],
        position_group: None,
        target_entry_ids: vec!["enter_long".to_string()],
    }];

    let mut stop_loss_params = StrategyParameterMap::new();
    stop_loss_params.insert("percentage".to_string(), StrategyParamValue::Number(0.5));
    let mut take_profit_params = StrategyParameterMap::new();
    take_profit_params.insert("percentage".to_string(), StrategyParamValue::Number(0.8));
    let stop_handlers = vec![
        StopHandlerSpec {
            id: "stop_loss_pct".to_string(),
            name: "Stop Loss Pct".to_string(),
            handler_name: "StopLossPct".to_string(),
            timeframe: timeframe.clone(),
            price_field: PriceField::Close,
            parameters: stop_loss_params,
            direction: PositionDirection::Long,
            priority: 10,
            tags: vec!["stop".to_string(), "risk".to_string()],
            target_entry_ids: vec!["enter_long".to_string()],
        },
        StopHandlerSpec {
            id: "take_profit_pct".to_string(),
            name: "Take Profit Pct".to_string(),
            handler_name: "TakeProfitPct".to_string(),
            timeframe: timeframe.clone(),
            price_field: PriceField::Close,
            parameters: take_profit_params,
            direction: PositionDirection::Long,
            priority: 20,
            tags: vec!["stop".to_string(), "target".to_string()],
            target_entry_ids: vec!["enter_long".to_string()],
        },
    ];

    let timeframe_requirements = vec![
        TimeframeRequirement {
            alias: fast_alias.clone(),
            timeframe: timeframe.clone(),
        },
        TimeframeRequirement {
            alias: slow_alias.clone(),
            timeframe: timeframe.clone(),
        },
    ];

    StrategyDefinition {
        metadata: StrategyMetadata {
            id: "SMA_CROSSOVER_LONG".to_string(),
            name: "SMA Crossover Long".to_string(),
            description: Some("Простая стратегия пересечения скользящих средних".to_string()),
            version: Some("1.0.0".to_string()),
            author: Some("System".to_string()),
            categories: vec![super::types::StrategyCategory::TrendFollowing],
            tags: vec!["sma".to_string(), "trend".to_string()],
            created_at: None,
            updated_at: None,
        },
        parameters: Vec::new(),
        indicator_bindings,
        formulas,
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        timeframe_requirements,
        defaults: StrategyParameterMap::new(),
        optimizer_hints: BTreeMap::new(),
    }
}
