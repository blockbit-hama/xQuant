use async_trait::async_trait;
use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderStatus};
use crate::models::trade::Trade;
use std::time::{SystemTime, UNIX_EPOCH};

/// A no-op exchange connector that records orders without sending them
pub struct DryRunExchange;

impl DryRunExchange {
  pub fn new() -> Self { Self }
  fn ts() -> i64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64 }
}

#[async_trait]
impl Exchange for DryRunExchange {
  async fn submit_order(&mut self, order: Order) -> Result<OrderId, TradingError> {
    let id = OrderId(format!("dry-{}-{}", order.symbol, Self::ts()));
    Ok(id)
  }

  async fn cancel_order(&mut self, _order_id: &OrderId) -> Result<(), TradingError> { Ok(()) }

  async fn modify_order(&mut self, _order_id: &OrderId, _order: Order) -> Result<OrderId, TradingError> {
    Err(TradingError::ExchangeError("dry-run modify not supported".into()))
  }

  async fn get_order_status(&self, _order_id: &OrderId) -> Result<OrderStatus, TradingError> {
    Ok(OrderStatus::New)
  }

  async fn get_open_orders(&self) -> Result<Vec<Order>, TradingError> { Ok(vec![]) }

  async fn get_recent_trades(&self, _symbol: &str, _limit: Option<usize>) -> Result<Vec<Trade>, TradingError> { Ok(vec![]) }

  async fn get_market_data(&self, symbol: &str) -> Result<MarketData, TradingError> {
    Ok(MarketData { symbol: symbol.to_string(), timestamp: Self::ts(), open: 0.0, high: 0.0, low: 0.0, close: 0.0, volume: 0.0 })
  }

  async fn get_historical_data(&self, _symbol: &str, _interval: &str, _start_time: i64, _end_time: Option<i64>, _limit: Option<usize>) -> Result<Vec<MarketData>, TradingError> { Ok(vec![]) }

  async fn get_balance(&self, _asset: &str) -> Result<f64, TradingError> { Ok(0.0) }
}
