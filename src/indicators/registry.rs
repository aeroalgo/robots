use crate::indicators::{
    base::Indicator,
    implementations::*,
    runtime::IndicatorRuntimeEngine,
    types::{
        IndicatorCategory, IndicatorError, IndicatorId, IndicatorType, OHLCData, ParameterSet,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;

/// Конфигурация индикатора для сериализации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorConfig {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    pub parameters: HashMap<String, f32>,
    pub description: String,
}

/// Реестр индикаторов
pub struct IndicatorRegistry {
    indicators: HashMap<String, Box<dyn Indicator + Send + Sync>>,
    categories: HashMap<IndicatorCategory, Vec<String>>,
    types: HashMap<IndicatorType, Vec<String>>,
}

impl IndicatorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            indicators: HashMap::new(),
            categories: HashMap::new(),
            types: HashMap::new(),
        };

        // Регистрируем все доступные индикаторы
        registry.register_all_indicators();
        registry
    }

    /// Регистрируем все доступные индикаторы
    fn register_all_indicators(&mut self) {
        // OHLC индикаторы
        if let Ok(atr) = ATR::new(14.0) {
            self.register_indicator("ATR", Box::new(atr));
        }

        if let Ok(tr) = TrueRange::new() {
            self.register_indicator("TrueRange", Box::new(tr));
        }

        if let Ok(supertrend) = SuperTrend::new(10.0, 3.0) {
            self.register_indicator("SuperTrend", Box::new(supertrend));
        }

        if let Ok(stochastic) = Stochastic::new(14.0) {
            self.register_indicator("Stochastic", Box::new(stochastic));
        }

        if let Ok(watr) = WATR::new(14.0) {
            self.register_indicator("WATR", Box::new(watr));
        }

        if let Ok(vtrand) = VTRAND::new(14.0) {
            self.register_indicator("VTRAND", Box::new(vtrand));
        }

        if let Ok(maxfor) = MAXFOR::new(14.0) {
            self.register_indicator("MAXFOR", Box::new(maxfor));
        }

        if let Ok(minfor) = MINFOR::new(14.0) {
            self.register_indicator("MINFOR", Box::new(minfor));
        }

        // Простые индикаторы
        if let Ok(sma) = SMA::new(20.0) {
            self.register_indicator("SMA", Box::new(sma));
        }

        if let Ok(ema) = EMA::new(20.0) {
            self.register_indicator("EMA", Box::new(ema));
        }

        if let Ok(rsi) = RSI::new(14.0) {
            self.register_indicator("RSI", Box::new(rsi));
        }

        if let Ok(wma) = WMA::new(20.0) {
            self.register_indicator("WMA", Box::new(wma));
        }

        if let Ok(ama) = AMA::new(20.0) {
            self.register_indicator("AMA", Box::new(ama));
        }

        if let Ok(zlema) = ZLEMA::new(20.0) {
            self.register_indicator("ZLEMA", Box::new(zlema));
        }

        if let Ok(geomean) = GEOMEAN::new(20.0) {
            self.register_indicator("GEOMEAN", Box::new(geomean));
        }

        if let Ok(amma) = AMMA::new(20.0) {
            self.register_indicator("AMMA", Box::new(amma));
        }

        if let Ok(sqwma) = SQWMA::new(20.0) {
            self.register_indicator("SQWMA", Box::new(sqwma));
        }

        if let Ok(sinewma) = SINEWMA::new(20.0) {
            self.register_indicator("SINEWMA", Box::new(sinewma));
        }

        if let Ok(tpbf) = TPBF::new(20.0) {
            self.register_indicator("TPBF", Box::new(tpbf));
        }

        // Bollinger Bands компоненты
        if let Ok(bb_middle) = BBMiddle::new(20.0, 2.0) {
            self.register_indicator("BBMiddle", Box::new(bb_middle));
        }

        if let Ok(bb_upper) = BBUpper::new(20.0, 2.0) {
            self.register_indicator("BBUpper", Box::new(bb_upper));
        }

        if let Ok(bb_lower) = BBLower::new(20.0, 2.0) {
            self.register_indicator("BBLower", Box::new(bb_lower));
        }

        // Keltner Channel компоненты
        if let Ok(kc_middle) = KCMiddle::new(20.0) {
            self.register_indicator("KCMiddle", Box::new(kc_middle));
        }

        if let Ok(kc_upper) = KCUpper::new(20.0, 10.0, 2.0) {
            self.register_indicator("KCUpper", Box::new(kc_upper));
        }

        if let Ok(kc_lower) = KCLower::new(20.0, 10.0, 2.0) {
            self.register_indicator("KCLower", Box::new(kc_lower));
        }
    }

    /// Регистрируем индикатор
    pub fn register_indicator(&mut self, name: &str, indicator: Box<dyn Indicator + Send + Sync>) {
        let category = indicator.category();
        let indicator_type = indicator.indicator_type();

        // Добавляем в основной список
        self.indicators.insert(name.to_string(), indicator);

        // Добавляем в категорию
        self.categories
            .entry(category.clone())
            .or_insert_with(Vec::new)
            .push(name.to_string());

        // Добавляем в тип
        self.types
            .entry(indicator_type.clone())
            .or_insert_with(Vec::new)
            .push(name.to_string());
    }

    /// Получить индикатор по имени
    pub fn get_indicator(&self, name: &str) -> Option<&Box<dyn Indicator + Send + Sync>> {
        self.indicators.get(name)
    }

    /// Получить все индикаторы определенной категории
    pub fn get_indicators_by_category(
        &self,
        category: &IndicatorCategory,
    ) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.categories
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.indicators.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Получить все индикаторы определенного типа
    pub fn get_indicators_by_type(
        &self,
        indicator_type: &IndicatorType,
    ) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.types
            .get(indicator_type)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.indicators.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Получить все OHLC индикаторы
    pub fn get_ohlc_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_type(&IndicatorType::OHLC)
    }

    /// Получить все простые индикаторы
    pub fn get_simple_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_type(&IndicatorType::Simple)
    }

    /// Получить все универсальные индикаторы
    pub fn get_universal_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_type(&IndicatorType::Universal)
    }

    /// Получить все трендовые индикаторы
    pub fn get_trend_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::Trend)
    }

    /// Получить все осцилляторы
    pub fn get_oscillator_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::Oscillator)
    }

    /// Получить все канальные индикаторы
    pub fn get_channel_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::Channel)
    }

    /// Получить все индикаторы волатильности
    pub fn get_volatility_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::Volatility)
    }

    /// Получить все объемные индикаторы
    pub fn get_volume_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::Volume)
    }

    /// Получить все индикаторы поддержки и сопротивления
    pub fn get_support_resistance_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::SupportResistance)
    }

    /// Получить все пользовательские индикаторы
    pub fn get_custom_indicators(&self) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.get_indicators_by_category(&IndicatorCategory::Custom)
    }

    /// Получить список всех имен индикаторов
    pub fn get_all_indicator_names(&self) -> Vec<String> {
        self.indicators.keys().cloned().collect()
    }

    /// Получить статистику реестра
    pub fn get_stats(&self) -> RegistryStats {
        RegistryStats {
            total_indicators: self.indicators.len(),
            by_category: self
                .categories
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
            by_type: self
                .types
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
        }
    }

    /// Поиск индикаторов по ключевым словам
    pub fn search_indicators(&self, query: &str) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        let query_lower = query.to_lowercase();

        self.indicators
            .iter()
            .filter(|(name, indicator)| {
                name.to_lowercase().contains(&query_lower)
                    || indicator
                        .description()
                        .to_lowercase()
                        .contains(&query_lower)
            })
            .map(|(_, indicator)| indicator)
            .collect()
    }

    /// Получить индикаторы с определенным количеством параметров
    pub fn get_indicators_with_parameter_count(
        &self,
        count: usize,
    ) -> Vec<&Box<dyn Indicator + Send + Sync>> {
        self.indicators
            .iter()
            .filter(|(_, indicator)| indicator.parameters().len() == count)
            .map(|(_, indicator)| indicator)
            .collect()
    }

    // Метод get_indicators_with_output_type удален - все индикаторы возвращают Vec<f32>
}

