use axum::{routing::{get, post}, Router, extract::{Path, State}, response::IntoResponse};
use axum::extract::ws::{WebSocketUpgrade, Message, WebSocket};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};

use crate::core::strategy_manager::StrategyManager;
use crate::exchange::traits::Exchange;
use crate::strategies::Strategy;
use crate::order_core::manager::OrderManager;
use crate::core::risk_manager::RiskManager;
use crate::models::order::{Order, OrderSide, OrderType, OrderId};

#[derive(Clone)]
pub struct AppState {
  pub exchange: Arc<RwLock<dyn Exchange>>, 
  pub strategy_manager: Arc<RwLock<StrategyManager>>, 
  // Note: OrderManager is in main runtime; for API calls we recreate lightweight paths via exchange+repo if needed.
}

#[derive(Debug, Serialize)]
struct Health { status: &'static str }

pub fn build_router(state: AppState) -> Router {
  let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any);

  Router::new()
    .route("/health", get(|| async { axum::Json(Health { status: "ok" }) }))
    .route("/strategies", get(list_strategies))
    .route("/strategies/ta", post(create_ta_strategy))
    .route("/strategies/vwap", post(create_vwap_strategy))
    .route("/strategies/twap", post(create_twap_strategy))
    .route("/strategies/iceberg", post(create_iceberg_strategy))
    .route("/strategies/trailing", post(create_trailing_strategy))
    .route("/strategies/:name/toggle", post(toggle_strategy))
    .route("/strategies/:name", get(get_strategy_info).delete(delete_strategy))
    // futures settings
    .route("/futures/position_mode", post(set_position_mode))
    .route("/futures/margin_mode", post(set_margin_mode))
    .route("/futures/leverage", post(set_leverage))
    .route("/futures/settings", post(apply_futures_settings))
    // market data
    .route("/market/:symbol", get(get_market_snapshot))
    .route("/positions", get(get_positions))
    // orders
    .route("/orders", post(create_order))
    .route("/orders/:id", get(get_order_status).delete(cancel_order))
    .route("/ws/prices/:symbol", get(ws_prices))
    .route("/ws/orders", get(ws_orders))
    .route("/ws/positions", get(ws_positions))
    .route("/ws/strategies", get(ws_strategies))
    .with_state(state)
    .layer(cors)
}

async fn list_strategies(State(state): State<AppState>) -> Result<axum::Json<Vec<(String, bool)>>, axum::http::StatusCode> {
  let mgr = state.strategy_manager.read().await;
  Ok(axum::Json(mgr.list_strategies()))
}

#[derive(Debug, Deserialize)]
struct CreateReq {
  symbol: String,
  strategy_type: String,
  params: serde_json::Value,
}

async fn create_ta_strategy(State(state): State<AppState>, axum::Json(req): axum::Json<CreateReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  use crate::strategies::technical::TechnicalStrategy;

  let strategy_result = match req.strategy_type.as_str() {
    "ma_crossover" => {
      let fast = req.params.get("fast_period").and_then(|v| v.as_u64()).unwrap_or(12) as usize;
      let slow = req.params.get("slow_period").and_then(|v| v.as_u64()).unwrap_or(26) as usize;
      TechnicalStrategy::ma_crossover(req.symbol.clone(), fast, slow)
    }
    , "rsi" => {
      let period = req.params.get("period").and_then(|v| v.as_u64()).unwrap_or(14) as usize;
      let oversold = req.params.get("oversold").and_then(|v| v.as_f64()).unwrap_or(30.0);
      let overbought = req.params.get("overbought").and_then(|v| v.as_f64()).unwrap_or(70.0);
      TechnicalStrategy::rsi(req.symbol.clone(), period, oversold, overbought)
    }
    , _ => Err(crate::error::TradingError::InvalidStrategy("unknown".into()))
  };

  match strategy_result {
    Ok(strategy) => {
      let name = Strategy::name(&strategy).to_string();
      let mut mgr = state.strategy_manager.write().await;
      if let Err(_) = mgr.add_strategy(Box::new(strategy)) { return Err(axum::http::StatusCode::BAD_REQUEST); }
      Ok(axum::Json(serde_json::json!({"status":"success","strategy_name": name})))
    }
    Err(_) => Err(axum::http::StatusCode::BAD_REQUEST)
  }
}

