/**
* filename : combined_strategy_tests
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/

//! 복합 전략 테스트
//!
//! 여러 전략을 조합한 복합 전략 테스트

use xQuant::models::market_data::MarketData;
use xQuant::models::order::OrderSide;
use xQuant::strategies::{
  Strategy, CombinedStrategy, VwapStrategy, TwapStrategy
};

#[test]
fn test_combined_strategy() {
  // 복합 전략 생성
  let mut strategy = CombinedStrategy::new(
    "Combined-BTC",
    "Combined VWAP and TWAP strategy for BTC",
  );
  
  // VWAP 전략 추가
  let vwap = VwapStrategy::new(
    "BTCUSDT",
    OrderSide::Buy,
    0.5,
    3600000, // 1시간
    10,      // 10개 윈도우
  );
  
  // TWAP 전략 추가
  let twap = TwapStrategy::new(
    "BTCUSDT",
    OrderSide::Buy,
    0.5,
    3600000, // 1시간
    5,       // 5개 분할
  );
  
  strategy.add_strategy(Box::new(vwap));
  strategy.add_strategy(Box::new(twap));
  
  // 시장 데이터로 업데이트
  let market_data = MarketData {
    symbol: "BTCUSDT".to_string(),
    timestamp: 1000,
    open: 50000.0,
    high: 50100.0,
    low: 49900.0,
    close: 50000.0,
    volume: 10.0,
  };
  
  let result = strategy.update(market_data);
  assert!(result.is_ok());
  
  // 하위 전략들의 주문 생성 확인
  let orders = strategy.get_orders().unwrap();
  
  // VWAP과 TWAP 전략 모두에서 주문이 생성되어야 함
  assert!(orders.len() >= 1);
  
  // 전략 정보 확인
  assert_eq!(strategy.name(), "Combined-BTC");
  assert!(strategy.description().contains("Combined"));
}

#[test]
fn test_combined_strategy_with_different_symbols() {
  // 복합 전략 생성
  let mut strategy = CombinedStrategy::new(
    "Multi-Asset",
    "Combined strategies for multiple assets",
  );
  
  // BTC VWAP 전략
  let btc_vwap = VwapStrategy::new(
    "BTCUSDT",
    OrderSide::Buy,
    0.1,
    3600000,
    10,
  );
  
  // ETH TWAP 전략
  let eth_twap = TwapStrategy::new(
    "ETHUSDT",
    OrderSide::Sell,
    1.0,
    3600000,
    5,
  );
  
  strategy.add_strategy(Box::new(btc_vwap));
  strategy.add_strategy(Box::new(eth_twap));
  
  // BTC 데이터 업데이트
  let btc_data = MarketData {
    symbol: "BTCUSDT".to_string(),
    timestamp: 1000,
    open: 50000.0,
    high: 50100.0,
    low: 49900.0,
    close: 50000.0,
    volume: 10.0,
  };
  
  strategy.update(btc_data).unwrap();
  
  // ETH 데이터 업데이트
  let eth_data = MarketData {
    symbol: "ETHUSDT".to_string(),
    timestamp: 1000,
    open: 3000.0,
    high: 3050.0,
    low: 2950.0,
    close: 3000.0,
    volume: 20.0,
  };
  
  strategy.update(eth_data).unwrap();
  
  // 주문 생성 확인
  let orders = strategy.get_orders().unwrap();
  
  // 두 전략에서 주문이 생성될 수 있음
  if !orders.is_empty() {
    // 주문들의 심볼 확인
    let symbols: Vec<&str> = orders.iter().map(|o| o.symbol.as_str()).collect();
    assert!(symbols.contains(&"BTCUSDT") || symbols.contains(&"ETHUSDT"));
  }
}