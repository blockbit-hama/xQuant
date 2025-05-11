/**
* filename : lib
* author : HAMA
* date: 2025. 5. 8.
* description:
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

    // API 라우트 초기화
    let routes = routes::create_routes(
        exchange.clone(),
        order_manager.clone(),
        config.clone(),
    );
    log::info!("API 라우트 초기화 완료");

    // Warp 서버 시작
    let addr = ([127, 0, 0, 1], 3030);
    log::info!("서버 시작: http://{}:{}/", addr.0.join("."), addr.1);
    warp::serve(routes).run(addr).await;

    Ok(())
}

async fn run_backtest() -> Result<(), anyhow::Error> {
    log::info!("백테스트 모드 시작...");

    // 백테스트 시나리오 생성
    let mut scenario = BacktestScenarioBuilder::new("VWAP 전략 테스트")
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

    // 백테스트 실행
    log::info!("백테스트 실행 중...");
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

    Ok(())
}