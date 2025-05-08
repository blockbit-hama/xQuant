/**
* filename : twap_splitter
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};

/// TWAP 기반 주문 분할기
pub struct TwapSplitter {
  /// 거래소 인스턴스
  exchange: Arc<RwLock<dyn Exchange>>,
  /// 거래 심볼
  symbol: String,
  /// 주문 방향 (매수/매도)
  side: OrderSide,
  /// 총 주문 수량
  total_quantity: f64,
  /// 실행 간격 (밀리초)
  execution_interval: i64,
  /// 분할 수
  num_slices: usize,
  /// 이미 실행한 수량
  executed_quantity: f64,
  /// 실행 중 여부
  is_active: bool,
  /// 생성된 하위 주문 ID 목록
  child_orders: Vec<OrderId>,
}

impl TwapSplitter {
  /// 새 TWAP 분할기 생성
  pub fn new(
    exchange: Arc<RwLock<dyn Exchange>>,
    symbol: impl Into<String>,
    side: OrderSide,
    total_quantity: f64,
    execution_interval: i64,
    num_slices: usize,
  ) -> Self {
    TwapSplitter {
      exchange,
      symbol: symbol.into(),
      side,
      total_quantity,
      execution_interval,
      num_slices,
      executed_quantity: 0.0,
      is_active: false,
      child_orders: Vec::new(),
    }
  }
  
  /// TWAP 실행 알고리즘 시작
  pub async fn start(&mut self) -> Result<(), TradingError> {
    if self.is_active {
      return Err(TradingError::AlreadyRunning("TWAP execution already running".to_string()));
    }
    
    self.is_active = true;
    self.executed_quantity = 0.0;
    self.child_orders.clear();
    
    // 분할 크기 계산
    let slice_quantity = self.total_quantity / self.num_slices as f64;
    let time_between_slices = self.execution_interval / self.num_slices as i64;
    
    // 분할 실행을 위한 타이머 생성
    let mut interval_timer = interval(Duration::from_millis(time_between_slices as u64));
    
    let mut remaining_quantity = self.total_quantity;
    
    for i in 0..self.num_slices {
      interval_timer.tick().await;
      
      if !self.is_active {
        break;
      }
      
      // 현재 분할 수량 계산 (마지막 분할에서 반올림 오차 처리)
      let adjusted_quantity = if i == self.num_slices - 1 {
        remaining_quantity
      } else {
        slice_quantity.min(remaining_quantity)
      };
      
      if adjusted_quantity > 0.0 {
        // 하위 주문 생성 및 제출
        let order_result = self.create_child_order(adjusted_quantity).await;
        
        match order_result {
          Ok(order_id) => {
            self.child_orders.push(order_id);
            remaining_quantity -= adjusted_quantity;
            self.executed_quantity += adjusted_quantity;
          },
          Err(e) => {
            log::error!("Failed to create TWAP child order: {}", e);
          }
        }
      }
      
      // 총 수량 완료 여부 확인
      if remaining_quantity <= 0.0 {
        break;
      }
    }
    
    self.is_active = false;
    Ok(())
  }
  
  /// TWAP 실행 중지 및 모든 활성 주문 취소
  pub async fn stop(&mut self) -> Result<(), TradingError> {
    if !self.is_active {
      return Ok(());
    }
    
    self.is_active = false;
    
    // 모든 활성 하위 주문 취소
    for order_id in &self.child_orders {
      let mut exchange = self.exchange.write().await;
      let status = exchange.get_order_status(order_id).await?;
      
      if status == OrderStatus::New || status == OrderStatus::PartiallyFilled {
        let _ = exchange.cancel_order(order_id).await;
      }
    }
    
    Ok(())
  }
  
  /// TWAP 실행 상태 조회
  pub fn status(&self) -> (bool, f64, f64) {
    (self.is_active, self.executed_quantity, self.total_quantity)
  }
  
  /// 하위 주문 생성
  async fn create_child_order(&self, quantity: f64) -> Result<OrderId, TradingError> {
    let mut exchange = self.exchange.write().await;
    
    // 현재 시장 데이터로 가격 조회
    let market_data = exchange.get_market_data(&self.symbol).await?;
    
    // 시장가 주문 생성
    let order = Order::new(
      self.symbol.clone(),
      self.side.clone(),
      OrderType::Market,
      quantity,
      market_data.close,
    );
    
    exchange.submit_order(order).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::exchange::mocks::MockExchange;
  use crate::config::Config;
  
  #[tokio::test]
  async fn test_twap_splitter() {
    // 테스트 환경 설정
    let config = Config::default();
    let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
    
    // TWAP 분할기 생성
    let mut twap = TwapSplitter::new(
      exchange.clone(),
      "BTCUSDT",
      OrderSide::Buy,
      1.0,
      3600000,  // 1시간 (밀리초)
      5,        // 5개 분할
    );
    
    // 실행 시작
    let result = twap.start().await;
    assert!(result.is_ok());
    
    // 상태 확인
    let (is_active, executed, total) = twap.status();
    assert!(!is_active); // 테스트에서는 즉시 완료됨
    assert!(executed > 0.0);
    assert_eq!(total, 1.0);
    
    // 하위 주문 수 확인
    assert!(twap.child_orders.len() <= 5);
  }
}