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
struct PriceData {
    open: Option<Arc<Vec<f32>>>,
    high: Option<Arc<Vec<f32>>>,
    low: Option<Arc<Vec<f32>>>,
    close: Option<Arc<Vec<f32>>>,
    volume: Option<Arc<Vec<f32>>>,
}

impl PriceData {
    fn new() -> Self {
        Self {
            open: None,
            high: None,
            low: None,
            close: None,
            volume: None,
        }
    }

    fn get(&self, field: &PriceField) -> Option<&[f32]> {
        match field {
            PriceField::Open => self.open.as_deref().map(|v| v.as_slice()),
            PriceField::High => self.high.as_deref().map(|v| v.as_slice()),
            PriceField::Low => self.low.as_deref().map(|v| v.as_slice()),
            PriceField::Close => self.close.as_deref().map(|v| v.as_slice()),
            PriceField::Volume => self.volume.as_deref().map(|v| v.as_slice()),
        }
    }

    fn set(&mut self, field: PriceField, series: Arc<Vec<f32>>) {
        match field {
            PriceField::Open => self.open = Some(series),
            PriceField::High => self.high = Some(series),
            PriceField::Low => self.low = Some(series),
            PriceField::Close => self.close = Some(series),
            PriceField::Volume => self.volume = Some(series),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TimeframeData {
    timeframe: TimeFrame,
    symbol: Option<Symbol>,
    index: usize,
    prices: PriceData,
    indicators: Vec<Arc<Vec<f32>>>,
    indicator_alias_order: Vec<String>,
    custom: HashMap<String, Arc<Vec<f32>>>,
    condition_results: Vec<Option<Arc<ConditionResultData>>>,
    condition_id_to_index: HashMap<String, usize>,
    ohlc: Option<Arc<OHLCData>>,
    timestamps: Option<Arc<Vec<i64>>>,
    /// Служебные индикаторы для стоп-обработчиков (ATR, MINFOR, MAXFOR и т.д.)
    /// Эти индикаторы не являются частью торговой логики стратегии
    auxiliary_indicators: HashMap<String, Arc<Vec<f32>>>,
}

impl TimeframeData {
    pub fn new(timeframe: TimeFrame) -> Self {
        Self {
            timeframe,
            symbol: None,
            index: 0,
            prices: PriceData::new(),
            indicators: Vec::with_capacity(16),
            indicator_alias_order: Vec::with_capacity(16),
            custom: HashMap::with_capacity(8),
            condition_results: Vec::new(),
            condition_id_to_index: HashMap::new(),
            ohlc: None,
            timestamps: None,
            auxiliary_indicators: HashMap::with_capacity(4),
        }
    }

    pub fn ensure_condition_capacity(&mut self, capacity: usize) {
        if self.condition_results.len() < capacity {
            self.condition_results.resize(capacity, None);
        }
    }

    pub fn register_condition_id(&mut self, condition_id: String, index: usize) {
        self.condition_id_to_index.insert(condition_id, index);
        if self.condition_results.len() <= index {
            self.condition_results.resize(index + 1, None);
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
        data.prices.set(PriceField::Open, open);
        data.prices.set(PriceField::High, high);
        data.prices.set(PriceField::Low, low);
        data.prices.set(PriceField::Close, close);
        if let Some(volume_vec) = volume {
            data.prices.set(PriceField::Volume, Arc::new(volume_vec));
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
        self.prices.set(field, Arc::new(series));
    }

    pub fn insert_price_series_arc(&mut self, field: PriceField, series: Arc<Vec<f32>>) {
        self.prices.set(field, series);
    }

    pub fn insert_indicator(&mut self, alias: impl Into<String>, series: Vec<f32>) {
        self.insert_indicator_arc(alias, Arc::new(series));
    }

    pub fn insert_indicator_arc(&mut self, alias: impl Into<String>, series: Arc<Vec<f32>>) {
        let alias = alias.into();
        if let Some(index) = self.indicator_alias_order.iter().position(|a| a == &alias) {
            self.indicators[index] = series;
        } else {
            self.indicator_alias_order.push(alias);
            self.indicators.push(series);
        }
    }

    pub fn indicator_by_index(&self, index: usize) -> Option<&[f32]> {
        self.indicators
            .get(index)
            .map(|data| data.as_ref().as_slice())
    }

    fn find_indicator_index(&self, alias: &str) -> Option<usize> {
        self.indicator_alias_order.iter().position(|a| a == alias)
    }

    pub fn indicator_index(&self, alias: &str) -> Option<usize> {
        self.find_indicator_index(alias)
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
        self.prices.set(PriceField::Open, open);
        self.prices.set(PriceField::High, high);
        self.prices.set(PriceField::Low, low);
        self.prices.set(PriceField::Close, close);
        if let Some(volume_vec) = volume {
            self.prices.set(PriceField::Volume, Arc::new(volume_vec));
        }
        self.timestamps = timestamps.map(|vec| Arc::new(vec));
    }

    pub fn price_series_slice(&self, field: &PriceField) -> Option<&[f32]> {
        self.prices.get(field)
    }

    pub fn indicator_series_slice(&self, alias: &str) -> Option<&[f32]> {
        self.find_indicator_index(alias)
            .and_then(|index| self.indicators.get(index))
            .map(|data| data.as_ref().as_slice())
    }

    pub fn indicator_value_at(&self, alias: &str, candle_index: usize) -> Option<f32> {
        self.find_indicator_index(alias)
            .and_then(|index| self.indicators.get(index))
            .and_then(|data| data.get(candle_index).copied())
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

    // ===== Методы для служебных индикаторов (auxiliary) =====

    /// Добавить служебный индикатор (для стоп-обработчиков)
    pub fn insert_auxiliary(&mut self, alias: impl Into<String>, series: Vec<f32>) {
        self.auxiliary_indicators
            .insert(alias.into(), Arc::new(series));
    }

    /// Добавить служебный индикатор (Arc версия)
    pub fn insert_auxiliary_arc(&mut self, alias: impl Into<String>, series: Arc<Vec<f32>>) {
        self.auxiliary_indicators.insert(alias.into(), series);
    }

    /// Получить слайс служебного индикатора
    pub fn auxiliary_series_slice(&self, alias: &str) -> Option<&[f32]> {
        self.auxiliary_indicators
            .get(alias)
            .map(|data| data.as_ref().as_slice())
    }

    /// Получить значение служебного индикатора по индексу
    pub fn auxiliary_value_at(&self, alias: &str, candle_index: usize) -> Option<f32> {
        self.auxiliary_indicators
            .get(alias)
            .and_then(|data| data.get(candle_index).copied())
    }

    /// Проверить наличие служебного индикатора
    pub fn has_auxiliary(&self, alias: &str) -> bool {
        self.auxiliary_indicators.contains_key(alias)
    }

    /// Удалить служебный индикатор
    pub fn remove_auxiliary(&mut self, alias: &str) -> Option<Arc<Vec<f32>>> {
        self.auxiliary_indicators.remove(alias)
    }

    /// Очистить все служебные индикаторы
    pub fn clear_auxiliary(&mut self) {
        self.auxiliary_indicators.clear();
    }

    /// Получить список всех алиасов служебных индикаторов
    pub fn auxiliary_aliases(&self) -> Vec<&String> {
        self.auxiliary_indicators.keys().collect()
    }

    pub fn insert_condition_result(
        &mut self,
        condition_id: impl Into<String>,
        result: ConditionResultData,
    ) {
        let condition_id = condition_id.into();
        if let Some(&index) = self.condition_id_to_index.get(&condition_id) {
            if self.condition_results.len() <= index {
                self.condition_results.resize(index + 1, None);
            }
            self.condition_results[index] = Some(Arc::new(result));
        }
    }

    pub fn insert_condition_result_arc(
        &mut self,
        condition_id: impl Into<String>,
        result: Arc<ConditionResultData>,
    ) {
        let condition_id = condition_id.into();
        if let Some(&index) = self.condition_id_to_index.get(&condition_id) {
            if self.condition_results.len() <= index {
                self.condition_results.resize(index + 1, None);
            }
            self.condition_results[index] = Some(result);
        }
    }

    pub fn insert_condition_result_by_index(
        &mut self,
        index: usize,
        result: Arc<ConditionResultData>,
    ) {
        if self.condition_results.len() <= index {
            self.condition_results.resize(index + 1, None);
        }
        self.condition_results[index] = Some(result);
    }

    pub fn condition_result(&self, condition_id: &str) -> Option<&ConditionResultData> {
        self.condition_id_to_index
            .get(condition_id)
            .and_then(|&index| self.condition_results.get(index))
            .and_then(|opt| opt.as_ref())
            .map(|arc| arc.as_ref())
    }

    pub fn condition_result_by_index(&self, index: usize) -> Option<&ConditionResultData> {
        self.condition_results
            .get(index)
            .and_then(|opt| opt.as_ref())
            .map(|arc| arc.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct StrategyContext {
    timeframes: Vec<TimeframeData>,
    timeframe_order: Vec<TimeFrame>,
    pub user_settings: StrategyUserSettings,
    pub metadata: HashMap<String, String>,
    pub runtime_parameters: StrategyParameterMap,
    active_positions: HashMap<String, ActivePosition>,
}

impl StrategyContext {
    pub fn new() -> Self {
        Self {
            timeframes: Vec::with_capacity(4),
            timeframe_order: Vec::with_capacity(4),
            user_settings: HashMap::with_capacity(8),
            metadata: HashMap::with_capacity(8),
            runtime_parameters: HashMap::with_capacity(16),
            active_positions: HashMap::with_capacity(8),
        }
    }

    pub fn with_timeframes(timeframes: HashMap<TimeFrame, TimeframeData>) -> Self {
        let timeframes_len = timeframes.len();
        let mut timeframe_vec = Vec::with_capacity(timeframes_len);
        let mut timeframe_order = Vec::with_capacity(timeframes_len);

        for (timeframe, data) in timeframes {
            timeframe_order.push(timeframe.clone());
            timeframe_vec.push(data);
        }

        Self {
            timeframes: timeframe_vec,
            timeframe_order,
            user_settings: HashMap::with_capacity(8),
            metadata: HashMap::with_capacity(8),
            runtime_parameters: HashMap::with_capacity(16),
            active_positions: HashMap::with_capacity(timeframes_len.max(8)),
        }
    }

    pub fn with_timeframes_ordered(
        timeframe_order: &[TimeFrame],
        timeframes: HashMap<TimeFrame, TimeframeData>,
    ) -> Self {
        let mut timeframe_vec = Vec::with_capacity(timeframe_order.len());
        let mut order = Vec::with_capacity(timeframe_order.len());

        for timeframe in timeframe_order {
            if let Some(data) = timeframes.get(timeframe) {
                order.push(timeframe.clone());
                timeframe_vec.push(data.clone());
            }
        }

        let timeframes_len = timeframe_vec.len();
        Self {
            timeframes: timeframe_vec,
            timeframe_order: order,
            user_settings: HashMap::with_capacity(8),
            metadata: HashMap::with_capacity(8),
            runtime_parameters: HashMap::with_capacity(16),
            active_positions: HashMap::with_capacity(timeframes_len.max(8)),
        }
    }

    fn find_timeframe_index(&self, timeframe: &TimeFrame) -> Option<usize> {
        self.timeframe_order.iter().position(|tf| tf == timeframe)
    }

    pub fn insert_timeframe(&mut self, timeframe: TimeFrame, data: TimeframeData) {
        if let Some(index) = self.find_timeframe_index(&timeframe) {
            self.timeframes[index] = data;
        } else {
            self.timeframe_order.push(timeframe.clone());
            self.timeframes.push(data);
        }
    }

    pub fn timeframe(&self, timeframe: &TimeFrame) -> Result<&TimeframeData, StrategyError> {
        self.find_timeframe_index(timeframe)
            .and_then(|index| self.timeframes.get(index))
            .ok_or_else(|| StrategyError::MissingTimeframe(timeframe.clone()))
    }

    pub fn timeframe_mut(
        &mut self,
        timeframe: &TimeFrame,
    ) -> Result<&mut TimeframeData, StrategyError> {
        let index = self
            .find_timeframe_index(timeframe)
            .ok_or_else(|| StrategyError::MissingTimeframe(timeframe.clone()))?;
        self.timeframes
            .get_mut(index)
            .ok_or_else(|| StrategyError::MissingTimeframe(timeframe.clone()))
    }

    pub fn timeframe_by_index(&self, index: usize) -> Result<&TimeframeData, StrategyError> {
        self.timeframes.get(index).ok_or_else(|| {
            StrategyError::MissingTimeframe(
                self.timeframe_order
                    .get(index)
                    .cloned()
                    .unwrap_or(TimeFrame::Minutes(1)),
            )
        })
    }

    pub fn timeframe_by_index_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut TimeframeData, StrategyError> {
        self.timeframes.get_mut(index).ok_or_else(|| {
            StrategyError::MissingTimeframe(
                self.timeframe_order
                    .get(index)
                    .cloned()
                    .unwrap_or(TimeFrame::Minutes(1)),
            )
        })
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

    pub fn active_positions_mut(&mut self) -> &mut HashMap<String, ActivePosition> {
        &mut self.active_positions
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

    pub fn update_position_metadata(
        &mut self,
        position_id: &str,
        updates: HashMap<String, String>,
    ) {
        if let Some(position) = self.active_positions.get_mut(position_id) {
            position.metadata.extend(updates);
        }
    }
}
