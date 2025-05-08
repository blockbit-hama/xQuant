//! Trailing Stop 관련 테스트
//!
//! Trailing Stop 관리자 및 전략 테스트

use std::sync::Arc;
use tokio::sync::RwLock;
use xQuant::config::Config;
use xQuant::core::TrailingStopManager;
use xQuant::exchange::mock::MockExchange;
use xQuant::models::market_data::MarketData;
use xQuant::models::order::{OrderSide, OrderType};
use xQuant::strategies::{Strategy, TrailingStopStrategy};

#[tokio::test]
async fn test_trailing_stop_manager_execution() {
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
  
  // 델타 업데이트 테스트
  let update_result = trailing_stop.update_delta(3.0).await;
  assert!(update_result.is_ok());
  
  // 상태 확인
  let (is_active, executed, trigger_price, quantity) = trailing_stop.status();
  assert!(is_active);
  assert!(!executed);
  assert!(trigger_price > 0.0);
  assert_eq!(quantity, 1.0);
  
  // 중지
  let stop_result = trailing_stop.stop().await;
  assert!(stop_result.is_ok());
  
  // 상태 재확인
  let (is_active, _, _, _) = trailing_stop.status();
  assert!(!is_active);
}

#[test]
fn test_trailing_stop_strategy() {
  // Trailing Stop 전략 생성 (매도)
  let mut strategy = TrailingStopStrategy::new(
    "BTCUSDT",
    OrderSide::Sell,
    1.0,
    2.0,  // 2% 트레일링 델타
    None, // 활성화 가격 없음 (즉시 활성화)
  );
  
  // 초기 상태 확인
  assert_eq!(strategy.name(), "TrailingStop-BTCUSDT");
  assert!(strategy.description().contains("trailing"));
  
  // 진입 가격 설정
  strategy.set_entry_price(50000.0);
  
  // 가격 하락 시 스탑 가격 이동 확인
  let market_data1 = MarketData {
    symbol: "BTCUSDT".to_string(),
    timestamp: 1000,
    open: 50000.0,
    high: 50000.0,
    low: 49500.0,
    close: 49500.0,
    volume: 10.0,
  };
  
  strategy.update(market_data1).unwrap();
  
  // 이 시점에서는 스탑 조건 충족하지 않음
  let orders1 = strategy.get_orders().unwrap();
  assert!(orders1.is_empty());
  
  // 가격 급등 시 스탑 조건 충족 확인
  let market_data2 = MarketData {
    symbol: "BTCUSDT".to_string(),
    timestamp: 2000,
    open: 49500.0,
    high: 50500.0,
    low: 49500.0,
    close: 50500.0, // 가격 급등 (50500 > 49500 * 1.02)
    volume: 15.0,
  };
  
  strategy.update(market_data2).unwrap();
  
  // 스탑 조건 충족하여 주문 생성
  let orders2 = strategy.get_orders().unwrap();
  assert!(!orders2.is_empty());
  
  // 주문 내용 확인
  let order = &orders2[0];
  assert_eq!(order.symbol, "BTCUSDT");
  assert_eq!(order.side, OrderSide::Sell);
  assert_eq!(order.order_type, OrderType::Market);
  assert_eq!(order.quantity, 1.0);
  
  // 주문 생성 후 비활성화 확인
  let orders3 = strategy.get_orders().unwrap();
  assert!(orders3.is_empty());
}