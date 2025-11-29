use std::collections::HashMap;
use std::sync::Arc;

use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::indicators::formula::{FormulaDefinition, FormulaEvaluationContext};
use crate::indicators::runtime::IndicatorRuntimeEngine;
use crate::strategy::base::Strategy;
use crate::strategy::context::StrategyContext;
use crate::strategy::types::{IndicatorBindingSpec, IndicatorSourceSpec, StrategyError};

use super::BacktestError;

pub struct IndicatorEngine {
    runtime: IndicatorRuntimeEngine,
}

impl IndicatorEngine {
    pub fn new() -> Self {
        Self {
            runtime: IndicatorRuntimeEngine::new(),
        }
    }

    pub fn populate_indicators(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError> {
        let bindings_count = strategy.indicator_bindings().len();
        let mut grouped: HashMap<TimeFrame, Vec<IndicatorBindingSpec>> =
            HashMap::with_capacity(bindings_count / 2 + 1);

        for binding in strategy.indicator_bindings() {
            grouped
                .entry(binding.timeframe.clone())
                .or_default()
                .push(binding.clone());
        }

        for (timeframe, bindings) in grouped {
            let frame = frames.get(&timeframe).ok_or_else(|| {
                BacktestError::Feed(format!("timeframe {:?} not available in feed", timeframe))
            })?;

            Self::ensure_timeframe_data(context, &timeframe, frame);

            let ohlc = frame.to_indicator_ohlc();
            let plan = IndicatorComputationPlan::build(&bindings)?;
            let mut computed: HashMap<String, Arc<Vec<f32>>> =
                HashMap::with_capacity(bindings.len());

            for binding in plan.ordered() {
                match &binding.source {
                    IndicatorSourceSpec::Registry { name, parameters } => {
                        let values = self
                            .runtime
                            .compute_registry(&timeframe, name, parameters, &ohlc)
                            .map_err(|err| {
                                BacktestError::Feed(format!(
                                    "indicator {} calculation failed: {}",
                                    name, err
                                ))
                            })?;
                        Self::store_indicator_series(
                            context,
                            &timeframe,
                            &binding.alias,
                            Arc::clone(&values),
                        )?;
                        computed.insert(binding.alias.clone(), Arc::clone(&values));
                    }
                    IndicatorSourceSpec::Formula { .. } => {
                        let definition = plan.formula(&binding.alias).ok_or_else(|| {
                            BacktestError::Feed(format!(
                                "missing formula definition for alias {}",
                                binding.alias
                            ))
                        })?;
                        let eval_context = FormulaEvaluationContext::new(&ohlc, &computed);
                        let values = self
                            .runtime
                            .compute_formula(&timeframe, definition, &eval_context)
                            .map_err(|err| BacktestError::Feed(err.to_string()))?;
                        Self::store_indicator_series(
                            context,
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

    pub fn populate_auxiliary_indicators(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError> {
        let auxiliary_specs = strategy.auxiliary_indicator_specs();
        if auxiliary_specs.is_empty() {
            return Ok(());
        }

        let timeframe = frames
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| BacktestError::Feed("No frames available".to_string()))?;

        let frame = frames.get(&timeframe).ok_or_else(|| {
            BacktestError::Feed(format!("timeframe {:?} not available in feed", timeframe))
        })?;

        let ohlc = frame.to_indicator_ohlc();

        let computed =
            crate::risk::compute_auxiliary_indicators(auxiliary_specs, &ohlc).map_err(|e| {
                BacktestError::Feed(format!("Auxiliary indicator error: {}", e))
            })?;

        let data = context
            .timeframe_mut(&timeframe)
            .map_err(|e| BacktestError::Strategy(e))?;

        for (alias, values) in computed {
            data.insert_auxiliary(alias, values);
        }

        Ok(())
    }

    pub fn populate_custom_data(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError> {
        for requirement in strategy.timeframe_requirements() {
            let frame = frames.get(&requirement.timeframe);
            if frame.is_none() {
                continue;
            }
            let frame = frame.unwrap();

            Self::ensure_timeframe_data(context, &requirement.timeframe, frame);

            let data = context
                .timeframe_mut(&requirement.timeframe)
                .map_err(|e| BacktestError::Strategy(e))?;

            for custom in &requirement.custom_data {
                match custom.as_str() {
                    "close" | "price" => {
                        let close_values: Vec<f32> = (0..frame.len())
                            .filter_map(|i| frame.get(i).map(|bar| bar.close()))
                            .collect();
                        data.insert_custom(custom.clone(), Arc::new(close_values));
                    }
                    "high" => {
                        let high_values: Vec<f32> = (0..frame.len())
                            .filter_map(|i| frame.get(i).map(|bar| bar.high()))
                            .collect();
                        data.insert_custom(custom.clone(), Arc::new(high_values));
                    }
                    "low" => {
                        let low_values: Vec<f32> = (0..frame.len())
                            .filter_map(|i| frame.get(i).map(|bar| bar.low()))
                            .collect();
                        data.insert_custom(custom.clone(), Arc::new(low_values));
                    }
                    "open" => {
                        let open_values: Vec<f32> = (0..frame.len())
                            .filter_map(|i| frame.get(i).map(|bar| bar.open()))
                            .collect();
                        data.insert_custom(custom.clone(), Arc::new(open_values));
                    }
                    "volume" => {
                        let volume_values: Vec<f32> = (0..frame.len())
                            .filter_map(|i| frame.get(i).map(|bar| bar.volume()))
                            .collect();
                        data.insert_custom(custom.clone(), Arc::new(volume_values));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn ensure_timeframe_data(
        context: &mut StrategyContext,
        timeframe: &TimeFrame,
        frame: &Arc<QuoteFrame>,
    ) {
        use crate::strategy::context::TimeframeData;
        if context.timeframe(timeframe).is_err() {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            context.insert_timeframe(timeframe.clone(), data);
        }
    }

    fn store_indicator_series(
        context: &mut StrategyContext,
        timeframe: &TimeFrame,
        alias: &str,
        values: Arc<Vec<f32>>,
    ) -> Result<(), BacktestError> {
        let data = context
            .timeframe_mut(timeframe)
            .map_err(|e| BacktestError::Strategy(e))?;
        data.insert_indicator(alias.to_string(), values);
        Ok(())
    }
}

impl Default for IndicatorEngine {
    fn default() -> Self {
        Self::new()
    }
}

struct IndicatorComputationPlan<'a> {
    ordered: Vec<&'a IndicatorBindingSpec>,
    formulas: HashMap<String, FormulaDefinition>,
}

impl<'a> IndicatorComputationPlan<'a> {
    fn build(bindings: &'a [IndicatorBindingSpec]) -> Result<Self, BacktestError> {
        let mut ordered = Vec::with_capacity(bindings.len());
        let mut formulas = HashMap::new();
        let mut remaining: Vec<&IndicatorBindingSpec> = bindings.iter().collect();
        let mut resolved: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut last_size = remaining.len() + 1;

        while !remaining.is_empty() && remaining.len() < last_size {
            last_size = remaining.len();
            let mut next_remaining = Vec::new();

            for binding in remaining {
                match &binding.source {
                    IndicatorSourceSpec::Registry { .. } => {
                        ordered.push(binding);
                        resolved.insert(binding.alias.clone());
                    }
                    IndicatorSourceSpec::Formula { expression } => {
                        let formula = FormulaDefinition::parse(expression).map_err(|e| {
                            BacktestError::Feed(format!(
                                "formula parse error for {}: {}",
                                binding.alias, e
                            ))
                        })?;

                        let deps_resolved = formula
                            .dependencies()
                            .iter()
                            .all(|dep| resolved.contains(dep));

                        if deps_resolved {
                            ordered.push(binding);
                            resolved.insert(binding.alias.clone());
                            formulas.insert(binding.alias.clone(), formula);
                        } else {
                            next_remaining.push(binding);
                        }
                    }
                }
            }
            remaining = next_remaining;
        }

        if !remaining.is_empty() {
            let unresolved: Vec<_> = remaining.iter().map(|b| b.alias.as_str()).collect();
            return Err(BacktestError::Feed(format!(
                "circular or unresolved dependencies: {:?}",
                unresolved
            )));
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
