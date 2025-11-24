use crate::data_model::vector_ops::unsafe_ops;
use crate::indicators::{
    base::{
        Indicator, OHLCIndicator, OscillatorIndicator, OverboughtOversoldZones, SimpleIndicator,
        TrendDirection, TrendIndicator, VolatilityIndicator,
    },
    parameters::*,
    types::{
        IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet, ParameterType,
    },
};

/// Диапазон оптимизации параметра
#[derive(Debug, Clone)]
pub struct OptimizationRange {
    pub start: f32,
    pub end: f32,
    pub step: f32,
}

impl OptimizationRange {
    pub fn new(start: f32, end: f32, step: f32) -> Self {
        Self { start, end, step }
    }
}

/// Возвращает диапазон оптимизации для параметра индикатора
///
/// # Аргументы
/// * `indicator_name` - имя индикатора (например, "SMA", "RSI", "BBUpper")
/// * `param_name` - имя параметра (например, "period", "deviation", "coeff_atr")
/// * `param_type` - тип параметра
///
/// # Возвращает
/// Опциональный диапазон оптимизации (start, end, step)
pub fn get_optimization_range(
    indicator_name: &str,
    param_name: &str,
    param_type: &ParameterType,
) -> Option<OptimizationRange> {
    ParameterPresets::get_range_for_parameter(indicator_name, param_name, param_type).map(|range| {
        let step = match param_type {
            ParameterType::Period => 10.0,
            ParameterType::Multiplier
                if matches!(
                    param_name.to_lowercase().as_str(),
                    "coeff_atr" | "atr_multiplier" | "atr_coefficient"
                ) =>
            {
                0.2
            }
            ParameterType::Multiplier => 0.2,
            _ => range.step,
        };
        OptimizationRange::new(range.start, range.end, step)
    })
}

/// Возвращает диапазон оптимизации для пороговых значений осцилляторов
/// Использует централизованную систему ParameterPresets
pub fn get_oscillator_threshold_range(
    indicator_name: &str,
    param_name: &str,
) -> Option<OptimizationRange> {
    ParameterPresets::get_range_for_parameter(indicator_name, param_name, &ParameterType::Threshold)
        .map(|range| OptimizationRange::new(range.start, range.end, range.step))
}

fn adjust_period(period: usize, len: usize) -> Option<usize> {
    if len == 0 {
        None
    } else {
        Some(period.max(1).min(len))
    }
}

// ============================================================================
// ВСПОМОГАТЕЛЬНЫЕ ИНДИКАТОРЫ
// ============================================================================

/// MAXFOR - максимальное значение за период
pub struct MAXFOR {
    parameters: ParameterSet,
}

impl MAXFOR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета MAXFOR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    /// Создать MAXFOR без валидации параметров (для внутреннего использования)
    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета MAXFOR",
        ));
        Self { parameters: params }
    }
}

impl Indicator for MAXFOR {
    fn name(&self) -> &str {
        "MAXFOR"
    }
    fn description(&self) -> &str {
        "Максимальное значение за период"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut max_values = Vec::with_capacity(len);

        for i in 0..len {
            let window = period.min(i + 1);
            let start_idx = i + 1 - window;
            let end_idx = i + 1;
            let max_value = unsafe_ops::max_f32_fast(&data.high[start_idx..end_idx])
                .unwrap_or(f32::NEG_INFINITY);
            max_values.push(max_value);
        }

        Ok(max_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

/// MINFOR - минимальное значение за период
pub struct MINFOR {
    parameters: ParameterSet,
}

impl MINFOR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета MINFOR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    /// Создать MINFOR без валидации параметров (для внутреннего использования)
    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета MINFOR",
        ));
        Self { parameters: params }
    }
}

