use std::collections::HashMap;

use crate::strategy::types::PositionDirection;

#[derive(Clone, Debug)]
pub struct PositionRiskState {
    pub position_id: String,
    pub direction: PositionDirection,
    pub entry_price: f64,
    pub max_high_since_entry: f64,
    pub min_low_since_entry: f64,
    pub current_stop: Option<f64>,
    pub entry_bar_index: Option<usize>,
}

impl PositionRiskState {
    pub fn new(
        position_id: String,
        direction: PositionDirection,
        entry_price: f64,
        initial_high: f64,
        initial_low: f64,
    ) -> Self {
        Self {
            position_id,
            direction,
            entry_price,
            max_high_since_entry: initial_high,
            min_low_since_entry: initial_low,
            current_stop: None,
            entry_bar_index: None,
        }
    }

    pub fn update_price_extremes(&mut self, high: f64, low: f64) {
        self.max_high_since_entry = self.max_high_since_entry.max(high);
        self.min_low_since_entry = self.min_low_since_entry.min(low);
    }

    pub fn update_stop(&mut self, new_stop: f64) {
        match self.direction {
            PositionDirection::Long => {
                self.current_stop = Some(
                    self.current_stop
                        .map(|prev| prev.max(new_stop))
                        .unwrap_or(new_stop),
                );
            }
            PositionDirection::Short => {
                self.current_stop = Some(
                    self.current_stop
                        .map(|prev| prev.min(new_stop))
                        .unwrap_or(new_stop),
                );
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RiskStateBook {
    states: HashMap<String, PositionRiskState>,
}

impl RiskStateBook {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn insert(&mut self, state: PositionRiskState) {
        self.states.insert(state.position_id.clone(), state);
    }

    pub fn get(&self, position_id: &str) -> Option<&PositionRiskState> {
        self.states.get(position_id)
    }

    pub fn get_mut(&mut self, position_id: &str) -> Option<&mut PositionRiskState> {
        self.states.get_mut(position_id)
    }

    pub fn remove(&mut self, position_id: &str) -> Option<PositionRiskState> {
        self.states.remove(position_id)
    }

    pub fn contains(&self, position_id: &str) -> bool {
        self.states.contains_key(position_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PositionRiskState)> {
        self.states.iter()
    }

    pub fn clear(&mut self) {
        self.states.clear();
    }

    pub fn position_ids(&self) -> Vec<String> {
        self.states.keys().cloned().collect()
    }
}