#[derive(Debug, Deserialize)]
struct VwapReq { symbol: String, side: String, quantity: f64, window: i64, participation: Option<f64> }
async fn create_vwap_strategy(State(state): State<AppState>, axum::Json(req): axum::Json<VwapReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  use crate::strategies::vwap::VwapStrategy;
  use crate::models::order::OrderSide;
  let side = match req.side.to_lowercase().as_str() { "buy" => OrderSide::Buy, "sell" => OrderSide::Sell, _ => return Err(axum::http::StatusCode::BAD_REQUEST)};
  let s = VwapStrategy::new(&req.symbol, side, req.quantity, req.window as i64, (req.participation.unwrap_or(0.1)*100.0) as usize);
  let mut mgr = state.strategy_manager.write().await;
  mgr.add_strategy(Box::new(s)).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"success","strategy_name": format!("VWAP-{}", req.symbol)})))
}

#[derive(Debug, Deserialize)]
struct TwapReq { symbol: String, side: String, quantity: f64, window: i64 }
async fn create_twap_strategy(State(state): State<AppState>, axum::Json(req): axum::Json<TwapReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  use crate::strategies::twap::TwapStrategy;
  use crate::models::order::OrderSide;
  let side = match req.side.to_lowercase().as_str() { "buy" => OrderSide::Buy, "sell" => OrderSide::Sell, _ => return Err(axum::http::StatusCode::BAD_REQUEST)};
  let s = TwapStrategy::new(&req.symbol, side, req.quantity, req.window as i64, 5);
  let mut mgr = state.strategy_manager.write().await;
  mgr.add_strategy(Box::new(s)).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"success","strategy_name": format!("TWAP-{}", req.symbol)})))
}

#[derive(Debug, Deserialize)]
struct IcebergReq { symbol: String, side: String, total_qty: f64, visible_qty: f64, price: f64 }
async fn create_iceberg_strategy(State(state): State<AppState>, axum::Json(req): axum::Json<IcebergReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  use crate::strategies::iceberg::IcebergStrategy;
  use crate::models::order::OrderSide;
  let side = match req.side.to_lowercase().as_str() { "buy" => OrderSide::Buy, "sell" => OrderSide::Sell, _ => return Err(axum::http::StatusCode::BAD_REQUEST)};
  let s = IcebergStrategy::new(req.symbol.clone(), side, req.total_qty, req.visible_qty, req.price);
  let mut mgr = state.strategy_manager.write().await;
  mgr.add_strategy(Box::new(s)).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"success","strategy_name": format!("ICEBERG-{}", req.symbol)})))
}

#[derive(Debug, Deserialize)]
struct TrailingReq { symbol: String, side: String, qty: f64, callback: f64, activation: Option<f64> }
async fn create_trailing_strategy(State(state): State<AppState>, axum::Json(req): axum::Json<TrailingReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  use crate::strategies::trailing_stop::TrailingStopStrategy;
  use crate::models::order::OrderSide;
  let side = match req.side.to_lowercase().as_str() { "buy" => OrderSide::Buy, "sell" => OrderSide::Sell, _ => return Err(axum::http::StatusCode::BAD_REQUEST)};
  let mut s = TrailingStopStrategy::new(req.symbol.clone(), side, req.qty, req.callback, req.activation);
  let mut mgr = state.strategy_manager.write().await;
  mgr.add_strategy(Box::new(s)).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"success","strategy_name": format!("TRAIL-{}", req.symbol)})))
}

#[derive(Debug, Deserialize)]
struct ToggleReq { active: bool }

async fn toggle_strategy(Path(name): Path<String>, State(state): State<AppState>, axum::Json(body): axum::Json<ToggleReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let mut mgr = state.strategy_manager.write().await;
  mgr.set_strategy_active(&name, body.active).map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
  Ok(axum::Json(serde_json::json!({"status":"ok","name":name,"active":body.active})))
}

