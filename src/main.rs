/**
* filename : lib
* author : HAMA
* date: 2025. 5. 11.
* description: 기술적 분석(TA) 기능이 추가된 자동 트레이딩 시스템
**/

mod api;
mod backtest;
mod config;
mod core;
mod error;
mod exchange;
mod market_data;
mod models;
mod order_core;
mod strategies;
mod utils;
// 새로 추가된 TA 관련 모듈
mod indicators;
mod signals;
mod trading_bots;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::Filter;
use chrono::{Utc, Duration};

use crate::api::routes;
use crate::backtest::scenario::BacktestScenarioBuilder;
use crate::config::Config;
use crate::exchange::mocks::MockExchange;
use crate::market_data::provider::MarketDataManager;
use crate::market_data::stream::MarketDataStream;
use crate::market_data::websocket::WebSocketProvider;
use crate::order_core::manager::OrderManager;
use crate::order_core::repository::InMemoryOrderRepository;
use crate::strategies::vwap::VwapStrategy;
use crate::utils::logging;
use crate::models::order::OrderSide;
// 새로 추가된 TA 관련 임포트
use crate::strategies::technical::TechnicalStrategy;
use crate::strategies::combined::CombinedStrategy;
use crate::core::strategy_manager::StrategyManager;
use crate::trading_bots::bot_config::TradingBotConfig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  // 로깅 초기화
  logging::init()?;
  log::info!("자동 트레이딩 시스템 시작...");
  
  // 설정 로드
  let config = Config::load()?;
  log::info!("설정 로드 완료");
  
  // 명령줄 인수 확인
  let args: Vec<String> = std::env::args().collect();
  
  if args.len() > 1 && args[1] == "backtest" {
    run_backtest().await?;
  } else {
    run_live_trading(config).await?;
  }
  
  Ok(())
}

async fn run_live_trading(config: Config) -> Result<(), anyhow::Error> {
  // 시장 데이터 스트림 생성
  let market_stream = Arc::new(RwLock::new(MarketDataStream::new(1000)));
  
  // WebSocket 제공자 생성
  let ws_provider = Arc::new(RwLock::new(WebSocketProvider::new(
    "wss://stream.binance.com:9443/ws",
    market_stream.clone(),
  )));
  
  // 시장 데이터 관리자 생성
  let mut market_manager = MarketDataManager::new();
  market_manager.add_provider(ws_provider.clone());
  
  // 모든 제공자 연결
  market_manager.connect_all().await?;
  
  // 거래소 인스턴스 생성
  let exchange = Arc::new(RwLock::new(MockExchange::new(config.clone())));
  log::info!("모의 거래소 초기화 완료");
  
  // 주문 저장소 생성
  let order_repo = Arc::new(RwLock::new(InMemoryOrderRepository::new()));
  
  // 주문 관리자 생성
  let order_manager = Arc::new(RwLock::new(OrderManager::new(
    exchange.clone(),
    order_repo.clone(),
  )));
  
  // 주문 상태 감시 시작
  {
    let manager = order_manager.write().await;
    manager.start_order_monitoring().await?;
  }
  
  // 전략 매니저 생성 (신규)
  let strategy_manager = Arc::new(RwLock::new(StrategyManager::new()));
  log::info!("전략 매니저 초기화 완료");
  
  // 기본 기술적 분석 전략 추가 (예시)
  if config.enable_ta_strategies {
    setup_technical_strategies(strategy_manager.clone(), exchange.clone(), market_stream.clone()).await?;
    log::info!("기술적 분석 전략 초기화 완료");
  }
  
  // API 라우트 초기화 (전략 매니저 추가)
  let routes = routes::create_routes(
    exchange.clone(),
    order_manager.clone(),
    strategy_manager.clone(),  // 전략 매니저 추가
    config.clone(),
  );
  log::info!("API 라우트 초기화 완료");
  
  // Warp 서버 시작
  let addr = ([127, 0, 0, 1], 3030);
  log::info!("서버 시작: http://{}:{}/", addr.0.join("."), addr.1);
  warp::serve(routes).run(addr).await;
  
  Ok(())
}

// TA 전략 설정 함수 (신규)
async fn setup_technical_strategies(
  strategy_manager: Arc<RwLock<StrategyManager>>,
  exchange: Arc<RwLock<dyn Exchange>>,
  market_stream: Arc<RwLock<MarketDataStream>>,
) -> Result<(), anyhow::Error> {
  // 1. 이동평균 크로스오버 전략
  let ma_strategy = TechnicalStrategy::ma_crossover(
    "BTCUSDT".to_string(),
    12,  // 빠른 이동평균 기간
    26,  // 느린 이동평균 기간
  )?;
  
  // 2. RSI 전략
  let rsi_strategy = TechnicalStrategy::rsi(
    "ETHUSDT".to_string(),
    14,    // RSI 기간
    30.0,  // 과매도 기준점
    70.0,  // 과매수 기준점
  )?;
  
  // 3. MACD 전략
  let macd_strategy = TechnicalStrategy::macd(
    "BTCUSDT".to_string(),
    12,  // 빠른 EMA
    26,  // 느린 EMA
    9,   // 시그널 EMA
  )?;
  
  // 4. 복합 지표 전략
  let multi_strategy = TechnicalStrategy::multi_indicator(
    "ETHUSDT".to_string(),
  )?;
  
  // 5. RSI 신호 + TWAP 실행의 복합 전략
  let combined_strategy = CombinedStrategy::rsi_twap(
    "BTCUSDT".to_string(),
    14,      // RSI 기간
    30.0,    // 과매도 기준점
    70.0,    // 과매수 기준점
    60,      // 60분(TWAP 분할 기간)
  )?;
  
  // 6. MACD 신호 + VWAP 실행의 복합 전략
  let macd_vwap_strategy = CombinedStrategy::macd_vwap(
    "ETHUSDT".to_string(),
    12,    // 빠른 EMA
    26,    // 느린 EMA
    9,     // 시그널 EMA
    0.1,   // 거래량 참여율 10%
  )?;
  
  // 전략 매니저에 전략 추가
  let mut manager = strategy_manager.write().await;
  manager.add_strategy(Box::new(ma_strategy))?;
  manager.add_strategy(Box::new(rsi_strategy))?;
  manager.add_strategy(Box::new(macd_strategy))?;
  manager.add_strategy(Box::new(multi_strategy))?;
  manager.add_strategy(Box::new(combined_strategy))?;
  manager.add_strategy(Box::new(macd_vwap_strategy))?;
  
  // 시장 데이터 스트림에 전략 매니저 연결
  {
    let mut stream = market_stream.write().await;
    stream.register_strategy_manager(strategy_manager.clone());
  }
  
  Ok(())
}

