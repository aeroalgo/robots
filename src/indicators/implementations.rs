use crate::indicators::{
    base::{
        Indicator, OHLCIndicator, OscillatorIndicator, OverboughtOversoldZones, SimpleIndicator,
        TrendDirection, TrendIndicator, VolatilityIndicator,
    },
    parameters::*,
    types::{IndicatorCategory, IndicatorError, IndicatorType, OHLCData, ParameterSet},
};
use async_trait::async_trait;
use std::collections::HashMap;

// ============================================================================
// ВСПОМОГАТЕЛЬНЫЕ ИНДИКАТОРЫ
// ============================================================================

/// MAXFOR - максимальное значение за период
pub struct MAXFOR {
    parameters: ParameterSet,
}

impl MAXFOR {
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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
}

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let mut max_values = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            let start_idx = if i >= period - 1 { i - period + 1 } else { 0 };
            let end_idx = i + 1;
            let max_value = data.high[start_idx..end_idx]
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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
}

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let mut min_values = Vec::with_capacity(data.len());

        for i in 0..data.len() {
            let start_idx = if i >= period - 1 { i - period + 1 } else { 0 };
            let end_idx = i + 1;
            let min_value = data.low[start_idx..end_idx]
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut atr_values = vec![0.0; data.len()];

        for i in 0..data.len() {
            let true_ranges = self.true_range_simple(data, period, i).await;
            let atr = true_ranges.iter().sum::<f64>() / period as f64;
            atr_values[i] = atr;
        }

        Ok(atr_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let mut atr_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            atr_values.push(0.0);
        }

        // Вычисляем ATR
        for i in period - 1..data.len() {
            let mut true_ranges = Vec::new();

            for j in i - period + 1..=i {
                let true_range = self.true_range_ohlc(data, j);
                true_ranges.push(true_range);
            }

            let atr = true_ranges.iter().sum::<f64>() / period as f64;
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
    async fn true_range_simple(&self, data: &[f64], period: usize, bar_num: usize) -> Vec<f64> {
        let mut true_ranges = Vec::new();
        let new_period = if bar_num < period { bar_num } else { period };

        for i in bar_num.saturating_sub(new_period - 1)..=bar_num {
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
    fn true_range_ohlc(&self, data: &OHLCData, j: usize) -> f64 {
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
#[async_trait]
impl VolatilityIndicator for ATR {
    async fn get_volatility_level(&self, data: &[f64]) -> Result<f64, IndicatorError> {
        let values = self.calculate_simple(data).await?;
        Ok(values.last().copied().unwrap_or(0.0))
    }
}

/// SuperTrend - трендовый индикатор
pub struct SuperTrend {
    parameters: ParameterSet,
}

impl SuperTrend {
    pub fn new(period: f64, coeff_atr: f64) -> Result<Self, IndicatorError> {
        let mut params = ParameterSet::new();
        params
            .add_parameter(create_period_parameter(
                "period",
                period,
                "Период для расчета ATR",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;
        params
            .add_parameter(create_multiplier_parameter_with_range(
                "coeff_atr",
                coeff_atr,
                ParameterPresets::atr_multiplier(),
                "Коэффициент ATR для SuperTrend",
            ))
            .map_err(|e| IndicatorError::InvalidParameter(e))?;

        Ok(Self { parameters: params })
    }
}

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        // Используем ATR индикатор для вычисления
        let atr_indicator = ATR::new(period as f64)?;
        let atr_values = atr_indicator.calculate_simple(data).await?;
        let mut supertrend_values = vec![0.0; data.len()];

        for i in 2..data.len() {
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

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let coeff_atr = self.parameters.get_value("coeff_atr").unwrap();

        // Создаем временный ATR индикатор для расчета
        let atr_indicator = ATR::new(period as f64)?;
        let atr_values = atr_indicator.calculate_ohlc(data).await?;

        let median_prices = self.calculate_median_price(data).await;
        let mut supertrend_values = vec![0.0; data.len()];

        for i in 2..data.len() {
            let atr = atr_values[i];
            let median_price = median_prices[i];

            let upper_band = median_price + (coeff_atr * atr);
            let lower_band = median_price - (coeff_atr * atr);

            let prev_supertrend = supertrend_values[i - 1];
            let current_close = data.close[i];

            let supertrend = if current_close > upper_band {
                lower_band
            } else if current_close < lower_band {
                upper_band
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
    async fn calculate_median_price(&self, data: &OHLCData) -> Vec<f64> {
        data.get_median_price()
    }
}

#[async_trait]
impl TrendIndicator for SuperTrend {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let mut stochastic_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            stochastic_values.push(0.0);
        }

        // Вычисляем Stochastic
        for i in period - 1..data.len() {
            let start_idx = i - period + 1;
            let end_idx = i + 1;

            let highest_high = data.high[start_idx..end_idx]
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            let lowest_low = data.low[start_idx..end_idx]
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b));

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

#[async_trait]
impl OscillatorIndicator for Stochastic {
    async fn get_overbought_oversold_zones(
        &self,
        data: &[f64],
    ) -> Result<OverboughtOversoldZones, IndicatorError> {
        let values = self.calculate_simple(data).await?;
        if let Some(current_value) = values.last() {
            Ok(OverboughtOversoldZones::new(80.0, 20.0, *current_value))
        } else {
            Err(IndicatorError::CalculationError(
                "No values calculated".to_string(),
            ))
        }
    }
}

/// WATR (Wilder's Average True Range)
pub struct WATR {
    parameters: ParameterSet,
}

impl WATR {
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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
}

#[async_trait]
impl Indicator for WATR {
    fn name(&self) -> &str {
        "WATR"
    }
    fn description(&self) -> &str {
        "Wilder's Average True Range - средний истинный диапазон Уайлдера"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::OHLC
    }
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let mut watr_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            watr_values.push(0.0);
        }

        // Вычисляем WATR (Wilder's ATR)
        let mut sum_tr = 0.0;
        for i in 0..period {
            let high_low = data.high[i] - data.low[i];
            let high_close_prev = if i > 0 {
                (data.high[i] - data.close[i - 1]).abs()
            } else {
                0.0
            };
            let low_close_prev = if i > 0 {
                (data.low[i] - data.close[i - 1]).abs()
            } else {
                0.0
            };

            let true_range = high_low.max(high_close_prev).max(low_close_prev);
            sum_tr += true_range;
        }

        let mut watr = sum_tr / period as f64;
        watr_values.push(watr);

        // Вычисляем остальные значения
        for i in period..data.len() {
            let high_low = data.high[i] - data.low[i];
            let high_close_prev = (data.high[i] - data.close[i - 1]).abs();
            let low_close_prev = (data.low[i] - data.close[i - 1]).abs();

            let true_range = high_low.max(high_close_prev).max(low_close_prev);

            // Wilder's smoothing: (prev * (period - 1) + current) / period
            watr = (watr * (period as f64 - 1.0) + true_range) / period as f64;
            watr_values.push(watr);
        }

        Ok(watr_values)
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

#[async_trait]
impl VolatilityIndicator for WATR {
    async fn get_volatility_level(&self, data: &[f64]) -> Result<f64, IndicatorError> {
        let values = self.calculate_simple(data).await?;
        Ok(values.last().copied().unwrap_or(0.0))
    }
}

/// VTRAND - среднее между MAXFOR и MINFOR
pub struct VTRAND {
    parameters: ParameterSet,
}

impl VTRAND {
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        // Создаем временные индикаторы для расчета
        let max_indicator = MAXFOR::new(period as f64)?;
        let min_indicator = MINFOR::new(period as f64)?;

        let max_result = max_indicator.calculate_ohlc(data).await?;
        let min_result = min_indicator.calculate_ohlc(data).await?;

        // VTRAND = (MAXFOR + MINFOR) / 2
        let vtrand_values: Vec<f64> = max_result
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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
}

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut sma_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            sma_values.push(0.0);
        }

        // Вычисляем SMA
        for i in period - 1..data.len() {
            let sum: f64 = data[i - period + 1..=i].iter().sum();
            sma_values.push(sum / period as f64);
        }

        Ok(sma_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для SMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SMA {}

#[async_trait]
impl TrendIndicator for SMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as f64;
        let multiplier = 2.0 / (period + 1.0);

        if data.is_empty() {
            return Err(IndicatorError::InsufficientData {
                required: 1,
                actual: 0,
            });
        }

        let mut ema_values = Vec::with_capacity(data.len());

        // Первое значение - простое среднее
        let first_sma: f64 = data[..period.min(data.len() as f64) as usize].iter().sum();
        let first_ema = first_sma / period.min(data.len() as f64);
        ema_values.push(first_ema);

        // Остальные значения - EMA
        for i in 1..data.len() {
            let prev_ema = ema_values[i - 1];
            let current_price = data[i];
            let ema = (current_price * multiplier) + (prev_ema * (1.0 - multiplier));
            ema_values.push(ema);
        }

        Ok(ema_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для EMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for EMA {}

#[async_trait]
impl TrendIndicator for EMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period + 1 {
            return Err(IndicatorError::InsufficientData {
                required: period + 1,
                actual: data.len(),
            });
        }

        let mut rsi_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period {
            rsi_values.push(0.0);
        }

        // Вычисляем RSI
        for i in period..data.len() {
            let mut gains = 0.0;
            let mut losses = 0.0;

            for j in i - period + 1..=i {
                let change = data[j] - data[j - 1];
                if change > 0.0 {
                    gains += change;
                } else {
                    losses += change.abs();
                }
            }

            let avg_gain = gains / period as f64;
            let avg_loss = losses / period as f64;

            let rs = if avg_loss == 0.0 {
                100.0
            } else {
                avg_gain / avg_loss
            };
            let rsi = 100.0 - (100.0 / (1.0 + rs));

            rsi_values.push(rsi);
        }

        Ok(rsi_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для RSI используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for RSI {}

#[async_trait]
impl OscillatorIndicator for RSI {
    async fn get_overbought_oversold_zones(
        &self,
        data: &[f64],
    ) -> Result<OverboughtOversoldZones, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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
}

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut wma_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            wma_values.push(0.0);
        }

        // Вычисляем WMA
        for i in period - 1..data.len() {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            for j in 0..period {
                let weight = (j + 1) as f64;
                weighted_sum += data[i - j] * weight;
                weight_sum += weight;
            }

            wma_values.push(weighted_sum / weight_sum);
        }

        Ok(wma_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для WMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for WMA {}

#[async_trait]
impl TrendIndicator for WMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period * 2 {
            return Err(IndicatorError::InsufficientData {
                required: period * 2,
                actual: data.len(),
            });
        }

        let mut ama_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period * 2 - 1 {
            ama_values.push(0.0);
        }

        // Вычисляем AMA
        for i in period * 2 - 1..data.len() {
            let sma1 = SMA::new(period as f64)?
                .calculate_simple(&data[i - period * 2 + 1..=i])
                .await?;
            let sma2 = SMA::new((period * 2) as f64)?
                .calculate_simple(&data[i - period * 2 + 1..=i])
                .await?;

            let sma1_value = *sma1.last().unwrap_or(&0.0);
            let sma2_value = *sma2.last().unwrap_or(&0.0);

            let amma = (sma1_value + sma2_value) / 2.0;
            ama_values.push(amma);
        }

        Ok(ama_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для AMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for AMA {}

#[async_trait]
impl TrendIndicator for AMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as f64;
        let lag = (period - 1.0) / 2.0;

        if data.len() < lag as usize + 1 {
            return Err(IndicatorError::InsufficientData {
                required: lag as usize + 1,
                actual: data.len(),
            });
        }

        let mut zlema_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..lag as usize {
            zlema_values.push(0.0);
        }

        // Вычисляем ZLEMA
        for i in lag as usize..data.len() {
            let error = data[i] - data[i - lag as usize];
            let zlema = data[i] + error;
            zlema_values.push(zlema);
        }

        Ok(zlema_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для ZLEMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for ZLEMA {}

#[async_trait]
impl TrendIndicator for ZLEMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut geomean_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            geomean_values.push(0.0);
        }

        // Вычисляем GEOMEAN
        for i in period - 1..data.len() {
            let product: f64 = data[i - period + 1..=i].iter().product();
            let geomean = product.powf(1.0 / period as f64);
            geomean_values.push(geomean);
        }

        Ok(geomean_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для GEOMEAN используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for GEOMEAN {}

#[async_trait]
impl TrendIndicator for GEOMEAN {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period * 2 {
            return Err(IndicatorError::InsufficientData {
                required: period * 2,
                actual: data.len(),
            });
        }

        let mut amma_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period * 2 - 1 {
            amma_values.push(0.0);
        }

        // Вычисляем AMMA
        for i in period * 2 - 1..data.len() {
            let sma1 = SMA::new(period as f64)?
                .calculate_simple(&data[i - period * 2 + 1..=i])
                .await?;
            let sma2 = SMA::new((period * 2) as f64)?
                .calculate_simple(&data[i - period * 2 + 1..=i])
                .await?;

            let sma1_value = *sma1.last().unwrap_or(&0.0);
            let sma2_value = *sma2.last().unwrap_or(&0.0);

            let amma = (sma1_value + sma2_value) / 2.0;
            amma_values.push(amma);
        }

        Ok(amma_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для AMMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for AMMA {}

#[async_trait]
impl TrendIndicator for AMMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut sqwma_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            sqwma_values.push(0.0);
        }

        // Вычисляем SQWMA
        for i in period - 1..data.len() {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            for j in 0..period {
                let weight = ((j + 1) as f64).sqrt();
                weighted_sum += data[i - j] * weight;
                weight_sum += weight;
            }

            sqwma_values.push(weighted_sum / weight_sum);
        }

        Ok(sqwma_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для SQWMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SQWMA {}

#[async_trait]
impl TrendIndicator for SQWMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut sinewma_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            sinewma_values.push(0.0);
        }

        // Вычисляем SINEWMA
        for i in period - 1..data.len() {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;

            for j in 0..period {
                let weight = ((j + 1) as f64 * std::f64::consts::PI / period as f64).sin();
                weighted_sum += data[i - j] * weight;
                weight_sum += weight;
            }

            sinewma_values.push(weighted_sum / weight_sum);
        }

        Ok(sinewma_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для SINEWMA используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for SINEWMA {}

#[async_trait]
impl TrendIndicator for SINEWMA {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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
}

#[async_trait]
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
    // output_type удален - все индикаторы возвращают Vec<f64>
    fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }
    fn min_data_points(&self) -> usize {
        self.parameters.get_value("period").unwrap() as usize
    }

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut tpbf_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            tpbf_values.push(0.0);
        }

        // Упрощенная реализация TPBF (в реальности это сложный фильтр)
        for i in period - 1..data.len() {
            let sum: f64 = data[i - period + 1..=i].iter().sum();
            let tpbf = sum / period as f64;
            tpbf_values.push(tpbf);
        }

        Ok(tpbf_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для TPBF используем close цены
        self.calculate_simple(&data.close).await
    }

    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(Self::new(self.parameters.get_value("period").unwrap()).unwrap())
    }
}

impl SimpleIndicator for TPBF {}

#[async_trait]
impl TrendIndicator for TPBF {
    async fn get_trend_direction(&self, data: &[f64]) -> Result<TrendDirection, IndicatorError> {
        let values = self.calculate_simple(data).await?;
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
    pub fn new(period: f64, deviation: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut sma_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            sma_values.push(0.0);
        }

        // Вычисляем SMA
        for i in period - 1..data.len() {
            let sum: f64 = data[i - period + 1..=i].iter().sum();
            let sma = sum / period as f64;
            sma_values.push(sma);
        }

        Ok(sma_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для BB используем close цены
        self.calculate_simple(&data.close).await
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
    pub fn new(period: f64, deviation: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let deviation = self.parameters.get_value("deviation").unwrap();

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut upper_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            upper_values.push(0.0);
        }

        // Вычисляем верхнюю линию BB
        for i in period - 1..data.len() {
            let window = &data[i - period + 1..=i];
            let sma: f64 = window.iter().sum::<f64>() / period as f64;

            // Вычисляем стандартное отклонение
            let variance: f64 =
                window.iter().map(|&x| (x - sma).powi(2)).sum::<f64>() / period as f64;
            let std_dev = variance.sqrt();

            let upper = sma + (deviation * std_dev);
            upper_values.push(upper);
        }

        Ok(upper_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для BB используем close цены
        self.calculate_simple(&data.close).await
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
    pub fn new(period: f64, deviation: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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

    async fn calculate_simple(&self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let deviation = self.parameters.get_value("deviation").unwrap();

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut lower_values = Vec::with_capacity(data.len());

        // Заполняем начальные значения нулями
        for _ in 0..period - 1 {
            lower_values.push(0.0);
        }

        // Вычисляем нижнюю линию BB
        for i in period - 1..data.len() {
            let window = &data[i - period + 1..=i];
            let sma: f64 = window.iter().sum::<f64>() / period as f64;

            // Вычисляем стандартное отклонение
            let variance: f64 =
                window.iter().map(|&x| (x - sma).powi(2)).sum::<f64>() / period as f64;
            let std_dev = variance.sqrt();

            let lower = sma - (deviation * std_dev);
            lower_values.push(lower);
        }

        Ok(lower_values)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        // Для BB используем close цены
        self.calculate_simple(&data.close).await
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
    pub fn new(period: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;

        if data.len() < period {
            return Err(IndicatorError::InsufficientData {
                required: period,
                actual: data.len(),
            });
        }

        let mut ema_values = Vec::with_capacity(data.len());
        let mut ema = 0.0;

        // Вычисляем типичную цену и EMA
        for i in 0..data.len() {
            let high = data.high[i];
            let low = data.low[i];
            let close = data.close[i];

            let typical_price = (high + low + close) / 3.0;

            if i == 0 {
                ema = typical_price;
            } else {
                let multiplier = 2.0 / (period as f64 + 1.0);
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
    pub fn new(period: f64, atr_period: f64, atr_multiplier: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();

        if data.len() < std::cmp::max(period, atr_period) {
            return Err(IndicatorError::InsufficientData {
                required: std::cmp::max(period, atr_period),
                actual: data.len(),
            });
        }

        // Вычисляем среднюю линию (EMA)
        let middle_indicator = KCMiddle::new(period as f64).unwrap();
        let middle_values = middle_indicator.calculate_ohlc(data).await?;

        // Вычисляем ATR
        let atr_indicator = ATR::new(atr_period as f64).unwrap();
        let atr_values = atr_indicator.calculate_ohlc(data).await?;

        // Вычисляем верхнюю линию
        let mut upper_values = Vec::with_capacity(data.len());
        for (middle, atr) in middle_values.iter().zip(atr_values.iter()) {
            let upper = middle + (atr * atr_multiplier);
            upper_values.push(upper);
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
    pub fn new(period: f64, atr_period: f64, atr_multiplier: f64) -> Result<Self, IndicatorError> {
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

#[async_trait]
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

    async fn calculate_simple(&self, _data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        Err(IndicatorError::OHLCDataRequired)
    }

    async fn calculate_ohlc(&self, data: &OHLCData) -> Result<Vec<f64>, IndicatorError> {
        let period = self.parameters.get_value("period").unwrap() as usize;
        let atr_period = self.parameters.get_value("atr_period").unwrap() as usize;
        let atr_multiplier = self.parameters.get_value("atr_multiplier").unwrap();

        if data.len() < std::cmp::max(period, atr_period) {
            return Err(IndicatorError::InsufficientData {
                required: std::cmp::max(period, atr_period),
                actual: data.len(),
            });
        }

        // Вычисляем среднюю линию (EMA)
        let middle_indicator = KCMiddle::new(period as f64).unwrap();
        let middle_values = middle_indicator.calculate_ohlc(data).await?;

        // Вычисляем ATR
        let atr_indicator = ATR::new(atr_period as f64).unwrap();
        let atr_values = atr_indicator.calculate_ohlc(data).await?;

        // Вычисляем нижнюю линию
        let mut lower_values = Vec::with_capacity(data.len());
        for (middle, atr) in middle_values.iter().zip(atr_values.iter()) {
            let lower = middle - (atr * atr_multiplier);
            lower_values.push(lower);
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
