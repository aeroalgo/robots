use std::collections::HashMap;

use thiserror::Error;

use crate::indicators::implementations::OptimizationRange;
use crate::indicators::types::OHLCData;
use crate::position::view::ActivePosition;
use crate::strategy::context::TimeframeData;
use crate::strategy::types::{PositionDirection, PriceField, StopSignalKind, StrategyParamValue};

/// Спецификация служебного индикатора, необходимого для работы стоп-обработчика.
/// Эти индикаторы вычисляются автоматически и не являются частью торговой логики стратегии.
#[derive(Clone, Debug)]
pub struct AuxiliaryIndicatorSpec {
    /// Имя индикатора ("ATR", "MINFOR", "MAXFOR")
    pub indicator_name: String,
    /// Параметры индикатора (например, {"period": 14.0})
    pub parameters: HashMap<String, f64>,
    /// Уникальный алиас для хранения (например, "aux_ATR_14")
    pub alias: String,
}

impl AuxiliaryIndicatorSpec {
    pub fn atr(period: u32) -> Self {
        Self {
            indicator_name: "ATR".to_string(),
            parameters: [("period".to_string(), period as f64)]
                .into_iter()
                .collect(),
            alias: format!("aux_ATR_{}", period),
        }
    }

    pub fn minfor(period: u32) -> Self {
        Self {
            indicator_name: "MINFOR".to_string(),
            parameters: [("period".to_string(), period as f64)]
                .into_iter()
                .collect(),
            alias: format!("aux_MINFOR_{}", period),
        }
    }

    pub fn maxfor(period: u32) -> Self {
        Self {
            indicator_name: "MAXFOR".to_string(),
            parameters: [("period".to_string(), period as f64)]
                .into_iter()
                .collect(),
            alias: format!("aux_MAXFOR_{}", period),
        }
    }
}

pub struct StopEvaluationContext<'a> {
    pub position: &'a ActivePosition,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> StopEvaluationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct StopValidationContext<'a> {
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub timeframe_data: &'a TimeframeData,
    pub price_field: PriceField,
    pub index: usize,
    pub current_price: f64,
}

impl<'a> StopValidationContext<'a> {
    pub fn price_series(&self) -> Option<&[f32]> {
        self.timeframe_data.price_series_slice(&self.price_field)
    }
}

pub struct StopValidationResult {
    pub stop_level: f64,
    pub is_valid: bool,
    pub reason: Option<String>,
}

pub struct StopOutcome {
    pub exit_price: f64,
    pub kind: StopSignalKind,
    pub metadata: HashMap<String, String>,
}

pub trait StopHandler: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome>;

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![]
    }
}

pub struct StopHandlerFactory;

#[derive(Debug, Error)]
pub enum StopHandlerError {
    #[error("unknown stop handler: {0}")]
    UnknownHandler(String),
    #[error("invalid parameter {0}")]
    InvalidParameter(String),
}

