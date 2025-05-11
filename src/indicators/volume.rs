/**
* filename : volume
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::VecDeque;
use crate::error::TradingError;
use super::{Indicator, IndicatorResult, IndicatorSignal};

#[derive(Debug)]
pub struct VolumeWeightedAveragePrice {
  name: String,
  period: usize,
  prices: VecDeque<f64>,
  volumes: VecDeque<f64>,
  price_volume_products: VecDeque<f64>,
}

impl VolumeWeightedAveragePrice {
  pub fn new(period: usize) -> Self {
    VolumeWeightedAveragePrice {
      name: format!("VWAP-{}", period),
      period,
      prices: VecDeque::with_capacity(period),
      volumes: VecDeque::with_capacity(period),
      price_volume_products: VecDeque::with_capacity(period),
    }
  }
}

impl Indicator for VolumeWeightedAveragePrice {
  fn name(&self) -> &str {
    &self.name
  }
  
  fn update(&mut self, price: f64, volume: Option<f64>) -> Result<(), TradingError> {
    let volume = volume.ok_or(TradingError::MissingData("Volume data required for VWAP".to_string()))?;
    
    self.prices.push_back(price);
    self.volumes.push_back(volume);
    self.price_volume_products.push_back(price * volume);
    
    // 오래된 데이터 제거
    if self.prices.len() > self.period {
      self.prices.pop_front();
      self.volumes.pop_front();
      self.price_volume_products.pop_front();
    }
    
    Ok(())
  }
  
  fn calculate(&self) -> Result<IndicatorResult, TradingError> {
    if !self.is_ready() {
      return Err(TradingError::InsufficientData);
    }
    
    let total_volume: f64 = self.volumes.iter().sum();
    
    if total_volume == 0.0 {
      return Err(TradingError::CalculationError("Total volume is zero".to_string()));
    }
    
    let total_price_volume: f64 = self.price_volume_products.iter().sum();
    let vwap = total_price_volume / total_volume;
    
    // 마지막 가격과 VWAP 비교
    let mut signals = Vec::new();
    
    if !self.prices.is_empty() {
      let last_price = self.prices.back().unwrap();
      
      // 가격이 VWAP보다 높음 (약한 매수 신호)
      if *last_price > vwap {
        let ratio = last_price / vwap;
        let strength = 0.2 + ((ratio - 1.0) * 2.0).min(0.3); // 0.2 ~ 0.5
        
        signals.push(IndicatorSignal {
          name: "Price Above VWAP".to_string(),
          strength,
          message: format!("Price is {:.2}% above VWAP", (ratio - 1.0) * 100.0),
        });
      }
      // 가격이 VWAP보다 낮음 (약한 매도 신호)
      else if *last_price < vwap {
        let ratio = vwap / last_price;
        let strength = -0.2 - ((ratio - 1.0) * 2.0).min(0.3); // -0.2 ~ -0.5
        
        signals.push(IndicatorSignal {
          name: "Price Below VWAP".to_string(),
          strength,
          message: format!("Price is {:.2}% below VWAP", (ratio - 1.0) * 100.0),
        });
      }
    }
    
    Ok(IndicatorResult {
      value: vwap,
      signals,
    })
  }
  
  fn is_ready(&self) -> bool {
    !self.prices.is_empty() && !self.volumes.is_empty()
  }
  
  fn reset(&mut self) {
    self.prices.clear();
    self.volumes.clear();
    self.price_volume_products.clear();
  }
}