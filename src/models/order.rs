use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash,PartialEq)]
pub struct OrderId(pub String);

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLimit,
    TrailingStop,
    Iceberg,
    VWAP,
    TWAP,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: f64,
    pub stop_price: Option<f64>,
    pub time_in_force: String,
    pub created_at: i64,
    pub client_order_id: Option<String>,

    // Advanced order parameters
    pub iceberg_qty: Option<f64>,           // For Iceberg orders
    pub trailing_delta: Option<f64>,        // For Trailing Stop orders
    pub execution_interval: Option<i64>,    // For TWAP/VWAP orders
    pub target_percentage: Option<f64>,     // For participation rate
}

impl Order {
    pub fn new(
        symbol: impl Into<String>,
        side: OrderSide,
        order_type: OrderType,
        quantity: f64,
        price: f64,
    ) -> Self {
        Order {
            id: OrderId("".to_string()),  // This will be set by the exchange
            symbol: symbol.into(),
            side,
            order_type,
            quantity,
            price,
            stop_price: None,
            time_in_force: "GTC".to_string(),  // Good Till Cancelled
            created_at: chrono::Utc::now().timestamp_millis(),
            client_order_id: None,
            iceberg_qty: None,
            trailing_delta: None,
            execution_interval: None,
            target_percentage: None,
        }
    }

    pub fn with_stop_price(mut self, stop_price: f64) -> Self {
        self.stop_price = Some(stop_price);
        self
    }

    pub fn with_time_in_force(mut self, time_in_force: impl Into<String>) -> Self {
        self.time_in_force = time_in_force.into();
        self
    }

    pub fn with_client_order_id(mut self, client_order_id: impl Into<String>) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }

    pub fn with_iceberg_qty(mut self, iceberg_qty: f64) -> Self {
        self.iceberg_qty = Some(iceberg_qty);
        self.order_type = OrderType::Iceberg;
        self
    }

    pub fn with_trailing_delta(mut self, trailing_delta: f64) -> Self {
        self.trailing_delta = Some(trailing_delta);
        self.order_type = OrderType::TrailingStop;
        self
    }

    pub fn with_vwap_params(mut self, execution_interval: i64, target_percentage: Option<f64>) -> Self {
        self.execution_interval = Some(execution_interval);
        self.target_percentage = target_percentage;
        self.order_type = OrderType::VWAP;
        self
    }

    pub fn with_twap_params(mut self, execution_interval: i64) -> Self {
        self.execution_interval = Some(execution_interval);
        self.order_type = OrderType::TWAP;
        self
    }
}