pub fn get_optimization_range(handler_name: &str, param_name: &str) -> Option<OptimizationRange> {
    match handler_name.to_uppercase().as_str() {
        "STOPLOSSPCT" | "STOP_LOSS_PCT" | "STOPLOSS_PCT" => {
            match param_name.to_lowercase().as_str() {
                "percentage" | "stop_loss" | "stop" | "value" | "pct" => {
                    Some(OptimizationRange::new(2.0, 10.0, 0.5))
                }
                _ => None,
            }
        }
        "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => {
            match param_name.to_lowercase().as_str() {
                "period" => Some(OptimizationRange::new(10.0, 150.0, 10.0)),
                "coeff_atr" | "coeff" | "atr_coeff" => Some(OptimizationRange::new(2.0, 8.0, 0.2)),
                _ => None,
            }
        }
        "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
            match param_name.to_lowercase().as_str() {
                "period" => Some(OptimizationRange::new(10.0, 150.0, 10.0)),
                _ => None,
            }
        }
        "PERCENTTRAILSTOP" | "PERCENTTRAILINGSTOP" | "PERCENT_TRAIL_STOP" | "PERCENT_TRAIL" => {
            match param_name.to_lowercase().as_str() {
                "percentage" | "stop_loss" | "stop" | "value" | "pct" => {
                    Some(OptimizationRange::new(2.0, 10.0, 0.5))
                }
                _ => None,
            }
        }
        "INDICATORSTOP" | "INDICATOR_STOP" | "IND_STOP" => {
            match param_name.to_lowercase().as_str() {
                // period: период индикатора
                "period" => Some(OptimizationRange::new(10.0, 200.0, 10.0)),
                // coeff_atr, multiplier: коэффициенты
                "coeff_atr" | "coeff" | "multiplier" => Some(OptimizationRange::new(1.0, 5.0, 0.5)),
                // offset_percent: от -5% до +5% от значения индикатора
                "offset_percent" | "offset" | "offset_pct" => {
                    Some(OptimizationRange::new(-0.05, 0.05, 0.005))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

impl StopHandlerFactory {
    pub fn create(
        handler_name: &str,
        parameters: &HashMap<String, StrategyParamValue>,
    ) -> Result<Box<dyn StopHandler>, StopHandlerError> {
        match handler_name.to_ascii_uppercase().as_str() {
            "STOPLOSSPCT" | "STOP_LOSS_PCT" | "STOPLOSS_PCT" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "stop_loss", "stop", "value"],
                    0.2,
                )?;
                Ok(Box::new(StopLossPctHandler { percentage }))
            }
            "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => {
                let period = extract_number(parameters, &["period"], 14.0)?;
                let coeff_atr =
                    extract_number(parameters, &["coeff_atr", "coeff", "atr_coeff"], 5.0)?;
                Ok(Box::new(ATRTrailStopHandler { period, coeff_atr }))
            }
            "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
                let period = extract_number(parameters, &["period"], 14.0)?;
                Ok(Box::new(HILOTrailingStopHandler { period }))
            }
            "PERCENTTRAILSTOP" | "PERCENTTRAILINGSTOP" | "PERCENT_TRAIL_STOP" | "PERCENT_TRAIL" => {
                let percentage = extract_percentage(
                    parameters,
                    &["percentage", "stop_loss", "stop", "value", "pct"],
                    1.0,
                )?;
                Ok(Box::new(PercentTrailingStopHandler { percentage }))
            }
            "INDICATORSTOP" | "INDICATOR_STOP" | "IND_STOP" => {
                // Получаем тип индикатора
                let indicator_name = extract_string(parameters, &["indicator_name", "indicator"])?
                    .unwrap_or_else(|| "SMA".to_string())
                    .to_uppercase();

                // Собираем все параметры индикатора (все кроме служебных)
                let reserved_keys = [
                    "indicator_name",
                    "indicator",
                    "offset_percent",
                    "offset",
                    "offset_pct",
                    "trailing",
                    "trail",
                ];
                let mut indicator_params: HashMap<String, f64> = HashMap::new();

                for (key, value) in parameters {
                    let key_lower = key.to_lowercase();
                    if !reserved_keys.iter().any(|&r| key_lower == r) {
                        if let Some(num) = value.as_f64() {
                            indicator_params.insert(key_lower, num);
                        }
                    }
                }

                // Нормализуем параметры: удаляем лишние, добавляем недостающие
                let indicator_params =
                    normalize_indicator_params(&indicator_name, &indicator_params);

                let offset_percent =
                    extract_number(parameters, &["offset_percent", "offset", "offset_pct"], 0.0)?;
                let trailing = extract_bool(parameters, &["trailing", "trail"], true);

                Ok(Box::new(IndicatorStopHandler {
                    indicator_name,
                    indicator_params,
                    offset_percent,
                    trailing,
                }))
            }
            other => Err(StopHandlerError::UnknownHandler(other.to_string())),
        }
    }
}

fn extract_percentage(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, StopHandlerError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(StopHandlerError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

fn extract_number(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: f64,
) -> Result<f64, StopHandlerError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(number) = value.as_f64() {
                    return Ok(number);
                }
                return Err(StopHandlerError::InvalidParameter(key.clone()));
            }
        }
    }
    Ok(default_value)
}

