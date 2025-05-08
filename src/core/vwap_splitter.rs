//! VWAP 기반 주문 분할 알고리즘 구현
//!
//! 대량 주문을 시장 거래량에 비례하여 여러 작은 주문으로 분할하는 알고리즘

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};

/// VWAP 기반 주문 분할기
pub struct VwapSplitter {
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
  /// 목표 거래량 비율 (%)
  target_percentage: Option<f64>,
  /// 이미 실행한 수량
  executed_quantity: f64,
  /// 실행 중 여부
  is_active: bool,
  /// 생성된 하위 주문 ID 목록
  child_orders: Vec<OrderId>,
}

impl VwapSplitter {
  /// 새 VWAP 분할기 생성
  pub fn new(
    exchange: Arc<RwLock<dyn Exchange>>,
    symbol: impl Into<String>,
    side: OrderSide,
    total_quantity: f64,
    execution_interval: i64,
    target_percentage: Option<f64>,
  ) -> Self {
    VwapSplitter {
      exchange,
      symbol: symbol.into(),
      side,
      total_quantity,
      execution_interval,
      target_percentage,
      executed_quantity: 0.0,
      is_active: false,
      child_orders: Vec::new(),
    }
  }
  
  /// VWAP 실행 알고리즘 시작
  pub async fn start(&mut self) -> Result<(), TradingError> {
    if self.is_active {
      return Err(TradingError::AlreadyRunning("VWAP execution already running".to_string()));
    }
    
    self.is_active = true;
    self.executed_quantity = 0.0;
    self.child_orders.clear();
    
    // 시간 구간 계산
    let now = chrono::Utc::now().timestamp_millis();
    let end_time = now + self.execution_interval;
    let remaining_time = end_time - now;
    
    // 과거 VWAP 데이터 가져오기
    let historical_data = self.get_historical_vwap_data().await?;
    
    // 거래량 프로필 계산
    let volume_profile = self.calculate_volume_profile(historical_data);
    
    // 분할 수와 각 분할별 수량 계산
    let num_slices = 10; // 요구사항에 따라 조정 가능
    let mut remaining_quantity = self.total_quantity;
    
    // 분할 실행을 위한 타이머 생성
    let mut interval_timer = interval(Duration::from_millis(
      (remaining_time / num_slices as i64) as u64
    ));
    
    for i in 0..num_slices {
      interval_timer.tick().await;
      
      if !self.is_active {
        break;
      }
      
      // 거래량 프로필 기반으로 분할 수량 계산
      let volume_ratio = volume_profile[i];
      let slice_quantity = self.total_quantity * volume_ratio;
      
      // 남은 수량 기준으로 조정
      let adjusted_quantity = if i == num_slices - 1 {
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
            log::error!("Failed to create VWAP child order: {}", e);
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
  
  /// VWAP 실행 중지 및 모든 활성 주문 취소
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
  
  /// VWAP 실행 상태 조회
  pub fn status(&self) -> (bool, f64, f64) {
    (self.is_active, self.executed_quantity, self.total_quantity)
  }
  
  /// 과거 VWAP 데이터 가져오기
  async fn get_historical_vwap_data(&self) -> Result<Vec<MarketData>, TradingError> {
    let exchange = self.exchange.read().await;
    
    // 전날 같은 시간대의 과거 데이터 가져오기
    let now = chrono::Utc::now();
    let one_day_ago = now - chrono::Duration::days(1);
    
    let start_time = one_day_ago.timestamp_millis();
    let end_time = start_time + self.execution_interval;
    
    exchange.get_historical_data(
      &self.symbol,
      "1m",  // 1분 간격
      start_time,
      Some(end_time),
      None,
    ).await
  }
  
  /// 거래량 프로필 계산
  fn calculate_volume_profile(&self, data: Vec<MarketData>) -> Vec<f64> {
    // 데이터가 없으면 균등 분배
    if data.is_empty() {
      let num_slices = 10;
      return vec![1.0 / num_slices as f64; num_slices];
    }
    
    // 총 거래량 계산
    let total_volume: f64 = data.iter().map(|d| d.volume).sum();
    
    if total_volume == 0.0 {
      let num_slices = 10;
      return vec![1.0 / num_slices as f64; num_slices];
    }
    
    // 10개 시간대로 데이터 그룹화하여 거래량 비율 계산
    let period_duration = data.len() / 10;
    let mut volume_profile = Vec::with_capacity(10);
    
    for i in 0..10 {
      let start_idx = i * period_duration;
      let end_idx = ((i + 1) * period_duration).min(data.len());
      
      let period_volume: f64 = data[start_idx..end_idx].iter().map(|d| d.volume).sum();
      volume_profile.push(period_volume / total_volume);
    }
    
    volume_profile
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
  async fn test_vwap_splitter() {
    // 테스트 환경 설정
    let config = Config::default();
    let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
    
    // VWAP 분할기 생성
    let mut vwap = VwapSplitter::new(
      exchange.clone(),
      "BTCUSDT",
      OrderSide::Buy,
      1.0,
      3600000,  // 1시간 (밀리초)
      Some(10.0),
    );
    
    // 실행 시작
    let result = vwap.start().await;
    assert!(result.is_ok());
    
    // 상태 확인
    let (is_active, executed, total) = vwap.status();
    assert!(!is_active); // 테스트에서는 즉시 완료됨
    assert!(executed > 0.0);
    assert_eq!(total, 1.0);
  }
}