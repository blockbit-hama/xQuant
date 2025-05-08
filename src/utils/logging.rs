//! 로깅 유틸리티
//!
//! 로그 초기화 및 유틸리티 함수 제공

use std::io;
use env_logger::Builder;
use log::LevelFilter;
use std::env;

use crate::error::TradingError;

/// 로깅 시스템 초기화
pub fn init() -> Result<(), TradingError> {
    let mut builder = Builder::from_default_env();
    
    // RUST_LOG 환경변수 확인
    let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    
    // 로그 레벨 파싱
    let level_filter = match log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };
    
    builder
      .filter_level(level_filter)
      .format_timestamp_millis()
      .init();
    
    log::info!("로깅 시스템 초기화 완료: 레벨 = {}", log_level);
    
    Ok(())
}

/// 트레이딩 작업 시작 로그
pub fn log_trading_start(strategy_name: &str, symbol: &str) {
    log::info!("전략 시작: {} - 심볼: {}", strategy_name, symbol);
}

/// 트레이딩 작업 종료 로그
pub fn log_trading_end(strategy_name: &str, symbol: &str, result: &str) {
    log::info!("전략 종료: {} - 심볼: {} - 결과: {}", strategy_name, symbol, result);
}

/// 주문 생성 로그
pub fn log_order_created(order_id: &str, symbol: &str, side: &str, quantity: f64, price: f64) {
    log::info!("주문 생성: {} - 심볼: {} - 방향: {} - 수량: {} - 가격: {}", 
               order_id, symbol, side, quantity, price);
}

/// 주문 취소 로그
pub fn log_order_cancelled(order_id: &str) {
    log::info!("주문 취소: {}", order_id);
}

/// 주문 체결 로그
pub fn log_order_filled(order_id: &str, symbol: &str, quantity: f64, price: f64) {
    log::info!("주문 체결: {} - 심볼: {} - 수량: {} - 가격: {}", 
               order_id, symbol, quantity, price);
}

/// 오류 로그
pub fn log_error(context: &str, error: &TradingError) {
    log::error!("오류 발생 - {}: {}", context, error);
}