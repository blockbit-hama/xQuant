//! TWAP 전략
//!
//! 시간 가중 평균 가격 기반 주문 실행 전략

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide, OrderType};
use crate::strategies::Strategy;

/// TWAP 매매 전략
pub struct TwapStrategy {
    /// 전략 이름
    name: String,
    /// 전략 설명
    description: String,
    /// 거래 심볼
    symbol: String,
    /// 매매 방향
    side: OrderSide,
    /// 목표 총 수량
    total_quantity: f64,
    /// 실행 간격 (밀리초)
    execution_interval: i64,
    /// 분할 수
    num_slices: usize,
    /// 이미 실행한 수량
    executed_quantity: f64,
    /// 현재 시장 데이터
    current_market_data: Option<MarketData>,
    /// 전략 활성 여부
    is_active: bool,
    /// 마지막 주문 시간
    last_order_time: i64,
    /// 주문 간격 (밀리초)
    slice_interval: i64,
}

impl TwapStrategy {
    /// 새 TWAP 전략 생성
    pub fn new(
        symbol: impl Into<String>,
        side: OrderSide,
        total_quantity: f64,
        execution_interval: i64,
        num_slices: usize,
    ) -> Self {
        let symbol_str = symbol.into();
        let slice_interval = execution_interval / num_slices as i64;
        
        TwapStrategy {
            name: format!("TWAP-{}", symbol_str),
            description: "Time Weighted Average Price based execution strategy".to_string(),
            symbol: symbol_str,
            side,
            total_quantity,
            execution_interval,
            num_slices,
            executed_quantity: 0.0,
            current_market_data: None,
            is_active: true,
            last_order_time: 0,
            slice_interval,
        }
    }
}

impl Strategy for TwapStrategy {
    fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
        if market_data.symbol != self.symbol {
            return Ok(());
        }
        
        // 시장 데이터 업데이트
        self.current_market_data = Some(market_data);
        
        Ok(())
    }
    
    fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
        if !self.is_active || self.executed_quantity >= self.total_quantity {
            return Ok(Vec::new());
        }
        
        if let Some(market_data) = &self.current_market_data {
            let current_time = market_data.timestamp;
            
            // 시간 간격 체크
            if self.last_order_time > 0 && current_time - self.last_order_time < self.slice_interval {
                return Ok(Vec::new());
            }
            
            // 남은 수량 계산
            let remaining = self.total_quantity - self.executed_quantity;
            
            if remaining <= 0.0 {
                self.is_active = false;
                return Ok(Vec::new());
            }
            
            // 분할 크기 계산
            let slice_quantity = (self.total_quantity / self.num_slices as f64).min(remaining);
            
            // 시장가 주문 생성
            let order = Order::new(
                self.symbol.clone(),
                self.side.clone(),
                OrderType::Market,
                slice_quantity,
                market_data.close,
            ).with_twap_params(self.slice_interval);
            
            // 주문 추적 업데이트
            self.executed_quantity += slice_quantity;
            self.last_order_time = current_time;
            
            // 목표량 도달 시 비활성화
            if self.executed_quantity >= self.total_quantity {
                self.is_active = false;
            }
            
            return Ok(vec![order]);
        }
        
        Ok(Vec::new())
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
    
    #[test]
    fn test_twap_strategy() {
        // TWAP 전략 생성
        let mut strategy = TwapStrategy::new(
            "BTCUSDT",
            OrderSide::Buy,
            1.0,
            3600000, // 1시간 (밀리초)
            5,       // 5개 분할
        );
        
        // 시장 데이터 추가
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
        
        // 주문 생성 확인
        let orders = strategy.get_orders().unwrap();
        assert!(!orders.is_empty());
        
        // 주문 내용 확인
        let first_order = &orders[0];
        assert_eq!(first_order.symbol, "BTCUSDT");
        assert_eq!(first_order.side, OrderSide::Buy);
        assert_eq!(first_order.order_type, OrderType::Market);
        assert_eq!(first_order.quantity, 0.2); // 1.0/5
        assert_eq!(first_order.price, 50000.0);
        
        // 다음 interval 내에는 주문 생성 안함 확인
        let market_data2 = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1001, // 같은 interval 내
            open: 50050.0,
            high: 50150.0,
            low: 49950.0,
            close: 50050.0,
            volume: 12.0,
        };
        
        strategy.update(market_data2).unwrap();
        let orders2 = strategy.get_orders().unwrap();
        assert!(orders2.is_empty());
        
        // 다음 interval에는 주문 생성 확인
        let market_data3 = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000 + strategy.slice_interval, // 다음 interval
            open: 50100.0,
            high: 50200.0,
            low: 50000.0,
            close: 50100.0,
            volume: 15.0,
        };
        
        strategy.update(market_data3).unwrap();
        let orders3 = strategy.get_orders().unwrap();
        assert!(!orders3.is_empty());
    }
}