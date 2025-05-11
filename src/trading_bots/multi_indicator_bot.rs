/**
* filename : multi_indicator_bot
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::HashMap;
use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::Order;
use crate::indicators::{Indicator, IndicatorResult, moving_averages::MovingAverageCrossover, oscillators::RelativeStrengthIndex, trend::MACD};
use crate::signals::signal_types::{SignalType, SignalWithMetadata};
use crate::signals::signal_analyzer::SignalAnalyzer;
use crate::signals::position_sizing::{PositionSizer, FixedSizePositionSizer};
use super::bot_config::TradingBotConfig;
use super::base_bot::{TradingBot, create_order_from_signal};

pub struct MultiIndicatorBot {
  symbol: String,
  config: TradingBotConfig,
  indicators: Vec<Box<dyn Indicator>>,
  signal_analyzer: SignalAnalyzer,
  position_sizer: FixedSizePositionSizer,
  last_signals: Vec<SignalWithMetadata>,
  current_position: f64,
}

impl MultiIndicatorBot {
  pub fn new(symbol: String, config: TradingBotConfig) -> Result<Self, TradingError> {
    // 지표 생성
    let mut indicators: Vec<Box<dyn Indicator>> = Vec::new();
    
    // 기본 지표 설정
    // 1. MA 크로스오버
    if let Ok(fast_ma) = config.get_usize("ma_fast_period") {
      let slow_ma = config.get_usize("ma_slow_period").unwrap_or(26);
      indicators.push(Box::new(MovingAverageCrossover::with_ema(fast_ma, slow_ma)));
    }
    
    // 2. RSI
    if let Ok(rsi_period) = config.get_usize("rsi_period") {
      let overbought = config.get_f64("rsi_overbought").unwrap_or(70.0);
      let oversold = config.get_f64("rsi_oversold").unwrap_or(30.0);
      indicators.push(Box::new(RelativeStrengthIndex::new(rsi_period, Some(overbought), Some(oversold))));
    }
    
    // 3. MACD
    if let Ok(macd_fast) = config.get_usize("macd_fast_period") {
      let macd_slow = config.get_usize("macd_slow_period").unwrap_or(26);
      let macd_signal = config.get_usize("macd_signal_period").unwrap_or(9);
      indicators.push(Box::new(MACD::new(macd_fast, macd_slow, macd_signal)));
    }
    
    // 지표가 없으면 기본값 설정
    if indicators.is_empty() {
      indicators.push(Box::new(MovingAverageCrossover::with_ema(12, 26)));
      indicators.push(Box::new(RelativeStrengthIndex::new(14, Some(70.0), Some(30.0))));
      indicators.push(Box::new(MACD::new(12, 26, 9)));
    }
    
    // 신호 분석기 생성
    let signal_analyzer = SignalAnalyzer::new();
    
    // 포지션 사이저 설정
    let base_position_size = config.get_f64("base_position_size").unwrap_or(1.0);
    let strength_multiplier = config.get_f64("strength_multiplier").unwrap_or(0.5);
    let position_sizer = FixedSizePositionSizer::new(base_position_size, strength_multiplier);
    
    Ok(MultiIndicatorBot {
      symbol,
      config,
      indicators,
      signal_analyzer,
      position_sizer,
      last_signals: Vec::new(),
      current_position: 0.0,
    })
  }
}

impl TradingBot for MultiIndicatorBot {
  fn update(&mut self, market_data: &MarketData) -> Result<(), TradingError> {
    // 모든 지표 업데이트
    for indicator in &mut self.indicators {
      indicator.update(market_data.close_price, Some(market_data.volume))?;
    }
    
    // 각 지표의 결과 계산
    let mut indicator_results = Vec::new();
    
    for indicator in &self.indicators {
      if indicator.is_ready() {
        if let Ok(result) = indicator.calculate() {
          indicator_results.push(result);
        }
      }
    }
    
    // 신호 분석
    self.last_signals = self.signal_analyzer.analyze_indicator_results(&indicator_results);
    
    Ok(())
  }
  
  fn evaluate_signals(&self) -> Result<Vec<SignalWithMetadata>, TradingError> {
    Ok(self.last_signals.clone())
  }
  
  fn generate_orders(&self) -> Result<Vec<Order>, TradingError> {
    if self.last_signals.is_empty() {
      return Ok(vec![]);
    }
    
    // 가장 강한 신호 찾기
    let strongest_signal = self.last_signals.iter()
      .max_by(|a, b| {
        (a.strength.abs() * a.confidence)
          .partial_cmp(&(b.strength.abs() * b.confidence))
          .unwrap()
      })
      .unwrap();
    
    // 포지션 크기 계산
    let position_size = self.position_sizer.calculate_position_size(
      strongest_signal,
      10000.0, // 예시 가용 자본
      None,
      0.0, // 예시 가격
    );
    
    // 신호에 따른 주문 생성
    if let Some(order) = create_order_from_signal(
      &self.symbol,
      strongest_signal,
      position_size,
      self.current_position,
    ) {
      return Ok(vec![order]);
    }
    
    Ok(vec![])
  }
  
  fn config(&self) -> &TradingBotConfig {
    &self.config
  }
  
  fn update_config(&mut self, config: TradingBotConfig) -> Result<(), TradingError> {
    // 설정 업데이트
    self.config = config;
    
    // 지표 재생성
    let mut new_indicators: Vec<Box<dyn Indicator>> = Vec::new();
    
    // 1. MA 크로스오버
    if let Ok(fast_ma) = self.config.get_usize("ma_fast_period") {
      let slow_ma = self.config.get_usize("ma_slow_period").unwrap_or(26);
      new_indicators.push(Box::new(MovingAverageCrossover::with_ema(fast_ma, slow_ma)));
    }
    
    // 2. RSI
    if let Ok(rsi_period) = self.config.get_usize("rsi_period") {
      let overbought = self.config.get_f64("rsi_overbought").unwrap_or(70.0);
      let oversold = self.config.get_f64("rsi_oversold").unwrap_or(30.0);
      new_indicators.push(Box::new(RelativeStrengthIndex::new(rsi_period, Some(overbought), Some(oversold))));
    }
    
    // 3. MACD
    if let Ok(macd_fast) = self.config.get_usize("macd_fast_period") {
      let macd_slow = self.config.get_usize("macd_slow_period").unwrap_or(26);
      let macd_signal = self.config.get_usize("macd_signal_period").unwrap_or(9);
      new_indicators.push(Box::new(MACD::new(macd_fast, macd_slow, macd_signal)));
    }
    
    // 지표가 없으면 기본값 설정
    if new_indicators.is_empty() {
      new_indicators.push(Box::new(MovingAverageCrossover::with_ema(12, 26)));
      new_indicators.push(Box::new(RelativeStrengthIndex::new(14, Some(70.0), Some(30.0))));
      new_indicators.push(Box::new(MACD::new(12, 26, 9)));
    }
    
    self.indicators = new_indicators;
    
    // 포지션 사이저 재설정
    let base_position_size = self.config.get_f64("base_position_size").unwrap_or(1.0);
    let strength_multiplier = self.config.get_f64("strength_multiplier").unwrap_or(0.5);
    self.position_sizer = FixedSizePositionSizer::new(base_position_size, strength_multiplier);
    
    Ok(())
  }
  
  fn reset(&mut self) {
    for indicator in &mut self.indicators {
      indicator.reset();
    }
    self.last_signals.clear();
  }
}