fn extract_string(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
) -> Result<Option<String>, StopHandlerError> {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(s) = value.as_str() {
                    return Ok(Some(s.to_string()));
                }
                // Также попробуем извлечь из Number как строку (для алиаса типа "SMA_20")
                if let StrategyParamValue::Text(s) = value {
                    return Ok(Some(s.clone()));
                }
                return Err(StopHandlerError::InvalidParameter(format!(
                    "{} must be a string",
                    key
                )));
            }
        }
    }
    Ok(None)
}

fn extract_bool(
    parameters: &HashMap<String, StrategyParamValue>,
    keys: &[&str],
    default_value: bool,
) -> bool {
    for (key, value) in parameters {
        for target in keys {
            if key.eq_ignore_ascii_case(target) {
                if let Some(b) = value.as_bool() {
                    return b;
                }
            }
        }
    }
    default_value
}

struct StopLossPctHandler {
    percentage: f64,
}

impl StopLossPctHandler {
    fn level(&self, position: &ActivePosition) -> Option<f64> {
        let ratio = self.percentage / 100.0;
        match position.direction {
            PositionDirection::Long => Some(position.entry_price * (1.0 - ratio)),
            PositionDirection::Short => Some(position.entry_price * (1.0 + ratio)),
            _ => None,
        }
    }
}

impl StopHandler for StopLossPctHandler {
    fn name(&self) -> &str {
        "StopLossPct"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let level = self.level(ctx.position)?;

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match ctx.position.direction {
            PositionDirection::Long => low_price <= level,
            PositionDirection::Short => high_price >= level,
            _ => false,
        };
        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), level.to_string());
            let triggered_price = match ctx.position.direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            return Some(StopOutcome {
                exit_price: level,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

struct ATRTrailStopHandler {
    period: f64,
    coeff_atr: f64,
}

impl ATRTrailStopHandler {
    /// Получить алиас служебного ATR индикатора
    fn auxiliary_alias(&self) -> String {
        format!("aux_ATR_{}", self.period as u32)
    }

    fn get_atr_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        // Сначала ищем в служебных индикаторах (primary)
        let aux_alias = self.auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        // Fallback: ищем в обычных индикаторах для обратной совместимости
        let atr_alias = format!("ATR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&atr_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("ATR", ctx.index))
    }

    fn get_current_bar_extremes(&self, ctx: &StopEvaluationContext<'_>) -> (f64, f64) {
        let low_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .unwrap_or(&[]);

        let high_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .unwrap_or(&[]);

        let current_low = low_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let current_high = high_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        (current_low, current_high)
    }

    fn calculate_stop_level(
        &self,
        ctx: &StopEvaluationContext<'_>,
        min_price: f64,
        max_price: f64,
        atr: f32,
    ) -> f64 {
        let atr_f64 = atr as f64;
        let offset = atr_f64 * self.coeff_atr;

        match ctx.position.direction {
            PositionDirection::Long => min_price - offset,
            PositionDirection::Short => max_price + offset,
            _ => ctx.current_price,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        let stop_key = format!("{}_current_stop", self.name());

        if let Some(current_stop) = position
            .metadata
            .get(&stop_key)
            .and_then(|s| s.parse::<f64>().ok())
        {
            match direction {
                PositionDirection::Long => new_stop.max(current_stop),
                PositionDirection::Short => new_stop.min(current_stop),
                _ => new_stop,
            }
        } else {
            new_stop
        }
    }
}

impl StopHandler for ATRTrailStopHandler {
    fn name(&self) -> &str {
        "ATRTrailStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let atr_value = self.get_atr_value(ctx)?;

        let (min_price, max_price) = self.get_current_bar_extremes(ctx);

        let new_stop = self.calculate_stop_level(ctx, min_price, max_price, atr_value);

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert("atr_value".to_string(), atr_value.to_string());
            metadata.insert("min_price".to_string(), min_price.to_string());
            metadata.insert("max_price".to_string(), max_price.to_string());
            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            metadata.insert(format!("{}_min_price", self.name()), min_price.to_string());
            metadata.insert(format!("{}_max_price", self.name()), max_price.to_string());
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![AuxiliaryIndicatorSpec::atr(self.period as u32)]
    }
}

struct HILOTrailingStopHandler {
    period: f64,
}

impl HILOTrailingStopHandler {
    /// Получить алиасы служебных индикаторов
    fn minfor_auxiliary_alias(&self) -> String {
        format!("aux_MINFOR_{}", self.period as u32)
    }

    fn maxfor_auxiliary_alias(&self) -> String {
        format!("aux_MAXFOR_{}", self.period as u32)
    }

    fn get_minfor_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        // Сначала ищем в служебных индикаторах (primary)
        let aux_alias = self.minfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        // Fallback: ищем в обычных индикаторах для обратной совместимости
        let minfor_alias = format!("MINFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&minfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MINFOR", ctx.index))
    }

    fn get_maxfor_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        // Сначала ищем в служебных индикаторах (primary)
        let aux_alias = self.maxfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        // Fallback: ищем в обычных индикаторах для обратной совместимости
        let maxfor_alias = format!("MAXFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&maxfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MAXFOR", ctx.index))
    }

    fn get_minfor_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        // Сначала ищем в служебных индикаторах
        let aux_alias = self.minfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        // Fallback
        let minfor_alias = format!("MINFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&minfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MINFOR", ctx.index))
    }

    fn get_maxfor_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        // Сначала ищем в служебных индикаторах
        let aux_alias = self.maxfor_auxiliary_alias();
        if let Some(value) = ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index) {
            return Some(value);
        }

        // Fallback
        let maxfor_alias = format!("MAXFOR_{}", self.period as u32);
        ctx.timeframe_data
            .indicator_value_at(&maxfor_alias, ctx.index)
            .or_else(|| ctx.timeframe_data.indicator_value_at("MAXFOR", ctx.index))
    }

    fn calculate_stop_level(&self, ctx: &StopEvaluationContext<'_>) -> Option<f64> {
        match ctx.position.direction {
            PositionDirection::Long => {
                let minfor = self.get_minfor_value(ctx)?;
                Some(minfor as f64)
            }
            PositionDirection::Short => {
                let maxfor = self.get_maxfor_value(ctx)?;
                Some(maxfor as f64)
            }
            _ => None,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        let stop_key = format!("{}_current_stop", self.name());

        if let Some(current_stop) = position
            .metadata
            .get(&stop_key)
            .and_then(|s| s.parse::<f64>().ok())
        {
            match direction {
                PositionDirection::Long => new_stop.max(current_stop),
                PositionDirection::Short => new_stop.min(current_stop),
                _ => new_stop,
            }
        } else {
            new_stop
        }
    }
}

