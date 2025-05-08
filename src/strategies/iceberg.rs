//! Iceberg 전략
//!
//! 대량 포지션을 시장에 드러나지 않게 구축하는 전략

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide, OrderType};
use crate::strategies::Strategy;

/// Iceberg 매매 전략
pub struct IcebergStrategy {
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
    /// 지정가 가격
    limit_price: f64,
    /// 시장에 노출할 부분 수량
    display_quantity: f64,
    /// 이미 실행한 수량
    executed_quantity: f64,
    /// 전략 활성 여부
    is_active: bool,
    /// 현재 시장 데이터
    current_market_data: Option<MarketData>,
    /// 가격 조건 충족 여부
    price_condition_met: bool,
}

impl IcebergStrategy {
    /// 새 Iceberg 전략 생성
    pub fn new(
        symbol: impl Into<String>,
        side: OrderSide,
        total_quantity: f64,
        limit_price: f64,
        display_quantity: f64,
    ) -> Self {
        let symbol_str = symbol.into();
        let display_quantity = display_quantity.min(total_quantity);
        
        IcebergStrategy {
            name: format!("Iceberg-{}", symbol_str),
            description: "Hidden large order execution strategy".to_string(),
            symbol: symbol_str,
            side,
            total_quantity,
            limit_price,
            display_quantity,
            executed_quantity: 0.0,
            is_active: true,
            current_market_data: None,
            price_condition_met: false,
        }
    }
    
    /// 가격 조건 충족 여부 확인
    fn check_price_condition(&self) -> bool {
        if let Some(data) = &self.current_market_data {
            match self.side {
                OrderSide::Buy => data.close <= self.limit_price,
                OrderSide::Sell => data.close >= self.limit_price,
            }
        } else {
            false
        }
    }
}

impl Strategy for IcebergStrategy {
    fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
        if market_data.symbol != self.symbol {
            return Ok(());
        }
        
        // 시장 데이터 업데이트
        self.current_market_data = Some(market_data);
        
        // 가격 조건 확인
        self.price_condition_met = self.check_price_condition();
        
        Ok(())
    }
    
    fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
        if !self.is_active || self.executed_quantity >= self.total_quantity {
            return Ok(Vec::new());
        }
        
        // 가격 조건이 충족되지 않으면 주문 생성하지 않음
        if !self.price_condition_met {
            return Ok(Vec::new());
        }
        
        // 남은 수량 계산
        let remaining = self.total_quantity - self.executed_quantity;
        
        if remaining <= 0.0 {
            self.is_active = false;
            return Ok(Vec::new());
        }
        
        // 다음 주문 크기 계산
        let next_quantity = self.display_quantity.min(remaining);
        
        // 지정가 주문 생성
        let order = Order::new(
            self.symbol.clone(),
            self.side.clone(),
            OrderType::Limit,
            next_quantity,
            self.limit_price,
        ).with_iceberg_qty(next_quantity);
        
        // 주문 추적 업데이트
        self.executed_quantity += next_quantity;
        
        // 목표량 도달 시 비활성화
        if self.executed_quantity >= self.total_quantity {
            self.is_active = false;
        }
        
        Ok(vec![order])
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
    fn test_iceberg_strategy() {
        // Iceberg 전략 생성
        let mut strategy = IcebergStrategy::new(
            "BTCUSDT",
            OrderSide::Buy,
            10.0,
            50000.0,
            1.0,
        );
        
        // 가격 조건 충족하는 시장 데이터 추가
        let market_data = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 1000,
            open: 50100.0,
            high: 50200.0,
            low: 49900.0,
            close: 49900.0, // 매수 조건 충족 (현재가 < 지정가)
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
        assert_eq!(first_order.order_type, OrderType::Limit);
        assert_eq!(first_order.quantity, 1.0);
        assert_eq!(first_order.price, 50000.0);
        assert_eq!(first_order.iceberg_qty, Some(1.0));
        
        // 가격 조건 불충족 시 주문 생성 안함 확인
        let market_data2 = MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: 2000,
            open: 50100.0,
            high: 50200.0,
            low: 50100.0,
            close: 50100.0, // 매수 조건 불충족 (현재가 > 지정가)
            volume: 15.0,
        };
        
        strategy.update(market_data2).unwrap();
        let orders2 = strategy.get_orders().unwrap();
        assert!(orders2.is_empty());
    }
}