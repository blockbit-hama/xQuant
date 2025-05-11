/**
* filename : position_sizing
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use crate::models::position::Position;
use super::signal_types::{SignalType, SignalWithMetadata};

pub trait PositionSizer {
  fn calculate_position_size(
    &self,
    signal: &SignalWithMetadata,
    available_capital: f64,
    current_position: Option<&Position>,
    price: f64
  ) -> f64;
}

pub struct FixedSizePositionSizer {
  base_position_size: f64,
  strength_multiplier: f64,
}

impl FixedSizePositionSizer {
  pub fn new(base_size: f64, strength_multiplier: f64) -> Self {
    FixedSizePositionSizer {
      base_position_size: base_size,
      strength_multiplier: strength_multiplier,
    }
  }
}

impl PositionSizer for FixedSizePositionSizer {
  fn calculate_position_size(
    &self,
    signal: &SignalWithMetadata,
    _available_capital: f64,
    _current_position: Option<&Position>,
    _price: f64
  ) -> f64 {
    // 신호 강도에 따라 포지션 크기 조정
    let strength_factor = 1.0 + (signal.strength.abs() * self.strength_multiplier);
    
    self.base_position_size * strength_factor
  }
}

pub struct KellyPositionSizer {
  max_risk_percentage: f64, // 최대 위험 비율 (0.0 - 1.0)
  win_rate: f64,            // 예상 승률 (0.0 - 1.0)
  reward_risk_ratio: f64,   // 보상 대 위험 비율
}

impl KellyPositionSizer {
  pub fn new(max_risk: f64, win_rate: f64, reward_risk: f64) -> Self {
    KellyPositionSizer {
      max_risk_percentage: max_risk,
      win_rate: win_rate,
      reward_risk_ratio: reward_risk,
    }
  }
}

impl PositionSizer for KellyPositionSizer {
  fn calculate_position_size(
    &self,
    signal: &SignalWithMetadata,
    available_capital: f64,
    _current_position: Option<&Position>,
    _price: f64
  ) -> f64 {
    // 켈리 수식: f = (bp - q) / b
    // f: 투자 비율, b: 보상/위험 비율, p: 승률, q: 패률 (1-p)
    let p = self.win_rate * signal.confidence; // 신호의 신뢰도로 승률 조정
    let q = 1.0 - p;
    let b = self.reward_risk_ratio;
    
    let kelly_fraction = ((b * p) - q) / b;
    
    // 음수 켈리 또는 0인 경우 최소값 사용
    let fraction = kelly_fraction.max(0.01);
    
    // 최대 위험 비율로 제한
    let capped_fraction = fraction.min(self.max_risk_percentage);
    
    // 계산된 비율로 포지션 크기 결정
    available_capital * capped_fraction
  }
}