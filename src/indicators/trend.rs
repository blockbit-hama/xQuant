/**
* filename : trend
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::VecDeque;
use crate::error::TradingError;
use super::{Indicator, IndicatorResult, IndicatorSignal, moving_averages::ExponentialMovingAverage};

#[derive(Debug)]
pub struct MACD {
  name: String,
  fast_ema: ExponentialMovingAverage,
  slow_ema: ExponentialMovingAverage,
  signal_ema: ExponentialMovingAverage,
  histogram_values: VecDeque<f64>,
}

impl MACD {
  pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
    MACD {
      name: format!("MACD-{}-{}-{}", fast_period, slow_period, signal_period),
      fast_ema: ExponentialMovingAverage::new(fast_period),
      slow_ema: ExponentialMovingAverage::new(slow_period),
      signal_ema: ExponentialMovingAverage::new(signal_period),
      histogram_values: VecDeque::with_capacity(signal_period),
    }
  }
}

impl Indicator for MACD {
  fn name(&self) -> &str {
    &self.name
  }
  
  fn update(&mut self, price: f64, volume: Option<f64>) -> Result<(), TradingError> {
    // 빠른 EMA와 느린 EMA 업데이트
    self.fast_ema.update(price, volume)?;
    self.slow_ema.update(price, volume)?;
    
    // 두 EMA가 준비되면 MACD 라인 계산하고 시그널 EMA 업데이트
    if self.fast_ema.is_ready() && self.slow_ema.is_ready() {
      let fast_value = self.fast_ema.calculate()?.value;
      let slow_value = self.slow_ema.calculate()?.value;
      
      let macd_line = fast_value - slow_value;
      
      // 시그널 라인 업데이트
      self.signal_ema.update(macd_line, None)?;
      
      // 히스토그램 값 저장 (시그널 라인이 준비된 경우)
      if self.signal_ema.is_ready() {
        let signal_value = self.signal_ema.calculate()?.value;
        let histogram = macd_line - signal_value;
        
        self.histogram_values.push_back(histogram);
        
        // 히스토그램 값 관리 (메모리 효율성)
        if self.histogram_values.len() > 3 {
          self.histogram_values.pop_front();
        }
      }
    }
    
    Ok(())
  }
  
  fn calculate(&self) -> Result<IndicatorResult, TradingError> {
    if !self.is_ready() {
      return Err(TradingError::InsufficientData);
    }
    
    let fast_value = self.fast_ema.calculate()?.value;
    let slow_value = self.slow_ema.calculate()?.value;
    let macd_line = fast_value - slow_value;
    let signal_value = self.signal_ema.calculate()?.value;
    let histogram = macd_line - signal_value;
    
    let mut signals = Vec::new();
    
    // 최소 2개의 히스토그램 값이 있는 경우, 크로스오버 확인
    if self.histogram_values.len() >= 2 {
      let prev_histogram = self.histogram_values[self.histogram_values.len() - 2];
      
      // 시그널 크로스오버 (매수 신호)
      if prev_histogram < 0.0 && histogram > 0.0 {
        let strength = 0.7; // 강한 매수 신호
        signals.push(IndicatorSignal {
          name: "MACD Bullish Crossover".to_string(),
          strength,
          message: "MACD crossed above signal line".to_string(),
        });
      }
      // 시그널 크로스언더 (매도 신호)
      else if prev_histogram > 0.0 && histogram < 0.0 {
        let strength = -0.7; // 강한 매도 신호
        signals.push(IndicatorSignal {
          name: "MACD Bearish Crossover".to_string(),
          strength,
          message: "MACD crossed below signal line".to_string(),
        });
      }
    }
    
    // 중심선 크로스오버 (추가 신호)
    if macd_line > 0.0 && histogram > 0.0 {
      signals.push(IndicatorSignal {
        name: "MACD Above Zero".to_string(),
        strength: 0.3, // 약한 매수 신호
        message: "MACD is above zero line".to_string(),
      });
    }
    else if macd_line < 0.0 && histogram < 0.0 {
      signals.push(IndicatorSignal {
        name: "MACD Below Zero".to_string(),
        strength: -0.3, // 약한 매도 신호
        message: "MACD is below zero line".to_string(),
      });
    }
    
    Ok(IndicatorResult {
      value: histogram, // 히스토그램 값을 주요 값으로 반환
      signals,
    })
  }
  
  fn is_ready(&self) -> bool {
    self.fast_ema.is_ready() && self.slow_ema.is_ready() && self.signal_ema.is_ready()
  }
  
  fn reset(&mut self) {
    self.fast_ema.reset();
    self.slow_ema.reset();
    self.signal_ema.reset();
    self.histogram_values.clear();
  }
}