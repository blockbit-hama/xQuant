/**
* filename : signal_types
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::indicators::IndicatorSignal;

#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
  Buy,             // 일반 매수 신호
  StrongBuy,       // 강한 매수 신호
  Sell,            // 일반 매도 신호
  StrongSell,      // 강한 매도 신호
  ReduceLong,      // 롱 포지션 축소
  ReduceShort,     // 숏 포지션 축소
  CloseLong,       // 롱 포지션 청산
  CloseShort,      // 숏 포지션 청산
  Neutral,         // 중립 신호
}

impl SignalType {
  // 신호 강도를 기반으로 신호 유형 결정 (-1.0 ~ 1.0)
  pub fn from_strength(strength: f64) -> Self {
    match strength {
      s if s > 0.7 => SignalType::StrongBuy,
      s if s > 0.3 => SignalType::Buy,
      s if s > 0.0 => SignalType::ReduceShort, // 약한 매수 신호는 숏 포지션 축소
      s if s < -0.7 => SignalType::StrongSell,
      s if s < -0.3 => SignalType::Sell,
      s if s < 0.0 => SignalType::ReduceLong, // 약한 매도 신호는 롱 포지션 축소
      _ => SignalType::Neutral,
    }
  }
  
  // 신호가 매수 방향인지 확인
  pub fn is_buy(&self) -> bool {
    matches!(self, SignalType::Buy | SignalType::StrongBuy)
  }
  
  // 신호가 매도 방향인지 확인
  pub fn is_sell(&self) -> bool {
    matches!(self, SignalType::Sell | SignalType::StrongSell)
  }
  
  // 신호가 포지션 축소/청산 방향인지 확인
  pub fn is_reduce(&self) -> bool {
    matches!(self, SignalType::ReduceLong | SignalType::ReduceShort |
                      SignalType::CloseLong | SignalType::CloseShort)
  }
}

#[derive(Debug, Clone)]
pub struct SignalWithMetadata {
  pub signal_type: SignalType,
  pub source: String,
  pub strength: f64,  // -1.0 ~ 1.0
  pub timestamp: DateTime<Utc>,
  pub confidence: f64, // 0.0 ~ 1.0
  pub additional_info: HashMap<String, String>,
}

impl SignalWithMetadata {
  pub fn new(signal_type: SignalType, source: String, strength: f64) -> Self {
    SignalWithMetadata {
      signal_type,
      source,
      strength,
      timestamp: Utc::now(),
      confidence: 1.0,
      additional_info: HashMap::new(),
    }
  }
  
  pub fn from_indicator_signal(indicator_signal: &IndicatorSignal) -> Self {
    let signal_type = SignalType::from_strength(indicator_signal.strength);
    
    SignalWithMetadata {
      signal_type,
      source: indicator_signal.name.clone(),
      strength: indicator_signal.strength,
      timestamp: Utc::now(),
      confidence: 1.0,
      additional_info: HashMap::new(),
    }
  }
  
  pub fn with_confidence(mut self, confidence: f64) -> Self {
    self.confidence = confidence;
    self
  }
  
  pub fn add_info(mut self, key: &str, value: &str) -> Self {
    self.additional_info.insert(key.to_string(), value.to_string());
    self
  }
}