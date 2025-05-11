/**
* filename : mod
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/
pub mod moving_averages;
pub mod oscillators;
pub mod trend;
pub mod volume;
pub mod utils;

pub use moving_averages::*;
pub use oscillators::*;
pub use trend::*;
pub use volume::*;
pub use utils::*;

use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct IndicatorResult {
  pub value: f64,
  pub signals: Vec<IndicatorSignal>,
}

#[derive(Debug, Clone)]
pub struct IndicatorSignal {
  pub name: String,
  pub strength: f64,  // -1.0(강력 매도) ~ 1.0(강력 매수)
  pub message: String,
}

pub trait Indicator: Debug + Send + Sync {
  fn name(&self) -> &str;
  
  // 새로운 데이터로 지표 업데이트
  fn update(&mut self, price: f64, volume: Option<f64>) -> Result<(), crate::error::TradingError>;
  
  // 현재 지표 값 반환
  fn calculate(&self) -> Result<IndicatorResult, crate::error::TradingError>;
  
  // 지표가 계산 가능한지 (충분한 데이터가 있는지) 확인
  fn is_ready(&self) -> bool;
  
  // 지표 상태 리셋
  fn reset(&mut self);
}