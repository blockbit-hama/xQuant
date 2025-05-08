//! VWAP 관련 테스트
//!
//! VWAP 분할기 및 전략 테스트

use std::sync::Arc;
use tokio::sync::RwLock;
use xQuant::config::Config;
use xQuant::core::VwapSplitter;
use xQuant::exchange::mock::MockExchange;
use xQuant::models::market_data::MarketData;
use xQuant::models::order::{OrderSide, OrderType};
use xQuant::strategies::{Strategy, VwapStrategy};

#[tokio::test]
async fn test_vwap_splitter_execution() {
  // 설정 및 모의 거래소 초기화
  let config = Config::default();
  let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
  
  // VWAP 분할기 생성
  let mut vwap = VwapSplitter::new(
    exchange.clone(),
    "BTCUSDT",
    OrderSide::Buy,
    1.0,
    3600000,  // 1시간 (밀리초)
    Some(10.0),
  );
  
  // 실행 시작
  let result = vwap.start().await;
  assert!(result.is_ok());
  
  // 상태 확인
  let (is_active, executed, total) = vwap.status();
  assert!(!is_active);  // 테스트에서는 즉시 완료
  assert!(executed > 0.0);
  assert_eq!(total, 1.0);
  
  // 중지 기능 테스트
  let stop_result = vwap.stop().await;
  assert!(stop_result.is_ok());
}

#[test]
fn test_vwap_strategy() {
  // VWAP 전략 생성
  let mut strategy = VwapStrategy::new(
    "BTCUSDT",
    OrderSide::Buy,
    1.0,
    3600000,
    10,
  );
  
  // 초기 상태 확인
  assert_eq!(strategy.name(), "VWAP-BTCUSDT");
  assert!(strategy.description().contains("VWAP"));
  
  // 시장 데이터 추가
  for i in 0..15 {
    let market_data = MarketData {
      symbol: "BTCUSDT".to_string(),
      timestamp: 1000 + i * 60000,
      open: 50000.0 + i as f64 * 10.0,
      high: 50100.0 + i as f64 * 10.0,
      low: 49900.0 + i as f64 * 10.0,
      close: 50000.0 + i as f64 * 10.0,
      volume: 10.0 + i as f64,
    };
    
    let result = strategy.update(market_data);
    assert!(result.is_ok());
  }
  
  // 주문 생성 확인
  let orders = strategy.get_orders().unwrap();
  assert!(!orders.is_empty());
  
  // 주문 내용 확인
  let first_order = &orders[0];
  assert_eq!(first_order.symbol, "BTCUSDT");
  assert_eq!(first_order.side, OrderSide::Buy);
  assert_eq!(first_order.order_type, OrderType::Limit);
  assert!(first_order.quantity > 0.0);
  assert!(first_order.price > 0.0);
}