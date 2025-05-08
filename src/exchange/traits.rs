use async_trait::async_trait;
use uuid::Uuid;

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderStatus, OrderType};
use crate::models::trade::Trade;

/// The `Exchange` trait defines the interface for interacting with trading exchanges.
/// It will be implemented by real exchange connectors and mock implementations.
#[async_trait]
pub trait Exchange: Send + Sync {
    /// Submit a new order to the exchange
    async fn submit_order(&mut self, order: Order) -> Result<OrderId, TradingError>;

    /// Cancel an existing order
    async fn cancel_order(&mut self, order_id: &OrderId) -> Result<(), TradingError>;

    /// Modify an existing order
    async fn modify_order(&mut self, order_id: &OrderId, order: Order) -> Result<OrderId, TradingError>;

    /// Get the status of an order
    async fn get_order_status(&self, order_id: &OrderId) -> Result<OrderStatus, TradingError>;

    /// Get all open orders
    async fn get_open_orders(&self) -> Result<Vec<Order>, TradingError>;

    /// Get recent trades for a specific symbol
    async fn get_recent_trades(&self, symbol: &str, limit: Option<usize>) -> Result<Vec<Trade>, TradingError>;

    /// Get current market data for a symbol
    async fn get_market_data(&self, symbol: &str) -> Result<MarketData, TradingError>;

    /// Get historical market data for a symbol
    async fn get_historical_data(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<MarketData>, TradingError>;

    /// Get account balance
    async fn get_balance(&self, asset: &str) -> Result<f64, TradingError>;
}