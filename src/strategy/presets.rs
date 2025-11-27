use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;

use super::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, PriceField,
    StopHandlerSpec, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyRuleSpec, StrategySignalType, TakeHandlerSpec,
    UserFormulaMetadata,
};

pub fn default_strategy_definitions() -> Vec<StrategyDefinition> {
    vec![
        sma_crossover_definition(),
        bollinger_bands_definition(),
        supertrend_atr_trailing_definition(),
    ]
}

fn sma_crossover_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);
    let higher_timeframe = TimeFrame::minutes(240);

    let fast_alias = "fast_sma".to_string();
    let slow_alias = "slow_sma".to_string();
    let trend_alias = "trend_sma".to_string();
    let ema_alias = "ema_240".to_string();

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

    // Период EMA остается 50, но на таймфрейме 240 минут
    // Warmup = 50 * 2 = 100 баров на таймфрейме 240 минут
    // Это эквивалентно 100 * 4 = 400 барам на исходном таймфрейме 60 минут
    indicator_bindings.push(IndicatorBindingSpec {
        alias: ema_alias.clone(),
        timeframe: higher_timeframe.clone(),
        source: IndicatorSourceSpec::Registry {
            name: "EMA".to_string(),
            parameters: HashMap::from([("period".to_string(), 50.0)]),
        },
        tags: vec![
            "trend".to_string(),
            "higher_tf".to_string(),
            "confirmation".to_string(),
        ],
    });

    let formulas = vec![spread_formula];

    let dual_input = ConditionInputSpec::Dual {
        primary: DataSeriesSource::indicator(fast_alias.clone()),
        secondary: DataSeriesSource::indicator(slow_alias.clone()),
    };
    let trend_dual_input = ConditionInputSpec::Dual {
        primary: DataSeriesSource::indicator(fast_alias.clone()),
        secondary: DataSeriesSource::indicator(trend_alias.clone()),
    };

    let close_above_ema_input = ConditionInputSpec::Dual {
        primary: DataSeriesSource::price_with_timeframe(
            PriceField::Close,
            higher_timeframe.clone(),
        ),
        secondary: DataSeriesSource::indicator_with_timeframe(
            ema_alias.clone(),
            higher_timeframe.clone(),
        ),
    };

    let sma_above_close_percent_input = ConditionInputSpec::DualWithPercent {
        primary: DataSeriesSource::indicator(fast_alias.clone()),
        secondary: DataSeriesSource::price(PriceField::Close),
        percent: 2.5,
    };

    let sma_above_slow_percent_input = ConditionInputSpec::DualWithPercent {
        primary: DataSeriesSource::indicator(fast_alias.clone()),
        secondary: DataSeriesSource::indicator(slow_alias.clone()),
        percent: 1.5,
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
        ConditionBindingSpec {
            id: "close_above_ema_240".to_string(),
            name: "Close base TF above EMA compressed TF".to_string(),
            timeframe: higher_timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::GreaterThan,
                &close_above_ema_input,
            ),
            parameters: HashMap::new(),
            input: close_above_ema_input,
            weight: 1.0,
            tags: vec![
                "entry".to_string(),
                "higher_tf".to_string(),
                "trend".to_string(),
            ],
            user_formula: None,
        },
        ConditionBindingSpec {
            id: "sma_rising_trend".to_string(),
            name: "SMA RisingTrend (period: 20)".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::GreaterThan,
                &ConditionInputSpec::Single {
                    source: DataSeriesSource::indicator(fast_alias.clone()),
                },
            ),
            parameters: HashMap::from([("period".to_string(), 20.0)]),
            input: ConditionInputSpec::Single {
                source: DataSeriesSource::indicator(fast_alias.clone()),
            },
            weight: 1.0,
            tags: vec!["entry".to_string(), "trend".to_string()],
            user_formula: None,
        },
        ConditionBindingSpec {
            id: "sma_above_close_percent".to_string(),
            name: "SMA GreaterThan Close на 2.5%".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::GreaterThan,
                &sma_above_close_percent_input,
            ),
            parameters: HashMap::from([("percentage".to_string(), 2.5)]),
            input: sma_above_close_percent_input,
            weight: 1.0,
            tags: vec!["entry".to_string(), "filter".to_string()],
            user_formula: None,
        },
        ConditionBindingSpec {
            id: "fast_sma_above_slow_percent".to_string(),
            name: "Fast SMA GreaterThan Slow SMA на 1.5%".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::GreaterThan,
                &sma_above_slow_percent_input,
            ),
            parameters: HashMap::from([("percentage".to_string(), 1.5)]),
            input: sma_above_slow_percent_input,
            weight: 1.0,
            tags: vec!["entry".to_string(), "filter".to_string()],
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
                "close_above_ema_240".to_string(),
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
                "close_above_ema_240".to_string(),
            ],
            signal: StrategySignalType::Entry,
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["trend".to_string()],
            position_group: Some("enter_long_trend".to_string()),
            target_entry_ids: Vec::new(),
        },
        StrategyRuleSpec {
            id: "enter_long_rising_trend".to_string(),
            name: "Enter long on rising trend".to_string(),
            logic: super::types::RuleLogic::All,
            conditions: vec![
                "sma_rising_trend".to_string(),
                "close_above_ema_240".to_string(),
            ],
            signal: StrategySignalType::Entry,
            direction: PositionDirection::Long,
            quantity: None,
            tags: vec!["trend".to_string(), "rising".to_string()],
            position_group: Some("enter_long_rising_trend".to_string()),
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

fn bollinger_bands_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let bb_middle_alias = "bb_middle".to_string();
    let bb_upper_alias = "bb_upper".to_string();
    let bb_lower_alias = "bb_lower".to_string();

    let indicator_bindings = vec![
        IndicatorBindingSpec {
            alias: bb_middle_alias.clone(),
            timeframe: timeframe.clone(),
            source: IndicatorSourceSpec::Registry {
                name: "BBMiddle".to_string(),
                parameters: HashMap::from([
                    ("period".to_string(), 20.0),
                    ("deviation".to_string(), 2.0),
                ]),
            },
            tags: vec!["bb".to_string(), "middle".to_string()],
        },
        IndicatorBindingSpec {
            alias: bb_upper_alias.clone(),
            timeframe: timeframe.clone(),
            source: IndicatorSourceSpec::Registry {
                name: "BBUpper".to_string(),
                parameters: HashMap::from([
                    ("period".to_string(), 20.0),
                    ("deviation".to_string(), 2.0),
                ]),
            },
            tags: vec!["bb".to_string(), "upper".to_string()],
        },
        IndicatorBindingSpec {
            alias: bb_lower_alias.clone(),
            timeframe: timeframe.clone(),
            source: IndicatorSourceSpec::Registry {
                name: "BBLower".to_string(),
                parameters: HashMap::from([
                    ("period".to_string(), 20.0),
                    ("deviation".to_string(), 2.0),
                ]),
            },
            tags: vec!["bb".to_string(), "lower".to_string()],
        },
    ];

    let price_above_upper = ConditionInputSpec::Dual {
        primary: DataSeriesSource::price(PriceField::Close),
        secondary: DataSeriesSource::indicator(bb_upper_alias.clone()),
    };

    let price_below_lower = ConditionInputSpec::Dual {
        primary: DataSeriesSource::price(PriceField::Close),
        secondary: DataSeriesSource::indicator(bb_lower_alias.clone()),
    };

    let condition_bindings = vec![
        ConditionBindingSpec {
            id: "price_above_upper".to_string(),
            name: "Price above BB Upper".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::GreaterThan,
                &price_above_upper,
            ),
            parameters: HashMap::new(),
            input: price_above_upper,
            weight: 1.0,
            tags: vec!["exit".to_string()],
            user_formula: None,
        },
        ConditionBindingSpec {
            id: "price_below_lower".to_string(),
            name: "Price below BB Lower".to_string(),
            timeframe: timeframe.clone(),
            declarative: ConditionDeclarativeSpec::from_input(
                ConditionOperator::LessThan,
                &price_below_lower,
            ),
            parameters: HashMap::new(),
            input: price_below_lower,
            weight: 1.0,
            tags: vec!["entry".to_string()],
            user_formula: None,
        },
    ];

    let entry_rules = vec![StrategyRuleSpec {
        id: "enter_long_bb".to_string(),
        name: "Enter long on BB Lower touch".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["price_below_lower".to_string()],
        signal: StrategySignalType::Entry,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["bb".to_string(), "entry".to_string()],
        position_group: Some("enter_long_bb".to_string()),
        target_entry_ids: Vec::new(),
    }];

    let exit_rules = vec![];

    let mut stop_loss_params = StrategyParameterMap::new();
    stop_loss_params.insert("percentage".to_string(), StrategyParamValue::Number(1.0));
    let mut take_profit_params = StrategyParameterMap::new();
    take_profit_params.insert("percentage".to_string(), StrategyParamValue::Number(2.0));

    let stop_handlers = vec![StopHandlerSpec {
        id: "stop_loss_pct_bb".to_string(),
        name: "Stop Loss Pct BB".to_string(),
        handler_name: "StopLossPct".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: stop_loss_params,
        direction: PositionDirection::Long,
        priority: 10,
        tags: vec!["stop".to_string(), "risk".to_string()],
        target_entry_ids: vec!["enter_long_bb".to_string()],
    }];

    let take_handlers = vec![TakeHandlerSpec {
        id: "take_profit_pct_bb".to_string(),
        name: "Take Profit Pct BB".to_string(),
        handler_name: "TakeProfitPct".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: take_profit_params,
        direction: PositionDirection::Long,
        priority: 20,
        tags: vec!["take".to_string(), "target".to_string()],
        target_entry_ids: vec!["enter_long_bb".to_string()],
    }];

    StrategyDefinition::new(
        StrategyMetadata {
            id: "BOLLINGER_BANDS_TEST".to_string(),
            name: "Bollinger Bands Test".to_string(),
            description: Some(
                "Тестовая стратегия для проверки Bollinger Bands индикаторов".to_string(),
            ),
            version: Some("1.0.0".to_string()),
            author: Some("System".to_string()),
            categories: vec![super::types::StrategyCategory::MeanReversion],
            tags: vec!["bb".to_string(), "test".to_string()],
            created_at: None,
            updated_at: None,
        },
        Vec::new(),
        indicator_bindings,
        Vec::new(),
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        take_handlers,
        StrategyParameterMap::new(),
        BTreeMap::new(),
    )
}

