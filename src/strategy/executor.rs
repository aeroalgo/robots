use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use thiserror::Error;

use crate::condition::types::ConditionResultData;

use crate::candles::aggregator::TimeFrameAggregator;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::di::ServiceContainer;
use crate::indicators::formula::{FormulaDefinition, FormulaEvaluationContext};
use crate::indicators::runtime::IndicatorRuntimeEngine;

use super::base::Strategy;
use super::builder::StrategyBuilder;
use super::context::{StrategyContext, TimeframeData};
use super::types::{
    IndicatorBindingSpec, IndicatorSourceSpec, PriceField, StrategyDecision, StrategyDefinition,
    StrategyError, StrategyParameterMap,
};
use crate::metrics::{BacktestAnalytics, BacktestReport};
use crate::position::{ExecutionReport, PositionBook, PositionError, PositionManager};
use crate::risk::{RiskManager, StopHandlerEntry};

#[derive(Debug, Error)]
pub enum StrategyExecutionError {
    #[error("strategy evaluation error: {0}")]
    Strategy(#[from] StrategyError),
    #[error("position manager error: {0}")]
    Position(#[from] PositionError),
    #[error("feed error: {0}")]
    Feed(String),
}

#[derive(Clone, Debug)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub use_full_capital: bool,
    pub reinvest_profits: bool,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            use_full_capital: false,
            reinvest_profits: false,
        }
    }
}

pub struct BacktestExecutor {
    strategy: Box<dyn Strategy>,
    position_manager: PositionManager,
    risk_manager: RiskManager,
    feed: HistoricalFeed,
    context: StrategyContext,
    analytics: BacktestAnalytics,
    warmup_bars: usize,
    deferred_decision: Option<StrategyDecision>,
    cached_session_duration: Option<chrono::Duration>,
    cached_equity: Option<f64>,
    last_equity_bar: usize,
    initial_capital: f64,
    config: BacktestConfig,
}

#[derive(Clone, Copy, Debug, Default)]
struct SessionState {
    is_session_start: bool,
    is_session_end: bool,
}