/// Статистика реестра
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_indicators: usize,
    pub by_category: HashMap<IndicatorCategory, usize>,
    pub by_type: HashMap<IndicatorType, usize>,
}

/// Фабрика индикаторов
pub struct IndicatorFactory;

impl IndicatorFactory {
    /// Создать экземпляр движка вычислений с кешированием
    pub fn runtime_engine() -> IndicatorRuntimeEngine {
        IndicatorRuntimeEngine::new()
    }

    /// Создать индикатор по имени и параметрам
    pub fn create_indicator(
        name: &str,
        parameters: HashMap<String, f32>,
    ) -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> {
        match name.to_uppercase().as_str() {
            // OHLC индикаторы
            "ATR" => {
                let period = parameters.get("period").copied().unwrap_or(14.0);
                Ok(Box::new(ATR::new(period)?))
            }
            "TRUERANGE" => Ok(Box::new(TrueRange::new()?)),
            "SUPERTREND" => {
                let period = parameters.get("period").copied().unwrap_or(10.0);
                let coeff_atr = parameters.get("coeff_atr").copied().unwrap_or(3.0);
                Ok(Box::new(SuperTrend::new(period, coeff_atr)?))
            }
            "STOCHASTIC" => {
                let k_period = parameters.get("k_period").copied().unwrap_or(14.0);
                Ok(Box::new(Stochastic::new(k_period)?))
            }
            "WATR" => {
                let period = parameters.get("period").copied().unwrap_or(14.0);
                Ok(Box::new(WATR::new(period)?))
            }
            "VTRAND" => {
                let period = parameters.get("period").copied().unwrap_or(14.0);
                Ok(Box::new(VTRAND::new(period)?))
            }
            "MAXFOR" => {
                let period = parameters.get("period").copied().unwrap_or(14.0);
                Ok(Box::new(MAXFOR::new(period)?))
            }
            "MINFOR" => {
                let period = parameters.get("period").copied().unwrap_or(14.0);
                Ok(Box::new(MINFOR::new(period)?))
            }

            // Простые индикаторы
            "SMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(SMA::new(period)?))
            }
            "EMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(EMA::new(period)?))
            }
            "RSI" => {
                let period = parameters.get("period").copied().unwrap_or(14.0);
                Ok(Box::new(RSI::new(period)?))
            }
            "WMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(WMA::new(period)?))
            }
            "AMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(AMA::new(period)?))
            }
            "ZLEMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(ZLEMA::new(period)?))
            }
            "GEOMEAN" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(GEOMEAN::new(period)?))
            }
            "AMMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(AMMA::new(period)?))
            }
            "SQWMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(SQWMA::new(period)?))
            }
            "SINEWMA" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(SINEWMA::new(period)?))
            }
            "TPBF" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(TPBF::new(period)?))
            }

            // Bollinger Bands компоненты
            "BBMIDDLE" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                let deviation = parameters.get("deviation").copied().unwrap_or(2.0);
                Ok(Box::new(BBMiddle::new(period, deviation)?))
            }
            "BBUPPER" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                let deviation = parameters.get("deviation").copied().unwrap_or(2.0);
                Ok(Box::new(BBUpper::new(period, deviation)?))
            }
            "BBLOWER" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                let deviation = parameters.get("deviation").copied().unwrap_or(2.0);
                Ok(Box::new(BBLower::new(period, deviation)?))
            }

            // Keltner Channel компоненты
            "KCMIDDLE" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                Ok(Box::new(KCMiddle::new(period)?))
            }
            "KCUPPER" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                let atr_period = parameters.get("atr_period").copied().unwrap_or(10.0);
                let atr_multiplier = parameters.get("atr_multiplier").copied().unwrap_or(2.0);
                Ok(Box::new(KCUpper::new(period, atr_period, atr_multiplier)?))
            }
            "KCLOWER" => {
                let period = parameters.get("period").copied().unwrap_or(20.0);
                let atr_period = parameters.get("atr_period").copied().unwrap_or(10.0);
                let atr_multiplier = parameters.get("atr_multiplier").copied().unwrap_or(2.0);
                Ok(Box::new(KCLower::new(period, atr_period, atr_multiplier)?))
            }

            _ => Err(IndicatorError::InvalidParameter(format!(
                "Неизвестный индикатор: {}",
                name
            ))),
        }
    }

    /// Создать индикатор по конфигурации
    pub fn create_from_config(
        config: &IndicatorConfig,
    ) -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> {
        Self::create_indicator(&config.name, config.parameters.clone())
    }

    /// Получить список всех доступных индикаторов
    pub fn get_available_indicators() -> Vec<&'static str> {
        vec![
            // OHLC индикаторы
            "ATR",
            "TrueRange",
            "SuperTrend",
            "Stochastic",
            "WATR",
            "VTRAND",
            "MAXFOR",
            "MINFOR",
            // Простые индикаторы
            "SMA",
            "EMA",
            "RSI",
            "WMA",
            "AMA",
            "ZLEMA",
            "GEOMEAN",
            "AMMA",
            "SQWMA",
            "SINEWMA",
            "TPBF",
            // Bollinger Bands компоненты
            "BBMiddle",
            "BBUpper",
            "BBLower",
            // Keltner Channel компоненты
            "KCMiddle",
            "KCUpper",
            "KCLower",
        ]
    }

    /// Получить информацию об индикаторе
    pub fn get_indicator_info(name: &str) -> Option<IndicatorMetadata> {
        match name.to_uppercase().as_str() {
            // OHLC индикаторы
            "ATR" => Some(IndicatorMetadata {
                name: "ATR".to_string(),
                category: IndicatorCategory::Volatility,
                indicator_type: IndicatorType::OHLC,
                description: "Average True Range - индикатор волатильности".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "TRUERANGE" => Some(IndicatorMetadata {
                name: "TrueRange".to_string(),
                category: IndicatorCategory::Volatility,
                indicator_type: IndicatorType::OHLC,
                description: "True Range - истинный диапазон без сглаживания".to_string(),
                parameters: Vec::new(),
            }),
            "SUPERTREND" => Some(IndicatorMetadata {
                name: "SuperTrend".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::OHLC,
                description: "SuperTrend - трендовый индикатор с полосами ATR".to_string(),
                parameters: vec!["period".to_string(), "coeff_atr".to_string()],
            }),
            "STOCHASTIC" => Some(IndicatorMetadata {
                name: "Stochastic".to_string(),
                category: IndicatorCategory::Oscillator,
                indicator_type: IndicatorType::OHLC,
                description: "Stochastic Oscillator - стохастический осциллятор".to_string(),
                parameters: vec!["k_period".to_string()],
            }),
            "WATR" => Some(IndicatorMetadata {
                name: "WATR".to_string(),
                category: IndicatorCategory::Volatility,
                indicator_type: IndicatorType::OHLC,
                description: "Weighted Average True Range - взвешенный ATR".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "VTRAND" => Some(IndicatorMetadata {
                name: "VTRAND".to_string(),
                category: IndicatorCategory::Volatility,
                indicator_type: IndicatorType::OHLC,
                description: "Volatility True Range Random - случайный волатильность".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "MAXFOR" => Some(IndicatorMetadata {
                name: "MAXFOR".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::OHLC,
                description: "Maximum For Period - максимальное значение за период".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "MINFOR" => Some(IndicatorMetadata {
                name: "MINFOR".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::OHLC,
                description: "Minimum For Period - минимальное значение за период".to_string(),
                parameters: vec!["period".to_string()],
            }),

            // Простые индикаторы
            "SMA" => Some(IndicatorMetadata {
                name: "SMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Simple Moving Average - простое скользящее среднее".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "EMA" => Some(IndicatorMetadata {
                name: "EMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Exponential Moving Average - экспоненциальное скользящее среднее"
                    .to_string(),
                parameters: vec!["period".to_string()],
            }),
            "RSI" => Some(IndicatorMetadata {
                name: "RSI".to_string(),
                category: IndicatorCategory::Oscillator,
                indicator_type: IndicatorType::Simple,
                description: "Relative Strength Index - индекс относительной силы".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "WMA" => Some(IndicatorMetadata {
                name: "WMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Weighted Moving Average - взвешенное скользящее среднее".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "AMA" => Some(IndicatorMetadata {
                name: "AMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Adaptive Moving Average - адаптивное скользящее среднее".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "ZLEMA" => Some(IndicatorMetadata {
                name: "ZLEMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Zero Lag Exponential Moving Average - EMA с нулевым лагом"
                    .to_string(),
                parameters: vec!["period".to_string()],
            }),
            "GEOMEAN" => Some(IndicatorMetadata {
                name: "GEOMEAN".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Geometric Mean - геометрическое среднее".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "AMMA" => Some(IndicatorMetadata {
                name: "AMMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description:
                    "Adaptive Moving Average Modified - модифицированное адаптивное среднее"
                        .to_string(),
                parameters: vec!["period".to_string()],
            }),
            "SQWMA" => Some(IndicatorMetadata {
                name: "SQWMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Square Weighted Moving Average - квадратично взвешенное среднее"
                    .to_string(),
                parameters: vec!["period".to_string()],
            }),
            "SINEWMA" => Some(IndicatorMetadata {
                name: "SINEWMA".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Sine Weighted Moving Average - синусоидально взвешенное среднее"
                    .to_string(),
                parameters: vec!["period".to_string()],
            }),
            "TPBF" => Some(IndicatorMetadata {
                name: "TPBF".to_string(),
                category: IndicatorCategory::Trend,
                indicator_type: IndicatorType::Simple,
                description: "Triple Exponential Moving Average - тройное экспоненциальное среднее"
                    .to_string(),
                parameters: vec!["period".to_string()],
            }),

            // Bollinger Bands компоненты
            "BBMIDDLE" => Some(IndicatorMetadata {
                name: "BBMiddle".to_string(),
                category: IndicatorCategory::Channel,
                indicator_type: IndicatorType::Simple,
                description: "Bollinger Bands Middle Line (SMA)".to_string(),
                parameters: vec!["period".to_string(), "deviation".to_string()],
            }),
            "BBUPPER" => Some(IndicatorMetadata {
                name: "BBUpper".to_string(),
                category: IndicatorCategory::Channel,
                indicator_type: IndicatorType::Simple,
                description: "Bollinger Bands Upper Line (SMA + deviation)".to_string(),
                parameters: vec!["period".to_string(), "deviation".to_string()],
            }),
            "BBLOWER" => Some(IndicatorMetadata {
                name: "BBLower".to_string(),
                category: IndicatorCategory::Channel,
                indicator_type: IndicatorType::Simple,
                description: "Bollinger Bands Lower Line (SMA - deviation)".to_string(),
                parameters: vec!["period".to_string(), "deviation".to_string()],
            }),

            // Keltner Channel компоненты
            "KCMIDDLE" => Some(IndicatorMetadata {
                name: "KCMiddle".to_string(),
                category: IndicatorCategory::Channel,
                indicator_type: IndicatorType::OHLC,
                description: "Keltner Channel Middle Line (EMA)".to_string(),
                parameters: vec!["period".to_string()],
            }),
            "KCUPPER" => Some(IndicatorMetadata {
                name: "KCUpper".to_string(),
                category: IndicatorCategory::Channel,
                indicator_type: IndicatorType::OHLC,
                description: "Keltner Channel Upper Line (EMA + ATR * multiplier)".to_string(),
                parameters: vec![
                    "period".to_string(),
                    "atr_period".to_string(),
                    "atr_multiplier".to_string(),
                ],
            }),
            "KCLOWER" => Some(IndicatorMetadata {
                name: "KCLower".to_string(),
                category: IndicatorCategory::Channel,
                indicator_type: IndicatorType::OHLC,
                description: "Keltner Channel Lower Line (EMA - ATR * multiplier)".to_string(),
                parameters: vec![
                    "period".to_string(),
                    "atr_period".to_string(),
                    "atr_multiplier".to_string(),
                ],
            }),
            _ => None,
        }
    }
}

/// Метаданные индикатора для реестра
#[derive(Debug, Clone)]
pub struct IndicatorMetadata {
    pub name: String,
    pub category: IndicatorCategory,
    pub indicator_type: IndicatorType,
    pub description: String,
    pub parameters: Vec<String>,
}

// Глобальный реестр индикаторов
pub static GLOBAL_REGISTRY: OnceLock<RwLock<IndicatorRegistry>> = OnceLock::new();

pub fn get_global_registry() -> &'static RwLock<IndicatorRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| RwLock::new(IndicatorRegistry::new()))
}

/// Утилиты для работы с реестром
pub mod registry_utils {
    use super::*;

    /// Получить все OHLC индикаторы из глобального реестра
    pub async fn get_all_ohlc_indicators() -> Vec<Box<dyn Indicator + Send + Sync>> {
        let registry = get_global_registry().read().await;
        registry
            .get_ohlc_indicators()
            .into_iter()
            .map(|indicator| indicator.clone_box())
            .collect()
    }

    /// Получить все простые индикаторы из глобального реестра
    pub async fn get_all_simple_indicators() -> Vec<Box<dyn Indicator + Send + Sync>> {
        let registry = get_global_registry().read().await;
        registry
            .get_simple_indicators()
            .into_iter()
            .map(|indicator| indicator.clone_box())
            .collect()
    }

    /// Получить индикаторы по категории из глобального реестра
    pub async fn get_indicators_by_category(
        category: &IndicatorCategory,
    ) -> Vec<Box<dyn Indicator + Send + Sync>> {
        let registry = get_global_registry().read().await;
        registry
            .get_indicators_by_category(category)
            .into_iter()
            .map(|indicator| indicator.clone_box())
            .collect()
    }

    /// Создать индикатор по имени и параметрам через глобальный реестр
    pub async fn create_indicator(
        name: &str,
        parameters: HashMap<String, f32>,
    ) -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> {
        IndicatorFactory::create_indicator(name, parameters)
    }
}

impl IndicatorFactory {
    /// Создать индикатор по имени и параметрам (асинхронная версия)
    pub async fn create_indicator_async(
        name: &str,
        parameters: HashMap<String, f32>,
    ) -> Result<Box<dyn Indicator + Send + Sync>, IndicatorError> {
        Self::create_indicator(name, parameters)
    }
}

// Расширение для клонирования Box<dyn Indicator>
pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync>;
}

// Реализуем CloneBox для Box<dyn Indicator>
impl CloneBox for Box<dyn Indicator + Send + Sync> {
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        // Создаем новый индикатор того же типа с теми же параметрами
        let name = self.name();
        let parameters = self.parameters().get_current_values();
        IndicatorFactory::create_indicator(name, parameters).unwrap_or_else(|_| {
            // Fallback: создаем пустой индикатор
            Box::new(EmptyIndicator)
        })
    }
}

// Пустой индикатор для fallback
struct EmptyIndicator;

impl Indicator for EmptyIndicator {
    fn name(&self) -> &str {
        "Empty"
    }
    fn description(&self) -> &str {
        "Empty indicator"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Custom
    }
    fn indicator_type(&self) -> IndicatorType {
        IndicatorType::Simple
    }
    // output_type удален - все индикаторы возвращают Vec<f32>
    fn parameters(&self) -> &ParameterSet {
        static EMPTY_PARAMS: OnceLock<ParameterSet> = OnceLock::new();
        EMPTY_PARAMS.get_or_init(|| ParameterSet::new())
    }
    fn min_data_points(&self) -> usize {
        1
    }
    fn calculate_simple(&self, _data: &[f32]) -> Result<Vec<f32>, IndicatorError> {
        Ok(vec![0.0])
    }
    fn calculate_ohlc(&self, _data: &OHLCData) -> Result<Vec<f32>, IndicatorError> {
        Ok(vec![0.0])
    }
    fn clone_box(&self) -> Box<dyn Indicator + Send + Sync> {
        Box::new(EmptyIndicator)
    }
}
