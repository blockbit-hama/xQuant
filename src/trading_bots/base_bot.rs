/**
* filename : base_bot
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::sync::Arc;
use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide, OrderType};
use crate::signals::signal_types::{SignalType, SignalWithMetadata};
use super::bot_config::TradingBotConfig;

pub trait TradingBot: Send + Sync {
  // 시장 데이터로 봇 상태 업데이트
  fn update(&mut self, market_data: &MarketData) -> Result<(), TradingError>;
  
  // 트레이딩 신호 평가
  fn evaluate_signals(&self) -> Result<Vec<SignalWithMetadata>, TradingError>;
  
  // 신호에 기반한 주문 생성
  fn generate_orders(&self) -> Result<Vec<Order>, TradingError>;
  
  // 봇 설정 가져오기
  fn config(&self) -> &TradingBotConfig;
  
  // 봇 설정 업데이트
  fn update_config(&mut self, config: TradingBotConfig) -> Result<(), TradingError>;
  
  // 봇 상태 리셋
  fn reset(&mut self);
  
  // 봇 이름 가져오기
  fn name(&self) -> &str {
    // config의 이름 필드를 공개하지 않으므로 기본 구현 제공
    "TradingBot"
  }
}

// 봇 팩토리 - 설정에 따라 적절한 봇 생성
pub fn create_bot(symbol: &str, config: TradingBotConfig) -> Result<Box<dyn TradingBot>, TradingError> {
  let bot_type = config.get_string("bot_type").unwrap_or_else(|_| "unknown".to_string());
  
  match bot_type.as_str() {
    "ma_crossover" => Ok(Box::new(super::ma_crossover_bot::MACrossoverBot::new(
      symbol.to_string(), config)?)),
    
    "rsi" => Ok(Box::new(super::rsi_bot::RSIBot::new(
      symbol.to_string(), config)?)),
    
    "macd" => Ok(Box::new(super::macd_bot::MACDBot::new(
      symbol.to_string(), config)?)),
    
    "multi_indicator" => Ok(Box::new(super::multi_indicator_bot::MultiIndicatorBot::new(
      symbol.to_string(), config)?)),
    
    _ => Err(TradingError::ConfigError(format!("Unknown bot type: {}", bot_type))),
  }
}

// 신호와 포지션을 기반으로 주문 생성 헬퍼 함수
pub fn create_order_from_signal(
  symbol: &str,
  signal: &SignalWithMetadata,
  position_size: f64,
  current_position: f64,
) -> Option<Order> {
  // 신호 유형에 따른 주문 생성
  match signal.signal_type {
    SignalType::Buy | SignalType::StrongBuy => {
      // 현재 롱 포지션이 없거나 숏 포지션인 경우만 매수
      if current_position <= 0.0 {
        Some(Order::new(symbol.to_string(), OrderSide::Buy, OrderType::Market, position_size, 0.0))
      } else {
        None
      }
    },
    
    SignalType::Sell | SignalType::StrongSell => {
      // 현재 숏 포지션이 없거나 롱 포지션인 경우만 매도
      if current_position >= 0.0 {
        Some(Order::new(symbol.to_string(), OrderSide::Sell, OrderType::Market, position_size, 0.0))
      } else {
        None
      }
    },
    
    SignalType::ReduceLong => {
      // 롱 포지션이 있는 경우만 일부 청산
      if current_position > 0.0 {
        let reduce_size = (current_position * 0.5).min(position_size);
        if reduce_size > 0.0 {
          Some(Order::new(symbol.to_string(), OrderSide::Sell, OrderType::Market, reduce_size, 0.0))
        } else {
          None
        }
      } else {
        None
      }
    },
    
    SignalType::ReduceShort => {
      // 숏 포지션이 있는 경우만 일부 청산
      if current_position < 0.0 {
        let reduce_size = (current_position.abs() * 0.5).min(position_size);
        if reduce_size > 0.0 {
          Some(Order::new(symbol.to_string(), OrderSide::Buy, OrderType::Market, reduce_size, 0.0))
        } else {
          None
        }
      } else {
        None
      }
    },
    
    SignalType::CloseLong => {
      // 롱 포지션 전체 청산
      if current_position > 0.0 {
        Some(Order::new(symbol.to_string(), OrderSide::Sell, OrderType::Market, current_position, 0.0))
      } else {
        None
      }
    },
    
    SignalType::CloseShort => {
      // 숏 포지션 전체 청산
      if current_position < 0.0 {
        Some(Order::new(symbol.to_string(), OrderSide::Buy, OrderType::Market, current_position.abs(), 0.0))
      } else {
        None
      }
    },
    
    SignalType::Neutral => None,
  }
}