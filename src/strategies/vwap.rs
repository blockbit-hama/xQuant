//! VWAP 기반 매매 전략
//!
//! 거래량 가중 평균 가격을 기준으로 매매 신호를 생성하는 전략

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide, OrderType};
use crate::strategies::Strategy;

/// VWAP 매매 전략
pub struct VwapStrategy {
    /// 전략 이름
    name: String,
    /// 전략 설명
    description: String,
    /// 거래 심볼
    symbol: String,
    /// 매매 방향
    side: OrderSide,
    /// 목표 총 수량
    target_quantity: f64,
    /// 실행 간격 (밀리초)
    execution_interval: i64,
    /// VWAP 계산 윈도우 크기
    vwap_window: usize,
    /// 가격 데이터 히스토리
    price_data: Vec<MarketData>,
    /// 이미 실행한 수량
    executed_quantity: f64,
    /// 전략 활성 여부
    is_active: bool,
    /// 마지막 주문 시간
    last_order_time: i64,
    /// 주문 간격 (밀리초)
    order_interval: i64,
}

impl VwapStrategy {
    /// 새 VWAP 전략 생성
    pub fn new(
        symbol: impl Into<String>,
        side: OrderSide,
        target_quantity: f64,
        execution_interval: i64,
        vwap_window: usize,
    ) -> Self {
        let symbol_str = symbol.into();
        
        VwapStrategy {
            name: format!("VWAP-{}", symbol_str),
            description: "Volume Weighted Average Price based execution strategy".to_string(),
            symbol: symbol_str,
            side,
            target_quantity,
            execution_interval,
            vwap_window,
            price_data: Vec::new(),
            executed_quantity: 0.0,
            is_active: true,
            last_order_time: 0,
            order_interval: execution_interval / 10, // 10개 분할 주문
        }
    }
    
    /// VWAP 계산
    fn calculate_vwap(&self) -> Option<f64> {
        if self.price_data.is_empty() {
            return None;
        }
        
        let window_data = if self.price_data.len() <= self.vwap_window {
            &self.price_data[..]
        } else {
            &self.price_data[self.price_data.len() - self.vwap_window..]
        };
        
        let volume_price_sum: f64 = window_data.iter()
          .map(|data| data.volume * data.close)
          .sum();
        
        let volume_sum: f64 = window_data.iter()
          .map(|data| data.volume)
          .sum();
        
        if volume_sum > 0.0 {
            Some(volume_price_sum / volume_sum)
        } else {
            None
        }
    }
}

impl Strategy for VwapStrategy {
    fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
        if market_data.symbol != self.symbol {
            return Ok(());
        }
        
        // 시장 데이터 저장
        self.price_data.push(market_data);
        
        // 윈도우 크기 유지
        if self.price_data.len() > self.vwap_window * 2 {
            self.price_data.remove(0);
        }
        
        Ok(())
    }
    
    fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
        if !self.is_active || self.executed_quantity >= self.target_quantity {
            return Ok(Vec::new());
        }
        
        let current_time = if let Some(last_data) = self.price_data.last() {
            last_data.timestamp
        } else {
            return Ok(Vec::new());
        };
        
        // 주문 간격 체크
        if current_time - self.last_order_time < self.order_interval {
            return Ok(Vec::new());
        }
        
        // VWAP 계산
        let vwap = match self.calculate_vwap() {
            Some(price) => price,
            None => return Ok(Vec::new()),
        };
        
        // 남은 수량 계산
        let remaining = self.target_quantity - self.executed_quantity;
        
        // 다음 주문 크기 계산
        let slice_size = (self.target_quantity / 10.0).min(remaining);
        
        if slice_size > 0.0 {
            // 주문 생성
            let order = Order::new(
                self.symbol.clone(),
                self.side.clone(),
                OrderType::Limit,
                slice_size,
                vwap,
            );
            
            // 주문 추적 업데이트
            self.executed_quantity += slice_size;
            self.last_order_time = current_time;
            
            // 목표량 도달 시 비활성화
            if self.executed_quantity >= self.target_quantity {
                self.is_active = false;
            }
            
            Ok(vec![order])
        } else {
            Ok(Vec::new())
        }
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
    fn test_vwap_strategy() {
        // VWAP 전략 생성
        let mut strategy = VwapStrategy::new(
            "BTCUSDT",
            OrderSide::Buy,
            1.0,
            3600000,
            10,
        );
        
        // 시장 데이터 추가
        for i in 0..15 {
            let market_data = MarketData {
                symbol: "BTCUSDT".to_string(),
                timestamp: 1000 + i * 60000,
                open: 50000.0 + i as f64 * 10.0,
                high: 50100.0 + i as f64 * 10.0,
                low: 49900.0 + i as f64 * 10.0,
                close: 50000.0 + i as f64 * 10.0,
                volume: 10.0 + i as f64,
            };
            
            let result = strategy.update(market_data);
            assert!(result.is_ok());
        }
        
        // 주문 생성 확인
        let orders = strategy.get_orders().unwrap();
        assert!(!orders.is_empty());
        
        // 주문 내용 확인
        let first_order = &orders[0];
        assert_eq!(first_order.symbol, "BTCUSDT");
        assert_eq!(first_order.side, OrderSide::Buy);
        assert_eq!(first_order.order_type, OrderType::Limit);
        assert!(first_order.quantity > 0.0);
        assert!(first_order.price > 0.0);
    }
}