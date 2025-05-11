pub mod vwap;
pub mod iceberg;
pub mod trailing_stop;
pub mod twap;
pub mod combined;
mod technical;

use async_trait::async_trait;

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::Order;

/// 트레이딩 전략 인터페이스
pub trait Strategy: Send + Sync {
    /// 시장 데이터로 전략 업데이트
    fn update(&mut self, market_data: MarketData) -> Result<(), TradingError>;

    /// 현재 생성된 주문 가져오기
    fn get_orders(&mut self) -> Result<Vec<Order>, TradingError>;

    /// 전략 이름 가져오기
    fn name(&self) -> &str;

    /// 전략 설명 가져오기
    fn description(&self) -> &str;
}

/// 전략 팩토리 인터페이스
pub trait StrategyFactory: Send + Sync {
    /// 전략 생성
    fn create(&self) -> Box<dyn Strategy>;

    /// 팩토리 이름 가져오기
    fn name(&self) -> &str;
}

// 핵심 전략 재노출
pub use vwap::VwapStrategy;
pub use iceberg::IcebergStrategy;
pub use trailing_stop::TrailingStopStrategy;
pub use twap::TwapStrategy;
pub use combined::CombinedStrategy;
