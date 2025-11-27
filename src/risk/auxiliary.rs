use std::collections::HashMap;

use crate::indicators::types::OHLCData;
use crate::strategy::types::StrategyParamValue;

use super::errors::StopHandlerError;

#[derive(Clone, Debug)]
pub struct AuxiliaryIndicatorSpec {
    pub indicator_name: String,
    pub parameters: HashMap<String, f64>,
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
        let mut params = HashMap::new();
        params.insert("period".to_string(), 20.0);
        params
    }
}

pub fn fill_missing_indicator_params(
    indicator_name: &str,
    existing_params: &mut HashMap<String, f64>,
) {
    let defaults = get_default_indicator_params(indicator_name);
    for (key, default_value) in defaults {
        existing_params.entry(key).or_insert(default_value);
    }
}

pub fn normalize_indicator_params(
    indicator_name: &str,
    existing_params: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let defaults = get_default_indicator_params(indicator_name);

    let mut result = HashMap::new();

    for (key, default_value) in &defaults {
        let value = existing_params.get(key).copied().unwrap_or(*default_value);
        result.insert(key.clone(), value);
    }

    result
}

pub fn collect_required_auxiliary_indicators(
    handlers: &[Box<dyn super::traits::StopHandler>],
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

pub fn compute_auxiliary_indicators(
    specs: &[AuxiliaryIndicatorSpec],
    ohlc: &OHLCData,
) -> Result<HashMap<String, Vec<f32>>, StopHandlerError> {
    use crate::indicators::registry::IndicatorFactory;

    let mut results = HashMap::new();

    for spec in specs {
        let parameters: HashMap<String, f32> = spec
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), *v as f32))
            .collect();

        let indicator = IndicatorFactory::create_indicator(&spec.indicator_name, parameters)
            .map_err(|e| {
                StopHandlerError::InvalidParameter(format!(
                    "Failed to create auxiliary indicator {}: {:?}",
                    spec.indicator_name, e
                ))
            })?;

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
            let indicator_name = parameters
                .iter()
                .find(|(k, _)| {
                    let k_lower = k.to_lowercase();
                    k_lower == "indicator_name" || k_lower == "indicator"
                })
                .and_then(|(_, v)| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "SMA".to_string())
                .to_uppercase();

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

            let indicator_params = normalize_indicator_params(&indicator_name, &indicator_params);

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