impl StopHandler for HILOTrailingStopHandler {
    fn name(&self) -> &str {
        "HILOTrailingStop"
    }

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        let stop_level = match ctx.direction {
            PositionDirection::Long => {
                let minfor = self.get_minfor_value_for_validation(ctx)?;
                minfor as f64
            }
            PositionDirection::Short => {
                let maxfor = self.get_maxfor_value_for_validation(ctx)?;
                maxfor as f64
            }
            _ => return None,
        };

        let is_valid = match ctx.direction {
            PositionDirection::Long => ctx.current_price > stop_level,
            PositionDirection::Short => ctx.current_price < stop_level,
            _ => false,
        };

        let reason = if !is_valid {
            match ctx.direction {
                PositionDirection::Long => Some(format!(
                    "Цена {} не выше MINFOR уровня {}",
                    ctx.current_price, stop_level
                )),
                PositionDirection::Short => Some(format!(
                    "Цена {} не ниже MAXFOR уровня {}",
                    ctx.current_price, stop_level
                )),
                _ => None,
            }
        } else {
            None
        };

        Some(StopValidationResult {
            stop_level,
            is_valid,
            reason,
        })
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let new_stop = self.calculate_stop_level(ctx)?;

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());

            if let Some(minfor) = self.get_minfor_value(ctx) {
                metadata.insert("minfor_value".to_string(), minfor.to_string());
            }
            if let Some(maxfor) = self.get_maxfor_value(ctx) {
                metadata.insert("maxfor_value".to_string(), maxfor.to_string());
            }

            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![
            AuxiliaryIndicatorSpec::minfor(self.period as u32),
            AuxiliaryIndicatorSpec::maxfor(self.period as u32),
        ]
    }
}

