use crate::position::PositionManager;

use super::constants;

pub struct EquityCalculator {
    cached_equity: Option<f64>,
    last_equity_bar: usize,
    initial_capital: f64,
}

impl EquityCalculator {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            cached_equity: None,
            last_equity_bar: 0,
            initial_capital,
        }
    }

    pub fn calculate(
        &mut self,
        position_manager: &PositionManager,
        has_open_positions: bool,
        equity_changed: bool,
        processed_bars: usize,
    ) -> f64 {
        let needs_snapshot = (has_open_positions
            && (equity_changed || self.cached_equity.is_none()))
            || (has_open_positions
                && processed_bars - self.last_equity_bar >= constants::EQUITY_UPDATE_INTERVAL)
            || self.cached_equity.is_none();

        if !needs_snapshot {
            return self.cached_equity.unwrap();
        }

        let total_equity = position_manager.portfolio_snapshot().total_equity;
        let current_equity = self.initial_capital + total_equity;

        let should_update_cache = if has_open_positions
            && (equity_changed || self.cached_equity.is_none())
        {
            self.cached_equity.map_or(true, |cached| {
                (cached - current_equity).abs() > constants::EQUITY_CACHE_THRESHOLD
            })
        } else if has_open_positions
            && processed_bars - self.last_equity_bar >= constants::EQUITY_UPDATE_INTERVAL
        {
            (self.cached_equity.unwrap() - current_equity).abs() > constants::EQUITY_CACHE_THRESHOLD
        } else {
            true
        };

        if should_update_cache {
            self.cached_equity = Some(current_equity);
            self.last_equity_bar = processed_bars;
        }

        current_equity
    }

    pub fn reset(&mut self) {
        self.cached_equity = None;
        self.last_equity_bar = 0;
    }
}
