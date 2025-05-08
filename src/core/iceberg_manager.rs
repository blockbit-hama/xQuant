/**
* filename : iceberg_manager
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tokio::time::{interval, Duration};

use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};

/// Iceberg 주문 관리자
pub struct IcebergManager {
  /// 거래소 인스턴스
  exchange: Arc<RwLock<dyn Exchange>>,
  /// 거래 심볼
  symbol: String,
  /// 주문 방향 (매수/매도)
  side: OrderSide,
  /// 총 주문 수량
  total_quantity: f64,
  /// 지정가 가격
  limit_price: f64,
  /// 노출할 부분 수량
  display_quantity: f64,
  /// 이미 실행한 수량
  executed_quantity: f64,
  /// 실행 중 여부
  is_active: bool,
  /// 현재 활성 주문 ID
  current_order_id: Option<OrderId>,
  /// 실행 동기화를 위한 뮤텍스
  execution_lock: Arc<Mutex<()>>,
}

impl IcebergManager {
  /// 새 Iceberg 관리자 생성
  pub fn new(
    exchange: Arc<RwLock<dyn Exchange>>,
    symbol: impl Into<String>,
    side: OrderSide,
    total_quantity: f64,
    limit_price: f64,
    display_quantity: f64,
  ) -> Self {
    let display_quantity = display_quantity.min(total_quantity);
    
    IcebergManager {
      exchange,
      symbol: symbol.into(),
      side,
      total_quantity,
      limit_price,
      display_quantity,
      executed_quantity: 0.0,
      is_active: false,
      current_order_id: None,
      execution_lock: Arc::new(Mutex::new(()))
    }
  }
  
  /// Iceberg 주문 실행 시작
  pub async fn start(&mut self) -> Result<(), TradingError> {
    if self.is_active {
      return Err(TradingError::AlreadyRunning("Iceberg execution already running".to_string()));
    }
    
    self.is_active = true;
    self.executed_quantity = 0.0;
    self.current_order_id = None;
    
    // 첫 번째 노출 부분 제출
    self.submit_visible_portion().await?;
    
    // 모니터링 태스크 시작
    let exchange_clone = self.exchange.clone();
    let symbol_clone = self.symbol.clone();
    let side_clone = self.side.clone();
    let total_quantity = self.total_quantity;
    let limit_price = self.limit_price;
    let display_quantity = self.display_quantity;
    let execution_lock = self.execution_lock.clone();
    
    // 자신에 대한 약한 참조 사용하여 메모리 누수 방지
    let is_active = Arc::new(RwLock::new(true));
    let is_active_clone = is_active.clone();
    let executed_quantity = Arc::new(RwLock::new(0.0));
    let executed_quantity_clone = executed_quantity.clone();
    let current_order_id = Arc::new(RwLock::new(None));
    let current_order_id_clone = current_order_id.clone();
    
    tokio::spawn(async move {
      let mut interval_timer = interval(Duration::from_millis(1000)); // 1초마다 확인
      
      while *is_active_clone.read().await {
        interval_timer.tick().await;
        
        // 한 번에 하나의 실행 프로세스만 실행하도록 락 사용
        let _lock = execution_lock.lock().await;
        
        // 현재 주문 상태 확인
        if let Some(order_id) = current_order_id_clone.read().await.clone() {
          let exchange = exchange_clone.read().await;
          let status = exchange.get_order_status(&order_id).await;
          
          if let Ok(status) = status {
            match status {
              OrderStatus::Filled => {
                // 주문 체결됨, 실행 수량 업데이트
                *executed_quantity_clone.write().await += display_quantity;
                
                // 완료 여부 확인
                if *executed_quantity_clone.read().await >= total_quantity {
                  *is_active_clone.write().await = false;
                  break;
                }
                
                // 다음 노출 부분 제출
                drop(exchange); // 락 해제
                
                let remaining = total_quantity - *executed_quantity_clone.read().await;
                if remaining > 0.0 {
                  let next_quantity = display_quantity.min(remaining);
                  
                  let mut exchange = exchange_clone.write().await;
                  let order = Order::new(
                    symbol_clone.clone(),
                    side_clone.clone(),
                    OrderType::Limit,
                    next_quantity,
                    limit_price,
                  );
                  
                  if let Ok(new_order_id) = exchange.submit_order(order).await {
                    *current_order_id_clone.write().await = Some(new_order_id);
                  }
                }
              },
              OrderStatus::Cancelled | OrderStatus::Rejected | OrderStatus::Expired => {
                // 주문 실패, 재시도
                drop(exchange); // 락 해제
                
                let remaining = total_quantity - *executed_quantity_clone.read().await;
                if remaining > 0.0 {
                  let next_quantity = display_quantity.min(remaining);
                  
                  let mut exchange = exchange_clone.write().await;
                  let order = Order::new(
                    symbol_clone.clone(),
                    side_clone.clone(),
                    OrderType::Limit,
                    next_quantity,
                    limit_price,
                  );
                  
                  if let Ok(new_order_id) = exchange.submit_order(order).await {
                    *current_order_id_clone.write().await = Some(new_order_id);
                  }
                }
              },
              _ => {
                // 주문 여전히 활성 상태, 계속 모니터링
              }
            }
          }
        }
      }
    });
    
    Ok(())
  }
  
  /// Iceberg 주문 실행 중지 및 활성 주문 취소
  pub async fn stop(&mut self) -> Result<(), TradingError> {
    if !self.is_active {
      return Ok(());
    }
    
    self.is_active = false;
    
    // 현재 주문 있으면 취소
    if let Some(order_id) = self.current_order_id.as_ref() {
      let mut exchange = self.exchange.write().await;
      let _ = exchange.cancel_order(order_id).await;
    }
    
    Ok(())
  }
  
  /// Iceberg 실행 상태 조회
  pub fn status(&self) -> (bool, f64, f64) {
    (self.is_active, self.executed_quantity, self.total_quantity)
  }
  
  /// 노출 부분 주문 제출
  async fn submit_visible_portion(&mut self) -> Result<(), TradingError> {
    let remaining_quantity = self.total_quantity - self.executed_quantity;
    
    if remaining_quantity <= 0.0 {
      self.is_active = false;
      return Ok(());
    }
    
    // 다음 노출 수량 계산
    let next_quantity = self.display_quantity.min(remaining_quantity);
    
    // 지정가 주문 생성 및 제출
    let order = Order::new(
      self.symbol.clone(),
      self.side.clone(),
      OrderType::Limit,
      next_quantity,
      self.limit_price,
    );
    
    let mut exchange = self.exchange.write().await;
    let order_id = exchange.submit_order(order).await?;
    self.current_order_id = Some(order_id);
    
    Ok(())
  }
  
  /// 향후 주문 제출을 위한 지정가 업데이트
  pub async fn update_price(&mut self, new_price: f64) -> Result<(), TradingError> {
    self.limit_price = new_price;
    
    // 활성 상태면 현재 주문 취소 후 새 가격으로 재제출
    if self.is_active {
      if let Some(order_id) = self.current_order_id.as_ref() {
        let mut exchange = self.exchange.write().await;
        let _ = exchange.cancel_order(order_id).await;
      }
      
      self.submit_visible_portion().await?;
    }
    
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::exchange::mocks::MockExchange;
  use crate::config::Config;
  
  #[tokio::test]
  async fn test_iceberg_manager() {
    // 테스트 환경 설정
    let config = Config::default();
    let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
    
    // Iceberg 관리자 생성
    let mut iceberg = IcebergManager::new(
      exchange.clone(),
      "BTCUSDT",
      OrderSide::Buy,
      10.0,
      50000.0,
      1.0,
    );
    
    // 실행 시작
    let result = iceberg.start().await;
    assert!(result.is_ok());
    
    // 잠시 대기
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 주문이 제출되었는지 확인
    assert!(iceberg.current_order_id.is_some());
    
    // 중지
    let stop_result = iceberg.stop().await;
    assert!(stop_result.is_ok());
    assert!(!iceberg.is_active);
  }
}