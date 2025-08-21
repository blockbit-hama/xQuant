// TA 관련 핸들러들

use std::collections::HashMap;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::reply::{json, with_status, Reply};
use tokio::sync::RwLock;
use crate::exchange::traits::Exchange;
use crate::indicators::Indicator;
use crate::trading_bots::bot_config::TradingBotConfig;
use crate::strategies::Strategy;
use crate::strategies::technical::TechnicalStrategy;
use crate::strategies::combined::CombinedStrategy;
use crate::core::strategy_manager::StrategyManager;
use serde::{Deserialize, Serialize};
use crate::order_core::manager::OrderManager;
use crate::models::order::{Order, OrderId, OrderSide, OrderType};

/// 전략 목록 조회 핸들러
pub async fn list_strategies(
  strategy_manager: Arc<RwLock<StrategyManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let manager = strategy_manager.read().await;
  let strategies = manager.list_strategies();
  
  Ok(with_status(json(&strategies), StatusCode::OK))
}

/// TA 전략 생성 요청 모델
#[derive(Debug, Deserialize)]
pub struct CreateTAStrategyRequest {
  pub symbol: String,
  pub strategy_type: String,
  pub params: serde_json::Value,
}

/// TA 전략 생성 핸들러
pub async fn create_ta_strategy(
  req: CreateTAStrategyRequest,
  strategy_manager: Arc<RwLock<StrategyManager>>,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  // 전략 타입에 따라 전략 인스턴스 생성
  let strategy_result = match req.strategy_type.as_str() {
    "ma_crossover" => {
      let fast_period = req.params["fast_period"].as_u64().unwrap_or(12) as usize;
      let slow_period = req.params["slow_period"].as_u64().unwrap_or(26) as usize;
      
      TechnicalStrategy::ma_crossover(req.symbol, fast_period, slow_period)
    },
    "rsi" => {
      let period = req.params["period"].as_u64().unwrap_or(14) as usize;
      let oversold = req.params["oversold"].as_f64().unwrap_or(30.0);
      let overbought = req.params["overbought"].as_f64().unwrap_or(70.0);
      
      TechnicalStrategy::rsi(req.symbol, period, oversold, overbought)
    },
    "macd" => {
      let fast_period = req.params["fast_period"].as_u64().unwrap_or(12) as usize;
      let slow_period = req.params["slow_period"].as_u64().unwrap_or(26) as usize;
      let signal_period = req.params["signal_period"].as_u64().unwrap_or(9) as usize;
      
      TechnicalStrategy::macd(req.symbol, fast_period, slow_period, signal_period)
    },
    "multi_indicator" => {
      TechnicalStrategy::multi_indicator(req.symbol)
    },
    "rsi_twap" => {
      let period = req.params["period"].as_u64().unwrap_or(14) as usize;
      let oversold = req.params["oversold"].as_f64().unwrap_or(30.0);
      let overbought = req.params["overbought"].as_f64().unwrap_or(70.0);
      let duration_minutes = req.params["duration_minutes"].as_u64().unwrap_or(60);
      // Combined 전략은 즉시 추가/응답 반환(타입 정합성 위해 여기서 처리)
      return match CombinedStrategy::rsi_twap(req.symbol, period, oversold, overbought, duration_minutes) {
        Ok(strategy) => {
          let strategy_name = strategy.name().to_string();
          let mut manager = strategy_manager.write().await;
          match manager.add_strategy(Box::new(strategy)) {
            Ok(_) => {
              let response = serde_json::json!({
                "status": "success",
                "message": "Strategy created successfully",
                "strategy_name": strategy_name
              });
              Ok(with_status(json(&response), StatusCode::CREATED))
            }
            Err(e) => {
              let error_response = serde_json::json!({"error": format!("Failed to add strategy: {}", e)});
              Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
            }
          }
        }
        Err(e) => {
          let error_response = serde_json::json!({"error": format!("Failed to create strategy: {}", e)});
          Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
        }
      };
    },
    "macd_vwap" => {
      let fast_period = req.params["fast_period"].as_u64().unwrap_or(12) as usize;
      let slow_period = req.params["slow_period"].as_u64().unwrap_or(26) as usize;
      let signal_period = req.params["signal_period"].as_u64().unwrap_or(9) as usize;
      let participation_rate = req.params["participation_rate"].as_f64().unwrap_or(0.1);
      // Combined 전략은 즉시 추가/응답 반환
      return match CombinedStrategy::macd_vwap(req.symbol, fast_period, slow_period, signal_period, participation_rate) {
        Ok(strategy) => {
          let strategy_name = strategy.name().to_string();
          let mut manager = strategy_manager.write().await;
          match manager.add_strategy(Box::new(strategy)) {
            Ok(_) => {
              let response = serde_json::json!({
                "status": "success",
                "message": "Strategy created successfully",
                "strategy_name": strategy_name
              });
              Ok(with_status(json(&response), StatusCode::CREATED))
            }
            Err(e) => {
              let error_response = serde_json::json!({"error": format!("Failed to add strategy: {}", e)});
              Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
            }
          }
        }
        Err(e) => {
          let error_response = serde_json::json!({"error": format!("Failed to create strategy: {}", e)});
          Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
        }
      };
    },
    _ => Err(crate::error::TradingError::InvalidStrategy(format!("Unknown strategy type: {}", req.strategy_type))),
  };
  
  // 전략 생성 결과 처리
  match strategy_result {
    Ok(strategy) => {
      // 전략 매니저에 추가
      let strategy_name = strategy.name().to_string();
      let mut manager = strategy_manager.write().await;
      match manager.add_strategy(Box::new(strategy)) {
        Ok(_) => {
          let response = serde_json::json!({
                        "status": "success",
                        "message": "Strategy created successfully",
                        "strategy_name": strategy_name
                    });
          
          Ok(with_status(json(&response), StatusCode::CREATED))
        },
        Err(e) => {
          let error_response = serde_json::json!({
                        "error": format!("Failed to add strategy: {}", e),
                    });
          
          Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
        }
      }
    },
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to create strategy: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// 전략 상태 조회 핸들러
pub async fn get_strategy_status(
  strategy_name: String,
  strategy_manager: Arc<RwLock<StrategyManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let manager = strategy_manager.read().await;
  match manager.get_strategy_status(&strategy_name) {
    Ok(status) => {
      Ok(with_status(json(&status), StatusCode::OK))
    },
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to get strategy status: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

/// 전략 삭제 핸들러
pub async fn delete_strategy(
  strategy_name: String,
  strategy_manager: Arc<RwLock<StrategyManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let mut manager = strategy_manager.write().await;
  match manager.remove_strategy(&strategy_name) {
    Ok(_) => {
      let response = serde_json::json!({
                "status": "success",
                "message": "Strategy deleted successfully",
                "strategy_name": strategy_name
            });
      
      Ok(with_status(json(&response), StatusCode::OK))
    },
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to delete strategy: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

/// 전략 활성화/비활성화 요청 모델
#[derive(Debug, Deserialize)]
pub struct ToggleStrategyRequest {
  pub active: bool,
}

/// 전략 활성화/비활성화 핸들러
pub async fn toggle_strategy(
  strategy_name: String,
  toggle_req: ToggleStrategyRequest,
  strategy_manager: Arc<RwLock<StrategyManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let mut manager = strategy_manager.write().await;
  match manager.set_strategy_active(&strategy_name, toggle_req.active) {
    Ok(_) => {
      let response = serde_json::json!({
                "status": "success",
                "message": if toggle_req.active { "Strategy activated" } else { "Strategy deactivated" },
                "strategy_name": strategy_name
            });
      
      Ok(with_status(json(&response), StatusCode::OK))
    },
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to toggle strategy: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

/// 인디케이터 쿼리 매개변수
#[derive(Debug, Deserialize)]
pub struct IndicatorQuery {
  pub indicator_type: String,
  pub period: Option<usize>,
  pub fast_period: Option<usize>,
  pub slow_period: Option<usize>,
  pub signal_period: Option<usize>,
  pub overbought: Option<f64>,
  pub oversold: Option<f64>,
  pub limit: Option<usize>,
}

/// 인디케이터 응답 모델
#[derive(Debug, Serialize)]
pub struct IndicatorResponse {
  pub symbol: String,
  pub indicator_type: String,
  pub values: Vec<IndicatorValue>,
  pub signals: Vec<SignalInfo>,
}

#[derive(Debug, Serialize)]
pub struct IndicatorValue {
  pub timestamp: i64,
  pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct SignalInfo {
  pub timestamp: i64,
  pub name: String,
  pub strength: f64,
  pub message: String,
}

/// 인디케이터 계산 핸들러
pub async fn calculate_indicators(
  symbol: String,
  query: IndicatorQuery,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  // 기본값 설정
  let limit = query.limit.unwrap_or(100);
  
  // 과거 시장 데이터 가져오기
  let exchange_instance = exchange.read().await;
  let interval = "1m"; // 기본 간격
  let end_time = chrono::Utc::now().timestamp_millis();
  let start_time = end_time - (limit as i64 * 60 * 1000); // limit분 전
  
  let historical_data = match exchange_instance.get_historical_data(
    &symbol,
    interval,
    start_time,
    Some(end_time),
    Some(limit),
  ).await {
    Ok(data) => data,
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to fetch historical data: {}", e),
            });
      return Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST));
    }
  };
  
  // 인디케이터 인스턴스 생성
  let mut indicator: Box<dyn Indicator> = match query.indicator_type.as_str() {
    "sma" => {
      let period = query.period.unwrap_or(14);
      Box::new(crate::indicators::moving_averages::SimpleMovingAverage::new(period))
    },
    "ema" => {
      let period = query.period.unwrap_or(14);
      Box::new(crate::indicators::moving_averages::ExponentialMovingAverage::new(period))
    },
    "ma_crossover" => {
      let fast_period = query.fast_period.unwrap_or(12);
      let slow_period = query.slow_period.unwrap_or(26);
      Box::new(crate::indicators::moving_averages::MovingAverageCrossover::with_ema(
        fast_period, slow_period
      ))
    },
    "rsi" => {
      let period = query.period.unwrap_or(14);
      let overbought = query.overbought.unwrap_or(70.0);
      let oversold = query.oversold.unwrap_or(30.0);
      Box::new(crate::indicators::oscillators::RelativeStrengthIndex::new(
        period, Some(overbought), Some(oversold)
      ))
    },
    "macd" => {
      let fast_period = query.fast_period.unwrap_or(12);
      let slow_period = query.slow_period.unwrap_or(26);
      let signal_period = query.signal_period.unwrap_or(9);
      Box::new(crate::indicators::trend::MACD::new(
        fast_period, slow_period, signal_period
      ))
    },
    "vwap" => {
      let period = query.period.unwrap_or(14);
      Box::new(crate::indicators::volume::VolumeWeightedAveragePrice::new(period))
    },
    _ => {
      let error_response = serde_json::json!({
                "error": format!("Unknown indicator type: {}", query.indicator_type),
            });
      return Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST));
    }
  };
  
  // 인디케이터 계산
  let mut values = Vec::new();
  let mut all_signals = Vec::new();
  
  for candle in &historical_data {
    // 인디케이터 업데이트
    if let Err(e) = indicator.update(candle.close, Some(candle.volume)) {
      continue; // 업데이트 실패 시 스킵
    }
    
    // 인디케이터 값 계산
    if indicator.is_ready() {
      if let Ok(result) = indicator.calculate() {
        values.push(IndicatorValue {
          timestamp: candle.timestamp,
          value: result.value,
        });
        
        // 신호 처리
        for signal in &result.signals {
          all_signals.push(SignalInfo {
            timestamp: candle.timestamp,
            name: signal.name.clone(),
            strength: signal.strength,
            message: signal.message.clone(),
          });
        }
      }
    }
  }
  
  // 응답 구성
  let response = IndicatorResponse {
    symbol: symbol.clone(),
    indicator_type: query.indicator_type.clone(),
    values,
    signals: all_signals,
  };
  
  Ok(with_status(json(&response), StatusCode::OK))
}

/// 백테스트 성과 지표 요청 모델
#[derive(Debug, Deserialize)]
pub struct BacktestPerformanceRequest {
  pub strategy_type: String,
  pub symbol: String,
  pub start_time: i64,
  pub end_time: i64,
  pub params: serde_json::Value,
}

/// 백테스트 성과 지표 계산 핸들러
pub async fn calculate_backtest_performance(
  req: BacktestPerformanceRequest,
) -> Result<impl Reply, warp::Rejection> {
  use crate::backtest::scenario::BacktestScenarioBuilder;
  use chrono::{DateTime, Utc};
  
  // 시간 변환
  let start_time = DateTime::<Utc>::from_timestamp_millis(req.start_time)
    .ok_or_else(|| warp::reject())?;
  
  let end_time = DateTime::<Utc>::from_timestamp_millis(req.end_time)
    .ok_or_else(|| warp::reject())?;
  
  // 데이터 파일 경로 생성
  let data_file = format!("./data/{}-1m.csv", req.symbol);
  
  // 전략 생성
  let strategy_result = match req.strategy_type.as_str() {
    "ma_crossover" => {
      let fast_period = req.params["fast_period"].as_u64().unwrap_or(12) as usize;
      let slow_period = req.params["slow_period"].as_u64().unwrap_or(26) as usize;
      
      TechnicalStrategy::ma_crossover(req.symbol.clone(), fast_period, slow_period)
    },
    "rsi" => {
      let period = req.params["period"].as_u64().unwrap_or(14) as usize;
      let oversold = req.params["oversold"].as_f64().unwrap_or(30.0);
      let overbought = req.params["overbought"].as_f64().unwrap_or(70.0);
      
      TechnicalStrategy::rsi(req.symbol.clone(), period, oversold, overbought)
    },
    "macd" => {
      let fast_period = req.params["fast_period"].as_u64().unwrap_or(12) as usize;
      let slow_period = req.params["slow_period"].as_u64().unwrap_or(26) as usize;
      let signal_period = req.params["signal_period"].as_u64().unwrap_or(9) as usize;
      
      TechnicalStrategy::macd(req.symbol.clone(), fast_period, slow_period, signal_period)
    },
    _ => {
      let error_response = serde_json::json!({
                "error": format!("Unknown strategy type: {}", req.strategy_type),
            });
      return Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST));
    }
  };
  
  match strategy_result {
    Ok(strategy) => {
      // 백테스트 시나리오 생성
      let mut builder = BacktestScenarioBuilder::new(format!("{} 백테스트", req.strategy_type))
        .description(format!("{} 기반 백테스트", req.strategy_type))
        .data_file(data_file)
        .period(start_time, end_time)
        .initial_balance("USDT", 10000.0)
        .fee_rate(0.001)
        .slippage(0.0005)
        .strategy(Box::new(strategy));
      
      // 백테스트 실행
      match builder.build() {
        Ok(scenario) => {
          match scenario.run().await {
            Ok(result) => {
              // 성능 지표 계산 및 응답
              let response = serde_json::json!({
                                "strategy_type": req.strategy_type,
                                "symbol": req.symbol,
                                "start_time": start_time.to_rfc3339(),
                                "end_time": end_time.to_rfc3339(),
                                "profit": result.profit,
                                "profit_percentage": result.profit_percentage,
                                "trade_count": result.trade_count(),
                                "win_rate": result.win_rate(),
                                "avg_profit_per_trade": result.average_profit_per_trade(),
                                "sharpe_ratio": result.sharpe_ratio(),
                                "max_drawdown": result.max_drawdown(),
                                "profit_factor": result.profit_factor(),
                                "car": result.car(),
                            });
              
              Ok(with_status(json(&response), StatusCode::OK))
            },
            Err(e) => {
              let error_response = serde_json::json!({
                                "error": format!("Failed to run backtest: {}", e),
                            });
              
              Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
            }
          }
        },
        Err(e) => {
          let error_response = serde_json::json!({
                        "error": format!("Failed to build backtest scenario: {}", e),
                    });
          
          Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
        }
      }
    },
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to create strategy: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// 포트폴리오 최적화 요청 모델
#[derive(Debug, Deserialize)]
pub struct PortfolioOptimizationRequest {
  pub symbols: Vec<String>,
  pub start_time: i64,
  pub end_time: i64,
  pub strategy_type: String,
  pub allocation_type: String,  // "equal_weight", "max_sharpe", "min_variance"
}

/// 포트폴리오 최적화 핸들러
pub async fn optimize_portfolio(
  req: PortfolioOptimizationRequest,
) -> Result<impl Reply, warp::Rejection> {
  // 이 부분은 실제 포트폴리오 최적화 로직을 구현해야 함
  // 현재는 간단한 예시로 대체
  
  let response = serde_json::json!({
        "status": "success",
        "message": "Portfolio optimization API is under development",
        "optimal_allocation": {
            "method": req.allocation_type,
            "allocations": req.symbols.iter().map(|s| (s.clone(), 1.0 / req.symbols.len() as f64)).collect::<HashMap<String, f64>>()
        }
    });
  
  Ok(with_status(json(&response), StatusCode::OK))
}

/// 거래 로그 분석 핸들러
pub async fn analyze_trade_logs(
  exchange: Arc<RwLock<dyn Exchange>>,
  strategy_manager: Arc<RwLock<StrategyManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // 이 부분은 실제 거래 로그 분석 로직을 구현해야 함
  // 현재는 간단한 예시로 대체
  
  let response = serde_json::json!({
        "status": "success",
        "message": "Trade log analysis API is under development",
        "summary": {
            "total_trades": 0,
            "winning_trades": 0,
            "losing_trades": 0,
            "win_rate": 0.0,
            "profit_loss": 0.0
        }
    });
  
  Ok(with_status(json(&response), StatusCode::OK))
}

// ====== 기본 주문/실행 관련 핸들러 스텁 ======

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
  pub symbol: String,
  pub side: OrderSide,
  pub order_type: OrderType,
  pub quantity: f64,
  pub price: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
  pub order_id: String,
}

pub async fn create_order(
  req: CreateOrderRequest,
  _exchange: Arc<RwLock<dyn Exchange>>, // reserved for validation
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // Build order
  let price = req.price.unwrap_or(0.0);
  let order = Order::new(req.symbol, req.side, req.order_type, req.quantity, price);

  let id_res = {
    let manager = order_manager.read().await;
    manager.create_order(order).await
  };

  match id_res {
    Ok(order_id) => Ok(with_status(json(&CreateOrderResponse { order_id: order_id.to_string() }), StatusCode::CREATED)),
    Err(e) => Ok(with_status(json(&serde_json::json!({"error": format!("failed to create order: {}", e)})), StatusCode::BAD_REQUEST)),
  }
}

pub async fn get_orders(
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let res = {
    let manager = order_manager.read().await;
    manager.get_open_orders().await
  };

  match res {
    Ok(list) => Ok(with_status(json(&list), StatusCode::OK)),
    Err(e) => Ok(with_status(json(&serde_json::json!({"error": format!("failed to fetch orders: {}", e)})), StatusCode::BAD_REQUEST)),
  }
}

pub async fn cancel_order(
  order_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let oid = OrderId(order_id);
  let res = {
    let manager = order_manager.read().await;
    manager.cancel_order(&oid).await
  };

  match res {
    Ok(_) => Ok(with_status(json(&serde_json::json!({"status":"cancelled"})), StatusCode::OK)),
    Err(e) => Ok(with_status(json(&serde_json::json!({"error": format!("failed to cancel order: {}", e)})), StatusCode::BAD_REQUEST)),
  }
}

// VWAP endpoints (stubs)
#[derive(Debug, Deserialize)]
pub struct CreateVwapRequest { pub symbol: String, pub side: OrderSide, pub target_quantity: f64, pub execution_interval_ms: i64, pub vwap_window: Option<usize> }

pub async fn create_vwap_order(
  _req: CreateVwapRequest,
  _exchange: Arc<RwLock<dyn Exchange>>,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"VWAP not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

pub async fn get_vwap_status(
  _id: String,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"VWAP status not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

pub async fn cancel_vwap_order(
  _id: String,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"VWAP cancel not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

// Iceberg endpoints (stubs)
#[derive(Debug, Deserialize)]
pub struct CreateIcebergRequest { pub symbol: String, pub side: OrderSide, pub total_quantity: f64, pub display_size: f64 }

pub async fn create_iceberg_order(
  _req: CreateIcebergRequest,
  _exchange: Arc<RwLock<dyn Exchange>>,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"Iceberg not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

pub async fn get_iceberg_status(
  _id: String,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"Iceberg status not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

pub async fn cancel_iceberg_order(
  _id: String,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"Iceberg cancel not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

// Trailing stop endpoints (stubs)
#[derive(Debug, Deserialize)]
pub struct CreateTrailingStopRequest { pub symbol: String, pub side: OrderSide, pub quantity: f64, pub trailing_delta: f64 }

pub async fn create_trailing_stop(
  _req: CreateTrailingStopRequest,
  _exchange: Arc<RwLock<dyn Exchange>>,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"Trailing stop not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

pub async fn get_trailing_stop_status(
  _id: String,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"Trailing stop status not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

pub async fn cancel_trailing_stop(
  _id: String,
  _order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  Ok(with_status(json(&serde_json::json!({"error":"Trailing stop cancel not implemented"})), StatusCode::NOT_IMPLEMENTED))
}

// Market data endpoint
#[derive(Debug, Serialize)]
pub struct MarketDataResponse { pub symbol: String, pub data: crate::models::market_data::MarketData }

pub async fn get_market_data(
  symbol: String,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  let res = {
    let ex = exchange.read().await;
    ex.get_market_data(&symbol).await
  };

  match res {
    Ok(md) => Ok(with_status(json(&MarketDataResponse { symbol, data: md }), StatusCode::OK)),
    Err(e) => Ok(with_status(json(&serde_json::json!({"error": format!("failed to get market data: {}", e)})), StatusCode::BAD_REQUEST)),
  }
}