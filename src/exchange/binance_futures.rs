use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};
use crate::models::trade::Trade;

type HmacSha256 = Hmac<Sha256>;

/// Binance USDT-M Futures REST connector (minimal subset)
pub struct BinanceFuturesExchange {
  pub base_url: String,
  pub api_key: String,
  pub api_secret: String,
  pub http: reqwest::Client,
}

impl BinanceFuturesExchange {
  pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
    BinanceFuturesExchange {
      base_url: base_url.into(),
      api_key: api_key.into(),
      api_secret: api_secret.into(),
      http: reqwest::Client::new(),
    }
  }

  fn timestamp_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
  }

  fn sign(&self, query: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes()).unwrap();
    mac.update(query.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
  }
}

#[async_trait]
impl Exchange for BinanceFuturesExchange {
  async fn submit_order(&mut self, order: Order) -> Result<OrderId, TradingError> {
    // Minimal MARKET/LIMIT order via Binance Futures API (simplified)
    let side = match order.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
    let order_type = match order.order_type { OrderType::Market => "MARKET", OrderType::Limit => "LIMIT", _ => "MARKET" };
    let ts = Self::timestamp_ms();
    let mut params = vec![
      format!("symbol={}", order.symbol),
      format!("side={}", side),
      format!("type={}", order_type),
      format!("quantity={}", order.quantity),
      format!("timestamp={}", ts),
    ];
    if let OrderType::Limit = order.order_type {
      params.push(format!("price={}", order.price));
      params.push("timeInForce=GTC".to_string());
    }
    let query = params.join("&");
    let signature = self.sign(&query);
    let url = format!("{}/fapi/v1/order?{}&signature={}", self.base_url, query, signature);
    let res = self.http
      .post(url)
      .header("X-MBX-APIKEY", &self.api_key)
      .send().await
      .map_err(|e| TradingError::ExchangeError(format!("submit_order http error: {}", e)))?;
    if !res.status().is_success() { return Err(TradingError::ExchangeError(format!("submit_order failed: {}", res.status()))); }
    // In real code parse orderId
    Ok(OrderId(format!("binfut-{}", ts)))
  }

  async fn cancel_order(&mut self, _order_id: &OrderId) -> Result<(), TradingError> {
    // Not implemented
    Ok(())
  }

  async fn modify_order(&mut self, _order_id: &OrderId, _order: Order) -> Result<OrderId, TradingError> {
    Err(TradingError::ExchangeError("modify not supported in connector".to_string()))
  }

  async fn get_order_status(&self, _order_id: &OrderId) -> Result<OrderStatus, TradingError> {
    Ok(OrderStatus::New)
  }

  async fn get_open_orders(&self) -> Result<Vec<Order>, TradingError> {
    Ok(Vec::new())
  }

  async fn get_recent_trades(&self, _symbol: &str, _limit: Option<usize>) -> Result<Vec<Trade>, TradingError> {
    Ok(Vec::new())
  }

  async fn get_market_data(&self, symbol: &str) -> Result<MarketData, TradingError> {
    // Use 24hr ticker as a simple data source
    let url = format!("{}/fapi/v1/ticker/24hr?symbol={}", self.base_url, symbol);
    let res = self.http.get(url)
      .send().await
      .map_err(|e| TradingError::ExchangeError(format!("market_data http error: {}", e)))?;
    let status = res.status();
    let json = res.json::<serde_json::Value>().await
      .map_err(|e| TradingError::ExchangeError(format!("market_data parse error: {}", e)))?;
    if !status.is_success() { return Err(TradingError::ExchangeError(format!("market_data failed: {}", status))); }
    let close = json.get("lastPrice").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let high = json.get("highPrice").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(close);
    let low = json.get("lowPrice").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(close);
    let volume = json.get("volume").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    Ok(MarketData { symbol: symbol.to_string(), timestamp: Self::timestamp_ms(), open: close, high, low, close, volume })
  }

  async fn get_historical_data(&self, _symbol: &str, _interval: &str, _start_time: i64, _end_time: Option<i64>, _limit: Option<usize>) -> Result<Vec<MarketData>, TradingError> {
    Ok(Vec::new())
  }

  async fn get_balance(&self, _asset: &str) -> Result<f64, TradingError> { Ok(0.0) }
}
