//! Trailing Stop 전략
//!
//! 추세 추종과 자동 손절매를 위한 전략

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide, OrderType};
use crate::strategies::Strategy;

/// Trailing Stop 매매 전략
pub struct TrailingStopStrategy {
  /// 전략 이름
  name: String,
  /// 전략 설명
  description: String,
  /// 거래 심볼
  symbol: String,
  /// 매매 방향
  side: OrderSide,
  /// 주문 수량
  quantity: f64,
  /// 포지션 진입 가격
  entry_price: Option<f64>,
  /// 트레일링 간격 (백분율)
  trailing_delta: f64,
  /// 활성화 가격 (선택사항)
  activation_price: Option<f64>,
  /// 최고 가격 (매수 추적용)
  highest_price: f64,
  /// 최저 가격 (매도 추적용)
  lowest_price: f64,
  /// 현재 시장 데이터
  current_market_data: Option<MarketData>,
  /// 트레일링 스탑 활성화 여부
  activated: bool,
  /// 전략 활성 여부
  is_active: bool,
  /// 주문 실행 여부
  executed: bool,
}

impl TrailingStopStrategy {
  /// 새 Trailing Stop 전략 생성
  pub fn new(
    symbol: impl Into<String>,
    side: OrderSide,
    quantity: f64,
    trailing_delta: f64,
    activation_price: Option<f64>,
  ) -> Self {
    let symbol_str = symbol.into();
    
    TrailingStopStrategy {
      name: format!("TrailingStop-{}", symbol_str),
      description: "Price trailing based stop order strategy".to_string(),
      symbol: symbol_str,
      side,
      quantity,
      entry_price: None,
      trailing_delta,
      activation_price,
      highest_price: 0.0,
      lowest_price: f64::MAX,
      current_market_data: None,
      activated: activation_price.is_none(),
      is_active: true,
      executed: false,
    }
  }
  
  /// 진입 가격 설정
  pub fn set_entry_price(&mut self, price: f64) {
    self.entry_price = Some(price);
    self.highest_price = price;
    self.lowest_price = price;
  }
  
  /// 트레일링 스탑 가격 계산
  fn calculate_stop_price(&self) -> Option<f64> {
    if let Some(market_data) = &self.current_market_data {
      match self.side {
        OrderSide::Buy => {
          // 매수 트레일링 스탑: 최고가에서 델타% 하락 시 트리거
          let delta_amount = self.highest_price * (self.trailing_delta / 100.0);
          Some(self.highest_price - delta_amount)
        },
        OrderSide::Sell => {
          // 매도 트레일링 스탑: 최저가에서 델타% 상승 시 트리거
          let delta_amount = self.lowest_price * (self.trailing_delta / 100.0);
          Some(self.lowest_price + delta_amount)
        }
      }
    } else {
      None
    }
  }
  
  /// 스탑 조건 충족 여부 확인
  fn is_stop_triggered(&self) -> bool {
    if let Some(market_data) = &self.current_market_data {
      if let Some(stop_price) = self.calculate_stop_price() {
        match self.side {
          OrderSide::Buy => market_data.close <= stop_price,
          OrderSide::Sell => market_data.close >= stop_price,
        }
      } else {
        false
      }
    } else {
      false
    }
  }
  
  /// 활성화 조건 충족 여부 확인
  fn check_activation(&mut self) {
    if self.activated {
      return;
    }
    
    if let Some(activation_price) = self.activation_price {
      if let Some(market_data) = &self.current_market_data {
        match self.side {
          OrderSide::Buy => {
            if market_data.close <= activation_price {
              self.activated = true;
              self.highest_price = market_data.close;
              self.lowest_price = market_data.close;
            }
          },
          OrderSide::Sell => {
            if market_data.close >= activation_price {
              self.activated = true;
              self.highest_price = market_data.close;
              self.lowest_price = market_data.close;
            }
          }
        }
      }
    }
  }
}

impl Strategy for TrailingStopStrategy {
  fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
    if market_data.symbol != self.symbol {
      return Ok(());
    }
    
    // 시장 데이터 업데이트
    self.current_market_data = Some(market_data.clone());
    
    // 활성화 조건 확인
    self.check_activation();
    
    if self.activated {
      // 가격 추적기 업데이트
      let price = market_data.close;
      
      if price > self.highest_price {
        self.highest_price = price;
      }
      
      if price < self.lowest_price {
        self.lowest_price = price;
      }
    }
    
    Ok(())
  }
  
  fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
    if !self.is_active || self.executed || !self.activated {
      return Ok(Vec::new());
    }
    
    // 스탑 조건 확인
    if self.is_stop_triggered() {
      // 현재 가격 가져오기
      if let Some(market_data) = &self.current_market_data {
        // 시장가 주문 생성
        let order = Order::new(
          self.symbol.clone(),
          self.side.clone(),
          OrderType::Market,
          self.quantity,
          market_data.close,
        );
        
        // 주문 추적 업데이트
        self.executed = true;
        self.is_active = false;
        
        return Ok(vec![order]);
      }
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
  fn test_trailing_stop_strategy() {
    // Trailing Stop 전략 생성 (매도)
    let mut strategy = TrailingStopStrategy::new(
      "BTCUSDT",
      OrderSide::Sell,
      1.0,
      2.0,  // 2% 트레일링 델타
      None, // 활성화 가격 없음 (즉시 활성화)
    );
    
    // 진입 가격 설정
    strategy.set_entry_price(50000.0);
    
    // 가격 하락 시 스탑 가격 이동 확인
    let market_data1 = MarketData {
      symbol: "BTCUSDT".to_string(),
      timestamp: 1000,
      open: 50000.0,
      high: 50000.0,
      low: 49500.0,
      close: 49500.0,
      volume: 10.0,
    };
    
    strategy.update(market_data1).unwrap();
    
    // 이 시점에서는 스탑 조건 충족하지 않음
    let orders1 = strategy.get_orders().unwrap();
    assert!(orders1.is_empty());
    
    // 가격 급등 시 스탑 조건 충족 확인
    let market_data2 = MarketData {
      symbol: "BTCUSDT".to_string(),
      timestamp: 2000,
      open: 49500.0,
      high: 50500.0,
      low: 49500.0,
      close: 50500.0, // 가격 급등 (50500 > 49500 * 1.02)
      volume: 15.0,
    };
    
    strategy.update(market_data2).unwrap();
    
    // 스탑 조건 충족하여 주문 생성
    let orders2 = strategy.get_orders().unwrap();
    assert!(!orders2.is_empty());
    
    // 주문 내용 확인
    let order = &orders2[0];
    assert_eq!(order.symbol, "BTCUSDT");
    assert_eq!(order.side, OrderSide::Sell);
    assert_eq!(order.order_type, OrderType::Market);
    assert_eq!(order.quantity, 1.0);
  }
}