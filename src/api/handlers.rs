

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use warp::http::StatusCode;
use warp::reply::{json, with_status, Reply};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::vwap_splitter::VwapSplitter;
use crate::core::iceberg_manager::IcebergManager;
use crate::core::trailing_stop_manager::TrailingStopManager;
use crate::core::twap_splitter::TwapSplitter;
use crate::exchange::traits::Exchange;
use crate::models::order::{Order, OrderId, OrderSide, OrderType, OrderStatus};
use crate::order_core::manager::OrderManager;
use crate::utils::logging;

// 전역 전략 매니저 저장소 (실제 구현에서는 데이터베이스를 사용할 수 있음)
lazy_static::lazy_static! {
    static ref VWAP_MANAGERS: RwLock<HashMap<String, Arc<RwLock<VwapSplitter>>>> = RwLock::new(HashMap::new());
    static ref ICEBERG_MANAGERS: RwLock<HashMap<String, Arc<RwLock<IcebergManager>>>> = RwLock::new(HashMap::new());
    static ref TRAILING_MANAGERS: RwLock<HashMap<String, Arc<RwLock<TrailingStopManager>>>> = RwLock::new(HashMap::new());
    static ref TWAP_MANAGERS: RwLock<HashMap<String, Arc<RwLock<TwapSplitter>>>> = RwLock::new(HashMap::new());
}

//
// 헬스 체크 핸들러
//

