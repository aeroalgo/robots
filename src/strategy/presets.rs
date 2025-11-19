use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;

use super::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, PriceField,
    StopHandlerSpec, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyRuleSpec, StrategySignalType, TakeHandlerSpec,
    TimeframeRequirement, UserFormulaMetadata,
};

pub fn default_strategy_definitions() -> Vec<StrategyDefinition> {
    vec![sma_crossover_definition()]
}

fn sma_crossover_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let fast_alias = "fast_sma".to_string();
    let slow_alias = "slow_sma".to_string();
    let trend_alias = "trend_sma".to_string();

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
        IndicatorBindingSpec {
            alias: trend_alias.clone(),
            timeframe: timeframe.clone(),
            source: IndicatorSourceSpec::Registry {
                name: "SMA".to_string(),
                parameters: HashMap::from([("period".to_string(), 40.0)]),
            },
            tags: vec!["trend".to_string(), "confirmation".to_string()],
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
        primary: DataSeriesSource::indicator(fast_alias.clone()),
        secondary: DataSeriesSource::indicator(slow_alias.clone()),
    };
    let trend_dual_input = ConditionInputSpec::Dual {
        primary: DataSeriesSource::indicator(fast_alias.clone()),
        secondary: DataSeriesSource::indicator(trend_alias.clone()),
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
        ConditionBindingSpec {
            id: "fast_cross_above_trend".to_string(),
            name: "Fast SMA crosses above trend".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::CrossesAbove,
                &trend_dual_input,
            ),
            parameters: HashMap::new(),
            input: trend_dual_input.clone(),
            weight: 1.0,
            tags: vec!["entry".to_string(), "trend".to_string()],
            user_formula: None,
        },
        ConditionBindingSpec {
            id: "fast_cross_below_trend".to_string(),
            name: "Fast SMA crosses below trend".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::CrossesBelow,
                &trend_dual_input,
            ),
            parameters: HashMap::new(),
            input: trend_dual_input,
            weight: 1.0,
            tags: vec!["exit".to_string(), "trend".to_string()],
            user_formula: None,
        },
    ];

    let entry_rules = vec![
        StrategyRuleSpec {
            id: "enter_long".to_string(),
            name: "Enter long".to_string(),
            logic: super::types::RuleLogic::All,
            conditions: vec![
                "fast_cross_above".to_string(),
            ],
            signal: StrategySignalType::Entry,
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["core".to_string()],
            position_group: Some("enter_long".to_string()),
            target_entry_ids: Vec::new(),
        },
        StrategyRuleSpec {
            id: "enter_long_trend".to_string(),
            name: "Enter long trend".to_string(),
            logic: super::types::RuleLogic::All,
            conditions: vec![
                "fast_cross_above_trend".to_string(),
            ],
            signal: StrategySignalType::Entry,
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["trend".to_string()],
            position_group: Some("enter_long_trend".to_string()),
            target_entry_ids: Vec::new(),
        },
    ];

    let exit_rules = vec![
        StrategyRuleSpec {
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
        },
        StrategyRuleSpec {
            id: "exit_long_trend".to_string(),
            name: "Exit long trend".to_string(),
            logic: super::types::RuleLogic::All,
            conditions: vec!["fast_cross_below_trend".to_string()],
            signal: StrategySignalType::Exit,
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["trend".to_string()],
            position_group: None,
            target_entry_ids: vec!["enter_long_trend".to_string()],
        },
    ];

    let mut stop_loss_params = StrategyParameterMap::new();
    stop_loss_params.insert("percentage".to_string(), StrategyParamValue::Number(0.5));
    let mut take_profit_params = StrategyParameterMap::new();
    take_profit_params.insert("percentage".to_string(), StrategyParamValue::Number(0.8));
    let mut trend_stop_loss_params = StrategyParameterMap::new();
    trend_stop_loss_params.insert("percentage".to_string(), StrategyParamValue::Number(0.7));
    let mut trend_take_profit_params = StrategyParameterMap::new();
    trend_take_profit_params.insert("percentage".to_string(), StrategyParamValue::Number(1.2));

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
            id: "stop_loss_pct_trend".to_string(),
            name: "Stop Loss Pct Trend".to_string(),
            handler_name: "StopLossPct".to_string(),
            timeframe: timeframe.clone(),
            price_field: PriceField::Close,
            parameters: trend_stop_loss_params,
            direction: PositionDirection::Long,
            priority: 15,
            tags: vec!["stop".to_string(), "trend".to_string()],
            target_entry_ids: vec!["enter_long_trend".to_string()],
        },
    ];

    let take_handlers = vec![
        TakeHandlerSpec {
            id: "take_profit_pct".to_string(),
            name: "Take Profit Pct".to_string(),
            handler_name: "TakeProfitPct".to_string(),
            timeframe: timeframe.clone(),
            price_field: PriceField::Close,
            parameters: take_profit_params,
            direction: PositionDirection::Long,
            priority: 20,
            tags: vec!["take".to_string(), "target".to_string()],
            target_entry_ids: vec!["enter_long".to_string()],
        },
        TakeHandlerSpec {
            id: "take_profit_pct_trend".to_string(),
            name: "Take Profit Pct Trend".to_string(),
            handler_name: "TakeProfitPct".to_string(),
            timeframe: timeframe.clone(),
            price_field: PriceField::Close,
            parameters: trend_take_profit_params,
            direction: PositionDirection::Long,
            priority: 25,
            tags: vec!["take".to_string(), "trend".to_string()],
            target_entry_ids: vec!["enter_long_trend".to_string()],
        },
    ];

    StrategyDefinition::new(
        StrategyMetadata {
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
        Vec::new(),
        indicator_bindings,
        formulas,
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        take_handlers,
        StrategyParameterMap::new(),
        BTreeMap::new(),
    )
}