struct PercentTrailingStopHandler {
    percentage: f64,
}

impl PercentTrailingStopHandler {
    fn get_current_bar_extremes(&self, ctx: &StopEvaluationContext<'_>) -> (f64, f64) {
        let low_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .unwrap_or(&[]);

        let high_series = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .unwrap_or(&[]);

        let current_low = low_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let current_high = high_series
            .get(ctx.index)
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        (current_low, current_high)
    }

    fn calculate_stop_level(
        &self,
        ctx: &StopEvaluationContext<'_>,
        min_price: f64,
        max_price: f64,
    ) -> f64 {
        let ratio = self.percentage / 100.0;

        match ctx.position.direction {
            PositionDirection::Long => min_price * (1.0 - ratio),
            PositionDirection::Short => max_price * (1.0 + ratio),
            _ => ctx.current_price,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        let stop_key = format!("{}_current_stop", self.name());

        if let Some(current_stop) = position
            .metadata
            .get(&stop_key)
            .and_then(|s| s.parse::<f64>().ok())
        {
            match direction {
                PositionDirection::Long => new_stop.max(current_stop),
                PositionDirection::Short => new_stop.min(current_stop),
                _ => new_stop,
            }
        } else {
            new_stop
        }
    }
}

impl StopHandler for PercentTrailingStopHandler {
    fn name(&self) -> &str {
        "PercentTrailingStop"
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let (min_price, max_price) = self.get_current_bar_extremes(ctx);

        let new_stop = self.calculate_stop_level(ctx, min_price, max_price);

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert("percentage".to_string(), self.percentage.to_string());
            metadata.insert("min_price".to_string(), min_price.to_string());
            metadata.insert("max_price".to_string(), max_price.to_string());
            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            metadata.insert(format!("{}_min_price", self.name()), min_price.to_string());
            metadata.insert(format!("{}_max_price", self.name()), max_price.to_string());
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }
}

/// Стоп-лосс на основе любого индикатора (SMA, EMA, SuperTrend и т.д.)
/// Автоматически вычисляет индикатор как auxiliary
/// Проверяет validate_before_entry чтобы цена была выше/ниже индикатора
struct IndicatorStopHandler {
    /// Тип индикатора ("SMA", "EMA", "SUPERTREND" и т.д.)
    indicator_name: String,
    /// Параметры индикатора (period, coeff_atr, multiplier и т.д.)
    indicator_params: HashMap<String, f64>,
    /// Смещение от индикатора (например -0.01 = на 1% ниже индикатора)
    offset_percent: f64,
    /// Трейлинг: стоп может только улучшаться (для Long - только вверх)
    trailing: bool,
}

impl IndicatorStopHandler {
    /// Генерирует уникальный алиас на основе индикатора и его параметров
    fn auxiliary_alias(&self) -> String {
        // Сортируем параметры для стабильного алиаса
        let mut params: Vec<_> = self.indicator_params.iter().collect();
        params.sort_by_key(|(k, _)| k.as_str());

        let params_str: String = params
            .iter()
            .map(|(k, v)| format!("{}_{}", k, **v as u32))
            .collect::<Vec<_>>()
            .join("_");

        format!(
            "aux_stop_{}_{}",
            self.indicator_name.to_uppercase(),
            params_str
        )
    }

    /// Описание индикатора для сообщений
    fn indicator_description(&self) -> String {
        let params_str: String = self
            .indicator_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}({})", self.indicator_name, params_str)
    }