async fn delete_strategy(Path(name): Path<String>, State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let mut mgr = state.strategy_manager.write().await;
  mgr.remove_strategy(&name).map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
  Ok(axum::Json(serde_json::json!({"status":"ok","deleted":name})))
}

async fn get_strategy_info(Path(name): Path<String>, State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let mgr = state.strategy_manager.read().await;
  match mgr.get_strategy_info(&name) {
    Ok((n, desc, active)) => Ok(axum::Json(serde_json::json!({"name":n, "description": desc, "active":active}))),
    Err(_) => Err(axum::http::StatusCode::NOT_FOUND)
  }
}

// =============== Futures settings ===============
#[derive(Debug, Deserialize)]
struct SetPositionModeRequest { hedge: bool }

#[derive(Debug, Deserialize)]
struct SetMarginModeRequest { symbol: String, isolated: bool }

#[derive(Debug, Deserialize)]
struct SetLeverageRequest { symbol: String, leverage: u32 }

#[derive(Debug, Deserialize)]
struct FuturesSettingsRequest {
  position_mode: Option<SetPositionModeRequest>,
  margins: Option<Vec<SetMarginModeRequest>>, 
  leverages: Option<Vec<SetLeverageRequest>>, 
}

#[derive(Debug, Serialize)]
struct FuturesSettingsResponse { applied: serde_json::Value }

async fn set_position_mode(State(state): State<AppState>, axum::Json(req): axum::Json<SetPositionModeRequest>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let mut ex = state.exchange.write().await;
  ex.set_futures_position_mode(req.hedge).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"ok","hedge":req.hedge})))
}

async fn set_margin_mode(State(state): State<AppState>, axum::Json(req): axum::Json<SetMarginModeRequest>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let mut ex = state.exchange.write().await;
  ex.set_futures_margin_mode(&req.symbol, req.isolated).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"ok","symbol":req.symbol,"isolated":req.isolated})))
}

async fn set_leverage(State(state): State<AppState>, axum::Json(req): axum::Json<SetLeverageRequest>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let mut ex = state.exchange.write().await;
  ex.set_futures_leverage(&req.symbol, req.leverage).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"ok","symbol":req.symbol,"leverage":req.leverage})))
}

async fn apply_futures_settings(State(state): State<AppState>, axum::Json(req): axum::Json<FuturesSettingsRequest>) -> Result<axum::Json<FuturesSettingsResponse>, axum::http::StatusCode> {
  let mut applied = serde_json::json!({"position_mode": null, "margins": [], "leverages": []});
  {
    if let Some(pm) = &req.position_mode {
      let mut ex = state.exchange.write().await;
      ex.set_futures_position_mode(pm.hedge).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
      applied["position_mode"] = serde_json::json!({"hedge": pm.hedge});
    }
  }
  if let Some(items) = &req.margins {
    for m in items {
      let mut ex = state.exchange.write().await;
      ex.set_futures_margin_mode(&m.symbol, m.isolated).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
      applied["margins"].as_array_mut().unwrap().push(serde_json::json!({"symbol": m.symbol, "isolated": m.isolated}));
    }
  }
  if let Some(items) = &req.leverages {
    for l in items {
      let mut ex = state.exchange.write().await;
      ex.set_futures_leverage(&l.symbol, l.leverage).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
      applied["leverages"].as_array_mut().unwrap().push(serde_json::json!({"symbol": l.symbol, "leverage": l.leverage}));
    }
  }
  Ok(axum::Json(FuturesSettingsResponse { applied }))
}

async fn get_market_snapshot(Path(symbol): Path<String>, State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let ex = state.exchange.read().await;
  match ex.get_market_data(&symbol).await {
    Ok(md) => Ok(axum::Json(serde_json::to_value(md).unwrap_or(serde_json::json!({"symbol":symbol})))),
    Err(_) => Err(axum::http::StatusCode::BAD_REQUEST)
  }
}

async fn ws_prices(ws: WebSocketUpgrade, Path(symbol): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
  ws.on_upgrade(move |socket| price_stream(socket, symbol, state))
}

