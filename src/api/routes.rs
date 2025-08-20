use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;

use crate::api::handlers;
use crate::config::Config;
use crate::exchange::traits::Exchange;
use crate::order_core::manager::OrderManager;
use crate::core::strategy_manager::StrategyManager;  // 추가됨

/// 트레이딩 시스템의 API 라우트 생성
pub fn create_routes(
    exchange: Arc<RwLock<dyn Exchange>>,
    order_manager: Arc<RwLock<OrderManager>>,
    strategy_manager: Arc<RwLock<StrategyManager>>,  // 추가됨
    config: Config,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // 헬스체크 라우트
    let health = warp::path("health")
      .and(warp::get())
      .map(|| warp::reply::json(&serde_json::json!({"status":"ok"})));
    
    // 상태 필터 생성
    let exchange_filter = warp::any().map(move || exchange.clone());
    let order_manager_filter = warp::any().map(move || order_manager.clone());
    let strategy_manager_filter = warp::any().map(move || strategy_manager.clone());  // 추가됨
    let config_filter = warp::any().map(move || config.clone());
    
    // 주문 관리 라우트
    let orders = warp::path("orders");
    
    let order_routes = orders
      .and(warp::post())
      .and(warp::body::json())
      .and(exchange_filter.clone())
      .and(order_manager_filter.clone())
      .and_then(handlers::create_order)
      .or(orders
        .and(warp::get())
        .and(order_manager_filter.clone())
        .and_then(handlers::get_orders))
      .or(orders
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(order_manager_filter.clone())
        .and_then(handlers::cancel_order));
    
    // VWAP 스플리팅 라우트
    let vwap = warp::path("vwap");
    
    let vwap_routes = vwap
      .and(warp::post())
      .and(warp::body::json())
      .and(exchange_filter.clone())
      .and(order_manager_filter.clone())
      .and_then(handlers::create_vwap_order)
      .or(vwap
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(order_manager_filter.clone())
        .and_then(handlers::get_vwap_status))
      .or(vwap
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(order_manager_filter.clone())
        .and_then(handlers::cancel_vwap_order));
    
    // Iceberg 주문 라우트
    let iceberg = warp::path("iceberg");
    
    let iceberg_routes = iceberg
      .and(warp::post())
      .and(warp::body::json())
      .and(exchange_filter.clone())
      .and(order_manager_filter.clone())
      .and_then(handlers::create_iceberg_order)
      .or(iceberg
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(order_manager_filter.clone())
        .and_then(handlers::get_iceberg_status))
      .or(iceberg
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(order_manager_filter.clone())
        .and_then(handlers::cancel_iceberg_order));
    
    // Trailing Stop 라우트
    let trailing = warp::path("trailing");
    
    let trailing_routes = trailing
      .and(warp::post())
      .and(warp::body::json())
      .and(exchange_filter.clone())
      .and(order_manager_filter.clone())
      .and_then(handlers::create_trailing_stop)
      .or(trailing
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(order_manager_filter.clone())
        .and_then(handlers::get_trailing_stop_status))
      .or(trailing
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(order_manager_filter.clone())
        .and_then(handlers::cancel_trailing_stop));
    
    // 시장 데이터 라우트
    let market = warp::path("market");
    
    let market_routes = market
      .and(warp::path::param::<String>())
      .and(warp::get())
      .and(exchange_filter.clone())
      .and_then(handlers::get_market_data);
    
    // TA 전략 관련 라우트 (신규)
    let strategies = warp::path("strategies");
    
    let strategy_routes = strategies
      .and(warp::get())
      .and(strategy_manager_filter.clone())
      .and_then(handlers::list_strategies)
      .or(strategies
        .and(warp::path("ta"))
        .and(warp::post())
        .and(warp::body::json())
        .and(strategy_manager_filter.clone())
        .and(exchange_filter.clone())
        .and_then(handlers::create_ta_strategy))
      .or(strategies
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(strategy_manager_filter.clone())
        .and_then(handlers::get_strategy_status))
      .or(strategies
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(strategy_manager_filter.clone())
        .and_then(handlers::delete_strategy))
      .or(strategies
        .and(warp::path::param::<String>())
        .and(warp::path("toggle"))
        .and(warp::post())
        .and(warp::body::json())
        .and(strategy_manager_filter.clone())
        .and_then(handlers::toggle_strategy));
    
    // TA 인디케이터 라우트 (신규)
    let indicators = warp::path("indicators");
    
    let indicator_routes = indicators
      .and(warp::path::param::<String>())  // 심볼
      .and(warp::get())
      .and(warp::query::<handlers::IndicatorQuery>())
      .and(exchange_filter.clone())
      .and_then(handlers::calculate_indicators);
    
    // 모든 라우트 결합
    health
      .or(order_routes)
      .or(vwap_routes)
      .or(iceberg_routes)
      .or(trailing_routes)
      .or(market_routes)
      .or(strategy_routes)  // 신규
      .or(indicator_routes)  // 신규
}