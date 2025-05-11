/**
* filename : moving_averages
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::VecDeque;
use crate::error::TradingError;
use super::{Indicator, IndicatorResult, IndicatorSignal};

#[derive(Debug)]
pub struct SimpleMovingAverage {
  name: String,
  period: usize,
  values: VecDeque<f64>,
  sum: f64,
}

impl SimpleMovingAverage {
  pub fn new(period: usize) -> Self {
    SimpleMovingAverage {
      name: format!("SMA-{}", period),
      period,
      values: VecDeque::with_capacity(period),
      sum: 0.0,
    }
  }
  
  pub fn period(&self) -> usize {
    self.period
  }
}

impl Indicator for SimpleMovingAverage {
  fn name(&self) -> &str {
    &self.name
  }
  
  fn update(&mut self, price: f64, _volume: Option<f64>) -> Result<(), TradingError> {
    // 새 가격 추가
    self.values.push_back(price);
    self.sum += price;
    
    // 오래된 가격 제거 (필요시)
    if self.values.len() > self.period {
      if let Some(old_value) = self.values.pop_front() {
        self.sum -= old_value;
      }
    }
    
    Ok(())
  }
  
  fn calculate(&self) -> Result<IndicatorResult, TradingError> {
    if !self.is_ready() {
      return Err(TradingError::InsufficientData);
    }
    
    let value = self.sum / self.values.len() as f64;
    
    // 여기서는 단순히 값만 반환하고, 신호는 다른 모듈에서 분석
    Ok(IndicatorResult {
      value,
      signals: vec![],
    })
  }
  
  fn is_ready(&self) -> bool {
    self.values.len() >= self.period
  }
  
  fn reset(&mut self) {
    self.values.clear();
    self.sum = 0.0;
  }
}

#[derive(Debug)]
pub struct ExponentialMovingAverage {
  name: String,
  period: usize,
  values: VecDeque<f64>,
  current_ema: Option<f64>,
  alpha: f64,
  count: usize,
}

impl ExponentialMovingAverage {
  pub fn new(period: usize) -> Self {
    let alpha = 2.0 / (period as f64 + 1.0);
    
    ExponentialMovingAverage {
      name: format!("EMA-{}", period),
      period,
      values: VecDeque::with_capacity(period),
      current_ema: None,
      alpha,
      count: 0,
    }
  }
  
  pub fn period(&self) -> usize {
    self.period
  }
}

impl Indicator for ExponentialMovingAverage {
  fn name(&self) -> &str {
    &self.name
  }
  
  fn update(&mut self, price: f64, _volume: Option<f64>) -> Result<(), TradingError> {
    self.values.push_back(price);
    self.count += 1;
    
    // 값 관리 (메모리 효율성)
    if self.values.len() > self.period * 2 {
      self.values.pop_front();
    }
    
    // EMA 초기화 (처음 period개의 가격으로 SMA 계산)
    if self.count == self.period {
      let sma = self.values.iter().sum::<f64>() / self.period as f64;
      self.current_ema = Some(sma);
    }
    // EMA 업데이트
    else if self.count > self.period {
      if let Some(prev_ema) = self.current_ema {
        let new_ema = price * self.alpha + prev_ema * (1.0 - self.alpha);
        self.current_ema = Some(new_ema);
      }
    }
    
    Ok(())
  }
  
  fn calculate(&self) -> Result<IndicatorResult, TradingError> {
    if let Some(ema) = self.current_ema {
      Ok(IndicatorResult {
        value: ema,
        signals: vec![],
      })
    } else {
      Err(TradingError::InsufficientData)
    }
  }
  
  fn is_ready(&self) -> bool {
    self.current_ema.is_some()
  }
  
  fn reset(&mut self) {
    self.values.clear();
    self.current_ema = None;
    self.count = 0;
  }
}

#[derive(Debug)]
pub struct MovingAverageCrossover {
  name: String,
  fast_ma: Box<dyn Indicator>,
  slow_ma: Box<dyn Indicator>,
  last_fast: Option<f64>,
  last_slow: Option<f64>,
}

impl MovingAverageCrossover {
  pub fn new(fast_ma: Box<dyn Indicator>, slow_ma: Box<dyn Indicator>) -> Self {
    let name = format!("{}/{} Crossover", fast_ma.name(), slow_ma.name());
    
    MovingAverageCrossover {
      name,
      fast_ma,
      slow_ma,
      last_fast: None,
      last_slow: None,
    }
  }
  
  pub fn with_sma(fast_period: usize, slow_period: usize) -> Self {
    MovingAverageCrossover::new(
      Box::new(SimpleMovingAverage::new(fast_period)),
      Box::new(SimpleMovingAverage::new(slow_period)),
    )
  }
  
  pub fn with_ema(fast_period: usize, slow_period: usize) -> Self {
    MovingAverageCrossover::new(
      Box::new(ExponentialMovingAverage::new(fast_period)),
      Box::new(ExponentialMovingAverage::new(slow_period)),
    )
  }
}

impl Indicator for MovingAverageCrossover {
  fn name(&self) -> &str {
    &self.name
  }
  
  fn update(&mut self, price: f64, volume: Option<f64>) -> Result<(), TradingError> {
    self.fast_ma.update(price, volume)?;
    self.slow_ma.update(price, volume)?;
    
    Ok(())
  }
  
  fn calculate(&self) -> Result<IndicatorResult, TradingError> {
    if !self.is_ready() {
      return Err(TradingError::InsufficientData);
    }
    
    let fast_result = self.fast_ma.calculate()?;
    let slow_result = self.slow_ma.calculate()?;
    
    let fast_value = fast_result.value;
    let slow_value = slow_result.value;
    
    let mut signals = Vec::new();
    
    // 이전 값이 있으면 크로스오버 확인
    if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast, self.last_slow) {
      // 골든 크로스 - 매수 신호
      if prev_fast <= prev_slow && fast_value > slow_value {
        let strength = 0.8; // 강한 매수 신호
        signals.push(IndicatorSignal {
          name: "Golden Cross".to_string(),
          strength,
          message: format!("{} crossed above {}", self.fast_ma.name(), self.slow_ma.name()),
        });
      }
      // 데드 크로스 - 매도 신호
      else if prev_fast >= prev_slow && fast_value < slow_value {
        let strength = -0.8; // 강한 매도 신호
        signals.push(IndicatorSignal {
          name: "Death Cross".to_string(),
          strength,
          message: format!("{} crossed below {}", self.fast_ma.name(), self.slow_ma.name()),
        });
      }
    }
    
    // 두 MA 간의 상대적 거리를 지표값으로 사용
    let diff = fast_value - slow_value;
    
    Ok(IndicatorResult {
      value: diff,
      signals,
    })
  }
  
  fn is_ready(&self) -> bool {
    self.fast_ma.is_ready() && self.slow_ma.is_ready()
  }
  
  fn reset(&mut self) {
    self.fast_ma.reset();
    self.slow_ma.reset();
    self.last_fast = None;
    self.last_slow = None;
  }
}