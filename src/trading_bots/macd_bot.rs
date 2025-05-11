/**
* filename : macd_bot
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::Order;
use crate::indicators::{Indicator, IndicatorResult, trend::MACD};
use crate::signals::signal_types::{SignalType, SignalWithMetadata};
use crate::signals::position_sizing::{PositionSizer, FixedSizePositionSizer};
use super::bot_config::TradingBotConfig;
use super::base_bot::{TradingBot, create_order_from_signal};

pub struct MACDBot {
  symbol: String,
  config: TradingBotConfig,
  macd: MACD,
  position_sizer: FixedSizePositionSizer,
  last_signal: Option<SignalWithMetadata>,
  current_position: f64,
}

impl MACDBot {
  pub fn new(symbol: String, config: TradingBotConfig) -> Result<Self, TradingError> {
    // 설정에서 파라미터 추출
    let fast_period = config.get_usize("fast_period")?;
    let slow_period = config.get_usize("slow_period")?;
    let signal_period = config.get_usize("signal_period")?;
    
    // MACD 지표 생성
    let macd = MACD::new(fast_period, slow_period, signal_period);
    
    // 포지션 사이저 설정
    let base_position_size = config.get_f64("base_position_size").unwrap_or(1.0);
    let strength_multiplier = config.get_f64("strength_multiplier").unwrap_or(0.5);
    let position_sizer = FixedSizePositionSizer::new(base_position_size, strength_multiplier);
    
    Ok(MACDBot {
      symbol,
      config,
      macd,
      position_sizer,
      last_signal: None,
      current_position: 0.0,
    })
  }
}

impl TradingBot for MACDBot {
  fn update(&mut self, market_data: &MarketData) -> Result<(), TradingError> {
    // 현재 시장 데이터로 지표 업데이트
    self.macd.update(market_data.close_price, Some(market_data.volume))?;
    
    // 신호 평가
    if self.macd.is_ready() {
      let result = self.macd.calculate()?;
      
      // 유의미한 신호가 있으면 저장
      if !result.signals.is_empty() {
        // 크로스오버 신호를 우선으로 선택, 아니면 가장 강한 신호
        let crossover_signal = result.signals.iter()
          .find(|s| s.name.contains("Crossover"));
        
        if let Some(signal) = crossover_signal {
          self.last_signal = Some(SignalWithMetadata::from_indicator_signal(signal));
        } else {
          let strongest_signal = result.signals.iter()
            .max_by(|a, b| a.strength.abs().partial_cmp(&b.strength.abs()).unwrap())
            .unwrap();
          
          self.last_signal = Some(SignalWithMetadata::from_indicator_signal(strongest_signal));
        }
      }
    }
    
    Ok(())
  }
  
  fn evaluate_signals(&self) -> Result<Vec<SignalWithMetadata>, TradingError> {
    // 마지막 신호 반환 (있는 경우)
    if let Some(signal) = &self.last_signal {
      Ok(vec![signal.clone()])
    } else {
      Ok(vec![])
    }
  }
  
  fn generate_orders(&self) -> Result<Vec<Order>, TradingError> {
    if let Some(signal) = &self.last_signal {
      // 포지션 크기 계산
      let position_size = self.position_sizer.calculate_position_size(
        signal,
        10000.0, // 예시 가용 자본
        None,
        0.0, // 예시 가격
      );
      
      // 신호에 따른 주문 생성
      if let Some(order) = create_order_from_signal(
        &self.symbol,
        signal,
        position_size,
        self.current_position,
      ) {
        return Ok(vec![order]);
      }
    }
    
    Ok(vec![])
  }
  
  fn config(&self) -> &TradingBotConfig {
    &self.config
  }
  
  fn update_config(&mut self, config: TradingBotConfig) -> Result<(), TradingError> {
    // 설정 업데이트
    self.config = config;
    
    // 지표 재구성
    let fast_period = self.config.get_usize("fast_period")?;
    let slow_period = self.config.get_usize("slow_period")?;
    let signal_period = self.config.get_usize("signal_period")?;
    
    // MACD 지표 재생성
    self.macd = MACD::new(fast_period, slow_period, signal_period);
    
    // 포지션 사이저 재설정
    let base_position_size = self.config.get_f64("base_position_size").unwrap_or(1.0);
    let strength_multiplier = self.config.get_f64("strength_multiplier").unwrap_or(0.5);
    self.position_sizer = FixedSizePositionSizer::new(base_position_size, strength_multiplier);
    
    Ok(())
  }
  
  fn reset(&mut self) {
    self.macd.reset();
    self.last_signal = None;
  }
}