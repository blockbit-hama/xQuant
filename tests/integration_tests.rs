//! 통합 테스트
//!
//! 전체 시스템 통합 테스트 수행

use std::sync::Arc;
use tokio::sync::RwLock;
use xQuant::config::Config;
use xQuant::core::{VwapSplitter, IcebergManager, TrailingStopManager};
use xQuant::exchange::mock::MockExchange;
use xQuant::models::order::{OrderSide, OrderType};

#[tokio::test]
async fn test_vwap_execution() {
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
}

#[tokio::test]
async fn test_iceberg_execution() {
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
  
  // 중지
  let stop_result = iceberg.stop().await;
  assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_trailing_stop_execution() {
  // 설정 및 모의 거래소 초기화
  let config = Config::default();
  let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
  
  // Trailing Stop 관리자 생성
  let mut trailing_stop = TrailingStopManager::new(
    exchange.clone(),
    "BTCUSDT",
    OrderSide::Sell,
    1.0,
    2.0,  // 2% 트레일링 델타
    None, // 활성화 가격 없음 (즉시 활성화)
  );
  
  // 실행 시작
  let result = trailing_stop.start().await;
  assert!(result.is_ok());
  
  // 약간 기다리기
  tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
  
  // 중지
  let stop_result = trailing_stop.stop().await;
  assert!(stop_result.is_ok());
}