use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicI64, Ordering};

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
  pub recv_window_ms: u64,
  pub min_interval_ms: u64,
  pub last_request_ms: AtomicI64,
  pub time_offset_ms: AtomicI64,
}

impl BinanceFuturesExchange {
  pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
    BinanceFuturesExchange {
      base_url: base_url.into(),
      api_key: api_key.into(),
      api_secret: api_secret.into(),
      http: reqwest::Client::new(),
      recv_window_ms: 5000,
      min_interval_ms: 50,
      last_request_ms: AtomicI64::new(0),
      time_offset_ms: AtomicI64::new(0),
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

  async fn throttle(&self) {
    let now = Self::timestamp_ms();
    let last = self.last_request_ms.load(Ordering::SeqCst);
    let elapsed = (now - last) as u64;
    if elapsed < self.min_interval_ms {
      let sleep_ms = self.min_interval_ms - elapsed;
      tokio::time::sleep(std::time::Duration::from_millis(sleep_ms)).await;
    }
    self.last_request_ms.store(Self::timestamp_ms(), Ordering::SeqCst);
  }

  fn ts_with_offset(&self) -> i64 { Self::timestamp_ms() + self.time_offset_ms.load(Ordering::SeqCst) }
}

#[async_trait]
impl Exchange for BinanceFuturesExchange {
  async fn submit_order(&mut self, order: Order) -> Result<OrderId, TradingError> {
    // Build Binance Futures order params by type
    let side = match order.side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
    let ts = self.ts_with_offset();
    let mut params = vec![
      format!("symbol={}", order.symbol),
      format!("side={}", side),
      format!("quantity={}", order.quantity),
      format!("timestamp={}", ts),
      format!("recvWindow={}", self.recv_window_ms),
    ];

    match order.order_type {
      OrderType::Market | OrderType::VWAP | OrderType::TWAP => {
        params.push("type=MARKET".to_string());
      }
      OrderType::Limit | OrderType::Iceberg => {
        params.push("type=LIMIT".to_string());
        params.push(format!("price={}", order.price));
        params.push(format!("timeInForce={}", order.time_in_force));
        if let Some(ice) = order.iceberg_qty { params.push(format!("icebergQty={}", ice)); }
      }
      OrderType::StopLoss | OrderType::StopLimit => {
        // Use STOP (limit) if price provided, else STOP_MARKET
        if order.price > 0.0 {
          params.push("type=STOP".to_string());
          params.push(format!("price={}", order.price));
          params.push(format!("timeInForce={}", order.time_in_force));
        } else {
          params.push("type=STOP_MARKET".to_string());
        }
        let stop = order.stop_price.unwrap_or(order.price);
        if stop > 0.0 { params.push(format!("stopPrice={}", stop)); }
      }
      OrderType::TrailingStop => {
        // Binance requires callbackRate in percent (0.1-5.0)
        params.push("type=TRAILING_STOP_MARKET".to_string());
        let cb = order.trailing_delta.unwrap_or(0.5).max(0.1).min(5.0);
        params.push(format!("callbackRate={}", cb));
        // Optional activationPrice maps from stop_price when available
        if let Some(act) = order.stop_price { params.push(format!("activationPrice={}", act)); }
      }
    }
    let query = params.join("&");
    let signature = self.sign(&query);
    let url = format!("{}/fapi/v1/order?{}&signature={}", self.base_url, query, signature);
    self.throttle().await;
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
    self.throttle().await;
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
    Ok(MarketData { symbol: symbol.to_string(), timestamp: self.ts_with_offset(), open: close, high, low, close, volume })
  }

  async fn get_historical_data(&self, _symbol: &str, _interval: &str, _start_time: i64, _end_time: Option<i64>, _limit: Option<usize>) -> Result<Vec<MarketData>, TradingError> {
    Ok(Vec::new())
  }

  async fn get_balance(&self, _asset: &str) -> Result<f64, TradingError> { Ok(0.0) }

  async fn set_futures_leverage(&mut self, symbol: &str, leverage: u32) -> Result<(), TradingError> {
    let ts = self.ts_with_offset();
    let q = format!("symbol={}&leverage={}&timestamp={}&recvWindow={}", symbol, leverage, ts, self.recv_window_ms);
    let url = format!("{}/fapi/v1/leverage?{}&signature={}", self.base_url, q, self.sign(&q));
    self.throttle().await;
    let res = self.http.post(url).header("X-MBX-APIKEY", &self.api_key).send().await
      .map_err(|e| TradingError::ExchangeError(format!("set leverage http error: {}", e)))?;
    if !res.status().is_success() { return Err(TradingError::ExchangeError(format!("set leverage failed: {}", res.status()))); }
    Ok(())
  }

  async fn set_futures_position_mode(&mut self, hedge: bool) -> Result<(), TradingError> {
    let ts = self.ts_with_offset();
    let q = format!("dualSidePosition={}&timestamp={}&recvWindow={}", if hedge {"true"} else {"false"}, ts, self.recv_window_ms);
    let url = format!("{}/fapi/v1/positionSide/dual?{}&signature={}", self.base_url, q, self.sign(&q));
    self.throttle().await;
    let res = self.http.post(url).header("X-MBX-APIKEY", &self.api_key).send().await
      .map_err(|e| TradingError::ExchangeError(format!("set position mode http error: {}", e)))?;
    if !res.status().is_success() { return Err(TradingError::ExchangeError(format!("set position mode failed: {}", res.status()))); }
    Ok(())
  }

  async fn set_futures_margin_mode(&mut self, symbol: &str, isolated: bool) -> Result<(), TradingError> {
    // NOTE: Binance uses marginType=ISOLATED|CROSSED
    let ts = self.ts_with_offset();
    let q = format!("symbol={}&marginType={}&timestamp={}&recvWindow={}", symbol, if isolated {"ISOLATED"} else {"CROSSED"}, ts, self.recv_window_ms);
    let url = format!("{}/fapi/v1/marginType?{}&signature={}", self.base_url, q, self.sign(&q));
    self.throttle().await;
    let res = self.http.post(url).header("X-MBX-APIKEY", &self.api_key).send().await
      .map_err(|e| TradingError::ExchangeError(format!("set margin mode http error: {}", e)))?;
    if !res.status().is_success() { return Err(TradingError::ExchangeError(format!("set margin mode failed: {}", res.status()))); }
    Ok(())
  }

  async fn sync_time(&mut self) -> Result<(), TradingError> {
    // GET /fapi/v1/time
    let url = format!("{}/fapi/v1/time", self.base_url);
    self.throttle().await;
    let res = self.http.get(url).send().await
      .map_err(|e| TradingError::ExchangeError(format!("time http error: {}", e)))?;
    if !res.status().is_success() { return Err(TradingError::ExchangeError(format!("time failed: {}", res.status()))); }
    let v = res.json::<serde_json::Value>().await
      .map_err(|e| TradingError::ExchangeError(format!("time parse error: {}", e)))?;
    if let Some(server_ts) = v.get("serverTime").and_then(|t| t.as_i64()) {
      let local = Self::timestamp_ms();
      let offset = server_ts - local;
      self.time_offset_ms.store(offset, Ordering::SeqCst);
      Ok(())
    } else {
      Err(TradingError::ParseError("serverTime missing".into()))
    }
  }
}
