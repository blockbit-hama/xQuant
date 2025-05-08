use serde::{Deserialize, Serialize};

use crate::models::order::{OrderId, OrderSide};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: i64,
    pub order_id: OrderId,
    pub side: OrderSide,
}

impl Trade {
    pub fn new(
        id: impl Into<String>,
        symbol: impl Into<String>,
        price: f64,
        quantity: f64,
        timestamp: i64,
        order_id: OrderId,
        side: OrderSide,
    ) -> Self {
        Trade {
            id: id.into(),
            symbol: symbol.into(),
            price,
            quantity,
            timestamp,
            order_id,
            side,
        }
    }

    pub fn value(&self) -> f64 {
        self.price * self.quantity
    }
}