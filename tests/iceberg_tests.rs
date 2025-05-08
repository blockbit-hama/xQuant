//! Iceberg 관련 테스트
//!
//! Iceberg 관리자 및 전략 테스트

use std::sync::Arc;
use tokio::sync::RwLock;
use xQuant::config::Config;
use xQuant::core::IcebergManager;
use xQuant::exchange::mock::MockExchange;
use xQuant::models::market_data::MarketData;
use xQuant::models::order::{OrderSide, OrderType};
use xQuant::strategies::{Strategy, IcebergStrategy};

#[tokio::test]
async fn test_iceberg_manager_execution() {
  // 설정 및 모의 거래소 초기화
  let config = Config::default();
  let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
  
  // Iceberg 관리자 생성
  let mut iceberg = IcebergManager::new(
    exchange.clone(),
    "BTCUSDT",
    OrderSide::Buy,
    10.0,
    50000.0,
    1.0,
  );
  
  // 실행 시작
  let result = iceberg.start().await;
  assert!(result.is_ok());
  
  // 약간 기다리기
  tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
  
  // 가격 업데이트 테스트
  let update_result = iceberg.update_price(51000.0).await;
  assert!(update_result.is_ok());
  
  // 중지
  let stop_result = iceberg.stop().await;
  assert!(stop_result.is_ok());
}

#[test]
fn test_iceberg_strategy() {
  // Iceberg 전략 생성
  let mut strategy = IcebergStrategy::new(
    "BTCUSDT",
    OrderSide::Buy,
    10.0,
    50000.0,
    1.0,
  );
  
  // 초기 상태 확인
  assert_eq!(strategy.name(), "Iceberg-BTCUSDT");
  assert!(strategy.description().contains("Iceberg"));
  
  // 가격 조건 충족하는 시장 데이터 추가
  let market_data = MarketData {
    symbol: "BTCUSDT".to_string(),
    timestamp: 1000,
    open: 50100.0,
    high: 50200.0,
    low: 49900.0,
    close: 49900.0, // 매수 조건 충족 (현재가 < 지정가)
    volume: 10.0,
  };
  
  let result = strategy.update(market_data);
  assert!(result.is_ok());
  
  // 주문 생성 확인
  let orders = strategy.get_orders().unwrap();
  assert!(!orders.is_empty());
  
  // 주문 내용 확인
  let first_order = &orders[0];
  assert_eq!(first_order.symbol, "BTCUSDT");
  assert_eq!(first_order.side, OrderSide::Buy);
  assert_eq!(first_order.order_type, OrderType::Limit);
  assert_eq!(first_order.quantity, 1.0);
  assert_eq!(first_order.price, 50000.0);
  assert_eq!(first_order.iceberg_qty, Some(1.0));
  
  // 가격 조건 불충족 시 주문 생성 안함 확인
  let market_data2 = MarketData {
    symbol: "BTCUSDT".to_string(),
    timestamp: 2000,
    open: 50100.0,
    high: 50200.0,
    low: 50100.0,
    close: 50100.0, // 매수 조건 불충족 (현재가 > 지정가)
    volume: 15.0,
  };
  
  strategy.update(market_data2).unwrap();
  let orders2 = strategy.get_orders().unwrap();
  assert!(orders2.is_empty());
}