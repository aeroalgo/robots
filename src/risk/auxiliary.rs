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

// Функции get_default_indicator_params, normalize_indicator_params, fill_missing_indicator_params
// перенесены в parameter_extractor.rs для устранения дублирования
// Используйте: crate::risk::get_default_indicator_params, etc.

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

// Функция extract_indicator_params_with_aliases перенесена в parameter_extractor.rs
// Используйте: crate::risk::extract_indicator_params_with_aliases

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
        "ATRTRAILINDICATORSTOP" | "ATR_TRAIL_INDICATOR_STOP" | "ATR_TRAIL_IND" => {
            let period = parameters
                .iter()
                .find(|(k, _)| k.to_lowercase() == "period")
                .and_then(|(_, v)| v.as_f64())
                .unwrap_or(14.0) as u32;

            let indicator_name = parameters
                .iter()
                .find(|(k, _)| {
                    let k_lower = k.to_lowercase();
                    k_lower == "indicator_name" || k_lower == "indicator"
                })
                .and_then(|(_, v)| v.as_str().map(|s| s.to_string()))
                .expect("indicator_name is required for ATRTrailIndicatorStop - this should be set by process_stop_handler_indicator")
                .to_uppercase();

            let reserved_keys = [
                "indicator_name",
                "indicator",
                "period",
                "coeff_atr",
                "coeff",
                "atr_coeff",
            ];
            
            // Извлекаем параметры индикатора с поддержкой alias (indicator_*, ind_*)
            let indicator_params = crate::risk::extract_indicator_params_with_aliases(parameters, &reserved_keys);

            let indicator_params = crate::risk::normalize_indicator_params(&indicator_name, &indicator_params);
            let mut params: Vec<_> = indicator_params.iter().collect();
            params.sort_by_key(|(k, _)| k.as_str());
            let params_str: String = params
                .iter()
                .map(|(k, v)| format!("{}_{}", k, **v as u32))
                .collect::<Vec<_>>()
                .join("_");
            let alias = format!("aux_stop_ind_{}_{}", indicator_name, params_str);

            vec![
                AuxiliaryIndicatorSpec::atr(period),
                AuxiliaryIndicatorSpec {
                    indicator_name,
                    parameters: indicator_params,
                    alias,
                },
            ]
        }
        "PERCENTTRAILINDICATORSTOP" | "PERCENT_TRAIL_INDICATOR_STOP" | "PERCENT_TRAIL_IND" => {
            let indicator_name = parameters
                .iter()
                .find(|(k, _)| {
                    let k_lower = k.to_lowercase();
                    k_lower == "indicator_name" || k_lower == "indicator"
                })
                .and_then(|(_, v)| v.as_str().map(|s| s.to_string()))
                .expect("indicator_name is required for PercentTrailIndicatorStop - this should be set by process_stop_handler_indicator")
                .to_uppercase();

            let reserved_keys = [
                "indicator_name",
                "indicator",
                "percentage",
                "stop_loss",
                "stop",
                "value",
                "pct",
            ];
            
            // Извлекаем параметры индикатора с поддержкой alias (indicator_*, ind_*)
            let indicator_params = crate::risk::extract_indicator_params_with_aliases(parameters, &reserved_keys);

            let indicator_params = crate::risk::normalize_indicator_params(&indicator_name, &indicator_params);
            let mut params: Vec<_> = indicator_params.iter().collect();
            params.sort_by_key(|(k, _)| k.as_str());
            let params_str: String = params
                .iter()
                .map(|(k, v)| format!("{}_{}", k, **v as u32))
                .collect::<Vec<_>>()
                .join("_");
            let alias = format!("aux_stop_ind_{}_{}", indicator_name, params_str);

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
