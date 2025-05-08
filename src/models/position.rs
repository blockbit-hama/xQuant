use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
}

impl Position {
    pub fn new(symbol: impl Into<String>, quantity: f64, entry_price: f64) -> Self {
        Position {
            symbol: symbol.into(),
            quantity,
            entry_price,
            current_price: entry_price,
            unrealized_pnl: 0.0,
        }
    }

    pub fn is_long(&self) -> bool {
        self.quantity > 0.0
    }

    pub fn is_short(&self) -> bool {
        self.quantity < 0.0
    }

    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
        self.calculate_pnl();
    }

    pub fn calculate_pnl(&mut self) {
        if self.quantity != 0.0 {
            let direction = self.quantity.signum();
            self.unrealized_pnl = direction * (self.current_price - self.entry_price).abs() * self.quantity.abs();
        } else {
            self.unrealized_pnl = 0.0;
        }
    }
}