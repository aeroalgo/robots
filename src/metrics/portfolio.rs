#[derive(Clone, Debug, Default)]
pub struct PortfolioSnapshot {
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub exposure: f64,
    pub total_equity: f64,
}

impl PortfolioSnapshot {
    pub fn reset(&mut self) {
        self.realized_pnl = 0.0;
        self.unrealized_pnl = 0.0;
        self.exposure = 0.0;
        self.total_equity = 0.0;
    }

    pub fn update_equity(&mut self) {
        self.total_equity = self.realized_pnl + self.unrealized_pnl;
    }
}