fn supertrend_atr_trailing_definition() -> StrategyDefinition {
    let timeframe = TimeFrame::minutes(60);

    let supertrend_alias = "supertrend".to_string();

    let indicator_bindings = vec![IndicatorBindingSpec {
        alias: supertrend_alias.clone(),
        timeframe: timeframe.clone(),
        source: IndicatorSourceSpec::Registry {
            name: "SUPERTREND".to_string(),
            parameters: HashMap::from([
                ("period".to_string(), 60.0),
                ("coeff_atr".to_string(), 6.5),
            ]),
        },
        tags: vec!["trend".to_string(), "supertrend".to_string()],
    }];

    let supertrend_below_close = ConditionInputSpec::Dual {
        primary: DataSeriesSource::indicator(supertrend_alias.clone()),
        secondary: DataSeriesSource::price(PriceField::Close),
    };

    let condition_bindings = vec![ConditionBindingSpec {
        id: "supertrend_below_close".to_string(),
        name: "SuperTrend < Close".to_string(),
        timeframe: timeframe.clone(),
        declarative: ConditionDeclarativeSpec::from_input(
            ConditionOperator::LessThan,
            &supertrend_below_close,
        ),
        parameters: HashMap::new(),
        input: supertrend_below_close,
        weight: 1.0,
        tags: vec!["entry".to_string(), "trend".to_string()],
        user_formula: None,
    }];

    let entry_rules = vec![StrategyRuleSpec {
        id: "enter_long_supertrend".to_string(),
        name: "Enter long when SuperTrend < Close".to_string(),
        logic: super::types::RuleLogic::All,
        conditions: vec!["supertrend_below_close".to_string()],
        signal: StrategySignalType::Entry,
        direction: PositionDirection::Long,
        quantity: None,
        tags: vec!["supertrend".to_string(), "entry".to_string()],
        position_group: Some("supertrend_long".to_string()),
        target_entry_ids: Vec::new(),
    }];

    let exit_rules = vec![];

    let mut atr_trail_params = StrategyParameterMap::new();
    atr_trail_params.insert("period".to_string(), StrategyParamValue::Number(30.0));
    atr_trail_params.insert("coeff_atr".to_string(), StrategyParamValue::Number(7.0));

    let stop_handlers = vec![StopHandlerSpec {
        id: "atr_trailing_stop".to_string(),
        name: "ATR Trailing Stop".to_string(),
        handler_name: "ATRTrailStop".to_string(),
        timeframe: timeframe.clone(),
        price_field: PriceField::Close,
        parameters: atr_trail_params,
        direction: PositionDirection::Long,
        priority: 10,
        tags: vec![
            "stop".to_string(),
            "trailing".to_string(),
            "atr".to_string(),
        ],
        target_entry_ids: vec!["enter_long_supertrend".to_string()],
    }];

    let take_handlers = vec![];

    StrategyDefinition::new(
        StrategyMetadata {
            id: "SUPERTREND_ATR_TRAILING".to_string(),
            name: "SuperTrend with ATR Trailing Stop".to_string(),
            description: Some(
                "Стратегия на основе SuperTrend с выходом по ATR Trailing Stop".to_string(),
            ),
            version: Some("1.0.0".to_string()),
            author: Some("System".to_string()),
            categories: vec![super::types::StrategyCategory::TrendFollowing],
            tags: vec![
                "supertrend".to_string(),
                "atr".to_string(),
                "trailing".to_string(),
            ],
            created_at: None,
            updated_at: None,
        },
        Vec::new(),
        indicator_bindings,
        Vec::new(),
        condition_bindings,
        entry_rules,
        exit_rules,
        stop_handlers,
        take_handlers,
        StrategyParameterMap::new(),
        BTreeMap::new(),
    )
}