async fn price_stream(mut socket: WebSocket, symbol: String, state: AppState) {
  loop {
    // simple 1s polling push
    let price = {
      let ex = state.exchange.read().await;
      ex.get_market_data(&symbol).await.ok().map(|md| md.close)
    };
    if let Some(p) = price { let _ = socket.send(Message::Text(format!("{{\"symbol\":\"{}\",\"price\":{}}}", symbol, p))).await; }
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
}

async fn ws_orders(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
  ws.on_upgrade(move |socket| orders_stream(socket, state))
}

async fn orders_stream(mut socket: WebSocket, state: AppState) {
  loop {
    let data = {
      let ex = state.exchange.read().await;
      ex.get_open_orders().await.ok()
    };
    if let Some(list) = data {
      if let Ok(text) = serde_json::to_string(&list) {
        let _ = socket.send(Message::Text(text)).await;
      }
    }
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
  }
}

async fn ws_positions(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
  ws.on_upgrade(move |socket| positions_stream(socket, state))
}

async fn positions_stream(mut socket: WebSocket, state: AppState) {
  loop {
    let data = {
      let ex = state.exchange.read().await;
      ex.get_positions().await.ok()
    };
    if let Some(list) = data {
      if let Ok(text) = serde_json::to_string(&list) {
        let _ = socket.send(Message::Text(text)).await;
      }
    }
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
  }
}

async fn ws_strategies(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
  ws.on_upgrade(move |socket| strategies_stream(socket, state))
}

async fn strategies_stream(mut socket: WebSocket, state: AppState) {
  loop {
    let snapshot = {
      let mgr = state.strategy_manager.read().await;
      mgr.list_strategies()
    };
    if let Ok(text) = serde_json::to_string(&snapshot) {
      let _ = socket.send(Message::Text(text)).await;
    }
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
  }
}

#[derive(Debug, Deserialize)]
struct CreateOrderReq {
  symbol: String,
  side: String,
  order_type: String,
  quantity: f64,
  price: Option<f64>,
  reduce_only: Option<bool>,
  position_side: Option<String>,
}

async fn create_order(State(state): State<AppState>, axum::Json(req): axum::Json<CreateOrderReq>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let side = match req.side.to_lowercase().as_str() { "buy" => OrderSide::Buy, "sell" => OrderSide::Sell, _ => return Err(axum::http::StatusCode::BAD_REQUEST) };
  let mut order = Order::new(req.symbol, side, match req.order_type.to_lowercase().as_str() {
    "market" => OrderType::Market,
    "limit" => OrderType::Limit,
    "stop" | "stoploss" => OrderType::StopLoss,
    _ => OrderType::Market,
  }, req.quantity, req.price.unwrap_or(0.0));
  if let Some(ro) = req.reduce_only { order = order.with_reduce_only(ro); }
  if let Some(ps) = req.position_side { order = order.with_position_side(ps); }

  // Minimal submit path via Exchange directly
  let oid = {
    let mut ex = state.exchange.write().await;
    ex.submit_order(order).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?
  };
  Ok(axum::Json(serde_json::json!({"status":"ok","order_id": oid.0})))
}

async fn cancel_order(Path(id): Path<String>, State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let order_id = OrderId(id);
  let mut ex = state.exchange.write().await;
  ex.cancel_order(&order_id).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
  Ok(axum::Json(serde_json::json!({"status":"ok","cancelled":true})))
}

async fn get_order_status(Path(id): Path<String>, State(state): State<AppState>) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
  let order_id = OrderId(id);
  let ex = state.exchange.read().await;
  match ex.get_order_status(&order_id).await {
    Ok(status) => Ok(axum::Json(serde_json::json!({"status": format!("{:?}", status)}))),
    Err(_) => Err(axum::http::StatusCode::BAD_REQUEST)
  }
}

async fn get_positions(State(state): State<AppState>) -> Result<axum::Json<Vec<crate::models::position::Position>>, axum::http::StatusCode> {
  // Prefer exchange-native positions if available
  let ex = state.exchange.read().await;
  match ex.get_positions().await {
    Ok(v) => Ok(axum::Json(v)),
    Err(_) => Err(axum::http::StatusCode::BAD_REQUEST)
  }
}
