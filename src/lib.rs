//! 자동 매매 시스템 라이브러리
//!
//! 다양한 주문 실행 전략과 백테스팅을 지원하는 트레이딩 시스템입니다.

pub mod api;
pub mod backtest;
pub mod config;
pub mod core;
pub mod error;
pub mod exchange;
pub mod market_data;
pub mod models;
pub mod order_core;
pub mod strategies;
pub mod utils;
mod trading_bots;

// 핵심 타입 재노출
pub use crate::error::TradingError;
pub use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};
pub use crate::models::trade::Trade;
pub use crate::models::market_data::MarketData;
pub use crate::models::position::Position;
pub use crate::exchange::traits::Exchange;
pub use crate::strategies::Strategy;

/// 버전 정보
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 결과 타입 별칭
pub type Result<T> = std::result::Result<T, TradingError>;