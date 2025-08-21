/**
* filename : lib
* author : HAMA
* date: 2025. 5. 11.
* description: 기술적 분석(TA) 기능이 추가된 자동 트레이딩 시스템
**/

mod api;
mod backtest;
mod config;
mod prediction_client;
mod core;
mod error;
mod http;
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
// use warp::Filter; // Warp 제거 예정: Axum 단일화
use chrono::{Utc, Duration};

// use crate::api::routes; // Warp 라우트 사용 중지
use crate::backtest::scenario::BacktestScenarioBuilder;
use crate::http::{build_router, AppState};
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
use crate::exchange::traits::Exchange;
use crate::prediction_client::{PredictionClient, SignalRequest};

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
  
  // 모든 제공자 연결(실패해도 경고 후 지속)
  if let Err(e) = market_manager.connect_all().await {
    log::warn!("market providers connect failed: {} — running with mocks only", e);
  }
  
  // 거래소 인스턴스 생성 (실거래/모의 선택)
  let exchange: Arc<RwLock<dyn Exchange>> = if !config.exchange.use_mock {
    let base = config.exchange.base_url.clone().unwrap_or("https://fapi.binance.com".to_string());
    let key = config.exchange.api_key.clone().unwrap_or_default();
    let sec = config.exchange.api_secret.clone().unwrap_or_default();
    Arc::new(RwLock::new(crate::exchange::binance_futures::BinanceFuturesExchange::new(base, key, sec)))
  } else {
    Arc::new(RwLock::new(MockExchange::new(config.clone())))
  };
  log::info!("거래소 초기화 완료 (mock: {})", config.exchange.use_mock);

  // 선물 기본 설정(실거래 사용 시): 레버리지/포지션모드/마진모드 적용
  if !config.exchange.use_mock {
    // 서버 시간 동기화
    {
      let mut ex = exchange.write().await;
      if let Err(e) = ex.sync_time().await { log::warn!("time sync failed: {}", e); }
    }
    // 설정 기반 기본값 적용
    let (symbols, lev, iso, hedge) = if let Some(f) = &config.futures {
      (if f.symbols.is_empty() { vec!["BTCUSDT".into()] } else { f.symbols.clone() }, f.leverage, f.isolated, f.hedge)
    } else { (vec!["BTCUSDT".into(), "ETHUSDT".into()], 20, false, false) };
    if let Err(e) = init_futures_defaults(exchange.clone(), symbols, lev, iso, hedge).await {
      log::warn!("futures defaults init failed: {}", e);
    }
  }
  
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
  
  // 예측 API 헬스체크 및 샘플 호출
  {
    let pred = PredictionClient::new(config.prediction_api.base_url.clone());
    match pred.health_check().await {
      Ok(true) => log::info!("Prediction API healthy: {}", config.prediction_api.base_url),
      Ok(false) => log::warn!("Prediction API unhealthy: {}", config.prediction_api.base_url),
      Err(e) => log::warn!("Prediction API check failed: {}", e),
    }

    // 샘플 시그널 1회 호출(실패해도 무시)
    let sample_req = SignalRequest {
      symbol: "BTC/USDT".to_string(),
      timeframe: "1h".to_string(),
      strategy: "trend_following".to_string(),
      lookback: 100,
    };
    match pred.get_signals(sample_req).await {
      Ok(sig) => log::info!("Sample signal: {} (conf {:.2})", sig.signal, sig.confidence),
      Err(e) => log::warn!("Sample signal fetch failed: {}", e),
    }
  }

  // 기본 기술적 분석 전략 추가 (예시)
  setup_technical_strategies(strategy_manager.clone(), exchange.clone(), market_stream.clone()).await?;
  log::info!("기술적 분석 전략 초기화 완료");
  
  // 전략 실행 런타임 시작: 거래소 시세 폴링 → 전략 업데이트 → 주문 제출
  start_strategy_runtime(
    strategy_manager.clone(),
    order_manager.clone(),
    exchange.clone(),
    vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
  );
  
  // Axum 서버 시작
  let axum_state = AppState { exchange: exchange.clone(), strategy_manager: strategy_manager.clone() };
  let axum_router = build_router(axum_state);
  let axum_addr = std::net::SocketAddr::from(([127,0,0,1], 4000));
  log::info!("Axum 서버 시작: http://127.0.0.1:4000/");
  let axum_task = tokio::spawn(async move {
    let listener = tokio::net::TcpListener::bind(axum_addr).await.unwrap();
    axum::serve(listener, axum_router.into_make_service()).await.unwrap();
  });
  let _ = tokio::join!(axum_task);
  
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
  
  // 시장 데이터 스트림 연결은 추후 구현 예정
  
  Ok(())
}

// 선물 기본설정: 심볼별 레버리지/마진모드, 계정 포지션모드 적용
async fn init_futures_defaults(
  exchange: Arc<RwLock<dyn Exchange>>,
  symbols: Vec<String>,
  leverage: u32,
  isolated: bool,
  hedge: bool,
) -> Result<(), anyhow::Error> {
  {
    let mut ex = exchange.write().await;
    // 포지션 모드(dualSidePosition): false=One-way, true=Hedge
    ex.set_futures_position_mode(hedge).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
  }

  for symbol in symbols {
    let mut ex = exchange.write().await;
    ex.set_futures_margin_mode(&symbol, isolated).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    ex.set_futures_leverage(&symbol, leverage).await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    log::info!("Applied futures settings: symbol={}, isolated={}, leverage={}, hedge={}", symbol, isolated, leverage, hedge);
  }

  Ok(())
}

// 실시간 전략 실행 루프: 심볼별로 거래소 시세를 폴링하여 전략을 업데이트하고 주문을 제출한다
fn start_strategy_runtime(
  strategy_manager: Arc<RwLock<StrategyManager>>,
  order_manager: Arc<RwLock<OrderManager>>,
  exchange: Arc<RwLock<dyn Exchange>>,
  symbols: Vec<String>,
) {
  // 심볼별 태스크 생성
  for symbol in symbols {
    let sm = strategy_manager.clone();
    let om = order_manager.clone();
    let ex = exchange.clone();
    tokio::spawn(async move {
      let mut ticker = tokio::time::interval(std::time::Duration::from_millis(1000));
      loop {
        ticker.tick().await;
        // 최신 시장 데이터 조회
        let md_res = {
          let exr = ex.read().await;
          exr.get_market_data(&symbol).await
        };
        let market_data = match md_res {
          Ok(md) => md,
          Err(e) => {
            log::warn!("market data fetch failed for {}: {}", symbol, e);
            continue;
          }
        };
        // 전략 업데이트 및 주문 수집
        let orders = {
          let mut manager = sm.write().await;
          if let Err(e) = manager.update_all(&market_data) {
            log::warn!("strategy update failed: {}", e);
            Vec::new()
          } else {
            match manager.get_all_orders() {
              Ok(os) => os,
              Err(e) => {
                log::warn!("collect orders failed: {}", e);
                Vec::new()
              }
            }
          }
        };
        // 주문 제출
        for order in orders {
          let submit_res = {
            let om_read = om.read().await;
            om_read.create_order(order).await
          };
          if let Err(e) = submit_res {
            log::warn!("order submit failed: {}", e);
          }
        }
      }
    });
  }
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
  let mut scenario = if args.len() > 2 {
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