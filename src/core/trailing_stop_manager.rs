/**
* filename : trailing_stop_manager
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

/// Trailing Stop 주문 관리자
pub struct TrailingStopManager {
  /// 거래소 인스턴스
  exchange: Arc<RwLock<dyn Exchange>>,
  /// 거래 심볼
  symbol: String,
  /// 주문 방향 (매수/매도)
  side: OrderSide,
  /// 주문 수량
  quantity: f64,
  /// 활성화 가격 (선택사항)
  activation_price: Option<f64>,
  /// 트레일링 간격 (백분율)
  trailing_delta: f64,
  /// 실행 여부
  executed: bool,
  /// 활성 여부
  is_active: bool,
  /// 최고 가격 (매수 추적용)
  highest_price: f64,
  /// 최저 가격 (매도 추적용)
  lowest_price: f64,
  /// 스탑 주문 ID
  stop_order_id: Option<OrderId>,
}

impl TrailingStopManager {
  /// 새 Trailing Stop 관리자 생성
  pub fn new(
    exchange: Arc<RwLock<dyn Exchange>>,
    symbol: impl Into<String>,
    side: OrderSide,
    quantity: f64,
    trailing_delta: f64,
    activation_price: Option<f64>,
  ) -> Self {
    TrailingStopManager {
      exchange,
      symbol: symbol.into(),
      side,
      quantity,
      activation_price,
      trailing_delta,
      executed: false,
      is_active: false,
      highest_price: 0.0,
      lowest_price: f64::MAX,
      stop_order_id: None,
    }
  }
  
  /// Trailing Stop 모니터링 시작
  pub async fn start(&mut self) -> Result<(), TradingError> {
    if self.is_active {
      return Err(TradingError::AlreadyRunning("Trailing stop already running".to_string()));
    }
    
    self.is_active = true;
    self.executed = false;
    
    // 초기 시장 가격 가져오기
    let initial_price = {
      let exchange = self.exchange.read().await;
      let market_data = exchange.get_market_data(&self.symbol).await?;
      market_data.close
    };
    
    // 가격 추적기 초기화
    self.highest_price = initial_price;
    self.lowest_price = initial_price;
    
    // 모니터링 태스크 생성
    let exchange_clone = self.exchange.clone();
    let symbol_clone = self.symbol.clone();
    let side_clone = self.side.clone();
    let quantity = self.quantity;
    let trailing_delta = self.trailing_delta;
    let activation_price = self.activation_price;
    
    // 약한 참조 사용하여 메모리 누수 방지
    let is_active = Arc::new(RwLock::new(true));
    let is_active_clone = is_active.clone();
    let highest_price = Arc::new(RwLock::new(initial_price));
    let lowest_price = Arc::new(RwLock::new(initial_price));
    let executed = Arc::new(RwLock::new(false));
    let stop_order_id = Arc::new(RwLock::new(None));
    
    tokio::spawn(async move {
      let mut interval_timer = interval(Duration::from_millis(500)); // 0.5초마다 확인
      let mut activated = activation_price.is_none();  // 활성화 가격 없으면 즉시 활성화
      
      while *is_active_clone.read().await {
        interval_timer.tick().await;
        
        // 현재 가격 가져오기
        let exchange = exchange_clone.read().await;
        let market_data_result = exchange.get_market_data(&symbol_clone).await;
        
        if let Ok(market_data) = market_data_result {
          let current_price = market_data.close;
          
          // 활성화 조건 확인 (아직 활성화되지 않은 경우)
          if !activated {
            if let Some(activation_price) = activation_price {
              if (side_clone == OrderSide::Buy && current_price <= activation_price) ||
                (side_clone == OrderSide::Sell && current_price >= activation_price) {
                activated = true;
              } else {
                continue;  // 아직 활성화되지 않음
              }
            }
          }
          
          // 가격 추적기 업데이트
          if current_price > *highest_price.read().await {
            *highest_price.write().await = current_price;
          }
          
          if current_price < *lowest_price.read().await {
            *lowest_price.write().await = current_price;
          }
          
          // 트레일링 델타 기반으로 스탑 가격 계산
          let stop_price = match side_clone {
            OrderSide::Buy => {
              // 매수 트레일링 스탑: 최고가에서 델타% 하락 시 트리거
              let delta_amount = *highest_price.read().await * (trailing_delta / 100.0);
              *highest_price.read().await - delta_amount
            },
            OrderSide::Sell => {
              // 매도 트레일링 스탑: 최저가에서 델타% 상승 시 트리거
              let delta_amount = *lowest_price.read().await * (trailing_delta / 100.0);
              *lowest_price.read().await + delta_amount
            }
          };
          
          // 스탑 조건 충족 여부 확인
          let stop_triggered = match side_clone {
            OrderSide::Buy => current_price <= stop_price,
            OrderSide::Sell => current_price >= stop_price,
          };
          
          if stop_triggered && !*executed.read().await {
            // 스탑 주문 실행
            let mut exchange = exchange_clone.write().await;
            
            let order = Order::new(
              symbol_clone.clone(),
              side_clone.clone(),
              OrderType::Market,  // 즉시 실행을 위한 시장가 주문
              quantity,
              current_price,
            );
            
            let order_result = exchange.submit_order(order).await;
            
            if let Ok(order_id) = order_result {
              *stop_order_id.write().await = Some(order_id);
              *executed.write().await = true;
              *is_active_clone.write().await = false;  // 모니터링 중지
              break;
            }
          }
        }
      }
    });
    
    Ok(())
  }
  
  /// Trailing Stop 모니터링 중지
  pub async fn stop(&mut self) -> Result<(), TradingError> {
    if !self.is_active {
      return Ok(());
    }
    
    self.is_active = false;
    
    // 스탑 주문 있고 아직 실행되지 않았으면 취소
    if let Some(order_id) = self.stop_order_id.as_ref() {
      if !self.executed {
        let mut exchange = self.exchange.write().await;
        let _ = exchange.cancel_order(order_id).await;
      }
    }
    
    Ok(())
  }
  
  /// Trailing Stop 현재 상태 조회
  pub fn status(&self) -> (bool, bool, f64, f64) {
    let current_trigger_price = match self.side {
      OrderSide::Buy => {
        let delta_amount = self.highest_price * (self.trailing_delta / 100.0);
        self.highest_price - delta_amount
      },
      OrderSide::Sell => {
        let delta_amount = self.lowest_price * (self.trailing_delta / 100.0);
        self.lowest_price + delta_amount
      }
    };
    
    (self.is_active, self.executed, current_trigger_price, self.quantity)
  }
  
  /// Trailing Delta 백분율 업데이트
  pub async fn update_delta(&mut self, new_delta: f64) -> Result<(), TradingError> {
    if new_delta <= 0.0 {
      return Err(TradingError::InvalidParameter("Trailing delta must be positive".to_string()));
    }
    
    self.trailing_delta = new_delta;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::exchange::mocks::MockExchange;
  use crate::config::Config;
  
  #[tokio::test]
  async fn test_trailing_stop() {
    // 테스트 환경 설정
    let config = Config::default();
    let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
    
    // Trailing Stop 관리자 생성
    let mut trailing_stop = TrailingStopManager::new(
      exchange.clone(),
      "BTCUSDT",
      OrderSide::Sell,
      0.1,
      2.0,  // 2% 트레일링 델타
      None, // 활성화 가격 없음 (즉시 활성화)
    );
    
    // 실행 시작
    let result = trailing_stop.start().await;
    assert!(result.is_ok());
    
    // 상태 확인
    let (is_active, executed, trigger_price, quantity) = trailing_stop.status();
    assert!(is_active);
    assert!(!executed);
    assert!(trigger_price > 0.0);
    assert_eq!(quantity, 0.1);
    
    // 중지
    let stop_result = trailing_stop.stop().await;
    assert!(stop_result.is_ok());
    assert!(!trailing_stop.is_active);
  }
}