use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarType {
    Time,
    Range {
        range_size: u32,
    },
    Volume {
        volume_size: u64,
    },
    Volatility {
        volatility_threshold: u32,
    },
    Renko {
        brick_size: u32,
    },
    HeikinAshi,
    Custom {
        name: String,
        parameters: Vec<(String, String)>,
    },
}

impl BarType {
    pub fn name(&self) -> String {
        match self {
            BarType::Time => "Time".to_string(),
            BarType::Range { .. } => "Range".to_string(),
            BarType::Volume { .. } => "Volume".to_string(),
            BarType::Volatility { .. } => "Volatility".to_string(),
            BarType::Renko { .. } => "Renko".to_string(),
            BarType::HeikinAshi => "HeikinAshi".to_string(),
            BarType::Custom { name, .. } => name.clone(),
        }
    }

    pub fn parameters(&self) -> Vec<(String, String)> {
        match self {
            BarType::Time => vec![],
            BarType::Range { range_size } => vec![("range_size".to_string(), range_size.to_string())],
            BarType::Volume { volume_size } => vec![("volume_size".to_string(), volume_size.to_string())],
            BarType::Volatility { volatility_threshold } => {
                vec![("volatility_threshold".to_string(), volatility_threshold.to_string())]
            }
            BarType::Renko { brick_size } => vec![("brick_size".to_string(), brick_size.to_string())],
            BarType::HeikinAshi => vec![],
            BarType::Custom { parameters, .. } => parameters.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BarTypeConfig {
    pub bar_type: BarType,
    pub timeframe: Option<crate::data_model::types::TimeFrame>,
}

impl BarTypeConfig {
    pub fn new(bar_type: BarType) -> Self {
        Self {
            bar_type,
            timeframe: None,
        }
    }

    pub fn with_timeframe(bar_type: BarType, timeframe: crate::data_model::types::TimeFrame) -> Self {
        Self {
            bar_type,
            timeframe: Some(timeframe),
        }
    }
}

