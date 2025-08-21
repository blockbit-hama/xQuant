use axum::{routing::{get, post}, Router, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};

use crate::core::strategy_manager::StrategyManager;
use crate::exchange::traits::Exchange;
use crate::strategies::Strategy;

#[derive(Clone)]
pub struct AppState {
  pub exchange: Arc<RwLock<dyn Exchange>>, 
  pub strategy_manager: Arc<RwLock<StrategyManager>>, 
}

#[derive(Debug, Serialize)]
struct Health { status: &'static str }

pub fn build_router(state: AppState) -> Router {
  let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any).allow_methods(Any);

  Router::new()
    .route("/health", get(|| async { axum::Json(Health { status: "ok" }) }))
    .route("/strategies", get(list_strategies))
    .route("/strategies/ta", post(create_ta_strategy))
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
