/**
* filename : technical
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::sync::{Arc, RwLock};
use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::Order;
use crate::trading_bots::{TradingBot, TradingBotConfig, bot_config};
use crate::strategies::Strategy;

// 기술적 분석 기반 전략
pub struct TechnicalStrategy {
  bot: Box<dyn TradingBot>,
  name: String,
  is_active: bool,
}

impl TechnicalStrategy {
  pub fn new(bot: Box<dyn TradingBot>, name: String) -> Self {
    TechnicalStrategy {
      bot,
      name,
      is_active: true,
    }
  }
  
  // 편의 생성자: MA 크로스오버 전략
  pub fn ma_crossover(symbol: String, fast_period: usize, slow_period: usize) -> Result<Self, TradingError> {
    let config = bot_config::TradingBotConfig::ma_crossover_config(fast_period, slow_period);
    let bot = crate::trading_bots::ma_crossover_bot::MACrossoverBot::new(symbol.clone(), config)?;
    
    Ok(TechnicalStrategy::new(
      Box::new(bot),
      format!("MA Crossover {}/{}", fast_period, slow_period),
    ))
  }
  
  // 편의 생성자: RSI 전략
  pub fn rsi(symbol: String, period: usize, oversold: f64, overbought: f64) -> Result<Self, TradingError> {
    let config = bot_config::TradingBotConfig::rsi_config(period, overbought, oversold);
    let bot = crate::trading_bots::rsi_bot::RSIBot::new(symbol.clone(), config)?;
    
    Ok(TechnicalStrategy::new(
      Box::new(bot),
      format!("RSI {}", period),
    ))
  }
  
  // 편의 생성자: MACD 전략
  pub fn macd(symbol: String, fast_period: usize, slow_period: usize, signal_period: usize) -> Result<Self, TradingError> {
    let config = bot_config::TradingBotConfig::macd_config(fast_period, slow_period, signal_period);
    let bot = crate::trading_bots::macd_bot::MACDBot::new(symbol.clone(), config)?;
    
    Ok(TechnicalStrategy::new(
      Box::new(bot),
      format!("MACD {}/{}/{}", fast_period, slow_period, signal_period),
    ))
  }
  
  // 편의 생성자: 복합 지표 전략
  pub fn multi_indicator(symbol: String) -> Result<Self, TradingError> {
    let mut config = TradingBotConfig::new()
      .with_name("Multi Indicator Strategy")
      .with_description("Combined strategy using multiple technical indicators");
    
    // 기본 설정 값
    config.set_param("ma_fast_period", 12u64);
    config.set_param("ma_slow_period", 26u64);
    config.set_param("rsi_period", 14u64);
    config.set_param("rsi_oversold", 30.0);
    config.set_param("rsi_overbought", 70.0);
    config.set_param("macd_fast_period", 12u64);
    config.set_param("macd_slow_period", 26u64);
    config.set_param("macd_signal_period", 9u64);
    config.set_param("base_position_size", 1.0);
    
    let bot = crate::trading_bots::multi_indicator_bot::MultiIndicatorBot::new(symbol.clone(), config)?;
    
    Ok(TechnicalStrategy::new(
      Box::new(bot),
      "Multi Indicator Strategy".to_string(),
    ))
  }
}

impl Strategy for TechnicalStrategy {
  fn update(&mut self, market_data: MarketData) -> Result<(), TradingError> {
    if !self.is_active {
      return Ok(());
    }
    
    self.bot.update(&market_data)
  }
  
  fn get_orders(&mut self) -> Result<Vec<Order>, TradingError> {
    if !self.is_active {
      return Ok(vec![]);
    }
    
    self.bot.generate_orders()
  }
  
  fn name(&self) -> &str {
    &self.name
  }
  
  fn description(&self) -> &str {
    // 이름을 설명으로 재사용(간단)
    &self.name
  }
  
  fn is_active(&self) -> bool {
    self.is_active
  }
  
  fn set_active(&mut self, active: bool) {
    self.is_active = active;
  }
}