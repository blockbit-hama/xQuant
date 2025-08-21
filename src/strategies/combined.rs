use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide};
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
    
    // TWAP 실행 전략 (시장가 분할). 예시: Buy, 1.0 총량, duration을 ms로 변환, 10개 분할
    let execution_strategy = crate::strategies::twap::TwapStrategy::new(
      symbol.clone(),
      OrderSide::Buy,
      1.0,
      (twap_duration_minutes as i64) * 60_000,
      10,
    );
    
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
    
    // VWAP 실행 전략. 예시: Sell, 총량을 참여율로 환산하지 않고 1.0 고정, 1시간 실행, 20 윈도우
    let execution_strategy = crate::strategies::vwap::VwapStrategy::new(
      symbol.clone(),
      OrderSide::Sell,
      vwap_participation_rate.max(0.01),
      3_600_000,
      20,
    );
    
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
    
    // Iceberg 실행 전략. 예시: Buy, 총 10.0, 지정가는 최근가 기반 외부에서 세팅한다고 가정하여 여기선 display_size만 사용
    let execution_strategy = crate::strategies::iceberg::IcebergStrategy::new(
      symbol.clone(),
      OrderSide::Buy,
      10.0,
      0.0,
      display_size.max(0.001),
    );
    
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
      // 신호 주문을 실행 전략에 다시 통과시켜 실 주문 생성
      // 실행 전략은 내부 로직에 따라 Market/Limit + 보조 파라미터(`with_*`)를 설정
      self.execution_strategy.update(market_data_from_order(&order))?;
      let mut exec_orders = self.execution_strategy.get_orders()?;
      execution_orders.append(&mut exec_orders);
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

// 보조: 신호 주문으로부터 최소한의 시장데이터 형태 구성
fn market_data_from_order(order: &Order) -> crate::models::market_data::MarketData {
  crate::models::market_data::MarketData {
    symbol: order.symbol.clone(),
    timestamp: chrono::Utc::now().timestamp_millis(),
    open: order.price,
    high: order.price,
    low: order.price,
    close: order.price,
    volume: order.quantity,
  }
}