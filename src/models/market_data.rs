use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl MarketData {
    pub fn new(
        symbol: impl Into<String>,
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        MarketData {
            symbol: symbol.into(),
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    pub fn vwap(&self) -> f64 {
        let typical_price = (self.high + self.low + self.close) / 3.0;
        typical_price  // In a real implementation, you'd multiply by volume and divide by total volume
    }
}