    fn get_indicator_value(&self, ctx: &StopEvaluationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index)
    }

    fn get_indicator_value_for_validation(&self, ctx: &StopValidationContext<'_>) -> Option<f32> {
        let aux_alias = self.auxiliary_alias();
        ctx.timeframe_data.auxiliary_value_at(&aux_alias, ctx.index)
    }

    fn calculate_stop_level(&self, indicator_value: f64, direction: &PositionDirection) -> f64 {
        let offset = indicator_value * self.offset_percent;
        match direction {
            // Для Long: стоп ниже индикатора (offset обычно отрицательный)
            PositionDirection::Long => indicator_value + offset,
            // Для Short: стоп выше индикатора (offset обычно положительный)
            PositionDirection::Short => indicator_value - offset,
            _ => indicator_value,
        }
    }

    fn compute_updated_stop(
        &self,
        position: &ActivePosition,
        new_stop: f64,
        direction: PositionDirection,
    ) -> f64 {
        if !self.trailing {
            return new_stop;
        }

        let stop_key = format!("{}_current_stop", self.name());

        if let Some(current_stop) = position
            .metadata
            .get(&stop_key)
            .and_then(|s| s.parse::<f64>().ok())
        {
            match direction {
                // Для Long: стоп может только расти (подтягиваться)
                PositionDirection::Long => new_stop.max(current_stop),
                // Для Short: стоп может только падать
                PositionDirection::Short => new_stop.min(current_stop),
                _ => new_stop,
            }
        } else {
            new_stop
        }
    }

    fn name(&self) -> &str {
        "IndicatorStop"
    }
}

impl StopHandler for IndicatorStopHandler {
    fn name(&self) -> &str {
        "IndicatorStop"
    }

    fn validate_before_entry(
        &self,
        ctx: &StopValidationContext<'_>,
    ) -> Option<StopValidationResult> {
        let indicator_value = self.get_indicator_value_for_validation(ctx)? as f64;
        let stop_level = self.calculate_stop_level(indicator_value, &ctx.direction);

        let is_valid = match ctx.direction {
            // Для Long: цена должна быть выше уровня стопа
            PositionDirection::Long => ctx.current_price > stop_level,
            // Для Short: цена должна быть ниже уровня стопа
            PositionDirection::Short => ctx.current_price < stop_level,
            _ => false,
        };

        let indicator_desc = self.indicator_description();
        let reason = if !is_valid {
            match ctx.direction {
                PositionDirection::Long => Some(format!(
                    "Цена {} не выше уровня стопа {} ({} = {})",
                    ctx.current_price, stop_level, indicator_desc, indicator_value
                )),
                PositionDirection::Short => Some(format!(
                    "Цена {} не ниже уровня стопа {} ({} = {})",
                    ctx.current_price, stop_level, indicator_desc, indicator_value
                )),
                _ => None,
            }
        } else {
            None
        };

        Some(StopValidationResult {
            stop_level,
            is_valid,
            reason,
        })
    }

    fn evaluate(&self, ctx: &StopEvaluationContext<'_>) -> Option<StopOutcome> {
        let indicator_value = self.get_indicator_value(ctx)? as f64;
        let new_stop = self.calculate_stop_level(indicator_value, &ctx.position.direction);

        let direction = ctx.position.direction.clone();
        let current_stop = self.compute_updated_stop(ctx.position, new_stop, direction.clone());

        let low_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::Low)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let high_price = ctx
            .timeframe_data
            .price_series_slice(&PriceField::High)
            .and_then(|series| series.get(ctx.index))
            .copied()
            .map(|p| p as f64)
            .unwrap_or(ctx.current_price);

        let triggered = match direction {
            PositionDirection::Long => low_price <= current_stop,
            PositionDirection::Short => high_price >= current_stop,
            _ => false,
        };

