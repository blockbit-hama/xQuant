/**
* filename : backtest_tests
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/


use std::collections::HashMap;
use chrono::{Duration, Utc};
use xQuant::backtest::engine::BacktestEngine;
use xQuant::backtest::data_provider::CsvDataProvider;
use xQuant::backtest::scenario::{BacktestScenario, BacktestScenarioBuilder};
use xQuant::models::order::OrderSide;
use xQuant::strategies::VwapStrategy;

#[tokio::test]
async fn test_backtest_engine() {
  // 테스트용 초기 잔고 설정
  let mut initial_balance = HashMap::new();
  initial_balance.insert("USDT".to_string(), 10000.0);
  
  // 백테스트 엔진 생성 (실제 파일이 없으므로 테스트는 생성만 확인)
  let engine_result = BacktestEngine::from_csv(
    "./data/test_data.csv",  // 실제로는 존재하지 않는 파일
    Utc::now() - Duration::days(30),
    Utc::now(),
    "BTCUSDT",
    initial_balance,
    0.001,  // 0.1% 수수료
    0.0005, // 0.05% 슬리피지
  );
  
  // 파일이 없으므로 에러가 예상됨
  assert!(engine_result.is_err());
}

#[test]
fn test_backtest_scenario_builder() {
  // 백테스트 시나리오 빌더 생성
  let scenario_result = BacktestScenarioBuilder::new("VWAP 전략 테스트")
    .description("BTCUSDT에 대한 VWAP 기반 매수 전략 테스트")
    .data_file("./data/BTCUSDT-1m.csv".into())
    .last_days(30)  // 최근 30일
    .initial_balance("USDT", 10000.0)
    .fee_rate(0.001)  // 0.1% 수수료
    .strategy(Box::new(VwapStrategy::new(
      "BTCUSDT",
      OrderSide::Buy,
      1.0,  // 1 BTC 매수
      86400000,  // 24시간(밀리초) 동안 실행
      100,  // 100개 캔들의 VWAP 윈도우
    )))
    .build();
  
  // 테스트 데이터 파일이 없으므로 빌더 자체는 작동하지만 파일 검사에서 에러 예상
  assert!(scenario_result.is_err());
}