//! 리스크 관리 모듈
//!
//! 포지션 크기, 손실 한도 등 리스크 관리 기능 구현

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::order::{Order, OrderSide};
use crate::models::position::Position;

/// 리스크 관리자
pub struct RiskManager {
    /// 거래소 인스턴스
    exchange: Arc<RwLock<dyn Exchange>>,
    /// 심볼별 최대 포지션 크기
    max_position_size: HashMap<String, f64>,
    /// 최대 낙폭 비율 (%)
    max_drawdown_percent: f64,
    /// 일일 최대 손실액
    max_daily_loss: f64,
    /// 당일 손실액
    daily_loss: f64,
    /// 현재 포지션
    positions: HashMap<String, Position>,
}

impl RiskManager {
    /// 새 리스크 관리자 생성
    pub fn new(
        exchange: Arc<RwLock<dyn Exchange>>,
        max_drawdown_percent: f64,
        max_daily_loss: f64,
    ) -> Self {
        RiskManager {
            exchange,
            max_position_size: HashMap::new(),
            max_drawdown_percent,
            max_daily_loss,
            daily_loss: 0.0,
            positions: HashMap::new(),
        }
    }
    
    /// 특정 심볼의 최대 포지션 크기 설정
    pub fn set_max_position_size(&mut self, symbol: impl Into<String>, size: f64) {
        self.max_position_size.insert(symbol.into(), size);
    }
    
    /// 주문이 리스크 관리 규칙을 통과하는지 확인
    pub async fn check_order(&mut self, order: &Order) -> Result<bool, TradingError> {
        // 포지션 업데이트
        self.update_positions().await?;
        
        // 포지션 크기 제한 확인
        if let Some(max_size) = self.max_position_size.get(&order.symbol) {
            let current_size = self.get_position_size(&order.symbol);
            let order_effect = match order.side {
                OrderSide::Buy => order.quantity,
                OrderSide::Sell => -order.quantity,
            };
            
            let new_size = current_size + order_effect;
            
            if new_size.abs() > *max_size {
                return Ok(false);
            }
        }
        
        // 낙폭 및 일일 손실 한도 확인
        if self.daily_loss >= self.max_daily_loss {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// 거래소 데이터로 포지션 업데이트
    pub async fn update_positions(&mut self) -> Result<(), TradingError> {
        let exchange = self.exchange.read().await;
        let mut positions = HashMap::new();
        
        // 미체결 주문 가져오기
        let open_orders = exchange.get_open_orders().await?;
        
        // 심볼별로 그룹화
        for order in open_orders {
            let position = positions.entry(order.symbol.clone()).or_insert_with(|| {
                Position {
                    symbol: order.symbol.clone(),
                    quantity: 0.0,
                    entry_price: 0.0,
                    current_price: 0.0,
                    unrealized_pnl: 0.0,
                }
            });
            
            // 포지션 크기 업데이트
            match order.side {
                OrderSide::Buy => position.quantity += order.quantity,
                OrderSide::Sell => position.quantity -= order.quantity,
            }
        }
        
        // 각 포지션의 현재 가격 및 미실현 손익 업데이트
        for (symbol, position) in positions.iter_mut() {
            let market_data = exchange.get_market_data(symbol).await?;
            position.current_price = market_data.close;
            
            // 미실현 손익 계산
            if position.quantity != 0.0 && position.entry_price != 0.0 {
                let direction = position.quantity.signum();
                position.unrealized_pnl = direction * (position.current_price - position.entry_price).abs() * position.quantity.abs();
            }
        }
        
        self.positions = positions;
        Ok(())
    }
    
    /// 심볼별 포지션 크기 조회
    pub fn get_position_size(&self, symbol: &str) -> f64 {
        self.positions.get(symbol).map_or(0.0, |p| p.quantity)
    }
    
    /// 심볼별 미실현 손익 조회
    pub fn get_unrealized_pnl(&self, symbol: &str) -> f64 {
        self.positions.get(symbol).map_or(0.0, |p| p.unrealized_pnl)
    }
    
    /// 실현 손익 기록
    pub fn record_pnl(&mut self, amount: f64) {
        if amount < 0.0 {
            self.daily_loss += amount.abs();
        }
    }
    
    /// 일일 손실 카운터 초기화
    pub fn reset_daily_loss(&mut self) {
        self.daily_loss = 0.0;
    }
    
    /// 모든 현재 포지션 조회
    pub fn get_positions(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchange::mocks::MockExchange;
    use crate::config::Config;
    use crate::models::order::OrderType;
    
    #[tokio::test]
    async fn test_risk_manager() {
        // 테스트 환경 설정
        let config = Config::default();
        let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
        
        // 리스크 관리자 생성
        let mut risk_manager = RiskManager::new(
            exchange.clone(),
            5.0,   // 5% 최대 낙폭
            1000.0, // $1000 일일 최대 손실
        );
        
        // 최대 포지션 크기 설정
        risk_manager.set_max_position_size("BTCUSDT", 1.0);
        
        // 포지션 업데이트
        let update_result = risk_manager.update_positions().await;
        assert!(update_result.is_ok());
        
        // 주문 검사
        let order = Order::new(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Market,
            0.5,
            50000.0,
        );
        
        let check_result = risk_manager.check_order(&order).await;
        assert!(check_result.is_ok());
        
        // 대용량 주문 검사 (거부되어야 함)
        let large_order = Order::new(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Market,
            2.0,
            50000.0,
        );
        
        let large_check_result = risk_manager.check_order(&large_order).await;
        assert!(large_check_result.is_ok());
        assert!(!large_check_result.unwrap());
    }
}