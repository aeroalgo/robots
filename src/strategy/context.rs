use std::collections::HashMap;
use std::sync::Arc;

use crate::condition::types::{ConditionInputData, ConditionResultData};
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::{timestamp_from_millis, Symbol, TimeFrame};
use crate::indicators::types::OHLCData;
use crate::position::view::{ActivePosition, PositionBook};

use super::types::{
    ConditionInputSpec, DataSeriesSource, PreparedCondition, PriceField, StrategyError,
    StrategyParameterMap, StrategyUserSettings,
};

#[derive(Clone, Debug)]
pub struct TimeframeData {
    timeframe: TimeFrame,
    symbol: Option<Symbol>,
    index: usize,
    prices: HashMap<PriceField, Arc<Vec<f32>>>,
    indicators: HashMap<String, Arc<Vec<f32>>>,
    custom: HashMap<String, Arc<Vec<f32>>>,
    condition_results: HashMap<String, Arc<ConditionResultData>>,
    ohlc: Option<Arc<OHLCData>>,
    timestamps: Option<Arc<Vec<i64>>>,
}

impl TimeframeData {
    pub fn new(timeframe: TimeFrame) -> Self {
        Self {
            timeframe,
            symbol: None,
            index: 0,
            prices: HashMap::with_capacity(4),
            indicators: HashMap::with_capacity(16),
            custom: HashMap::with_capacity(8),
            condition_results: HashMap::with_capacity(8),
            ohlc: None,
            timestamps: None,
        }
    }

    pub fn with_quote_frame(frame: &QuoteFrame, index: usize) -> Self {
        let timeframe = frame.timeframe();
        let symbol = frame.symbol();
        let mut data = Self::new(timeframe.clone());
        data.symbol = Some(symbol.clone());
        let length = frame.len();
        data.index = if length == 0 {
            0
        } else {
            index.min(length - 1)
        };
        let ohlc = frame.to_indicator_ohlc();
        let open = Arc::new(ohlc.open.clone());
        let high = Arc::new(ohlc.high.clone());
        let low = Arc::new(ohlc.low.clone());
        let close = Arc::new(ohlc.close.clone());
        let volume = ohlc.volume.clone();
        let timestamps = ohlc.timestamp.clone();
        data.ohlc = Some(Arc::new(ohlc));
        data.prices.insert(PriceField::Open, open);
        data.prices.insert(PriceField::High, high);
        data.prices.insert(PriceField::Low, low);
        data.prices.insert(PriceField::Close, close);
        if let Some(volume_vec) = volume {
            data.prices.insert(PriceField::Volume, Arc::new(volume_vec));
        }
        if let Some(ts_vec) = timestamps {
            data.timestamps = Some(Arc::new(ts_vec));
        }
        data
    }

    pub fn timeframe(&self) -> &TimeFrame {
        &self.timeframe
    }

    pub fn symbol(&self) -> Option<&Symbol> {
        self.symbol.as_ref()
    }

    pub fn set_symbol(&mut self, symbol: Symbol) {
        self.symbol = Some(symbol);
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn insert_price_series(&mut self, field: PriceField, series: Vec<f32>) {
        self.prices.insert(field, Arc::new(series));
    }

    pub fn insert_price_series_arc(&mut self, field: PriceField, series: Arc<Vec<f32>>) {
        self.prices.insert(field, series);
    }

    pub fn insert_indicator(&mut self, alias: impl Into<String>, series: Vec<f32>) {
        self.indicators.insert(alias.into(), Arc::new(series));
    }

    pub fn insert_indicator_arc(&mut self, alias: impl Into<String>, series: Arc<Vec<f32>>) {
        self.indicators.insert(alias.into(), series);
    }

    pub fn insert_custom_series(&mut self, key: impl Into<String>, series: Vec<f32>) {
        self.custom.insert(key.into(), Arc::new(series));
    }

    pub fn insert_custom_series_arc(&mut self, key: impl Into<String>, series: Arc<Vec<f32>>) {
        self.custom.insert(key.into(), series);
    }

    pub fn set_ohlc(&mut self, ohlc: OHLCData) {
        let open = Arc::new(ohlc.open.clone());
        let high = Arc::new(ohlc.high.clone());
        let low = Arc::new(ohlc.low.clone());
        let close = Arc::new(ohlc.close.clone());
        let volume = ohlc.volume.clone();
        let timestamps = ohlc.timestamp.clone();
        self.ohlc = Some(Arc::new(ohlc));
        self.prices.insert(PriceField::Open, open);
        self.prices.insert(PriceField::High, high);
        self.prices.insert(PriceField::Low, low);
        self.prices.insert(PriceField::Close, close);
        if let Some(volume_vec) = volume {
            self.prices.insert(PriceField::Volume, Arc::new(volume_vec));
        }
        self.timestamps = timestamps.map(|vec| Arc::new(vec));
    }

    pub fn price_series_slice(&self, field: &PriceField) -> Option<&[f32]> {
        self.prices.get(field).map(|data| data.as_ref().as_slice())
    }

    pub fn indicator_series_slice(&self, alias: &str) -> Option<&[f32]> {
        self.indicators
            .get(alias)
            .map(|data| data.as_ref().as_slice())
    }

    pub fn custom_series_slice(&self, key: &str) -> Option<&[f32]> {
        self.custom.get(key).map(|data| data.as_ref().as_slice())
    }

    pub fn ohlc_ref(&self) -> Option<&OHLCData> {
        self.ohlc.as_deref()
    }

    pub fn timestamp_millis_at(&self, index: usize) -> Option<i64> {
        self.timestamps.as_ref()?.get(index).copied()
    }

    pub fn current_timestamp_millis(&self) -> Option<i64> {
        self.timestamp_millis_at(self.index)
    }

    pub fn timestamp_at(&self, index: usize) -> Option<chrono::DateTime<chrono::Utc>> {
        let timestamps = self.timestamps.as_ref()?;
        timestamps
            .get(index)
            .and_then(|millis| timestamp_from_millis(*millis))
    }

    pub fn current_timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.timestamp_at(self.index)
    }