impl Indicator for MINFOR {
    fn name(&self) -> &str {
        "MINFOR"
    }
    fn description(&self) -> &str {
        "Минимальное значение за период"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut min_values = Vec::with_capacity(len);

        for i in 0..len {
            let window = period.min(i + 1);
            let start_idx = i + 1 - window;
            let end_idx = i + 1;
            let min_value =
                unsafe_ops::min_f32_fast(&data.low[start_idx..end_idx]).unwrap_or(f32::INFINITY);
            min_values.push(min_value);
        }

        Ok(min_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

/// ATR (Average True Range) - средний истинный диапазон
pub struct ATR {
    parameters: ParameterSet,
}

impl ATR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for ATR {
    fn name(&self) -> &str {
        "ATR"
    }
    fn description(&self) -> &str {
        "Average True Range - средний истинный диапазон (поддерживает Simple и OHLC данные)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Universal
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut atr_values = vec![0.0; len];

        for i in 0..len {
            let true_ranges = self.true_range_simple(data, period, i);
            let window_len = true_ranges.len().max(1) as f32;
            let atr = unsafe_ops::sum_f32_fast(&true_ranges) / window_len;
            atr_values[i] = atr;
        }

        Ok(atr_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut atr_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            atr_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let mut true_ranges = Vec::new();

            let start = i + 1 - period;
            for j in start..=i {
                let true_range = self.true_range_ohlc(data, j);
                true_ranges.push(true_range);
            }

            let atr = unsafe_ops::sum_f32_fast(&true_ranges) / period as f32;
            atr_values.push(atr);
        }

        Ok(atr_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl ATR {
    /// Вычисляет True Range по простым данным (как в any.rs)
    fn true_range_simple(&self, data: &[f32], period: usize, bar_num: usize) -> Vec<f32> {
        let mut true_ranges = Vec::new();

        if period == 0 {
            return true_ranges;
        }

        let available = bar_num + 1;
        let window = available.min(period);
        let start = bar_num + 1 - window;

        for i in start..=bar_num {
            if i > 0 {
                let true_range = (data[i] - data[i - 1]).abs();
                true_ranges.push(true_range);
            } else {
                true_ranges.push(0.0);
            }
        }

        true_ranges
    }

    /// Вычисляет True Range по OHLC данным
    fn true_range_ohlc(&self, data: &OHLCData, j: usize) -> f32 {
        let high_low = data.high[j] - data.low[j];
        let high_close_prev = if j > 0 {
            (data.high[j] - data.close[j - 1]).abs()
        } else {
            0.0
        };
        let low_close_prev = if j > 0 {
            (data.low[j] - data.close[j - 1]).abs()
        } else {
            0.0
        };

        high_low.max(high_close_prev).max(low_close_prev)
    }
}

// ATR теперь универсальный индикатор
impl VolatilityIndicator for ATR {
    fn get_volatility_level(&self, data: &[f32]) -> Result<f32, IndicatorError> {
        let values = self.calculate_simple(data)?;
        Ok(values.last().copied().unwrap_or(0.0))
    }
}

/// SuperTrend - трендовый индикатор
pub struct SuperTrend {
    parameters: ParameterSet,
}

impl SuperTrend {
    pub fn new(period: f32, coeff_atr: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "coeff_atr",
                coeff_atr,
                "Коэффициент ATR для SuperTrend",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for SuperTrend {
    fn name(&self) -> &str {
        "SuperTrend"
    }
    fn description(&self) -> &str {
        "SuperTrend - трендовый индикатор с полосами ATR (поддерживает Simple и OHLC данные)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Universal
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let atr_indicator = ATR::new(period as f32)?;
        let atr_values = atr_indicator.calculate_simple(data)?;
        let mut supertrend_values = vec![0.0; len];

        for i in 2..len {
            let atr = atr_values[i];
            let current_price = data[i];

            let upper_band = current_price + (coeff_atr * atr);
            let lower_band = current_price - (coeff_atr * atr);

            let prev_supertrend = supertrend_values[i - 1];
            let supertrend = if current_price >= prev_supertrend {
                if i > 0 && data[i - 1] < prev_supertrend {
                    lower_band
                } else {
                    if lower_band > prev_supertrend {
                        lower_band
                    } else {
                        prev_supertrend
                    }
                }
            } else if current_price < prev_supertrend {
                if i > 0 && data[i - 1] > prev_supertrend {
                    upper_band
                } else {
                    if upper_band < prev_supertrend {
                        upper_band
                    } else {
                        prev_supertrend
                    }
                }
            } else {
                prev_supertrend
            };

            supertrend_values[i] = supertrend;
        }

        Ok(supertrend_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let watr_indicator = WATR::new_unchecked(period as f32);
        let atr_values = watr_indicator.calculate_ohlc(data)?;

        let median_prices = self.calculate_median_price(data);
        let mut supertrend_values = vec![0.0; len];

        for i in 2..len {
            let atr = atr_values[i];
            let median_price = median_prices[i];

            let upper_band = median_price + (coeff_atr * atr);
            let lower_band = median_price - (coeff_atr * atr);

            let prev_supertrend = supertrend_values[i - 1];
            let current_close = data.close[i];
            let prev_close = data.close[i - 1];

            let supertrend = if current_close >= prev_supertrend {
                if prev_close < prev_supertrend {
                    lower_band
                } else if lower_band > prev_supertrend {
                    lower_band
                } else {
                    prev_supertrend
                }
            } else if current_close < prev_supertrend {
                if prev_close > prev_supertrend {
                    upper_band
                } else if upper_band < prev_supertrend {
                    upper_band
                } else {
                    prev_supertrend
                }
            } else {
                prev_supertrend
            };

            supertrend_values[i] = supertrend;
        }

        Ok(supertrend_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();
        Box::new(Self::new(period, coeff_atr).unwrap())
    }
}

impl SuperTrend {
    fn calculate_median_price(&self, data: &OHLCData) -> Vec<f32> {
        data.get_median_price()
    }
}

impl TrendIndicator for SuperTrend {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// Stochastic - стохастический осциллятор
pub struct Stochastic {
    parameters: ParameterSet,
}

impl Stochastic {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета Stochastic",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for Stochastic {
    fn name(&self) -> &str {
        "Stochastic"
    }
    fn description(&self) -> &str {
        "Stochastic - стохастический осциллятор"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillator
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let mut stochastic_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            stochastic_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let start_idx = i + 1 - period;
            let end_idx = i + 1;

            let highest_high = data.high[start_idx..end_idx]
                .iter()
                .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

            let lowest_low = data.low[start_idx..end_idx]
                .iter()
                .fold(f32::INFINITY, |a, &b| a.min(b));

            let current_close = data.close[i];

            let stochastic = if highest_high == lowest_low {
                50.0
            } else {
                ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
            };

            stochastic_values.push(stochastic);
        }

        Ok(stochastic_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl OscillatorIndicator for Stochastic {
    fn get_overbought_oversold_zones(
        &self,
        data: &[f32],
    ) -> Result<OverboughtOversoldZones, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(current_value) = values.last() {
            Ok(OverboughtOversoldZones::new(80.0, 20.0, *current_value))
        } else {
            Err(IndicatorError::CalculationError(
                "No values calculated".to_string(),
            ))
        }
    }
}

/// True Range (без параметров)
pub struct TrueRange {
    parameters: ParameterSet,
}

impl TrueRange {
    pub fn new() -> Result<Self, IndicatorError> {
        Ok(Self {
            parameters: ParameterSet::new(),
        })
    }

    /// Создать TrueRange без валидации (для внутреннего использования)
    pub fn new_unchecked() -> Self {
        Self {
            parameters: ParameterSet::new(),
        }
    }

    fn series(data: &OHLCData) -> Vec<f32> {
        let mut result = Vec::with_capacity(data.len());
        for idx in 0..data.len() {
            let high_low = data.high[idx] - data.low[idx];
            let high_close_prev = if idx > 0 {
                (data.high[idx] - data.close[idx - 1]).abs()
            } else {
                0.0
            };
            let low_close_prev = if idx > 0 {
                (data.low[idx] - data.close[idx - 1]).abs()
            } else {
                0.0
            };
            result.push(high_low.max(high_close_prev).max(low_close_prev));
        }
        result
    }
}

impl Indicator for TrueRange {
    fn name(&self) -> &str {
        "TrueRange"
    }
    fn description(&self) -> &str {
        "True Range - показатель истинного диапазона"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        1
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        Ok(Self::series(data))
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new().unwrap())
    }
}

impl VolatilityIndicator for TrueRange {
    fn get_volatility_level(&self, _data: &[f32]) -> Result<f32, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }
}

/// WATR (Weighted Average True Range на основе WMA)
pub struct WATR {
    parameters: ParameterSet,
}

impl WATR {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета WATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    /// Создать WATR без валидации параметров (для внутреннего использования)
    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета WATR",
        ));
        Self { parameters: params }
    }
}

impl Indicator for WATR {
    fn name(&self) -> &str {
        "WATR"
    }
    fn description(&self) -> &str {
        "Weighted Average True Range - средний истинный диапазон на основе WMA"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let tr_indicator = TrueRange::new_unchecked();
        let true_ranges = tr_indicator.calculate_ohlc(data)?;
        let wma_indicator = WMA::new_unchecked(period as f32);
        wma_indicator.calculate_simple(&true_ranges)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl VolatilityIndicator for WATR {
    fn get_volatility_level(&self, _data: &[f32]) -> Result<f32, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }
}

/// VTRAND - среднее между MAXFOR и MINFOR
pub struct VTRAND {
    parameters: ParameterSet,
}

impl VTRAND {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета VTRAND",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for VTRAND {
    fn name(&self) -> &str {
        "VTRAND"
    }
    fn description(&self) -> &str {
        "VTRAND - среднее между максимальным и минимальным значением за период"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        // Создаем временные индикаторы для расчета
        let max_indicator = MAXFOR::new_unchecked(period as f32);
        let min_indicator = MINFOR::new_unchecked(period as f32);

        let max_result = max_indicator.calculate_ohlc(data)?;
        let min_result = min_indicator.calculate_ohlc(data)?;

        // VTRAND = (MAXFOR + MINFOR) / 2
        let vtrand_values: Vec<f32> = max_result
            .into_iter()
            .zip(min_result)
            .map(|(max_val, min_val)| (max_val + min_val) / 2.0)
            .collect();

        Ok(vtrand_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

// ============================================================================
// ПРОСТЫЕ ИНДИКАТОРЫ
// ============================================================================

/// SMA (Simple Moving Average) - простое скользящее среднее
pub struct SMA {
    parameters: ParameterSet,
}

impl SMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    /// Создать SMA без валидации параметров (для внутреннего использования)
    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета SMA",
        ));
        Self { parameters: params }
    }
}

impl Indicator for SMA {
    fn name(&self) -> &str {
        "SMA"
    }
    fn description(&self) -> &str {
        "Simple Moving Average - простое скользящее среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut sma_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            sma_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let sum: f32 = unsafe_ops::sum_f32_fast(&data[start..=i]);
            sma_values.push(sum / current_window as f32);
        }

        Ok(sma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для SMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SMA {}

impl TrendIndicator for SMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// EMA (Exponential Moving Average) - экспоненциальное скользящее среднее
pub struct EMA {
    parameters: ParameterSet,
}

impl EMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    /// Создать EMA без валидации параметров (для внутреннего использования)
    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета EMA",
        ));
        Self { parameters: params }
    }
}

impl Indicator for EMA {
    fn name(&self) -> &str {
        "EMA"
    }
    fn description(&self) -> &str {
        "Exponential Moving Average - экспоненциальное скользящее среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap();
        if period <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "EMA period must be greater than 0".to_string(),
            ));
        }

        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }

        // Формула соответствует EMA_MT из EMA.cs (используется в GenEMA)
        // EMA_MT: array[0] = src[0], array[i] = array[i - 1] + num * (src[i] - array[i - 1])
        let multiplier = 2.0 / (1.0 + period);
        let mut ema_values = Vec::with_capacity(len);

        // Первое значение = первое значение данных
        ema_values.push(data[0]);

        // Остальные значения рассчитываются по формуле EMA
        for i in 1..len {
            let prev_ema = ema_values[i - 1];
            let price = data[i];
            ema_values.push(prev_ema + multiplier * (price - prev_ema));
        }

        Ok(ema_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для EMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for EMA {}

impl TrendIndicator for EMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// RSI (Relative Strength Index) - индекс относительной силы
pub struct RSI {
    parameters: ParameterSet,
}

impl RSI {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета RSI",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        "RSI"
    }
    fn description(&self) -> &str {
        "Relative Strength Index - индекс относительной силы"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillator
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let mut period = self.parameters.get_value("period").unwrap() as usize;
        if period == 0 {
            period = 1;
        }

        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }
        period = period.min(len);

        let mut gains = vec![0.0; len];
        let mut losses = vec![0.0; len];

        for i in 1..len {
            let change = data[i] - data[i - 1];
            if change > 0.0 {
                gains[i] = change;
            } else if change < 0.0 {
                losses[i] = -change;
            }
        }

        let ema_gains = EMA::new_unchecked(period as f32).calculate_simple(&gains)?;
        let ema_losses = EMA::new_unchecked(period as f32).calculate_simple(&losses)?;

        let mut rsi_values = vec![0.0; len];

        for i in 0..len {
            let gain = ema_gains[i];
            let loss = ema_losses[i];

            if loss == 0.0 {
                rsi_values[i] = 100.0;
            } else {
                let rs = gain / loss;
                if (rs - 1.0).abs() < f32::EPSILON {
                    rsi_values[i] = 0.0;
                } else {
                    rsi_values[i] = 100.0 - 100.0 / (1.0 + rs);
                }
            }
        }

        Ok(rsi_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для RSI используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for RSI {}

impl OscillatorIndicator for RSI {
    fn get_overbought_oversold_zones(
        &self,
        data: &[f32],
    ) -> Result<OverboughtOversoldZones, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(current_value) = values.last() {
            Ok(OverboughtOversoldZones::new(70.0, 30.0, *current_value))
        } else {
            Err(IndicatorError::CalculationError(
                "No values calculated".to_string(),
            ))
        }
    }
}

/// WMA (Weighted Moving Average) - взвешенное скользящее среднее
pub struct WMA {
    parameters: ParameterSet,
}

impl WMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета WMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    /// Создать WMA без валидации параметров (для внутреннего использования)
    pub fn new_unchecked(period: f32) -> Self {
        let mut params = ParameterSet::new();
        params.add_parameter_unchecked(create_period_parameter(
            "period",
            period,
            "Период для расчета WMA",
        ));
        Self { parameters: params }
    }
}

impl Indicator for WMA {
    fn name(&self) -> &str {
        "WMA"
    }
    fn description(&self) -> &str {
        "Weighted Moving Average - взвешенное скользящее среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut wma_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            wma_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            for (offset, value) in data[start..=i].iter().enumerate() {
                let weight = (offset + 1) as f32;
                weighted_sum += value * weight;
                weight_sum += weight;
            }

            wma_values.push(weighted_sum / weight_sum.max(1.0));
        }

        Ok(wma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для WMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for WMA {}

impl TrendIndicator for WMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// AMA (Adaptive Moving Average) - адаптивное скользящее среднее
pub struct AMA {
    parameters: ParameterSet,
}

impl AMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета AMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for AMA {
    fn name(&self) -> &str {
        "AMA"
    }
    fn description(&self) -> &str {
        "Adaptive Moving Average - адаптивное скользящее среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }

        if period == 0 {
            return Ok(vec![0.0; len]);
        }

        let mut ama_values = vec![0.0; len];
        let copy_limit = period.saturating_mul(2).min(len.saturating_sub(1));
        for i in 0..=copy_limit {
            ama_values[i] = data[i];
        }

        let mut ama_prev = if len < period || period + 1 >= len {
            0.0
        } else {
            data[period + 1]
        };

        for j in (period + 2)..len {
            if j < period || j >= len {
                continue;
            }

            let diff = (data[j] - data[j - period]).abs();
            let mut denom = 1e-9_f32;

            for k in 0..period {
                let idx = j - k;
                let prev_idx = idx - 1;
                denom += (data[idx] - data[prev_idx]).abs();
            }

            let efficiency = if denom <= 0.0 { 0.0 } else { diff / denom };
            let x = efficiency * 0.60215 + 0.06452;
            let smoothing = x * x;
            ama_prev = ama_prev + smoothing * (data[j] - ama_prev);
            ama_values[j] = ama_prev;
        }

        Ok(ama_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для AMMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for AMA {}

impl TrendIndicator for AMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// ZLEMA (Zero-Lag Exponential Moving Average) - экспоненциальное скользящее среднее с нулевым лагом
pub struct ZLEMA {
    parameters: ParameterSet,
}

impl ZLEMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ZLEMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for ZLEMA {
    fn name(&self) -> &str {
        "ZLEMA"
    }
    fn description(&self) -> &str {
        "Zero-Lag Exponential Moving Average - экспоненциальное скользящее среднее с нулевым лагом"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }

        if period == 0 {
            return Ok(vec![0.0; len]);
        }

        let alpha = 2.0 / (period as f32 + 1.0);
        let lag = (period.saturating_sub(1)) / 2;
        let mut zlema_values = vec![0.0; len];

        for i in lag..len {
            let prev_zlema = if i == 0 {
                data[0]
            } else {
                zlema_values[i.saturating_sub(1)]
            };

            let reference_index = if i >= lag { i - lag } else { 0 };
            let current_price = data[i];
            let reference_price = data[reference_index];
            let ema_input = current_price + (current_price - reference_price);

            zlema_values[i] = prev_zlema + alpha * (ema_input - prev_zlema);
        }

        Ok(zlema_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для ZLEMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for ZLEMA {}

impl TrendIndicator for ZLEMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// GEOMEAN (Geometric Mean) - геометрическое среднее
pub struct GEOMEAN {
    parameters: ParameterSet,
}

impl GEOMEAN {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета GEOMEAN",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for GEOMEAN {
    fn name(&self) -> &str {
        "GEOMEAN"
    }
    fn description(&self) -> &str {
        "Geometric Mean - геометрическое среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut geomean_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            geomean_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let product: f32 = data[start..=i].iter().product();
            let geomean = if current_window == 0 {
                0.0
            } else {
                product.powf(1.0 / current_window as f32)
            };
            geomean_values.push(geomean);
        }

        Ok(geomean_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для GEOMEAN используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for GEOMEAN {}

impl TrendIndicator for GEOMEAN {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// AMMA (Arithmetic Mean of Moving Averages) - арифметическое среднее скользящих средних
pub struct AMMA {
    parameters: ParameterSet,
}

impl AMMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета AMMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for AMMA {
    fn name(&self) -> &str {
        "AMMA"
    }
    fn description(&self) -> &str {
        "Arithmetic Mean of Moving Averages - арифметическое среднее скользящих средних"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        if len == 0 {
            return Ok(Vec::new());
        }

        let window_double = (period.saturating_mul(2)).max(1).min(len);
        let mut amma_values = Vec::with_capacity(len);

        for _ in 0..window_double.saturating_sub(1) {
            amma_values.push(0.0);
        }

        for i in window_double - 1..len {
            let current_window = (period.saturating_mul(2)).min(i + 1);
            let start = i + 1 - current_window;
            let slice = &data[start..=i];

            let sma1 = SMA::new_unchecked(period as f32).calculate_simple(slice)?;
            let sma2 =
                SMA::new_unchecked((period.saturating_mul(2)) as f32).calculate_simple(slice)?;

            let sma1_value = *sma1.last().unwrap_or(&0.0);
            let sma2_value = *sma2.last().unwrap_or(&0.0);

            let amma = (sma1_value + sma2_value) / 2.0;
            amma_values.push(amma);
        }

        Ok(amma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для AMMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for AMMA {}

impl TrendIndicator for AMMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// SQWMA (Square Root Weighted Moving Average) - квадратично-взвешенное скользящее среднее
pub struct SQWMA {
    parameters: ParameterSet,
}

impl SQWMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SQWMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for SQWMA {
    fn name(&self) -> &str {
        "SQWMA"
    }
    fn description(&self) -> &str {
        "Square Root Weighted Moving Average - квадратично-взвешенное скользящее среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut sqwma_values = Vec::with_capacity(len);

        let period_f = period as f32;
        let num = period_f * (period_f - 1.0) / 2.0;
        let num2 = period_f * (period_f - 1.0) * (2.0 * period_f - 1.0) / 6.0;

        for i in 0..len {
            if i < period {
                sqwma_values.push(data[i]);
                continue;
            }

            let mut sum = 0.0;
            let mut weighted_sum = 0.0;

            for j in 0..period {
                let value = data[i - j];
                let j_f = j as f32;
                sum += value;
                weighted_sum += value * j_f;
            }

            let denom = num2 * period_f - num * num;
            let slope = if denom.abs() < f32::EPSILON {
                0.0
            } else {
                (weighted_sum * period_f - num * sum) / denom
            };
            let intercept = (sum - num * slope) / period_f;
            sqwma_values.push(intercept);
        }

        Ok(sqwma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для SQWMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SQWMA {}

impl TrendIndicator for SQWMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// SINEWMA (Sine Weighted Moving Average) - синусоидально-взвешенное скользящее среднее
pub struct SINEWMA {
    parameters: ParameterSet,
}

impl SINEWMA {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SINEWMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for SINEWMA {
    fn name(&self) -> &str {
        "SINEWMA"
    }
    fn description(&self) -> &str {
        "Sine Weighted Moving Average - синусоидально-взвешенное скользящее среднее"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut sinewma_values = Vec::with_capacity(len);

        for i in 0..len {
            if i < period {
                sinewma_values.push(data[i]);
                continue;
            }

            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            let denom = period as f32 + 1.0;

            for j in 0..period.saturating_sub(1) {
                let weight = (std::f32::consts::PI * (j as f32 + 1.0) / denom).sin();
                let value = data[i - j];
                weighted_sum += value * weight;
                weight_sum += weight;
            }

            if weight_sum > 0.0 {
                sinewma_values.push(weighted_sum / weight_sum);
            } else {
                sinewma_values.push(0.0);
            }
        }

        Ok(sinewma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для SINEWMA используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SINEWMA {}

impl TrendIndicator for SINEWMA {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

/// TPBF (Three Pole Butterworth Filter) - трехполюсный фильтр Баттерворта
pub struct TPBF {
    parameters: ParameterSet,
}

impl TPBF {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета TPBF",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }

    fn filter(values: &[f32], period: usize) -> Vec<f32> {
        let len = values.len();
        if len == 0 {
            return Vec::new();
        }
        if period == 0 {
            return values.to_vec();
        }

        let pi = std::f32::consts::PI;
        let period_f = period as f32;
        let exp_term = (-pi / period_f).exp();
        let exp_term_sq = exp_term * exp_term;
        let cos_term = (pi * 3.0_f32.sqrt() / period_f).cos();

        let coef1 = (1.0 - 2.0 * exp_term * cos_term + exp_term_sq) * (1.0 - exp_term_sq) / 8.0;
        let coef2 = 2.0 * exp_term * cos_term + exp_term_sq;
        let coef3 = -(exp_term_sq + 2.0 * exp_term.powi(3) * cos_term);
        let coef4 = exp_term_sq * exp_term_sq;

        let mut result = vec![0.0; len];

        for i in 0..len {
            if i < 4 {
                result[i] = values[i];
            } else {
                let price_i = values[i];
                let price_1 = values[i - 1];
                let price_2 = values[i - 2];
                let price_3 = values[i - 3];

                result[i] = coef1 * (price_i + 3.0 * (price_1 + price_2) + price_3)
                    + coef2 * result[i - 1]
                    + coef3 * result[i - 2]
                    + coef4 * result[i - 3];
            }
        }

        result
    }
}

impl Indicator for TPBF {
    fn name(&self) -> &str {
        "TPBF"
    }
    fn description(&self) -> &str {
        "Three Pole Butterworth Filter - трехполюсный фильтр Баттерворта"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        Ok(Self::filter(data, period))
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let median_prices = data
            .high
            .iter()
            .zip(&data.low)
            .map(|(&h, &l)| (h + l) / 2.0)
            .collect::<Vec<_>>();
        Ok(Self::filter(&median_prices, period))
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for TPBF {}

impl TrendIndicator for TPBF {
    fn get_trend_direction(&self, data: &[f32]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data)?;
        if let Some(last_value) = values.last() {
            if let Some(prev_value) = values.get(values.len().saturating_sub(2)) {
                if last_value > prev_value {
                    Ok(TrendDirection::Up)
                } else if last_value < prev_value {
                    Ok(TrendDirection::Down)
                } else {
                    Ok(TrendDirection::Sideways)
                }
            } else {
                Ok(TrendDirection::Unknown)
            }
        } else {
            Ok(TrendDirection::Unknown)
        }
    }
}

// ============================================================================
// Bollinger Bands Components
// ============================================================================

/// Bollinger Bands Middle Line (SMA)
pub struct BBMiddle {
    parameters: ParameterSet,
}

impl BBMiddle {
    pub fn new(period: f32, deviation: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "deviation",
                deviation,
                "Стандартное отклонение множитель",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for BBMiddle {
    fn name(&self) -> &str {
        "BBMiddle"
    }
    fn description(&self) -> &str {
        "Bollinger Bands Middle Line (SMA)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut sma_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            sma_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let sum: f32 = data[start..=i].iter().sum();
            let sma = sum / current_window as f32;
            sma_values.push(sma);
        }

        Ok(sma_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для BB используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let deviation = self.parameters.get_value("deviation").unwrap();
        Box::new(Self::new(period, deviation).unwrap())
    }
}

impl SimpleIndicator for BBMiddle {}

/// Bollinger Bands Upper Line
pub struct BBUpper {
    parameters: ParameterSet,
}

impl BBUpper {
    pub fn new(period: f32, deviation: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "deviation",
                deviation,
                "Стандартное отклонение множитель",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for BBUpper {
    fn name(&self) -> &str {
        "BBUpper"
    }
    fn description(&self) -> &str {
        "Bollinger Bands Upper Line (SMA + deviation)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let deviation = self.parameters.get_value("deviation").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut upper_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            upper_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let end = i + 1;
            let window = &data[start..end];
            let sma: f32 = unsafe_ops::sum_f32_fast(window) / current_window as f32;

            let variance: f32 =
                unsafe_ops::sum_sq_diff_f32_fast(window, sma) / current_window as f32;
            let std_dev = variance.sqrt();

            let upper = sma + (deviation * std_dev);
            upper_values.push(upper);
        }

        Ok(upper_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для BB используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let deviation = self.parameters.get_value("deviation").unwrap();
        Box::new(Self::new(period, deviation).unwrap())
    }
}

impl SimpleIndicator for BBUpper {}

/// Bollinger Bands Lower Line
pub struct BBLower {
    parameters: ParameterSet,
}

impl BBLower {
    pub fn new(period: f32, deviation: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета SMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "deviation",
                deviation,
                "Стандартное отклонение множитель",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for BBLower {
    fn name(&self) -> &str {
        "BBLower"
    }
    fn description(&self) -> &str {
        "Bollinger Bands Lower Line (SMA - deviation)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let deviation = self.parameters.get_value("deviation").unwrap();
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut lower_values = Vec::with_capacity(len);

        for _ in 0..period.saturating_sub(1) {
            lower_values.push(0.0);
        }

        let start_index = period.saturating_sub(1);
        for i in start_index..len {
            let current_window = period.min(i + 1);
            let start = i + 1 - current_window;
            let end = i + 1;
            let window = &data[start..end];
            let sma: f32 = unsafe_ops::sum_f32_fast(window) / current_window as f32;

            let variance: f32 =
                unsafe_ops::sum_sq_diff_f32_fast(window, sma) / current_window as f32;
            let std_dev = variance.sqrt();

            let lower = sma - (deviation * std_dev);
            lower_values.push(lower);
        }

        Ok(lower_values)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        // Для BB используем close цены
        self.calculate_simple(&data.close)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let deviation = self.parameters.get_value("deviation").unwrap();
        Box::new(Self::new(period, deviation).unwrap())
    }
}

impl SimpleIndicator for BBLower {}

// ============================================================================
// Keltner Channel Components
// ============================================================================

/// Keltner Channel Middle Line
pub struct KCMiddle {
    parameters: ParameterSet,
}

impl KCMiddle {
    pub fn new(period: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for KCMiddle {
    fn name(&self) -> &str {
        "KCMiddle"
    }
    fn description(&self) -> &str {
        "Keltner Channel Middle Line (EMA)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let len = data.len();
        let Some(period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };

        let mut ema_values = Vec::with_capacity(len);
        let mut ema = 0.0;

        for i in 0..len {
            let high = data.high[i];
            let low = data.low[i];
            let close = data.close[i];

            let typical_price = (high + low + close) / 3.0;

            if i == 0 {
                ema = typical_price;
            } else {
                let multiplier = 2.0 / (period as f32 + 1.0);
                ema = (typical_price * multiplier) + (ema * (1.0 - multiplier));
            }

            ema_values.push(ema);
        }

        Ok(ema_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        Box::new(Self::new(period).unwrap())
    }
}

impl OHLCIndicator for KCMiddle {}

/// Keltner Channel Upper Line
pub struct KCUpper {
    parameters: ParameterSet,
}

impl KCUpper {
    pub fn new(period: f32, atr_period: f32, atr_multiplier: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_period_parameter(
                "atr_period",
                atr_period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "atr_multiplier",
                atr_multiplier,
                "Множитель ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for KCUpper {
    fn name(&self) -> &str {
        "KCUpper"
    }
    fn description(&self) -> &str {
        "Keltner Channel Upper Line (EMA + ATR * multiplier)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        std::cmp::max(period, atr_period)
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();
        let len = data.len();
        let Some(ema_period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let Some(atr_period) = adjust_period(atr_period, len) else {
            return Ok(Vec::new());
        };

        let middle_indicator = KCMiddle::new(ema_period as f32).unwrap();
        let middle_values = middle_indicator.calculate_ohlc(data)?;

        let atr_indicator = ATR::new(atr_period as f32).unwrap();
        let atr_values = atr_indicator.calculate_ohlc(data)?;

        let mut upper_values = Vec::with_capacity(len);
        for i in 0..len {
            let middle = middle_values.get(i).copied().unwrap_or(0.0);
            let atr = atr_values.get(i).copied().unwrap_or(0.0);
            upper_values.push(middle + (atr * atr_multiplier));
        }

        Ok(upper_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let atr_period = self.parameters.get_value("atr_period").unwrap();
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();
        Box::new(Self::new(period, atr_period, atr_multiplier).unwrap())
    }
}

impl OHLCIndicator for KCUpper {}

/// Keltner Channel Lower Line
pub struct KCLower {
    parameters: ParameterSet,
}

impl KCLower {
    pub fn new(period: f32, atr_period: f32, atr_multiplier: f32) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета EMA",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_period_parameter(
                "atr_period",
                atr_period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter(
                "atr_multiplier",
                atr_multiplier,
                "Множитель ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

impl Indicator for KCLower {
    fn name(&self) -> &str {
        "KCLower"
    }
    fn description(&self) -> &str {
        "Keltner Channel Lower Line (EMA - ATR * multiplier)"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Channel
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        std::cmp::max(period, atr_period)
    }

    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();
        let len = data.len();
        let Some(ema_period) = adjust_period(period, len) else {
            return Ok(Vec::new());
        };
        let Some(atr_period) = adjust_period(atr_period, len) else {
            return Ok(Vec::new());
        };

        let middle_indicator = KCMiddle::new(ema_period as f32).unwrap();
        let middle_values = middle_indicator.calculate_ohlc(data)?;

        let atr_indicator = ATR::new(atr_period as f32).unwrap();
        let atr_values = atr_indicator.calculate_ohlc(data)?;

        let mut lower_values = Vec::with_capacity(len);
        for i in 0..len {
            let middle = middle_values.get(i).copied().unwrap_or(0.0);
            let atr = atr_values.get(i).copied().unwrap_or(0.0);
            lower_values.push(middle - (atr * atr_multiplier));
        }

        Ok(lower_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        let period = self.parameters.get_value("period").unwrap();
        let atr_period = self.parameters.get_value("atr_period").unwrap();
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();
        Box::new(Self::new(period, atr_period, atr_multiplier).unwrap())
    }
}

impl OHLCIndicator for KCLower {}
