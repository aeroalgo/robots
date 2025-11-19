use std::collections::HashMap;
use std::sync::Arc;

use crate::data_model::types::TimeFrame;
use crate::indicators::formula::{FormulaDefinition, FormulaEvaluationContext};
use crate::indicators::registry::IndicatorFactory;
use crate::indicators::types::{IndicatorError, OHLCData};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct IndicatorCacheKey {
    timeframe: TimeFrame,
    name: String,
    params: Vec<(String, u32)>,
    length: usize,
}

impl IndicatorCacheKey {
    fn new(
        timeframe: TimeFrame,
        name: &str,
        parameters: &HashMap<String, f32>,
        length: usize,
    ) -> Self {
        let mut params: Vec<(String, u32)> = parameters
            .iter()
            .map(|(key, value)| (key.clone(), value.to_bits()))
            .collect();
        params.sort_by(|a, b| a.0.cmp(&b.0));
        Self {
            timeframe,
            name: name.to_ascii_uppercase(),
            params,
            length,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FormulaCacheKey {
    timeframe: TimeFrame,
    expression: String,
    length: usize,
}

impl FormulaCacheKey {
    fn new(timeframe: TimeFrame, expression: &str, length: usize) -> Self {
        Self {
            timeframe,
            expression: expression.trim().to_string(),
            length,
        }
    }
}

/// Выполняет вычисление индикаторов и формул с кешированием результатов между таймфреймами.
#[derive(Default, Debug)]
pub struct IndicatorRuntimeEngine {
    registry_cache: HashMap<IndicatorCacheKey, Arc<Vec<f32>>>,
    formula_cache: HashMap<FormulaCacheKey, Arc<Vec<f32>>>,
}

impl IndicatorRuntimeEngine {
    pub fn new() -> Self {
        Self {
            registry_cache: HashMap::new(),
            formula_cache: HashMap::new(),
        }
    }

    pub fn compute_registry(
        &mut self,
        timeframe: &TimeFrame,
        name: &str,
        parameters: &HashMap<String, f32>,
        data: &OHLCData,
    ) -> Result<Arc<Vec<f32>>, IndicatorError> {
        let key = IndicatorCacheKey::new(timeframe.clone(), name, parameters, data.len());
        if let Some(values) = self.registry_cache.get(&key) {
            return Ok(values.clone());
        }
        let indicator = IndicatorFactory::create_indicator(name, parameters.clone())?;
        let values = Arc::new(indicator.calculate_ohlc(data)?);
        self.registry_cache.insert(key, values.clone());
        Ok(values)
    }

    pub fn compute_formula(
        &mut self,
        timeframe: &TimeFrame,
        definition: &FormulaDefinition,
        context: &FormulaEvaluationContext<'_>,
    ) -> Result<Arc<Vec<f32>>, IndicatorError> {
        let length = context.length_for(definition)?;
        let key = FormulaCacheKey::new(timeframe.clone(), definition.expression(), length);
        if let Some(values) = self.formula_cache.get(&key) {
            return Ok(values.clone());
        }
        let values = Arc::new(definition.evaluate(context)?);
        self.formula_cache.insert(key, values.clone());
        Ok(values)
    }

    pub fn clear(&mut self) {
        self.registry_cache.clear();
        self.formula_cache.clear();
    }
}
