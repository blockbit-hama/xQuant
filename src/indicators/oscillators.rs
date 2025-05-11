/**
* filename : oscillators
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::VecDeque;
use crate::error::TradingError;
use super::{Indicator, IndicatorResult, IndicatorSignal};

#[derive(Debug)]
pub struct RelativeStrengthIndex {
  name: String,
  period: usize,
  prices: VecDeque<f64>,
  gains: VecDeque<f64>,
  losses: VecDeque<f64>,
  avg_gain: Option<f64>,
  avg_loss: Option<f64>,
  prev_price: Option<f64>,
  overbought_threshold: f64,
  oversold_threshold: f64,
}

impl RelativeStrengthIndex {
  pub fn new(period: usize, overbought: Option<f64>, oversold: Option<f64>) -> Self {
    RelativeStrengthIndex {
      name: format!("RSI-{}", period),
      period,
      prices: VecDeque::with_capacity(period + 1),
      gains: VecDeque::with_capacity(period),
      losses: VecDeque::with_capacity(period),
      avg_gain: None,
      avg_loss: None,
      prev_price: None,
      overbought_threshold: overbought.unwrap_or(70.0),
      oversold_threshold: oversold.unwrap_or(30.0),
    }
  }
  
  pub fn period(&self) -> usize {
    self.period
  }
}

impl Indicator for RelativeStrengthIndex {
  fn name(&self) -> &str {
    &self.name
  }
  
  fn update(&mut self, price: f64, _volume: Option<f64>) -> Result<(), TradingError> {
    self.prices.push_back(price);
    
    // 이전 가격과 비교하여 gain/loss 계산
    if let Some(prev_price) = self.prev_price {
      let change = price - prev_price;
      
      let gain = if change > 0.0 { change } else { 0.0 };
      let loss = if change < 0.0 { -change } else { 0.0 };
      
      self.gains.push_back(gain);
      self.losses.push_back(loss);
      
      // 초기 평균 계산
      if self.gains.len() == self.period {
        let total_gain: f64 = self.gains.iter().sum();
        let total_loss: f64 = self.losses.iter().sum();
        
        self.avg_gain = Some(total_gain / self.period as f64);
        self.avg_loss = Some(total_loss / self.period as f64);
      }
      // 평균 업데이트 (Wilder의 스무딩 방법)
      else if self.gains.len() > self.period {
        if let (Some(prev_avg_gain), Some(prev_avg_loss)) = (self.avg_gain, self.avg_loss) {
          let new_avg_gain = (prev_avg_gain * (self.period as f64 - 1.0) + gain) / self.period as f64;
          let new_avg_loss = (prev_avg_loss * (self.period as f64 - 1.0) + loss) / self.period as f64;
          
          self.avg_gain = Some(new_avg_gain);
          self.avg_loss = Some(new_avg_loss);
          
          // 오래된 데이터 제거
          if self.gains.len() > self.period {
            self.gains.pop_front();
            self.losses.pop_front();
          }
        }
      }
    }
    
    // 가장 오래된 가격 제거 (메모리 효율성)
    if self.prices.len() > self.period + 1 {
      self.prices.pop_front();
    }
    
    self.prev_price = Some(price);
    
    Ok(())
  }
  
  fn calculate(&self) -> Result<IndicatorResult, TradingError> {
    if !self.is_ready() {
      return Err(TradingError::InsufficientData);
    }
    
    let (avg_gain, avg_loss) = match (self.avg_gain, self.avg_loss) {
      (Some(g), Some(l)) => (g, l),
      _ => return Err(TradingError::CalculationError("RSI averages not calculated".to_string())),
    };
    
    // RS = 평균 상승폭 / 평균 하락폭
    let rs = if avg_loss == 0.0 {
      100.0 // 분모가 0인 경우 최대값으로 처리
    } else {
      avg_gain / avg_loss
    };
    
    // RSI = 100 - (100 / (1 + RS))
    let rsi = 100.0 - (100.0 / (1.0 + rs));
    
    let mut signals = Vec::new();
    
    // 과매수 영역 - 매도 신호
    if rsi > self.overbought_threshold {
      let strength = -0.5 - (rsi - self.overbought_threshold) / 60.0; // -0.5 ~ -1.0
      signals.push(IndicatorSignal {
        name: "RSI Overbought".to_string(),
        strength,
        message: format!("RSI is overbought at {:.2}", rsi),
      });
    }
    // 과매도 영역 - 매수 신호
    else if rsi < self.oversold_threshold {
      let strength = 0.5 + (self.oversold_threshold - rsi) / 60.0; // 0.5 ~ 1.0
      signals.push(IndicatorSignal {
        name: "RSI Oversold".to_string(),
        strength,
        message: format!("RSI is oversold at {:.2}", rsi),
      });
    }
    
    Ok(IndicatorResult {
      value: rsi,
      signals,
    })
  }
  
  fn is_ready(&self) -> bool {
    self.avg_gain.is_some() && self.avg_loss.is_some()
  }
  
  fn reset(&mut self) {
    self.prices.clear();
    self.gains.clear();
    self.losses.clear();
    self.avg_gain = None;
    self.avg_loss = None;
    self.prev_price = None;
  }
}