async fn run_backtest() -> Result<(), anyhow::Error> {
  log::info!("백테스트 모드 시작...");
  
  // 기본 백테스트 시나리오 생성
  let basic_scenario = BacktestScenarioBuilder::new("VWAP 전략 테스트")
    .description("BTCUSDT에 대한 VWAP 기반 매수 전략 테스트")
    .data_file("./data/BTCUSDT-1m.csv".into())
    .last_days(30)  // 최근 30일
    .initial_balance("USDT", 10000.0)
    .fee_rate(0.001)  // 0.1% 수수료
    .slippage(0.0005)  // 0.05% 슬리피지
    .strategy(Box::new(VwapStrategy::new(
      "BTCUSDT",
      OrderSide::Buy,
      1.0,  // 1 BTC 매수
      86400000,  // 24시간(밀리초) 동안 실행
      100,  // 100개 캔들의 VWAP 윈도우
    )))
    .build()?;
  
  // TA 전략 백테스트 시나리오 생성 (신규)
  let ta_scenario = BacktestScenarioBuilder::new("MA 크로스오버 전략 테스트")
    .description("BTCUSDT에 대한 이동평균 크로스오버 전략 테스트")
    .data_file("./data/BTCUSDT-1m.csv".into())
    .last_days(30)  // 최근 30일
    .initial_balance("USDT", 10000.0)
    .fee_rate(0.001)  // 0.1% 수수료
    .slippage(0.0005)  // 0.05% 슬리피지
    .strategy(Box::new(TechnicalStrategy::ma_crossover(
      "BTCUSDT".to_string(),
      12,  // 빠른 이동평균 기간
      26,  // 느린 이동평균 기간
    )?))
    .build()?;
  
  // 추가 TA 백테스트 시나리오 생성
  let rsi_scenario = BacktestScenarioBuilder::new("RSI 전략 테스트")
    .description("BTCUSDT에 대한 RSI 기반 전략 테스트")
    .data_file("./data/BTCUSDT-1m.csv".into())
    .last_days(30)  // 최근 30일
    .initial_balance("USDT", 10000.0)
    .fee_rate(0.001)  // 0.1% 수수료
    .slippage(0.0005)  // 0.05% 슬리피지
    .strategy(Box::new(TechnicalStrategy::rsi(
      "BTCUSDT".to_string(),
      14,    // RSI 기간
      30.0,  // 과매도 기준점
      70.0,  // 과매수 기준점
    )?))
    .build()?;
  
  // 명령줄 인수 확인 - 어떤 백테스트를 실행할지 결정
  let args: Vec<String> = std::env::args().collect();
  let scenario = if args.len() > 2 {
    match args[2].as_str() {
      "ma" => ta_scenario,
      "rsi" => rsi_scenario,
      _ => basic_scenario,
    }
  } else {
    basic_scenario
  };
  
  // 백테스트 실행
  log::info!("백테스트 실행 중: {}", scenario.name());
  let result = scenario.run().await?;
  
  // 결과 출력
  println!("\n{}", result.summary());
  
  // 상세 분석
  println!("\n=== 거래 내역 ===");
  println!("총 거래 횟수: {}", result.trade_count());
  println!("승률: {:.2}%", result.win_rate());
  println!("평균 거래당 수익: {:.2} USDT", result.average_profit_per_trade());
  
  if let Some((trade, profit)) = result.max_profit_trade() {
    println!("최대 수익 거래: {:.2} USDT ({})", profit, trade.id);
  }
  
  if let Some((trade, loss)) = result.max_loss_trade() {
    println!("최대 손실 거래: {:.2} USDT ({})", loss, trade.id);
  }
  
  println!("\n=== 최종 잔고 ===");
  for (currency, amount) in &result.final_balance {
    println!("{}: {:.8}", currency, amount);
  }
  
  // TA 관련 추가 분석 출력 (신규)
  if scenario.name().contains("MA") || scenario.name().contains("RSI") {
    println!("\n=== TA 전략 성능 지표 ===");
    println!("샤프 비율: {:.4}", result.sharpe_ratio());
    println!("최대 손실폭: {:.2}%", result.max_drawdown() * 100.0);
    println!("수익 대 위험 비율: {:.2}", result.profit_factor());
    println!("CAR (연간 복합 수익률): {:.2}%", result.car() * 100.0);
  }
  
  Ok(())
}
``