        if triggered {
            let mut metadata = HashMap::new();
            metadata.insert("level".to_string(), current_stop.to_string());
            metadata.insert("indicator_name".to_string(), self.indicator_name.clone());
            metadata.insert(
                "indicator_description".to_string(),
                self.indicator_description(),
            );
            metadata.insert("indicator_value".to_string(), indicator_value.to_string());
            // Добавляем все параметры индикатора
            for (key, value) in &self.indicator_params {
                metadata.insert(format!("indicator_{}", key), value.to_string());
            }
            let triggered_price = match direction {
                PositionDirection::Long => low_price,
                PositionDirection::Short => high_price,
                _ => ctx.current_price,
            };
            metadata.insert("triggered_price".to_string(), triggered_price.to_string());
            metadata.insert(
                format!("{}_current_stop", self.name()),
                current_stop.to_string(),
            );
            return Some(StopOutcome {
                exit_price: current_stop,
                kind: StopSignalKind::StopLoss,
                metadata,
            });
        }
        None
    }

    fn required_auxiliary_indicators(&self) -> Vec<AuxiliaryIndicatorSpec> {
        vec![AuxiliaryIndicatorSpec {
            indicator_name: self.indicator_name.clone(),
            parameters: self.indicator_params.clone(),
            alias: self.auxiliary_alias(),
        }]
    }
}

// ===== Дефолтные параметры индикаторов =====

/// Получает дефолтные параметры индикатора из registry
/// Если индикатор не найден, возвращает пустой HashMap
pub fn get_default_indicator_params(indicator_name: &str) -> HashMap<String, f64> {
    use crate::indicators::registry::IndicatorRegistry;

    let registry = IndicatorRegistry::new();
    if let Some(indicator) = registry.get_indicator(indicator_name) {
        indicator
            .parameters()
            .get_current_values()
            .into_iter()
            .map(|(k, v)| (k, v as f64))
            .collect()
    } else {
        // Fallback: базовый period
        let mut params = HashMap::new();
        params.insert("period".to_string(), 20.0);
        params
    }
}

/// Заполняет недостающие параметры индикатора дефолтными значениями
pub fn fill_missing_indicator_params(
    indicator_name: &str,
    existing_params: &mut HashMap<String, f64>,
) {
    let defaults = get_default_indicator_params(indicator_name);
    for (key, default_value) in defaults {
        existing_params.entry(key).or_insert(default_value);
    }
}

/// Нормализует параметры индикатора:
/// - Удаляет лишние параметры (которых нет у индикатора)
/// - Добавляет недостающие с дефолтными значениями
/// Используется при смене типа индикатора (например SuperTrend → SMA)
pub fn normalize_indicator_params(
    indicator_name: &str,
    existing_params: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let defaults = get_default_indicator_params(indicator_name);
    let valid_keys: std::collections::HashSet<&String> = defaults.keys().collect();

    let mut result = HashMap::new();

    // Берем только валидные параметры из existing, иначе дефолт
    for (key, default_value) in &defaults {
        let value = existing_params.get(key).copied().unwrap_or(*default_value);
        result.insert(key.clone(), value);
    }

    result
}

// ===== Вычисление служебных индикаторов =====

/// Собирает все необходимые служебные индикаторы от списка стоп-обработчиков
pub fn collect_required_auxiliary_indicators(
    handlers: &[Box<dyn StopHandler>],
) -> Vec<AuxiliaryIndicatorSpec> {
    let mut specs = Vec::new();
    let mut seen_aliases = std::collections::HashSet::new();

    for handler in handlers {
        for spec in handler.required_auxiliary_indicators() {
            if !seen_aliases.contains(&spec.alias) {
                seen_aliases.insert(spec.alias.clone());
                specs.push(spec);
            }
        }
    }

    specs
}

/// Вычисляет служебные индикаторы из OHLC данных
/// Возвращает HashMap с алиасом и вычисленными значениями
pub fn compute_auxiliary_indicators(
    specs: &[AuxiliaryIndicatorSpec],
    ohlc: &OHLCData,
) -> Result<HashMap<String, Vec<f32>>, StopHandlerError> {
    use crate::indicators::registry::IndicatorFactory;

    let mut results = HashMap::new();

    for spec in specs {
        // Конвертируем параметры из f64 в f32
        let parameters: HashMap<String, f32> = spec
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), *v as f32))
            .collect();

        // Создаем индикатор с нужными параметрами
        let indicator = IndicatorFactory::create_indicator(&spec.indicator_name, parameters)
            .map_err(|e| {
                StopHandlerError::InvalidParameter(format!(
                    "Failed to create auxiliary indicator {}: {:?}",
                    spec.indicator_name, e
                ))
            })?;

        // Вычисляем значения
        let values = indicator.calculate_ohlc(ohlc).map_err(|e| {
            StopHandlerError::InvalidParameter(format!(
                "Failed to compute auxiliary indicator {}: {:?}",
                spec.indicator_name, e
            ))
        })?;

        results.insert(spec.alias.clone(), values);
    }

    Ok(results)
}

