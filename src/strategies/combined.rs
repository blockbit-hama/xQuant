use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderSide, OrderType};
use crate::signals::signal_types::SignalType;
use crate::strategies::Strategy;
use super::technical::TechnicalStrategy;

// TA 기반 신호 생성 + 알고리즘 실행 최적화를 결합한 전략
pub struct CombinedStrategy {
  name: String,
  symbol: String,
  signal_strategy: Box<dyn Strategy>, // 신호 생성 전략
  execution_strategy: Box<dyn Strategy>, // 실행 최적화 전략
  is_active: bool,
}

impl CombinedStrategy {
  pub fn new(
    name: String,
    symbol: String,
    signal_strategy: Box<dyn Strategy>,
    execution_strategy: Box<dyn Strategy>,
  ) -> Self {
    CombinedStrategy {
      name,
      symbol,
      signal_strategy,
      execution_strategy,
      is_active: true,
    }
  }
  
  // 편의 생성자: RSI 신호 + TWAP 실행
  pub fn rsi_twap(
    symbol: String,
    rsi_period: usize,
    oversold: f64,
    overbought: f64,
    twap_duration_minutes: u64,
  ) -> Result<Self, TradingError> {
    // RSI 신호 전략
    let signal_strategy = TechnicalStrategy::rsi(
      symbol.clone(),
      rsi_period,
      oversold,
      overbought,
    )?;
    
    // TWAP 실행 전략 (가정: 기존 TWAP 전략 사용)
    let execution_strategy = crate::strategies::twap::TwapStrategy::new(
      symbol.clone(),
      twap_duration_minutes,
    )?;
    
    Ok(CombinedStrategy::new(
      format!("RSI-{} + TWAP-{}min", rsi_period, twap_duration_minutes),
      symbol,
      Box::new(signal_strategy),
      Box::new(execution_strategy),
    ))
  }
  
  // 편의 생성자: MACD 신호 + VWAP 실행
  pub fn macd_vwap(
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    vwap_participation_rate: f64,
  ) -> Result<Self, TradingError> {
    // MACD 신호 전략
    let signal_strategy = TechnicalStrategy::macd(
      symbol.clone(),
      fast_period,
      slow_period,
      signal_period,
    )?;
    
    // VWAP 실행 전략 (가정: 기존 VWAP 전략 사용)
    let execution_strategy = crate::strategies::vwap::VwapStrategy::new(
      symbol.clone(),
      vwap_participation_rate,
    )?;
    
    Ok(CombinedStrategy::new(
      format!("MACD-{}/{}/{} + VWAP-{}%", fast_period, slow_period, signal_period, vwap_participation_rate * 100.0),
      symbol,
      Box::new(signal_strategy),
      Box::new(execution_strategy),
    ))
  }
  
  // 편의 생성자: MA 크로스오버 신호 + Iceberg 실행
  pub fn ma_crossover_iceberg(
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    display_size: f64,
  ) -> Result<Self, TradingError> {
    // MA 크로스오버 신호 전략
    let signal_strategy = TechnicalStrategy::ma_crossover(
      symbol.clone(),
      fast_period,
      slow_period,
    )?;
    
    // Iceberg 실행 전략 (가정: 기존 Iceberg 전략 사용)
    let execution_strategy = crate::strategies::iceberg::IcebergStrategy::new(
      symbol.clone(),
      display_size,
    )?;
    
    Ok(CombinedStrategy::new(
      format!("MA-{}/{} + Iceberg-{}", fast_period, slow_period, display_size),
      symbol,
      Box::new(signal_strategy),
      Box::new(execution_strategy),
    ))
  }
}

impl Strategy for CombinedStrategy {
  fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
    if !self.is_active {
      return Ok(());
    }
    
    // 먼저 신호 전략 업데이트
    self.signal_strategy.update(market_data.clone())?;
    
    // 실행 전략도 업데이트
    self.execution_strategy.update(market_data)?;
    
    Ok(())
  }
  
  fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
    if !self.is_active {
      return Ok(vec![]);
    }
    
    // 신호 전략으로부터 기본 주문 가져오기
    let signal_orders = self.signal_strategy.get_orders()?;
    
    if signal_orders.is_empty() {
      return Ok(vec![]);
    }
    
    // 신호 주문을 실행 전략으로 변환
    let mut execution_orders = Vec::new();
    
    for order in signal_orders {
      // 기본 주문 속성 추출
      let symbol = order.symbol.clone();
      let side = order.side;
      let quantity = order.quantity;
      
      // 실행 전략에 맞게 주문 타입 수정
      // 이 예제에서는 주문 타입을 알고리즘으로 변경하고 원본 주문을 참조ID로 저장
      let mut execution_order = Order::new(
        symbol,
        side,
        quantity,
        // 실행 전략에 맞는 주문 타입으로 변경
        // 여기서는 간단히 예시로 보여주기 위해 임의 타입 사용
        OrderType::Algorithm {
          algo_type: self.execution_strategy.name().to_string(),
          params: vec![
            ("original_order_id".to_string(), order.id.to_string()),
            ("created_by".to_string(), "combined_strategy".to_string()),
          ],
        },
      );
      
      // 원본 주문 메타데이터 복사
      execution_order.meta = order.meta.clone();
      
      execution_orders.push(execution_order);
    }
    
    Ok(execution_orders)
  }
  
  fn name(&self) -> &str {
    &self.name
  }
  
  fn is_active(&self) -> bool {
    self.is_active
  }
  
  fn set_active(&mut self, active: bool) {
    self.is_active = active;
    self.signal_strategy.set_active(active);
    self.execution_strategy.set_active(active);
  }
}