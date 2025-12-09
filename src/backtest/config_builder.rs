use super::BacktestConfig;

pub struct BacktestConfigBuilder {
    initial_capital: Option<f64>,
    use_full_capital: Option<bool>,
    reinvest_profits: Option<bool>,
}

impl BacktestConfigBuilder {
    pub fn new() -> Self {
        Self {
            initial_capital: None,
            use_full_capital: None,
            reinvest_profits: None,
        }
    }

    pub fn with_initial_capital(mut self, capital: f64) -> Self {
        self.initial_capital = Some(capital);
        self
    }

    pub fn with_full_capital(mut self, use_full: bool) -> Self {
        self.use_full_capital = Some(use_full);
        self
    }

    pub fn with_reinvest_profits(mut self, reinvest: bool) -> Self {
        self.reinvest_profits = Some(reinvest);
        self
    }

    pub fn build(self) -> BacktestConfig {
        BacktestConfig {
            initial_capital: self.initial_capital.unwrap_or(10000.0),
            use_full_capital: self.use_full_capital.unwrap_or(false),
            reinvest_profits: self.reinvest_profits.unwrap_or(false),
        }
    }
}

impl Default for BacktestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