    pub fn insert_condition_result(
        &mut self,
        condition_id: impl Into<String>,
        result: ConditionResultData,
    ) {
        self.condition_results
            .insert(condition_id.into(), Arc::new(result));
    }

    pub fn insert_condition_result_arc(
        &mut self,
        condition_id: impl Into<String>,
        result: Arc<ConditionResultData>,
    ) {
        self.condition_results.insert(condition_id.into(), result);
    }

    pub fn condition_result(&self, condition_id: &str) -> Option<&ConditionResultData> {
        self.condition_results
            .get(condition_id)
            .map(|data| data.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct StrategyContext {
    timeframes: HashMap<TimeFrame, TimeframeData>,
    pub user_settings: StrategyUserSettings,
    pub metadata: HashMap<String, String>,
    pub runtime_parameters: StrategyParameterMap,
    active_positions: HashMap<String, ActivePosition>,
}

impl StrategyContext {
    pub fn new() -> Self {
        Self {
            timeframes: HashMap::with_capacity(4),
            user_settings: HashMap::with_capacity(8),
            metadata: HashMap::with_capacity(8),
            runtime_parameters: HashMap::with_capacity(16),
            active_positions: HashMap::with_capacity(8),
        }
    }

    pub fn with_timeframes(timeframes: HashMap<TimeFrame, TimeframeData>) -> Self {
        let timeframes_len = timeframes.len();
        Self {
            timeframes,
            user_settings: HashMap::with_capacity(8),
            metadata: HashMap::with_capacity(8),
            runtime_parameters: HashMap::with_capacity(16),
            active_positions: HashMap::with_capacity(timeframes_len.max(8)),
        }
    }

    pub fn insert_timeframe(&mut self, timeframe: TimeFrame, data: TimeframeData) {
        self.timeframes.insert(timeframe, data);
    }

    pub fn timeframe(&self, timeframe: &TimeFrame) -> Result<&TimeframeData, StrategyError> {
        self.timeframes
            .get(timeframe)
            .ok_or_else(|| StrategyError::MissingTimeframe(timeframe.clone()))
    }

    pub fn timeframe_mut(
        &mut self,
        timeframe: &TimeFrame,
    ) -> Result<&mut TimeframeData, StrategyError> {
        self.timeframes
            .get_mut(timeframe)
            .ok_or_else(|| StrategyError::MissingTimeframe(timeframe.clone()))
    }

    fn resolve_series<'a>(
        &'a self,
        default_timeframe: &'a TimeFrame,
        source: &'a DataSeriesSource,
    ) -> Result<&'a [f32], StrategyError> {
        let source_timeframe = source.timeframe().unwrap_or(default_timeframe);
        let data = self.timeframe(source_timeframe)?;
        match source {
            DataSeriesSource::Indicator { alias, .. } => data
                .indicator_series_slice(alias)
                .ok_or_else(|| StrategyError::MissingIndicator {
                    alias: alias.clone(),
                    timeframe: source_timeframe.clone(),
                }),
            DataSeriesSource::Price { field, .. } => {
                data.price_series_slice(field)
                    .ok_or_else(|| StrategyError::MissingPriceSeries {
                        field: field.clone(),
                        timeframe: source_timeframe.clone(),
                    })
            }
            DataSeriesSource::Custom { key, .. } => {
                data.custom_series_slice(key)
                    .ok_or_else(|| StrategyError::MissingCustomData {
                        key: key.clone(),
                        timeframe: source_timeframe.clone(),
                    })
            }
        }
    }

    pub fn prepare_condition_input<'a>(
        &'a self,
        condition: &'a PreparedCondition,
    ) -> Result<ConditionInputData<'a>, StrategyError> {
        let timeframe = &condition.timeframe;
        match &condition.input {
            ConditionInputSpec::Single { source } => {
                let series = self.resolve_series(timeframe, source)?;
                Ok(ConditionInputData::single(series))
            }
            ConditionInputSpec::Dual { primary, secondary } => {
                let primary_series = self.resolve_series(timeframe, primary)?;
                let secondary_series = self.resolve_series(timeframe, secondary)?;
                Ok(ConditionInputData::dual(primary_series, secondary_series))
            }
            ConditionInputSpec::DualWithPercent {
                primary,
                secondary,
                percent,
            } => {
                let primary_series = self.resolve_series(timeframe, primary)?;
                let secondary_series = self.resolve_series(timeframe, secondary)?;
                Ok(ConditionInputData::dual_with_percent(
                    primary_series,
                    secondary_series,
                    *percent,
                ))
            }
            ConditionInputSpec::Range {
                source,
                lower,
                upper,
            } => {
                let data_series = self.resolve_series(timeframe, source)?;
                let lower_series = self.resolve_series(timeframe, lower)?;
                let upper_series = self.resolve_series(timeframe, upper)?;
                Ok(ConditionInputData::range(
                    data_series,
                    lower_series,
                    upper_series,
                ))
            }
            ConditionInputSpec::Indexed {
                source,
                index_offset,
            } => {
                let series = self.resolve_series(timeframe, source)?;
                let data = self.timeframe(timeframe)?;
                let base_index = data.index();
                let index = if *index_offset > base_index {
                    0
                } else {
                    base_index - *index_offset
                };
                Ok(ConditionInputData::indexed(series, index))
            }
            ConditionInputSpec::Ohlc => {
                let data = self.timeframe(timeframe)?;
                let ohlc = data
                    .ohlc_ref()
                    .ok_or_else(|| StrategyError::MissingPriceSeries {
                        field: PriceField::Close,
                        timeframe: timeframe.clone(),
                    })?;
                Ok(ConditionInputData::ohlc(ohlc))
            }
        }
    }

    pub fn prepare_condition_input_with_index_offset<'a>(
        &'a self,
        condition: &'a PreparedCondition,
        index_offset: usize,
    ) -> Result<ConditionInputData<'a>, StrategyError> {
        let timeframe = &condition.timeframe;
        match &condition.input {
            ConditionInputSpec::Single { source } => {
                let series = self.resolve_series(timeframe, source)?;
                Ok(ConditionInputData::single(series))
            }
            ConditionInputSpec::Dual { primary, secondary } => {
                let primary_series = self.resolve_series(timeframe, primary)?;
                let secondary_series = self.resolve_series(timeframe, secondary)?;
                Ok(ConditionInputData::dual(primary_series, secondary_series))
            }
            ConditionInputSpec::DualWithPercent {
                primary,
                secondary,
                percent,
            } => {
                let primary_series = self.resolve_series(timeframe, primary)?;
                let secondary_series = self.resolve_series(timeframe, secondary)?;
                Ok(ConditionInputData::dual_with_percent(
                    primary_series,
                    secondary_series,
                    *percent,
                ))
            }
            ConditionInputSpec::Range {
                source,
                lower,
                upper,
            } => {
                let data_series = self.resolve_series(timeframe, source)?;
                let lower_series = self.resolve_series(timeframe, lower)?;
                let upper_series = self.resolve_series(timeframe, upper)?;
                Ok(ConditionInputData::range(
                    data_series,
                    lower_series,
                    upper_series,
                ))
            }
            ConditionInputSpec::Indexed {
                source,
                index_offset: spec_offset,
            } => {
                let series = self.resolve_series(timeframe, source)?;
                let data = self.timeframe(timeframe)?;
                let base_index = data.index().saturating_sub(index_offset);
                let index = if *spec_offset > base_index {
                    0
                } else {
                    base_index - *spec_offset
                };
                Ok(ConditionInputData::indexed(series, index))
            }
            ConditionInputSpec::Ohlc => {
                let data = self.timeframe(timeframe)?;
                let ohlc = data
                    .ohlc_ref()
                    .ok_or_else(|| StrategyError::MissingPriceSeries {
                        field: PriceField::Close,
                        timeframe: timeframe.clone(),
                    })?;
                Ok(ConditionInputData::ohlc(ohlc))
            }
        }
    }
}

impl StrategyContext {
    pub fn active_position(&self) -> Option<&ActivePosition> {
        self.active_positions.values().next()
    }

    pub fn active_positions(&self) -> &HashMap<String, ActivePosition> {
        &self.active_positions
    }

    pub fn set_active_positions(&mut self, book: PositionBook) {
        let entries = book.entries();
        self.active_positions = entries
            .iter()
            .map(|position| (position.id.clone(), position.clone()))
            .collect::<HashMap<_, _>>();
    }

    pub fn upsert_active_position(&mut self, position: ActivePosition) {
        self.active_positions.insert(position.id.clone(), position);
    }

    pub fn remove_active_position(&mut self, position_id: &str) -> Option<ActivePosition> {
        self.active_positions.remove(position_id)
    }
}