/// 시스템 헬스 체크 핸들러
pub async fn health_handler() -> Result<impl Reply, warp::Rejection> {
  Ok(json(&serde_json::json!({
        "status": "up",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

//
// 일반 주문 관련 핸들러
//

/// 주문 생성 요청 모델
#[derive(Debug, Deserialize)]
pub struct OrderRequest {
  pub symbol: String,
  pub side: OrderSide,
  pub order_type: OrderType,
  pub quantity: f64,
  pub price: Option<f64>,
  pub stop_price: Option<f64>,
  pub time_in_force: Option<String>,
  pub client_order_id: Option<String>,
}

/// 주문 응답 모델
#[derive(Debug, Serialize)]
pub struct OrderResponse {
  pub order_id: String,
  pub status: String,
}

/// 주문 생성 핸들러
pub async fn create_order(
  order_req: OrderRequest,
  exchange: Arc<RwLock<dyn Exchange>>,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // 가격 기본값 처리
  let price = order_req.price.unwrap_or(0.0);
  
  // 주문 객체 생성
  let mut order = Order::new(
    order_req.symbol.clone(),
    order_req.side.clone(),
    order_req.order_type.clone(),
    order_req.quantity,
    price,
  );
  
  // 추가 파라미터 설정
  if let Some(stop_price) = order_req.stop_price {
    order = order.with_stop_price(stop_price);
  }
  
  if let Some(tif) = &order_req.time_in_force {
    order = order.with_time_in_force(tif.clone());
  }
  
  if let Some(client_id) = &order_req.client_order_id {
    order = order.with_client_order_id(client_id.clone());
  }
  
  // 주문 관리자를 통해 주문 생성
  let manager = order_manager.read().await;
  match manager.create_order(order).await {
    Ok(order_id) => {
      // 로그 기록
      logging::log_order_created(
        &order_id.0,
        &order_req.symbol,
        &format!("{:?}", order_req.side),
        order_req.quantity,
        price
      );
      
      let response = OrderResponse {
        order_id: order_id.0,
        status: "success".to_string(),
      };
      
      Ok(with_status(json(&response), StatusCode::CREATED))
    },
    Err(e) => {
      // 에러 로그 기록
      logging::log_error("주문 생성 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to create order: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// 모든 주문 조회 핸들러
pub async fn get_orders(
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let manager = order_manager.read().await;
  match manager.get_open_orders().await {
    Ok(orders) => {
      Ok(with_status(json(&orders), StatusCode::OK))
    },
    Err(e) => {
      logging::log_error("주문 조회 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to fetch orders: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
    }
  }
}

/// 특정 주문 조회 핸들러
pub async fn get_order(
  order_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let order_id = OrderId(order_id);
  let manager = order_manager.read().await;
  
  match manager.get_order_status(&order_id).await {
    Ok(status) => {
      let response = serde_json::json!({
                "order_id": order_id.0,
                "status": format!("{:?}", status)
            });
      
      Ok(with_status(json(&response), StatusCode::OK))
    },
    Err(e) => {
      let error_response = serde_json::json!({
                "error": format!("Failed to get order status: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

/// 주문 취소 핸들러
pub async fn cancel_order(
  order_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let order_id = OrderId(order_id);
  
  let manager = order_manager.read().await;
  match manager.cancel_order(&order_id).await {
    Ok(_) => {
      // 로그 기록
      logging::log_order_cancelled(&order_id.0);
      
      let response = serde_json::json!({
                "order_id": order_id.0,
                "status": "cancelled",
            });
      
      Ok(with_status(json(&response), StatusCode::OK))
    },
    Err(e) => {
      logging::log_error("주문 취소 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to cancel order: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

//
// VWAP 관련 핸들러
//

/// VWAP 주문 요청 모델
#[derive(Debug, Deserialize)]
pub struct VwapRequest {
  pub symbol: String,
  pub side: OrderSide,
  pub quantity: f64,
  pub execution_interval: i64,  // 밀리초
  pub target_percentage: Option<f64>,
}

/// VWAP 응답 모델
#[derive(Debug, Serialize)]
pub struct VwapResponse {
  pub id: String,
  pub status: String,
}

/// VWAP 주문 생성 핸들러
pub async fn create_vwap_order(
  vwap_req: VwapRequest,
  exchange: Arc<RwLock<dyn Exchange>>,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // 고유 ID 생성
  let vwap_id = Uuid::new_v4().to_string();
  
  // VWAP 분할기 생성
  let vwap_splitter = VwapSplitter::new(
    exchange.clone(),
    vwap_req.symbol.clone(),
    vwap_req.side,
    vwap_req.quantity,
    vwap_req.execution_interval,
    vwap_req.target_percentage,
  );
  
  // 전역 저장소에 저장
  let vwap_arc = Arc::new(RwLock::new(vwap_splitter));
  let mut managers = VWAP_MANAGERS.write().await;
  managers.insert(vwap_id.clone(), vwap_arc.clone());
  
  // 실행 시작
  let mut vwap = vwap_arc.write().await;
  match vwap.start().await {
    Ok(_) => {
      let response = VwapResponse {
        id: vwap_id,
        status: "started".to_string(),
      };
      
      log::info!("VWAP 주문 시작: {} - 심볼: {} - 수량: {}",
                      vwap_id, vwap_req.symbol, vwap_req.quantity);
      
      Ok(with_status(json(&response), StatusCode::CREATED))
    },
    Err(e) => {
      // 오류 시 저장소에서 제거
      managers.remove(&vwap_id);
      
      logging::log_error("VWAP 주문 시작 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to start VWAP execution: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// VWAP 상태 조회 핸들러
pub async fn get_vwap_status(
  vwap_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = VWAP_MANAGERS.read().await;
  
  if let Some(vwap_arc) = managers.get(&vwap_id) {
    let vwap = vwap_arc.read().await;
    let (is_active, executed_qty, total_qty) = vwap.status();
    
    let progress_percentage = if total_qty > 0.0 {
      (executed_qty / total_qty) * 100.0
    } else {
      0.0
    };
    
    let response = serde_json::json!({
            "id": vwap_id,
            "is_active": is_active,
            "executed_quantity": executed_qty,
            "total_quantity": total_qty,
            "progress_percentage": progress_percentage,
        });
    
    Ok(with_status(json(&response), StatusCode::OK))
  } else {
    let error_response = serde_json::json!({
            "error": "VWAP execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

/// VWAP 주문 취소 핸들러
pub async fn cancel_vwap_order(
  vwap_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = VWAP_MANAGERS.read().await;
  
  if let Some(vwap_arc) = managers.get(&vwap_id) {
    let mut vwap = vwap_arc.write().await;
    match vwap.stop().await {
      Ok(_) => {
        log::info!("VWAP 주문 취소: {}", vwap_id);
        
        let response = serde_json::json!({
                    "id": vwap_id,
                    "status": "cancelled",
                });
        
        Ok(with_status(json(&response), StatusCode::OK))
      },
      Err(e) => {
        logging::log_error("VWAP 주문 취소 실패", &e);
        
        let error_response = serde_json::json!({
                    "error": format!("Failed to cancel VWAP execution: {}", e),
                });
        
        Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
      }
    }
  } else {
    let error_response = serde_json::json!({
            "error": "VWAP execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

//
// Iceberg 관련 핸들러
//

/// Iceberg 주문 요청 모델
#[derive(Debug, Deserialize)]
pub struct IcebergRequest {
  pub symbol: String,
  pub side: OrderSide,
  pub total_quantity: f64,
  pub limit_price: f64,
  pub display_quantity: f64,
}

/// Iceberg 응답 모델
#[derive(Debug, Serialize)]
pub struct IcebergResponse {
  pub id: String,
  pub status: String,
}

/// Iceberg 주문 생성 핸들러
pub async fn create_iceberg_order(
  iceberg_req: IcebergRequest,
  exchange: Arc<RwLock<dyn Exchange>>,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // 고유 ID 생성
  let iceberg_id = Uuid::new_v4().to_string();
  
  // Iceberg 관리자 생성
  let iceberg_manager = IcebergManager::new(
    exchange.clone(),
    iceberg_req.symbol.clone(),
    iceberg_req.side,
    iceberg_req.total_quantity,
    iceberg_req.limit_price,
    iceberg_req.display_quantity,
  );
  
  // 전역 저장소에 저장
  let iceberg_arc = Arc::new(RwLock::new(iceberg_manager));
  let mut managers = ICEBERG_MANAGERS.write().await;
  managers.insert(iceberg_id.clone(), iceberg_arc.clone());
  
  // 실행 시작
  let mut iceberg = iceberg_arc.write().await;
  match iceberg.start().await {
    Ok(_) => {
      let response = IcebergResponse {
        id: iceberg_id,
        status: "started".to_string(),
      };
      
      log::info!("Iceberg 주문 시작: {} - 심볼: {} - 총 수량: {} - 표시 수량: {}",
                      iceberg_id, iceberg_req.symbol, iceberg_req.total_quantity, iceberg_req.display_quantity);
      
      Ok(with_status(json(&response), StatusCode::CREATED))
    },
    Err(e) => {
      // 오류 시 저장소에서 제거
      managers.remove(&iceberg_id);
      
      logging::log_error("Iceberg 주문 시작 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to start Iceberg execution: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// Iceberg 상태 조회 핸들러
pub async fn get_iceberg_status(
  iceberg_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = ICEBERG_MANAGERS.read().await;
  
  if let Some(iceberg_arc) = managers.get(&iceberg_id) {
    let iceberg = iceberg_arc.read().await;
    let (is_active, executed_qty, total_qty) = iceberg.status();
    
    let progress_percentage = if total_qty > 0.0 {
      (executed_qty / total_qty) * 100.0
    } else {
      0.0
    };
    
    let response = serde_json::json!({
            "id": iceberg_id,
            "is_active": is_active,
            "executed_quantity": executed_qty,
            "total_quantity": total_qty,
            "progress_percentage": progress_percentage,
        });
    
    Ok(with_status(json(&response), StatusCode::OK))
  } else {
    let error_response = serde_json::json!({
            "error": "Iceberg execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

/// Iceberg 주문 취소 핸들러
pub async fn cancel_iceberg_order(
  iceberg_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = ICEBERG_MANAGERS.read().await;
  
  if let Some(iceberg_arc) = managers.get(&iceberg_id) {
    let mut iceberg = iceberg_arc.write().await;
    match iceberg.stop().await {
      Ok(_) => {
        log::info!("Iceberg 주문 취소: {}", iceberg_id);
        
        let response = serde_json::json!({
                    "id": iceberg_id,
                    "status": "cancelled",
                });
        
        Ok(with_status(json(&response), StatusCode::OK))
      },
      Err(e) => {
        logging::log_error("Iceberg 주문 취소 실패", &e);
        
        let error_response = serde_json::json!({
                    "error": format!("Failed to cancel Iceberg execution: {}", e),
                });
        
        Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
      }
    }
  } else {
    let error_response = serde_json::json!({
            "error": "Iceberg execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

/// Iceberg 가격 업데이트 요청 모델
#[derive(Debug, Deserialize)]
pub struct IcebergPriceUpdateRequest {
  pub new_price: f64,
}

/// Iceberg 가격 업데이트 핸들러
pub async fn update_iceberg_price(
  iceberg_id: String,
  update_req: IcebergPriceUpdateRequest,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = ICEBERG_MANAGERS.read().await;
  
  if let Some(iceberg_arc) = managers.get(&iceberg_id) {
    let mut iceberg = iceberg_arc.write().await;
    match iceberg.update_price(update_req.new_price).await {
      Ok(_) => {
        log::info!("Iceberg 가격 업데이트: {} - 새 가격: {}", iceberg_id, update_req.new_price);
        
        let response = serde_json::json!({
                    "id": iceberg_id,
                    "status": "price_updated",
                    "new_price": update_req.new_price,
                });
        
        Ok(with_status(json(&response), StatusCode::OK))
      },
      Err(e) => {
        logging::log_error("Iceberg 가격 업데이트 실패", &e);
        
        let error_response = serde_json::json!({
                    "error": format!("Failed to update Iceberg price: {}", e),
                });
        
        Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
      }
    }
  } else {
    let error_response = serde_json::json!({
            "error": "Iceberg execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

//
// Trailing Stop 관련 핸들러
//

/// Trailing Stop 요청 모델
#[derive(Debug, Deserialize)]
pub struct TrailingStopRequest {
  pub symbol: String,
  pub side: OrderSide,
  pub quantity: f64,
  pub trailing_delta: f64,
  pub activation_price: Option<f64>,
}

/// Trailing Stop 응답 모델
#[derive(Debug, Serialize)]
pub struct TrailingStopResponse {
  pub id: String,
  pub status: String,
}

/// Trailing Stop 생성 핸들러
pub async fn create_trailing_stop(
  trailing_req: TrailingStopRequest,
  exchange: Arc<RwLock<dyn Exchange>>,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // 고유 ID 생성
  let trailing_id = Uuid::new_v4().to_string();
  
  // Trailing Stop 관리자 생성
  let trailing_manager = TrailingStopManager::new(
    exchange.clone(),
    trailing_req.symbol.clone(),
    trailing_req.side,
    trailing_req.quantity,
    trailing_req.trailing_delta,
    trailing_req.activation_price,
  );
  
  // 전역 저장소에 저장
  let trailing_arc = Arc::new(RwLock::new(trailing_manager));
  let mut managers = TRAILING_MANAGERS.write().await;
  managers.insert(trailing_id.clone(), trailing_arc.clone());
  
  // 실행 시작
  let mut trailing = trailing_arc.write().await;
  match trailing.start().await {
    Ok(_) => {
      let response = TrailingStopResponse {
        id: trailing_id,
        status: "started".to_string(),
      };
      
      log::info!("Trailing Stop 시작: {} - 심볼: {} - 델타: {}%",
                      trailing_id, trailing_req.symbol, trailing_req.trailing_delta);
      
      Ok(with_status(json(&response), StatusCode::CREATED))
    },
    Err(e) => {
      // 오류 시 저장소에서 제거
      managers.remove(&trailing_id);
      
      logging::log_error("Trailing Stop 시작 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to start Trailing Stop: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// Trailing Stop 상태 조회 핸들러
pub async fn get_trailing_stop_status(
  trailing_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = TRAILING_MANAGERS.read().await;
  
  if let Some(trailing_arc) = managers.get(&trailing_id) {
    let trailing = trailing_arc.read().await;
    let (is_active, executed, trigger_price, quantity) = trailing.status();
    
    let response = serde_json::json!({
            "id": trailing_id,
            "is_active": is_active,
            "executed": executed,
            "trigger_price": trigger_price,
            "quantity": quantity,
        });
    
    Ok(with_status(json(&response), StatusCode::OK))
  } else {
    let error_response = serde_json::json!({
            "error": "Trailing Stop not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

/// Trailing Stop 취소 핸들러
pub async fn cancel_trailing_stop(
  trailing_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = TRAILING_MANAGERS.read().await;
  
  if let Some(trailing_arc) = managers.get(&trailing_id) {
    let mut trailing = trailing_arc.write().await;
    match trailing.stop().await {
      Ok(_) => {
        log::info!("Trailing Stop 취소: {}", trailing_id);
        
        let response = serde_json::json!({
                    "id": trailing_id,
                    "status": "cancelled",
                });
        
        Ok(with_status(json(&response), StatusCode::OK))
      },
      Err(e) => {
        logging::log_error("Trailing Stop 취소 실패", &e);
        
        let error_response = serde_json::json!({
                    "error": format!("Failed to cancel Trailing Stop: {}", e),
                });
        
        Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
      }
    }
  } else {
    let error_response = serde_json::json!({
            "error": "Trailing Stop not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

/// Trailing Delta 업데이트 요청 모델
#[derive(Debug, Deserialize)]
pub struct TrailingDeltaUpdateRequest {
  pub new_delta: f64,
}

/// Trailing Delta 업데이트 핸들러
pub async fn update_trailing_delta(
  trailing_id: String,
  update_req: TrailingDeltaUpdateRequest,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = TRAILING_MANAGERS.read().await;
  
  if let Some(trailing_arc) = managers.get(&trailing_id) {
    let mut trailing = trailing_arc.write().await;
    match trailing.update_delta(update_req.new_delta).await {
      Ok(_) => {
        log::info!("Trailing Delta 업데이트: {} - 새 델타: {}%", trailing_id, update_req.new_delta);
        
        let response = serde_json::json!({
                    "id": trailing_id,
                    "status": "delta_updated",
                    "new_delta": update_req.new_delta,
                });
        
        Ok(with_status(json(&response), StatusCode::OK))
      },
      Err(e) => {
        logging::log_error("Trailing Delta 업데이트 실패", &e);
        
        let error_response = serde_json::json!({
                    "error": format!("Failed to update Trailing Delta: {}", e),
                });
        
        Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
      }
    }
  } else {
    let error_response = serde_json::json!({
            "error": "Trailing Stop not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

//
// TWAP 관련 핸들러
//

/// TWAP 주문 요청 모델
#[derive(Debug, Deserialize)]
pub struct TwapRequest {
  pub symbol: String,
  pub side: OrderSide,
  pub total_quantity: f64,
  pub execution_interval: i64,  // 밀리초
  pub num_slices: usize,
}

/// TWAP 응답 모델
#[derive(Debug, Serialize)]
pub struct TwapResponse {
  pub id: String,
  pub status: String,
}

/// TWAP 주문 생성 핸들러
pub async fn create_twap_order(
  twap_req: TwapRequest,
  exchange: Arc<RwLock<dyn Exchange>>,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  // 고유 ID 생성
  let twap_id = Uuid::new_v4().to_string();
  
  // TWAP 분할기 생성
  let twap_splitter = TwapSplitter::new(
    exchange.clone(),
    twap_req.symbol.clone(),
    twap_req.side,
    twap_req.total_quantity,
    twap_req.execution_interval,
    twap_req.num_slices,
  );
  
  // 전역 저장소에 저장
  let twap_arc = Arc::new(RwLock::new(twap_splitter));
  let mut managers = TWAP_MANAGERS.write().await;
  managers.insert(twap_id.clone(), twap_arc.clone());
  
  // 실행 시작
  let mut twap = twap_arc.write().await;
  match twap.start().await {
    Ok(_) => {
      let response = TwapResponse {
        id: twap_id,
        status: "started".to_string(),
      };
      
      log::info!("TWAP 주문 시작: {} - 심볼: {} - 총 수량: {} - 분할 수: {}", 
                      twap_id, twap_req.symbol, twap_req.total_quantity, twap_req.num_slices);
      
      Ok(with_status(json(&response), StatusCode::CREATED))
    },
    Err(e) => {
      // 오류 시 저장소에서 제거
      managers.remove(&twap_id);
      
      logging::log_error("TWAP 주문 시작 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to start TWAP execution: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::BAD_REQUEST))
    }
  }
}

/// TWAP 상태 조회 핸들러
pub async fn get_twap_status(
  twap_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = TWAP_MANAGERS.read().await;
  
  if let Some(twap_arc) = managers.get(&twap_id) {
    let twap = twap_arc.read().await;
    let (is_active, executed_qty, total_qty) = twap.status();
    
    let progress_percentage = if total_qty > 0.0 {
      (executed_qty / total_qty) * 100.0
    } else {
      0.0
    };
    
    let response = serde_json::json!({
            "id": twap_id,
            "is_active": is_active,
            "executed_quantity": executed_qty,
            "total_quantity": total_qty,
            "progress_percentage": progress_percentage,
        });
    
    Ok(with_status(json(&response), StatusCode::OK))
  } else {
    let error_response = serde_json::json!({
            "error": "TWAP execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

/// TWAP 주문 취소 핸들러
pub async fn cancel_twap_order(
  twap_id: String,
  order_manager: Arc<RwLock<OrderManager>>,
) -> Result<impl Reply, warp::Rejection> {
  let managers = TWAP_MANAGERS.read().await;
  
  if let Some(twap_arc) = managers.get(&twap_id) {
    let mut twap = twap_arc.write().await;
    match twap.stop().await {
      Ok(_) => {
        log::info!("TWAP 주문 취소: {}", twap_id);
        
        let response = serde_json::json!({
                    "id": twap_id,
                    "status": "cancelled",
                });
        
        Ok(with_status(json(&response), StatusCode::OK))
      },
      Err(e) => {
        logging::log_error("TWAP 주문 취소 실패", &e);
        
        let error_response = serde_json::json!({
                    "error": format!("Failed to cancel TWAP execution: {}", e),
                });
        
        Ok(with_status(json(&error_response), StatusCode::INTERNAL_SERVER_ERROR))
      }
    }
  } else {
    let error_response = serde_json::json!({
            "error": "TWAP execution not found",
        });
    
    Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
  }
}

//
// 시장 데이터 관련 핸들러
//

/// 시장 데이터 조회 핸들러
pub async fn get_market_data(
  symbol: String,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  let exchange = exchange.read().await;
  match exchange.get_market_data(&symbol).await {
    Ok(market_data) => {
      Ok(with_status(json(&market_data), StatusCode::OK))
    },
    Err(e) => {
      logging::log_error("시장 데이터 조회 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to get market data: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

/// 과거 시장 데이터 요청 모델
#[derive(Debug, Deserialize)]
pub struct HistoricalDataRequest {
  pub start_time: i64,
  pub end_time: Option<i64>,
  pub interval: String,
  pub limit: Option<usize>,
}

/// 과거 시장 데이터 조회 핸들러
pub async fn get_historical_data(
  symbol: String,
  query: HistoricalDataRequest,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  let exchange = exchange.read().await;
  match exchange.get_historical_data(
    &symbol,
    &query.interval,
    query.start_time,
    query.end_time,
    query.limit,
  ).await {
    Ok(historical_data) => {
      Ok(with_status(json(&historical_data), StatusCode::OK))
    },
    Err(e) => {
      logging::log_error("과거 시장 데이터 조회 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to get historical data: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

/// 거래 내역 조회 핸들러
pub async fn get_recent_trades(
  symbol: String,
  limit: Option<usize>,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  let exchange = exchange.read().await;
  match exchange.get_recent_trades(&symbol, limit).await {
    Ok(trades) => {
      Ok(with_status(json(&trades), StatusCode::OK))
    },
    Err(e) => {
      logging::log_error("거래 내역 조회 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to get recent trades: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

//
// 계정 및 잔고 관련 핸들러
//

/// 자산 잔고 조회 핸들러
pub async fn get_balance(
  asset: String,
  exchange: Arc<RwLock<dyn Exchange>>,
) -> Result<impl Reply, warp::Rejection> {
  let exchange = exchange.read().await;
  match exchange.get_balance(&asset).await {
    Ok(balance) => {
      let response = serde_json::json!({
                "asset": asset,
                "balance": balance,
            });
      
      Ok(with_status(json(&response), StatusCode::OK))
    },
    Err(e) => {
      logging::log_error("잔고 조회 실패", &e);
      
      let error_response = serde_json::json!({
                "error": format!("Failed to get balance: {}", e),
            });
      
      Ok(with_status(json(&error_response), StatusCode::NOT_FOUND))
    }
  }
}

//
// 백테스트 관련 핸들러
//

/// 백테스트 요청 모델
#[derive(Debug, Deserialize)]
pub struct BacktestRequest {
  pub name: String,
  pub description: String,
  pub symbol: String,
  pub start_time: i64,
  pub end_time: i64,
  pub initial_balance: HashMap<String, f64>,
  pub strategy_type: String,
  pub strategy_params: serde_json::Value,
}

/// 백테스트 실행 핸들러
pub async fn run_backtest(
  backtest_req: BacktestRequest,
) -> Result<impl Reply, warp::Rejection> {
  use crate::backtest::scenario::BacktestScenarioBuilder;
  use chrono::{DateTime, Utc};
  
  // 백테스트 ID 생성
  let backtest_id = format!("backtest-{}", Uuid::new_v4());
  
  // 시간 변환
  let start_time = DateTime::<Utc>::from_timestamp_millis(backtest_req.start_time)
    .ok_or_else(|| warp::reject::custom(InvalidRequestError))?;
  
  let end_time = DateTime::<Utc>::from_timestamp_millis(backtest_req.end_time)
    .ok_or_else(|| warp::reject::custom(InvalidRequestError))?;
  
  // 데이터 파일 경로 생성
  let data_file = format!("./data/{}-1m.csv", backtest_req.symbol);
  
  // 시나리오 빌더 생성
  let mut builder = BacktestScenarioBuilder::new(backtest_req.name.clone())
    .description(backtest_req.description)
    .data_file(data_file.into())
    .period(start_time, end_time)
    .fee_rate(0.001); // 기본 수수료율
  
  // 초기 잔고 설정
  for (asset, amount) in &backtest_req.initial_balance {
    builder = builder.initial_balance(asset, *amount);
  }
  
  // 전략 생성 및 추가
  // 참고: 실제 구현에서는 strategy_params를 파싱하여 적절한 전략 인스턴스 생성
  // 여기서는 간단한 에러 응답만 반환
  
  let error_response = serde_json::json!({
        "error": "Backtest implementation not completed yet",
        "backtest_id": backtest_id,
    });
  
  Ok(with_status(json(&error_response), StatusCode::NOT_IMPLEMENTED))
}

//
// 오류 타입
//

/// 잘못된 요청 오류
#[derive(Debug)]
struct InvalidRequestError;

impl warp::reject::Reject for InvalidRequestError {}
        