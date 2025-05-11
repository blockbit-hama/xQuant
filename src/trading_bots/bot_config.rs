/**
* filename : bot_config
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use crate::error::TradingError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingBotConfig {
  #[serde(default)]
  params: HashMap<String, Value>,
  
  #[serde(default)]
  name: String,
  
  #[serde(default)]
  description: String,
}

impl TradingBotConfig {
  pub fn new() -> Self {
    TradingBotConfig {
      params: HashMap::new(),
      name: "DefaultBot".to_string(),
      description: "Default Trading Bot Configuration".to_string(),
    }
  }
  
  pub fn with_name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
  
  pub fn with_description(mut self, desc: &str) -> Self {
    self.description = desc.to_string();
    self
  }
  
  // 기본 파라미터 설정 헬퍼 메서드들
  pub fn set_param<T: Into<Value>>(&mut self, key: &str, value: T) {
    self.params.insert(key.to_string(), value.into());
  }
  
  pub fn get_param(&self, key: &str) -> Option<&Value> {
    self.params.get(key)
  }
  
  pub fn get_f64(&self, key: &str) -> Result<f64, TradingError> {
    match self.get_param(key) {
      Some(value) => match value.as_f64() {
        Some(f) => Ok(f),
        None => Err(TradingError::ConfigError(
          format!("Parameter '{}' is not a valid f64", key)
        )),
      },
      None => Err(TradingError::ConfigError(
        format!("Missing parameter: '{}'", key)
      )),
    }
  }
  
  pub fn get_usize(&self, key: &str) -> Result<usize, TradingError> {
    match self.get_param(key) {
      Some(value) => match value.as_u64() {
        Some(i) => Ok(i as usize),
        None => Err(TradingError::ConfigError(
          format!("Parameter '{}' is not a valid usize", key)
        )),
      },
      None => Err(TradingError::ConfigError(
        format!("Missing parameter: '{}'", key)
      )),
    }
  }
  
  pub fn get_bool(&self, key: &str) -> Result<bool, TradingError> {
    match self.get_param(key) {
      Some(value) => match value.as_bool() {
        Some(b) => Ok(b),
        None => Err(TradingError::ConfigError(
          format!("Parameter '{}' is not a valid boolean", key)
        )),
      },
      None => Err(TradingError::ConfigError(
        format!("Missing parameter: '{}'", key)
      )),
    }
  }
  
  pub fn get_string(&self, key: &str) -> Result<String, TradingError> {
    match self.get_param(key) {
      Some(value) => match value.as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(TradingError::ConfigError(
          format!("Parameter '{}' is not a valid string", key)
        )),
      },
      None => Err(TradingError::ConfigError(
        format!("Missing parameter: '{}'", key)
      )),
    }
  }
  
  // 기본 컨피그 생성 헬퍼
  pub fn ma_crossover_config(fast_period: usize, slow_period: usize) -> Self {
    let mut config = TradingBotConfig::new()
      .with_name(&format!("MA Crossover {}/{}", fast_period, slow_period))
      .with_description(&format!(
        "Moving Average Crossover Strategy with {} and {} periods",
        fast_period, slow_period
      ));
    
    config.set_param("fast_period", fast_period as u64);
    config.set_param("slow_period", slow_period as u64);
    config.set_param("ma_type", "EMA");
    config.set_param("base_position_size", 1.0);
    config.set_param("strength_multiplier", 0.5);
    
    config
  }
  
  pub fn rsi_config(period: usize, overbought: f64, oversold: f64) -> Self {
    let mut config = TradingBotConfig::new()
      .with_name(&format!("RSI {}", period))
      .with_description(&format!(
        "RSI Strategy with {} period, overbought at {}, oversold at {}",
        period, overbought, oversold
      ));
    
    config.set_param("period", period as u64);
    config.set_param("overbought", overbought);
    config.set_param("oversold", oversold);
    config.set_param("base_position_size", 1.0);
    config.set_param("strength_multiplier", 0.5);
    
    config
  }
  
  pub fn macd_config(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
    let mut config = TradingBotConfig::new()
      .with_name(&format!("MACD {}/{}/{}", fast_period, slow_period, signal_period))
      .with_description(&format!(
        "MACD Strategy with fast period {}, slow period {}, signal period {}",
        fast_period, slow_period, signal_period
      ));
    
    config.set_param("fast_period", fast_period as u64);
    config.set_param("slow_period", slow_period as u64);
    config.set_param("signal_period", signal_period as u64);
    config.set_param("base_position_size", 1.0);
    config.set_param("strength_multiplier", 0.5);
    
    config
  }
}