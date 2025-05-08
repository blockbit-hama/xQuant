//! 복합 전략
//!
//! 여러 전략을 조합하여 사용하는 고급 전략

use std::collections::HashMap;

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide, OrderType};
use crate::strategies::Strategy;

/// 여러 전략을 조합한 복합 전략
pub struct CombinedStrategy {
    /// 전략 이름
    name: String,
    /// 전략 설명
    description: String,
    /// 하위 전략 목록
    sub_strategies: Vec<Box<dyn Strategy>>,
    /// 심볼별 시장 데이터
    market_data: HashMap<String, MarketData>,
}

impl CombinedStrategy {
    /// 새 복합 전략 생성
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        CombinedStrategy {
            name: name.into(),
            description: description.into(),
            sub_strategies: Vec::new(),
            market_data: HashMap::new(),
        }
    }
    
    /// 하위 전략 추가
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.sub_strategies.push(strategy);
    }
}

impl Strategy for CombinedStrategy {
    fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
        // 시장 데이터 저장
        self.market_data.insert(market_data.symbol.clone(), market_data.clone());
        
        // 모든 하위 전략 업데이트
        for strategy in &mut self.sub_strategies {
            strategy.update(market_data.clone())?;
        }
        
        Ok(())
    }
    
    fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
        let mut all_orders = Vec::new();
        
        // 모든 하위 전략의 주문 수집
        for strategy in &mut self.sub_strategies {
            let orders = strategy.get_orders()?;
            all_orders.extend(orders);
        }
        
        // 필요시 주문 필터링 또는 충돌 해결 로직 추가할 수 있음
        // 예: 같은 심볼에 대한 반대 방향 주문 상쇄 등
        
        Ok(all_orders)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::{VwapStrategy, TwapStrategy};
    
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
    }
}