impl BacktestExecutor {
    /// Создать новый BacktestExecutor с прямым созданием зависимостей
    ///
    /// Этот метод создает зависимости напрямую (legacy подход).
    /// Для лучшей тестируемости используйте `new_with_provider`.
    pub fn new(
        strategy: Box<dyn Strategy>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, StrategyExecutionError> {
        Self::new_with_provider(strategy, frames, None)
    }

    /// Создать новый BacktestExecutor с использованием ServiceContainer
    ///
    /// Позволяет инжектировать зависимости через DI контейнер,
    /// что упрощает тестирование и делает код более гибким.
    ///
    /// Если `container` равен `None`, зависимости создаются напрямую (legacy режим).
    pub fn new_with_provider(
        strategy: Box<dyn Strategy>,
        frames: HashMap<TimeFrame, QuoteFrame>,
        container: Option<Arc<ServiceContainer>>,
    ) -> Result<Self, StrategyExecutionError> {
        if frames.is_empty() {
            return Err(StrategyExecutionError::Feed(
                "frames collection is empty".to_string(),
            ));
        }

        let mut required_timeframes: std::collections::HashSet<TimeFrame> =
            std::collections::HashSet::new();

        for binding in strategy.indicator_bindings() {
            required_timeframes.insert(binding.timeframe.clone());
        }

        for condition in strategy.conditions() {
            required_timeframes.insert(condition.timeframe.clone());
        }

        for requirement in strategy.timeframe_requirements() {
            required_timeframes.insert(requirement.timeframe.clone());
        }

        let required_timeframes: Vec<TimeFrame> = required_timeframes.into_iter().collect();

        let mut feed = HistoricalFeed::new_empty();
        let mut base_timeframes: Vec<TimeFrame> = Vec::new();

        for (tf, frame) in frames {
            let tf_for_map = tf.clone();
            let tf_for_vec = tf.clone();
            let tf_for_primary = tf.clone();
            let tf_for_len = tf.clone();
            feed.frames.insert(tf_for_map, Arc::new(frame));
            base_timeframes.push(tf_for_vec);
            if feed.primary_timeframe.is_none() {
                feed.primary_timeframe = Some(tf_for_primary);
            } else {
                let current_len = feed
                    .primary_timeframe
                    .as_ref()
                    .and_then(|ptf| feed.frames.get(ptf))
                    .map(|f| f.len())
                    .unwrap_or(0);
                let new_len = feed.frames.get(&tf_for_len).map(|f| f.len()).unwrap_or(0);
                if new_len > current_len {
                    feed.primary_timeframe = Some(tf_for_len);
                }
            }
        }

        let mut aggregated_frames = HashMap::new();
        for base_tf in &base_timeframes {
            if let Some(base_frame) = feed.frames.get(base_tf) {
                let all_required: Vec<TimeFrame> = required_timeframes
                    .iter()
                    .filter(|tf| {
                        HistoricalFeed::is_higher_timeframe(tf, base_tf)
                            && HistoricalFeed::is_multiple_of(base_tf, tf)
                    })
                    .cloned()
                    .collect();

                for target_tf in all_required {
                    if feed.frames.contains_key(&target_tf) {
                        continue;
                    }
                    if aggregated_frames.contains_key(&target_tf) {
                        continue;
                    }

                    match TimeFrameAggregator::aggregate(base_frame.as_ref(), target_tf.clone()) {
                        Ok(aggregated) => {
                            aggregated_frames.insert(target_tf.clone(), aggregated.frame);
                        }
                        Err(e) => {
                            return Err(StrategyExecutionError::Feed(format!(
                                "Failed to aggregate timeframe {:?} from {:?}: {}",
                                target_tf, base_tf, e
                            )));
                        }
                    }
                }

                let base_minutes = HistoricalFeed::timeframe_to_minutes(base_tf);
                if let Some(base_mins) = base_minutes {
                    let multipliers = match base_mins {
                        5 => vec![2, 3, 4],
                        15 => vec![2, 4],
                        30 => vec![2, 4],
                        60 => vec![2, 3, 4, 6, 8, 12],
                        120 => vec![2, 3],
                        180 => vec![2],
                        240 => vec![],
                        _ => vec![],
                    };

                    for mult in multipliers {
                        if let Some(target_tf) =
                            HistoricalFeed::create_derived_timeframe(base_tf, mult)
                        {
                            if feed.frames.contains_key(&target_tf) {
                                continue;
                            }
                            if aggregated_frames.contains_key(&target_tf) {
                                continue;
                            }
                            if !required_timeframes.contains(&target_tf) {
                                continue;
                            }

                            match TimeFrameAggregator::aggregate(
                                base_frame.as_ref(),
                                target_tf.clone(),
                            ) {
                                Ok(aggregated) => {
                                    aggregated_frames.insert(target_tf.clone(), aggregated.frame);
                                }
                                Err(e) => {
                                    return Err(StrategyExecutionError::Feed(format!(
                                        "Failed to aggregate timeframe {:?} from {:?}: {}",
                                        target_tf, base_tf, e
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        for (tf, frame) in aggregated_frames {
            feed.frames.insert(tf, Arc::new(frame));
        }

        if feed.primary_timeframe.is_none() {
            return Err(StrategyExecutionError::Feed(
                "No timeframes available after generation".to_string(),
            ));
        }

        let timeframe_order: Vec<TimeFrame> = strategy
            .timeframe_requirements()
            .iter()
            .map(|req| req.timeframe.clone())
            .collect();

        let context = feed.initialize_context_ordered(&timeframe_order);
        let config = BacktestConfig::default();

        // Разрешаем зависимости через DI или создаем напрямую
        let position_manager = if let Some(container) = &container {
            // Пытаемся получить из DI контейнера
            // Если сервис зарегистрирован, извлекаем его из Arc
            container
                .resolve::<PositionManager>()
                .and_then(|pm_arc| Arc::try_unwrap(pm_arc).ok())
                .unwrap_or_else(|| {
                    // Fallback на прямое создание
                    PositionManager::new(strategy.id().to_string()).with_capital(
                        config.initial_capital,
                        config.use_full_capital,
                        config.reinvest_profits,
                    )
                })
        } else {
            // Legacy режим - прямое создание
            PositionManager::new(strategy.id().to_string()).with_capital(
                config.initial_capital,
                config.use_full_capital,
                config.reinvest_profits,
            )
        };

        let risk_manager = if let Some(container) = &container {
            // Пытаемся получить из DI контейнера
            container
                .resolve::<RiskManager>()
                .and_then(|rm_arc| Arc::try_unwrap(rm_arc).ok())
                .unwrap_or_else(|| Self::build_risk_manager(&*strategy))
        } else {
            // Legacy режим - прямое создание
            Self::build_risk_manager(&*strategy)
        };
        let cached_session_duration = feed.primary_timeframe.as_ref().and_then(|tf| tf.duration());
        let initial_capital = config.initial_capital;
        Ok(Self {
            strategy,
            position_manager,
            risk_manager,
            feed,
            context,
            analytics: BacktestAnalytics::new(),
            warmup_bars: 0,
            deferred_decision: None,
            cached_session_duration,
            cached_equity: None,
            last_equity_bar: 0,
            initial_capital,
            config,
        })
    }

    pub fn with_config(mut self, config: BacktestConfig) -> Self {
        self.initial_capital = config.initial_capital;
        self.position_manager.set_capital(
            config.initial_capital,
            config.use_full_capital,
            config.reinvest_profits,
        );
        self.config = config;
        self
    }

    pub fn config(&self) -> &BacktestConfig {
        &self.config
    }

    fn build_risk_manager(strategy: &dyn Strategy) -> RiskManager {
        let mut risk_manager = RiskManager::new();

        for spec in strategy.stop_handler_specs() {
            if let Ok(handler) = crate::risk::factory::StopHandlerFactory::create(
                &spec.handler_name,
                &spec.parameters,
            ) {
                let entry = StopHandlerEntry {
                    handler: Arc::from(handler),
                    timeframe: spec.timeframe.clone(),
                    price_field: spec.price_field.clone(),
                    direction: spec.direction.clone(),
                    priority: spec.priority,
                };
                risk_manager.add_handler(entry);
            }
        }

        risk_manager
    }

    fn compute_warmup_bars(&self) -> usize {
        // Находим самый длинный период и его таймфрейм
        let mut max_warmup_bars = 0usize;

        for binding in self.strategy.indicator_bindings() {
            if let IndicatorSourceSpec::Registry { parameters, .. } = &binding.source {
                if let Some(period) = parameters.get("period") {
                    let period_usize = period.max(1.0).round() as usize;
                    // Warmup = период * 2 на данном таймфрейме
                    let warmup_on_tf = period_usize * 2;

                    // Пересчитываем warmup в бары базового таймфрейма
                    if let Some(primary_tf) = self.feed.primary_timeframe.as_ref() {
                        let warmup_base = Self::convert_warmup_to_base_timeframe(
                            &binding.timeframe,
                            primary_tf,
                            warmup_on_tf,
                        );
                        max_warmup_bars = max_warmup_bars.max(warmup_base);
                    }
                }
            }
        }

        max_warmup_bars
    }

    fn convert_warmup_to_base_timeframe(
        indicator_tf: &TimeFrame,
        base_tf: &TimeFrame,
        warmup_bars: usize,
    ) -> usize {
        let indicator_minutes = Self::timeframe_to_minutes(indicator_tf).unwrap_or(1);
        let base_minutes = Self::timeframe_to_minutes(base_tf).unwrap_or(1);

        if indicator_minutes >= base_minutes {
            // Старший таймфрейм: умножаем на соотношение
            let ratio = indicator_minutes / base_minutes;
            warmup_bars * ratio as usize
        } else {
            // Младший таймфрейм: делим на соотношение
            let ratio = base_minutes / indicator_minutes;
            (warmup_bars + ratio as usize - 1) / ratio as usize // Округление вверх
        }
    }

    pub fn from_definition(
        definition: StrategyDefinition,
        parameter_overrides: Option<StrategyParameterMap>,
        frames: HashMap<TimeFrame, QuoteFrame>,
    ) -> Result<Self, StrategyExecutionError> {
        let mut builder = StrategyBuilder::new(definition);
        if let Some(overrides) = parameter_overrides {
            builder = builder.with_parameters(overrides);
        }
        let strategy = builder.build().map_err(StrategyExecutionError::Strategy)?;

        static BACKTEST_COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let backtest_number =
            BACKTEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

        if backtest_number % 5 == 1 {
            println!(
                "\n      📋 StrategyDefinition (перед вызовом run_backtest, backtest #{}):",
                backtest_number
            );
            println!("{:#?}", strategy.definition());
        }

        let mut executor = Self::new(Box::new(strategy), frames)?;
        executor.warmup_bars = executor.compute_warmup_bars();
        Ok(executor)
    }

    fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        match tf {
            TimeFrame::Minutes(m) => Some(*m),
            TimeFrame::Hours(h) => Some(h * 60),
            TimeFrame::Days(d) => Some(d * 24 * 60),
            TimeFrame::Weeks(w) => Some(w * 7 * 24 * 60),
            TimeFrame::Months(m) => Some(m * 30 * 24 * 60),
            TimeFrame::Custom(_) => None,
        }
    }

    pub fn context(&self) -> &StrategyContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut StrategyContext {
        &mut self.context
    }

    pub fn position_manager(&self) -> &PositionManager {
        &self.position_manager
    }

    pub fn position_manager_mut(&mut self) -> &mut PositionManager {
        &mut self.position_manager
    }

    pub fn run_backtest(&mut self) -> Result<BacktestReport, StrategyExecutionError> {
        if self.warmup_bars == 0 {
            self.warmup_bars = self.compute_warmup_bars();
        }
        self.position_manager.reset();
        self.risk_manager.reset();
        self.context.set_active_positions(PositionBook::default());
        self.feed.reset();
        self.analytics.reset();
        self.deferred_decision = None;
        self.cached_equity = None;
        self.last_equity_bar = 0;
        for timeframe in self.feed.frames.keys() {
            if let Ok(data) = self.context.timeframe_mut(timeframe) {
                data.set_index(0);
            } else if let Some(frame) = self.feed.frames.get(timeframe) {
                let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
                self.context.insert_timeframe(timeframe.clone(), data);
            }
        }
        self.populate_indicators()?;
        self.populate_auxiliary_indicators()?;
        self.populate_custom_data()?;
        self.populate_conditions()?;
        self.analytics.push_equity_point(self.initial_capital);
        let mut processed_bars = 0usize;
        let mut needs_session_check = true;
        while self.feed.step(&mut self.context) {
            processed_bars += 1;

            if processed_bars < self.warmup_bars {
                self.analytics.push_equity_point(self.initial_capital);
                continue;
            }

            if needs_session_check {
                let session_state = self.session_state();
                if session_state.is_some() {
                    self.update_session_metadata(session_state);
                    needs_session_check = false;
                }
            } else if processed_bars % 10 == 0 {
                needs_session_check = true;
            }

            let has_open_positions = self.position_manager.open_position_count() > 0;
            self.analytics
                .increment_bars_in_positions_if_has_positions(has_open_positions);

            // if let Some(pending) = self.deferred_decision.take() {
            //     if !pending.is_empty() {
            //         self.context
            //             .metadata
            //             .insert("deferred_entries".to_string(), "true".to_string());
            //         let result = self
            //             .position_manager
            //             .process_decision(&mut self.context, &pending);
            //         self.context.metadata.remove("deferred_entries");
            //         let report = result.map_err(StrategyExecutionError::Position)?;
            //         self.collect_report(&report);
            //         self.cached_equity = None;
            //         if self.position_manager.open_position_count() > 0 {
            //             self.process_immediate_stop_checks()?;
            //         }
            //     }
            // }
            // Обновляем MFE и trailing stops на КАЖДОМ баре (ДО evaluate и проверки стопов)
            if self.position_manager.open_position_count() > 0 {
                self.update_trailing_stops();
            }

            let mut decision = self
                .strategy
                .evaluate(&self.context)
                .map_err(StrategyExecutionError::Strategy)?;

            if !decision.exits.is_empty() && !decision.entries.is_empty() {
                decision.entries.clear();
            }

            let equity_changed = !decision.is_empty();
            let had_new_entries = !decision.entries.is_empty();
            if equity_changed {
                // Валидация стоп-хендлеров перед открытием позиций
                let mut filtered_entries = Vec::new();
                for entry in &decision.entries {
                    // Получаем текущую цену из контекста
                    let entry_price = self
                        .context
                        .timeframe(&entry.timeframe)
                        .ok()
                        .and_then(|tf| {
                            tf.price_series_slice(&PriceField::Close)
                                .and_then(|series| series.get(tf.index()).copied())
                                .map(|p| p as f64)
                        })
                        .unwrap_or(0.0);

                    if entry_price > 0.0 {
                        if let Some(_reason) = self.risk_manager.validate_before_entry(
                            &self.context,
                            &entry.direction,
                            entry_price,
                            &entry.timeframe,
                            PriceField::Close,
                        ) {
                            // Пропускаем открытие позиции, если валидация не прошла
                            continue;
                        }
                    }
                    filtered_entries.push(entry.clone());
                }

                // Создаем новое решение только с валидными entry сигналами
                let mut validated_decision = decision.clone();
                validated_decision.entries = filtered_entries;

                let mut report = self
                    .position_manager
                    .process_decision(&mut self.context, &validated_decision)
                    .map_err(StrategyExecutionError::Position)?;

                for trade in &mut report.closed_trades {
                    let history = self.risk_manager.take_stop_history(&trade.position_id);
                    trade.stop_history = history
                        .into_iter()
                        .map(|h| crate::position::StopHistoryEntry {
                            bar_index: h.bar_index,
                            stop_level: h.stop_level,
                            max_high: h.max_high,
                            min_low: h.min_low,
                        })
                        .collect();
                }

                self.collect_report(&report);
                self.cached_equity = None;

                if had_new_entries && self.position_manager.open_position_count() > 0 {
                    self.update_trailing_stops();
                }
            }

            if self.position_manager.open_position_count() > 0 {
                self.process_immediate_stop_checks()?;
            }

            let equity = {
                let needs_snapshot = (has_open_positions
                    && (equity_changed || self.cached_equity.is_none()))
                    || (has_open_positions && processed_bars - self.last_equity_bar >= 10)
                    || self.cached_equity.is_none();

                if !needs_snapshot {
                    self.cached_equity.unwrap()
                } else {
                    let total_equity = self.position_manager.portfolio_snapshot().total_equity;
                    let current_equity = self.initial_capital + total_equity;

                    let should_update_cache = if has_open_positions
                        && (equity_changed || self.cached_equity.is_none())
                    {
                        self.cached_equity
                            .map_or(true, |cached| (cached - current_equity).abs() > 0.01)
                    } else if has_open_positions && processed_bars - self.last_equity_bar >= 10 {
                        (self.cached_equity.unwrap() - current_equity).abs() > 0.01
                    } else {
                        true
                    };

                    if should_update_cache {
                        self.cached_equity = Some(current_equity);
                        self.last_equity_bar = processed_bars;
                    }
                    current_equity
                }
            };
            self.analytics.push_equity_point(equity);
        }
        // if let Some(pending) = self.deferred_decision.take() {
        //     if !pending.is_empty() {
        //         self.context
        //             .metadata
        //             .insert("deferred_entries".to_string(), "true".to_string());
        //         let result = self
        //             .position_manager
        //             .process_decision(&mut self.context, &pending);
        //         self.context.metadata.remove("deferred_entries");
        //         let report = result.map_err(StrategyExecutionError::Position)?;
        //         self.collect_report(&report);
        //         self.process_immediate_stop_checks()?;
        //     }
        // }
        let initial_capital = self
            .analytics
            .equity_curve()
            .first()
            .copied()
            .unwrap_or(10000.0);

        let start_date = self
            .feed
            .primary_timeframe
            .as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .and_then(|frame| frame.first())
            .map(|quote| quote.timestamp());

        let end_date = self
            .feed
            .primary_timeframe
            .as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .and_then(|frame| frame.latest())
            .map(|quote| quote.timestamp());

        let total_bars = self
            .feed
            .primary_timeframe
            .as_ref()
            .and_then(|tf| self.feed.frames.get(tf))
            .map(|frame| frame.len())
            .unwrap_or(0);

        let bars_in_positions = self.analytics.bars_in_positions();

        Ok(self.analytics.build_report(
            initial_capital,
            start_date,
            end_date,
            total_bars,
            bars_in_positions,
            None,
        ))
    }

    fn populate_indicators(&mut self) -> Result<(), StrategyExecutionError> {
        let bindings_count = self.strategy.indicator_bindings().len();
        let mut grouped: HashMap<TimeFrame, Vec<IndicatorBindingSpec>> =
            HashMap::with_capacity(bindings_count / 2 + 1);
        for binding in self.strategy.indicator_bindings() {
            grouped
                .entry(binding.timeframe.clone())
                .or_default()
                .push(binding.clone());
        }
        let mut engine = IndicatorRuntimeEngine::new();
        for (timeframe, bindings) in grouped {
            let frame = {
                let frame_ref = self.feed.frames.get(&timeframe).ok_or_else(|| {
                    StrategyExecutionError::Feed(format!(
                        "timeframe {:?} not available in feed",
                        timeframe
                    ))
                })?;
                Arc::clone(frame_ref)
            };

            self.ensure_timeframe_data(&timeframe, &frame);
            let ohlc = frame.to_indicator_ohlc();
            let plan = IndicatorComputationPlan::build(&bindings)?;
            let mut computed: HashMap<String, Arc<Vec<f32>>> =
                HashMap::with_capacity(bindings.len());

            for binding in plan.ordered() {
                match &binding.source {
                    IndicatorSourceSpec::Registry { name, parameters } => {
                        let values = engine
                            .compute_registry(&timeframe, name, parameters, &ohlc)
                            .map_err(|err| {
                                StrategyExecutionError::Feed(format!(
                                    "indicator {} calculation failed: {}",
                                    name, err
                                ))
                            })?;
                        self.store_indicator_series(
                            &timeframe,
                            &binding.alias,
                            Arc::clone(&values),
                        )?;
                        computed.insert(binding.alias.clone(), Arc::clone(&values));
                    }
                    IndicatorSourceSpec::Formula { .. } => {
                        let definition = plan.formula(&binding.alias).ok_or_else(|| {
                            StrategyExecutionError::Feed(format!(
                                "missing formula definition for alias {}",
                                binding.alias
                            ))
                        })?;
                        let context = FormulaEvaluationContext::new(&ohlc, &computed);
                        let values = engine
                            .compute_formula(&timeframe, definition, &context)
                            .map_err(|err| StrategyExecutionError::Feed(err.to_string()))?;
                        self.store_indicator_series(
                            &timeframe,
                            &binding.alias,
                            Arc::clone(&values),
                        )?;
                        computed.insert(binding.alias.clone(), Arc::clone(&values));
                    }
                }
            }
        }
        Ok(())
    }

    /// Вычисляет служебные индикаторы для стоп-обработчиков (ATR, MINFOR, MAXFOR)
    fn populate_auxiliary_indicators(&mut self) -> Result<(), StrategyExecutionError> {
        let auxiliary_specs = self.strategy.auxiliary_indicator_specs();
        if auxiliary_specs.is_empty() {
            return Ok(());
        }

        // Получаем первый доступный таймфрейм для OHLC данных
        // (служебные индикаторы вычисляются на базовом таймфрейме)
        let timeframe = self
            .feed
            .frames
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| StrategyExecutionError::Feed("No frames available".to_string()))?;

        let frame = self.feed.frames.get(&timeframe).ok_or_else(|| {
            StrategyExecutionError::Feed(format!("timeframe {:?} not available in feed", timeframe))
        })?;

        let ohlc = frame.to_indicator_ohlc();

        // Вычисляем auxiliary индикаторы
        let computed =
            crate::risk::compute_auxiliary_indicators(auxiliary_specs, &ohlc).map_err(|e| {
                StrategyExecutionError::Feed(format!("Auxiliary indicator error: {}", e))
            })?;

        // Сохраняем в TimeframeData
        let data = self
            .context
            .timeframe_mut(&timeframe)
            .map_err(StrategyExecutionError::Strategy)?;

        for (alias, values) in computed {
            data.insert_auxiliary(alias, values);
        }

        Ok(())
    }

    fn ensure_timeframe_data(&mut self, timeframe: &TimeFrame, frame: &Arc<QuoteFrame>) {
        if self.context.timeframe(timeframe).is_err() {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            self.context.insert_timeframe(timeframe.clone(), data);
        }
    }

    fn store_indicator_series(
        &mut self,
        timeframe: &TimeFrame,
        alias: &str,
        values: Arc<Vec<f32>>,
    ) -> Result<(), StrategyExecutionError> {
        let data = self
            .context
            .timeframe_mut(timeframe)
            .map_err(StrategyExecutionError::Strategy)?;
        data.insert_indicator_arc(alias.to_string(), values);
        Ok(())
    }

    fn populate_custom_data(&mut self) -> Result<(), StrategyExecutionError> {
        use super::types::DataSeriesSource;
        use std::collections::HashMap;

        let mut constants_by_timeframe: HashMap<TimeFrame, HashMap<String, f32>> = HashMap::new();

        for condition in self.strategy.conditions() {
            let extract_constants = |source: &DataSeriesSource| -> Option<(String, f32)> {
                match source {
                    DataSeriesSource::Custom { key, .. } => {
                        if key.starts_with("constant_") {
                            if let Ok(value) = key.strip_prefix("constant_")?.parse::<f32>() {
                                return Some((key.clone(), value));
                            }
                        }
                        None
                    }
                    _ => None,
                }
            };

            match &condition.input {
                super::types::ConditionInputSpec::Single { source } => {
                    if let Some((key, value)) = extract_constants(source) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                }
                super::types::ConditionInputSpec::Dual { primary, secondary } => {
                    if let Some((key, value)) = extract_constants(primary) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                    if let Some((key, value)) = extract_constants(secondary) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                }
                super::types::ConditionInputSpec::DualWithPercent {
                    primary, secondary, ..
                } => {
                    if let Some((key, value)) = extract_constants(primary) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                    if let Some((key, value)) = extract_constants(secondary) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                }
                super::types::ConditionInputSpec::Range {
                    source,
                    lower,
                    upper,
                } => {
                    if let Some((key, value)) = extract_constants(source) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                    if let Some((key, value)) = extract_constants(lower) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                    if let Some((key, value)) = extract_constants(upper) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                }
                super::types::ConditionInputSpec::Indexed { source, .. } => {
                    if let Some((key, value)) = extract_constants(source) {
                        constants_by_timeframe
                            .entry(condition.timeframe.clone())
                            .or_default()
                            .insert(key, value);
                    }
                }
                super::types::ConditionInputSpec::Ohlc => {}
            }
        }

        for (timeframe, constants) in constants_by_timeframe {
            let frame = self.feed.frames.get(&timeframe).ok_or_else(|| {
                StrategyExecutionError::Feed(format!(
                    "timeframe {:?} not available for custom data",
                    timeframe
                ))
            })?;

            let frame_len = frame.len();
            let data = self
                .context
                .timeframe_mut(&timeframe)
                .map_err(StrategyExecutionError::Strategy)?;

            for (key, value) in constants {
                let constant_series: Vec<f32> = vec![value; frame_len];
                data.insert_custom_series(key, constant_series);
            }
        }

        Ok(())
    }

    fn populate_conditions(&mut self) -> Result<(), StrategyExecutionError> {
        let conditions_count = self.strategy.conditions().len();
        let mut grouped: HashMap<TimeFrame, Vec<usize>> =
            HashMap::with_capacity(conditions_count / 2 + 1);

        for (idx, condition) in self.strategy.conditions().iter().enumerate() {
            grouped
                .entry(condition.timeframe.clone())
                .or_default()
                .push(idx);
        }

        for (timeframe, condition_indices) in grouped {
            let frame = {
                let frame_ref = self.feed.frames.get(&timeframe).ok_or_else(|| {
                    StrategyExecutionError::Feed(format!(
                        "timeframe {:?} not available in feed for conditions",
                        timeframe
                    ))
                })?;
                Arc::clone(frame_ref)
            };

            self.ensure_timeframe_data(&timeframe, &frame);

            let mut results: Vec<(usize, Arc<ConditionResultData>)> = Vec::new();

            for &condition_idx in &condition_indices {
                let condition = self
                    .strategy
                    .conditions()
                    .get(condition_idx)
                    .ok_or_else(|| {
                        StrategyExecutionError::Feed(format!(
                            "condition at index {} not found",
                            condition_idx
                        ))
                    })?;

                {
                    let data = self
                        .context
                        .timeframe_mut(&timeframe)
                        .map_err(StrategyExecutionError::Strategy)?;
                    data.register_condition_id(condition.id.clone(), condition_idx);
                }

                let input = self
                    .context
                    .prepare_condition_input(condition)
                    .map_err(|err| StrategyExecutionError::Strategy(err))?;

                let result = condition.condition.check(input).map_err(|err| {
                    StrategyExecutionError::Strategy(StrategyError::ConditionFailure {
                        condition_id: condition.id.clone(),
                        source: err,
                    })
                })?;

                results.push((condition_idx, Arc::new(result)));
            }

            {
                let data = self
                    .context
                    .timeframe_mut(&timeframe)
                    .map_err(StrategyExecutionError::Strategy)?;
                for (condition_idx, result) in results {
                    data.insert_condition_result_by_index(condition_idx, result);
                }
            }
        }
        Ok(())
    }

    fn session_state(&self) -> Option<SessionState> {
        let primary = self.feed.primary_timeframe.as_ref()?;
        let frame = self.feed.frames.get(primary)?;
        let timeframe_data = self.context.timeframe(primary).ok()?;
        let idx = timeframe_data.index();
        if frame.len() == 0 || idx >= frame.len() {
            return None;
        }
        let duration = self.cached_session_duration?;
        let current = frame.get(idx)?;
        let mut state = SessionState::default();
        if idx == 0 {
            state.is_session_start = true;
        } else if let Some(prev) = frame.get(idx.saturating_sub(1)) {
            let delta = current.timestamp() - prev.timestamp();
            if delta > duration {
                state.is_session_start = true;
            }
        }
        if idx + 1 >= frame.len() {
            state.is_session_end = true;
        } else if let Some(next) = frame.get(idx + 1) {
            let delta = next.timestamp() - current.timestamp();
            if delta > duration {
                state.is_session_end = true;
            }
        }
        Some(state)
    }

    fn update_session_metadata(&mut self, state: Option<SessionState>) {
        const SESSION_START: &str = "session_start";
        const SESSION_END: &str = "session_end";
        match state {
            Some(state) => {
                if state.is_session_start {
                    self.context
                        .metadata
                        .insert(SESSION_START.to_string(), "true".to_string());
                } else {
                    self.context.metadata.remove(SESSION_START);
                }
                if state.is_session_end {
                    self.context
                        .metadata
                        .insert(SESSION_END.to_string(), "true".to_string());
                } else {
                    self.context.metadata.remove(SESSION_END);
                }
            }
            None => {
                self.context.metadata.remove(SESSION_START);
                self.context.metadata.remove(SESSION_END);
            }
        }
    }

    fn collect_report(&mut self, report: &ExecutionReport) {
        self.analytics.absorb_execution_report(report);
    }

    fn update_trailing_stops(&mut self) {
        self.risk_manager.sync_with_positions(&self.context);
        self.risk_manager.on_new_bar(&self.context);
    }

    fn process_immediate_stop_checks(&mut self) -> Result<(), StrategyExecutionError> {
        loop {
            let stop_signals = self.risk_manager.check_stops(&self.context);
            if stop_signals.is_empty() {
                break;
            }

            let closed_ids: Vec<String> = stop_signals
                .iter()
                .map(|s| s.signal.rule_id.clone())
                .collect();

            let mut decision = StrategyDecision::empty();
            decision.stop_signals = stop_signals;
            let mut report = self
                .position_manager
                .process_decision(&mut self.context, &decision)
                .map_err(StrategyExecutionError::Position)?;

            for trade in &mut report.closed_trades {
                let history = self.risk_manager.take_stop_history(&trade.position_id);
                trade.stop_history = history
                    .into_iter()
                    .map(|h| crate::position::StopHistoryEntry {
                        bar_index: h.bar_index,
                        stop_level: h.stop_level,
                        max_high: h.max_high,
                        min_low: h.min_low,
                    })
                    .collect();
            }

            self.collect_report(&report);

            for position_id in &closed_ids {
                self.risk_manager.on_position_closed(position_id);
            }
        }
        Ok(())
    }
}

struct HistoricalFeed {
    frames: HashMap<TimeFrame, Arc<QuoteFrame>>,
    indices: HashMap<TimeFrame, usize>,
    primary_timeframe: Option<TimeFrame>,
    higher_timeframe_timestamps: HashMap<TimeFrame, Vec<i64>>,
    cached_aligned_timestamps: HashMap<TimeFrame, i64>,
}

impl HistoricalFeed {
    fn new_empty() -> Self {
        Self {
            frames: HashMap::new(),
            indices: HashMap::new(),
            primary_timeframe: None,
            higher_timeframe_timestamps: HashMap::new(),
            cached_aligned_timestamps: HashMap::new(),
        }
    }

    fn initialize_context_ordered(&self, timeframe_order: &[TimeFrame]) -> StrategyContext {
        let mut map = HashMap::with_capacity(self.frames.len());
        for (timeframe, frame) in &self.frames {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            map.insert(timeframe.clone(), data);
        }
        StrategyContext::with_timeframes_ordered(timeframe_order, map)
    }

    fn timeframe_to_minutes(tf: &TimeFrame) -> Option<u32> {
        match tf {
            TimeFrame::Minutes(m) => Some(*m),
            TimeFrame::Hours(h) => Some(h * 60),
            TimeFrame::Days(d) => Some(d * 24 * 60),
            TimeFrame::Weeks(w) => Some(w * 7 * 24 * 60),
            TimeFrame::Months(m) => Some(m * 30 * 24 * 60),
            TimeFrame::Custom(_) => None,
        }
    }

    fn is_higher_timeframe(higher: &TimeFrame, lower: &TimeFrame) -> bool {
        let higher_min = Self::timeframe_to_minutes(higher).unwrap_or(0);
        let lower_min = Self::timeframe_to_minutes(lower).unwrap_or(0);
        higher_min > lower_min
    }

    fn is_multiple_of(base: &TimeFrame, target: &TimeFrame) -> bool {
        let base_min = Self::timeframe_to_minutes(base).unwrap_or(0);
        let target_min = Self::timeframe_to_minutes(target).unwrap_or(0);
        if base_min == 0 || target_min == 0 {
            return false;
        }
        target_min > base_min && target_min % base_min == 0
    }

    fn align_timestamp_millis_to_timeframe(
        timestamp_millis: i64,
        timeframe: &TimeFrame,
    ) -> Option<i64> {
        let minutes = Self::timeframe_to_minutes(timeframe)?;
        let total_minutes = timestamp_millis / (60 * 1000);
        let aligned_minutes = (total_minutes / minutes as i64) * minutes as i64;
        Some(aligned_minutes * 60 * 1000)
    }

    fn create_derived_timeframe(base: &TimeFrame, multiplier: u32) -> Option<TimeFrame> {
        let base_minutes = Self::timeframe_to_minutes(base)?;
        let target_minutes = base_minutes * multiplier;
        Self::minutes_to_timeframe(target_minutes)
    }

    fn minutes_to_timeframe(minutes: u32) -> Option<TimeFrame> {
        if minutes < 60 {
            Some(TimeFrame::Minutes(minutes))
        } else if minutes < 24 * 60 {
            let hours = minutes / 60;
            if minutes % 60 == 0 {
                Some(TimeFrame::Hours(hours))
            } else {
                Some(TimeFrame::Minutes(minutes))
            }
        } else if minutes < 7 * 24 * 60 {
            let days = minutes / (24 * 60);
            if minutes % (24 * 60) == 0 {
                Some(TimeFrame::Days(days))
            } else {
                Some(TimeFrame::Minutes(minutes))
            }
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.indices.clear();
        self.cached_aligned_timestamps.clear();
    }

    fn step(&mut self, context: &mut StrategyContext) -> bool {
        let primary_timeframe = match &self.primary_timeframe {
            Some(tf) => tf,
            None => return false,
        };
        let primary_frame = match self.frames.get(primary_timeframe) {
            Some(frame) => frame,
            None => return false,
        };
        let primary_len = primary_frame.len();
        if primary_len == 0 {
            return false;
        }
        let current_primary_index = {
            let entry = self.indices.get(primary_timeframe).copied().unwrap_or(0);
            if entry >= primary_len {
                return false;
            }
            entry
        };

        let current_quote = match primary_frame.get(current_primary_index) {
            Some(q) => q,
            None => return false,
        };
        let current_timestamp_millis = current_quote.timestamp_millis();

        let higher_timestamps = &self.higher_timeframe_timestamps;
        for (timeframe, frame) in &self.frames {
            let len = frame.len();
            if len == 0 {
                continue;
            }
            let idx = if timeframe == primary_timeframe {
                current_primary_index.min(len - 1)
            } else {
                let current_idx = self
                    .indices
                    .get(timeframe)
                    .copied()
                    .unwrap_or(0)
                    .min(len.saturating_sub(1));

                let new_idx = if Self::is_higher_timeframe(timeframe, primary_timeframe) {
                    if let Some(aligned_timestamp_millis) =
                        Self::align_timestamp_millis_to_timeframe(
                            current_timestamp_millis,
                            timeframe,
                        )
                    {
                        let cached_aligned = self.cached_aligned_timestamps.get(timeframe).copied();

                        if cached_aligned == Some(aligned_timestamp_millis) {
                            current_idx
                        } else {
                            let target_idx = if let Some(timestamps) =
                                higher_timestamps.get(timeframe)
                            {
                                let start_idx = current_idx.min(timestamps.len().saturating_sub(1));
                                let end_idx = timestamps.len();

                                let mut target_idx = start_idx;
                                let mut left = start_idx;
                                let mut right = end_idx;

                                while left < right {
                                    let mid = left + (right - left) / 2;
                                    if mid >= timestamps.len() {
                                        break;
                                    }
                                    if timestamps[mid] <= aligned_timestamp_millis {
                                        target_idx = mid;
                                        left = mid + 1;
                                    } else {
                                        right = mid;
                                    }
                                }
                                target_idx.min(len.saturating_sub(1))
                            } else {
                                let mut target_idx = current_idx;
                                while target_idx + 1 < len {
                                    if let Some(quote) = frame.get(target_idx + 1) {
                                        if quote.timestamp_millis() <= aligned_timestamp_millis {
                                            target_idx += 1;
                                        } else {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                                target_idx
                            };

                            self.cached_aligned_timestamps
                                .insert(timeframe.clone(), aligned_timestamp_millis);
                            target_idx
                        }
                    } else {
                        current_idx
                    }
                } else {
                    if current_idx + 1 < len {
                        if let Some(next_quote) = frame.get(current_idx + 1) {
                            if next_quote.timestamp_millis() <= current_timestamp_millis {
                                current_idx + 1
                            } else {
                                current_idx
                            }
                        } else {
                            current_idx
                        }
                    } else {
                        current_idx
                    }
                };
                self.indices.insert(timeframe.clone(), new_idx);
                new_idx
            };
            match context.timeframe_mut(timeframe) {
                Ok(data) => {
                    data.set_index(idx);
                    if data.symbol().is_none() {
                        data.set_symbol(frame.symbol().clone());
                    }
                }
                Err(_) => {
                    let mut data = TimeframeData::with_quote_frame(frame.as_ref(), idx);
                    data.set_index(idx);
                    context.insert_timeframe(timeframe.clone(), data);
                }
            }
        }
        self.indices.insert(
            primary_timeframe.clone(),
            current_primary_index.saturating_add(1),
        );
        true
    }
}

struct IndicatorComputationPlan<'a> {
    ordered: Vec<&'a IndicatorBindingSpec>,
    formulas: HashMap<String, FormulaDefinition>,
}

impl<'a> IndicatorComputationPlan<'a> {
    fn build(bindings: &'a [IndicatorBindingSpec]) -> Result<Self, StrategyExecutionError> {
        let mut binding_map: HashMap<String, &'a IndicatorBindingSpec> =
            HashMap::with_capacity(bindings.len());
        for binding in bindings {
            if binding_map.insert(binding.alias.clone(), binding).is_some() {
                return Err(StrategyExecutionError::Feed(format!(
                    "duplicate indicator alias {}",
                    binding.alias
                )));
            }
        }
        let formula_count = bindings
            .iter()
            .filter(|b| matches!(b.source, IndicatorSourceSpec::Formula { .. }))
            .count();
        let mut formulas = HashMap::with_capacity(formula_count);
        for binding in bindings {
            if let IndicatorSourceSpec::Formula { expression } = &binding.source {
                let definition = FormulaDefinition::parse(expression)
                    .map_err(|err| StrategyExecutionError::Feed(err.to_string()))?;
                formulas.insert(binding.alias.clone(), definition);
            }
        }
        let mut indegree: HashMap<String, usize> =
            binding_map.keys().map(|alias| (alias.clone(), 0)).collect();
        let mut edges: HashMap<String, Vec<String>> = HashMap::with_capacity(bindings.len());
        for binding in bindings {
            if let Some(definition) = formulas.get(&binding.alias) {
                for dependency in definition.data_dependencies() {
                    if !binding_map.contains_key(dependency) {
                        return Err(StrategyExecutionError::Feed(format!(
                            "missing dependency {} for alias {}",
                            dependency, binding.alias
                        )));
                    }
                    edges
                        .entry(dependency.clone())
                        .or_default()
                        .push(binding.alias.clone());
                    if let Some(value) = indegree.get_mut(&binding.alias) {
                        *value += 1;
                    }
                }
            }
        }
        let mut queue = VecDeque::with_capacity(bindings.len());
        for (alias, degree) in &indegree {
            if *degree == 0 {
                queue.push_back(alias.clone());
            }
        }
        let mut ordered = Vec::with_capacity(bindings.len());
        while let Some(alias) = queue.pop_front() {
            let binding = *binding_map
                .get(&alias)
                .expect("binding must exist for alias");
            ordered.push(binding);
            if let Some(children) = edges.get(&alias) {
                for child in children {
                    if let Some(entry) = indegree.get_mut(child) {
                        *entry -= 1;
                        if *entry == 0 {
                            queue.push_back(child.clone());
                        }
                    }
                }
            }
        }
        if ordered.len() != bindings.len() {
            return Err(StrategyExecutionError::Feed(
                "circular indicator dependency".to_string(),
            ));
        }
        Ok(Self { ordered, formulas })
    }

    fn ordered(&self) -> &[&'a IndicatorBindingSpec] {
        &self.ordered
    }

    fn formula(&self, alias: &str) -> Option<&FormulaDefinition> {
        self.formulas.get(alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::types::SignalStrength;
    use crate::data_model::quote::Quote;
    use crate::data_model::quote_frame::QuoteFrame;
    use crate::data_model::types::Symbol;
    use crate::strategy::context::StrategyContext;
    use crate::strategy::types::{
        IndicatorBindingSpec, PositionDirection, PreparedCondition, PriceField, StrategyDecision,
        StrategyId, StrategyMetadata, StrategyParameterMap, StrategyRuleSpec, StrategySignal,
        StrategySignalType, TimeframeRequirement,
    };
    use chrono::Utc;

    #[derive(Clone)]
    struct SimpleStrategy {
        id: StrategyId,
        metadata: StrategyMetadata,
        timeframe: TimeFrame,
        requirements: Vec<TimeframeRequirement>,
        parameters: StrategyParameterMap,
    }

    impl SimpleStrategy {
        fn new(timeframe: TimeFrame) -> Self {
            let id = "SIMPLE_ENTRY_EXIT".to_string();
            let metadata = StrategyMetadata::with_id(&id, "Simple Entry Exit");
            let requirements = vec![TimeframeRequirement {
                alias: "primary".to_string(),
                timeframe: timeframe.clone(),
            }];
            Self {
                id,
                metadata,
                timeframe,
                requirements,
                parameters: StrategyParameterMap::new(),
            }
        }
    }

    impl Strategy for SimpleStrategy {
        fn id(&self) -> &StrategyId {
            &self.id
        }

        fn metadata(&self) -> &StrategyMetadata {
            &self.metadata
        }

        fn parameters(&self) -> &StrategyParameterMap {
            &self.parameters
        }

        fn indicator_bindings(&self) -> &[IndicatorBindingSpec] {
            &[]
        }

        fn conditions(&self) -> &[PreparedCondition] {
            &[]
        }

        fn entry_rules(&self) -> &[StrategyRuleSpec] {
            &[]
        }

        fn exit_rules(&self) -> &[StrategyRuleSpec] {
            &[]
        }

        fn timeframe_requirements(&self) -> &[TimeframeRequirement] {
            &self.requirements
        }

        fn evaluate(&self, context: &StrategyContext) -> Result<StrategyDecision, StrategyError> {
            let data = context.timeframe(&self.timeframe)?;
            let idx = data.index();
            let series_len = data
                .price_series_slice(&PriceField::Close)
                .map(|slice| slice.len())
                .unwrap_or(0);
            let mut decision = StrategyDecision::empty();
            if idx == 0 {
                let signal = StrategySignal {
                    rule_id: "enter".to_string(),
                    signal_type: StrategySignalType::Entry,
                    direction: PositionDirection::Long,
                    timeframe: self.timeframe.clone(),
                    strength: SignalStrength::Strong,
                    quantity: Some(1.0),
                    entry_rule_id: Some("enter".to_string()),
                    tags: Vec::new(),
                    position_group: Some("enter".to_string()),
                    target_entry_ids: Vec::new(),
                };
                decision.entries.push(signal);
            } else if series_len > 0 && idx + 1 == series_len {
                let signal = StrategySignal {
                    rule_id: "exit".to_string(),
                    signal_type: StrategySignalType::Exit,
                    direction: PositionDirection::Long,
                    timeframe: self.timeframe.clone(),
                    strength: SignalStrength::Strong,
                    quantity: Some(1.0),
                    entry_rule_id: None,
                    tags: Vec::new(),
                    position_group: None,
                    target_entry_ids: vec!["enter".to_string()],
                };
                decision.exits.push(signal);
            }
            Ok(decision)
        }

        fn clone_box(&self) -> Box<dyn Strategy> {
            Box::new(self.clone())
        }
    }

    fn build_frame(prices: &[f32], timeframe: &TimeFrame) -> QuoteFrame {
        let symbol = Symbol::from_descriptor("TEST.TEST");
        let mut frame = QuoteFrame::new(symbol.clone(), timeframe.clone());
        for (idx, price) in prices.iter().enumerate() {
            let quote = Quote::from_parts(
                symbol.clone(),
                timeframe.clone(),
                Utc::now() + chrono::Duration::minutes(idx as i64),
                *price,
                *price,
                *price,
                *price,
                1.0,
            );
            frame.push(quote).unwrap();
        }
        frame
    }

    #[tokio::test]
    async fn backtest_executor_produces_trade_and_metrics() {
        let timeframe = TimeFrame::minutes(1);
        let frame = build_frame(&[100.0, 102.0, 105.0], &timeframe);
        let mut frames = HashMap::new();
        frames.insert(timeframe.clone(), frame);
        let strategy: Box<dyn Strategy> = Box::new(SimpleStrategy::new(timeframe.clone()));
        let mut executor = BacktestExecutor::new(strategy, frames).expect("executor creation");
        let report = executor.run_backtest().expect("backtest run");
        assert_eq!(report.trades.len(), 1);
        let trade = &report.trades[0];
        assert!((trade.pnl - 5.0).abs() < 1e-6);
        assert_eq!(report.metrics.total_trades, 1);
        assert!((report.metrics.total_profit - 5.0).abs() < 1e-6);
        assert!((report.metrics.winning_percentage - 1.0).abs() < 1e-6);
        assert_eq!(report.equity_curve.last().copied().unwrap_or_default(), 5.0);
    }
}
