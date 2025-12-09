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
            BarType::Range { range_size } => {
                vec![("range_size".to_string(), range_size.to_string())]
            }
            BarType::Volume { volume_size } => {
                vec![("volume_size".to_string(), volume_size.to_string())]
            }
            BarType::Volatility {
                volatility_threshold,
            } => {
                vec![(
                    "volatility_threshold".to_string(),
                    volatility_threshold.to_string(),
                )]
            }
            BarType::Renko { brick_size } => {
                vec![("brick_size".to_string(), brick_size.to_string())]
            }
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

    pub fn with_timeframe(
        bar_type: BarType,
        timeframe: crate::data_model::types::TimeFrame,
    ) -> Self {
        Self {
            bar_type,
            timeframe: Some(timeframe),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::types::TimeFrame;

    #[test]
    fn test_bar_type_time() {
        let bar_type = BarType::Time;
        assert_eq!(bar_type.name(), "Time");
        assert_eq!(bar_type.parameters(), vec![]);
    }

    #[test]
    fn test_bar_type_range() {
        let bar_type = BarType::Range { range_size: 100 };
        assert_eq!(bar_type.name(), "Range");
        assert_eq!(
            bar_type.parameters(),
            vec![("range_size".to_string(), "100".to_string())]
        );
    }

    #[test]
    fn test_bar_type_volume() {
        let bar_type = BarType::Volume { volume_size: 1000 };
        assert_eq!(bar_type.name(), "Volume");
        assert_eq!(
            bar_type.parameters(),
            vec![("volume_size".to_string(), "1000".to_string())]
        );
    }

    #[test]
    fn test_bar_type_volatility() {
        let bar_type = BarType::Volatility {
            volatility_threshold: 50,
        };
        assert_eq!(bar_type.name(), "Volatility");
        assert_eq!(
            bar_type.parameters(),
            vec![("volatility_threshold".to_string(), "50".to_string())]
        );
    }

    #[test]
    fn test_bar_type_renko() {
        let bar_type = BarType::Renko { brick_size: 25 };
        assert_eq!(bar_type.name(), "Renko");
        assert_eq!(
            bar_type.parameters(),
            vec![("brick_size".to_string(), "25".to_string())]
        );
    }

    #[test]
    fn test_bar_type_heikin_ashi() {
        let bar_type = BarType::HeikinAshi;
        assert_eq!(bar_type.name(), "HeikinAshi");
        assert_eq!(bar_type.parameters(), vec![]);
    }

    #[test]
    fn test_bar_type_custom() {
        let bar_type = BarType::Custom {
            name: "MyCustomBar".to_string(),
            parameters: vec![("param1".to_string(), "value1".to_string())],
        };
        assert_eq!(bar_type.name(), "MyCustomBar");
        assert_eq!(
            bar_type.parameters(),
            vec![("param1".to_string(), "value1".to_string())]
        );
    }

    #[test]
    fn test_bar_type_config_new() {
        let bar_type = BarType::Time;
        let config = BarTypeConfig::new(bar_type);
        assert_eq!(config.bar_type.name(), "Time");
        assert!(config.timeframe.is_none());
    }

    #[test]
    fn test_bar_type_config_with_timeframe() {
        let bar_type = BarType::Range { range_size: 100 };
        let timeframe = TimeFrame::Minutes(5);
        let config = BarTypeConfig::with_timeframe(bar_type, timeframe.clone());
        assert_eq!(config.bar_type.name(), "Range");
        assert_eq!(config.timeframe, Some(timeframe));
    }

    #[test]
    fn test_bar_type_equality() {
        let bar1 = BarType::Range { range_size: 100 };
        let bar2 = BarType::Range { range_size: 100 };
        let bar3 = BarType::Range { range_size: 200 };
        assert_eq!(bar1, bar2);
        assert_ne!(bar1, bar3);
    }

    #[test]
    fn test_bar_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(BarType::Time);
        set.insert(BarType::Range { range_size: 100 });
        set.insert(BarType::Volume { volume_size: 1000 });
        assert_eq!(set.len(), 3);
    }
}