/// Собирает auxiliary спецификации из StopHandlerSpec без создания хэндлеров
pub fn get_auxiliary_specs_from_handler_spec(
    handler_name: &str,
    parameters: &HashMap<String, StrategyParamValue>,
) -> Vec<AuxiliaryIndicatorSpec> {
    match handler_name.to_ascii_uppercase().as_str() {
        "ATRTRAILSTOP" | "ATR_TRAIL_STOP" | "ATR_TRAIL" => {
            let period = parameters
                .iter()
                .find(|(k, _)| k.to_lowercase() == "period")
                .and_then(|(_, v)| v.as_f64())
                .unwrap_or(14.0) as u32;
            vec![AuxiliaryIndicatorSpec::atr(period)]
        }
        "HILOTRAILSTOP" | "HILOTRAILINGSTOP" | "HILO_TRAIL_STOP" | "HILO_TRAIL" => {
            let period = parameters
                .iter()
                .find(|(k, _)| k.to_lowercase() == "period")
                .and_then(|(_, v)| v.as_f64())
                .unwrap_or(14.0) as u32;
            vec![
                AuxiliaryIndicatorSpec::minfor(period),
                AuxiliaryIndicatorSpec::maxfor(period),
            ]
        }
        "INDICATORSTOP" | "INDICATOR_STOP" | "IND_STOP" => {
            // Получаем тип индикатора
            let indicator_name = parameters
                .iter()
                .find(|(k, _)| {
                    let k_lower = k.to_lowercase();
                    k_lower == "indicator_name" || k_lower == "indicator"
                })
                .and_then(|(_, v)| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "SMA".to_string())
                .to_uppercase();

            // Собираем все параметры индикатора (все кроме служебных)
            let reserved_keys = [
                "indicator_name",
                "indicator",
                "offset_percent",
                "offset",
                "offset_pct",
                "trailing",
                "trail",
            ];
            let mut indicator_params: HashMap<String, f64> = HashMap::new();

            for (key, value) in parameters {
                let key_lower = key.to_lowercase();
                if !reserved_keys.iter().any(|&r| key_lower == r) {
                    if let Some(num) = value.as_f64() {
                        indicator_params.insert(key_lower, num);
                    }
                }
            }

            // Нормализуем параметры: удаляем лишние, добавляем недостающие
            let indicator_params = normalize_indicator_params(&indicator_name, &indicator_params);

            // Генерируем алиас
            let mut params: Vec<_> = indicator_params.iter().collect();
            params.sort_by_key(|(k, _)| k.as_str());
            let params_str: String = params
                .iter()
                .map(|(k, v)| format!("{}_{}", k, **v as u32))
                .collect::<Vec<_>>()
                .join("_");
            let alias = format!("aux_stop_{}_{}", indicator_name, params_str);

            vec![AuxiliaryIndicatorSpec {
                indicator_name,
                parameters: indicator_params,
                alias,
            }]
        }
        _ => vec![],
    }
}

/// Собирает все auxiliary спецификации из списка StopHandlerSpec
pub fn collect_auxiliary_specs_from_stop_handlers(
    stop_handlers: &[crate::strategy::types::StopHandlerSpec],
) -> Vec<AuxiliaryIndicatorSpec> {
    let mut specs = Vec::new();
    let mut seen_aliases = std::collections::HashSet::new();

    for handler in stop_handlers {
        for spec in
            get_auxiliary_specs_from_handler_spec(&handler.handler_name, &handler.parameters)
        {
            if !seen_aliases.contains(&spec.alias) {
                seen_aliases.insert(spec.alias.clone());
                specs.push(spec);
            }
        }
    }

    specs
}
