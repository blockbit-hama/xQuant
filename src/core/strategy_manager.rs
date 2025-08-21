/**
* filename : strategy_manager
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::Order;
use crate::strategies::Strategy;

// 전략 관리자 - 여러 전략 관리 및 조정
pub struct StrategyManager {
  strategies: HashMap<String, Box<dyn Strategy>>,
  active_strategies: Vec<String>,
}

impl StrategyManager {
  pub fn new() -> Self {
    StrategyManager {
      strategies: HashMap::new(),
      active_strategies: Vec::new(),
    }
  }
  
  // 전략 추가
  pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) -> Result<(), TradingError> {
    let name = strategy.name().to_string();
    
    if self.strategies.contains_key(&name) {
      return Err(TradingError::DuplicateStrategy(format!("Strategy '{}' already exists", name)));
    }
    
    let is_active = strategy.is_active();
    self.strategies.insert(name.clone(), strategy);
    
    if is_active {
      self.active_strategies.push(name);
    }
    
    Ok(())
  }
  
  // 전략 제거
  pub fn remove_strategy(&mut self, name: &str) -> Result<(), TradingError> {
    if !self.strategies.contains_key(name) {
      return Err(TradingError::StrategyNotFound(format!("Strategy '{}' not found", name)));
    }
    
    self.strategies.remove(name);
    self.active_strategies.retain(|s| s != name);
    
    Ok(())
  }
  
  // 전략 활성화/비활성화
  pub fn set_strategy_active(&mut self, name: &str, active: bool) -> Result<(), TradingError> {
    let strategy = self.strategies.get_mut(name)
      .ok_or_else(|| TradingError::StrategyNotFound(format!("Strategy '{}' not found", name)))?;
    
    strategy.set_active(active);
    
    if active {
      if !self.active_strategies.contains(&name.to_string()) {
        self.active_strategies.push(name.to_string());
      }
    } else {
      self.active_strategies.retain(|s| s != name);
    }
    
    Ok(())
  }
  
  // 모든 전략 업데이트
  pub fn update_all(&mut self, market_data: &MarketData) -> Result<(), TradingError> {
    for name in &self.active_strategies.clone() {
      if let Some(strategy) = self.strategies.get_mut(name) {
        strategy.update(market_data.clone())?;
      }
    }
    
    Ok(())
  }
  
  // 모든 활성 전략에서 주문 수집
  pub fn get_all_orders(&mut self) -> Result<Vec<Order>, TradingError> {
    let mut all_orders = Vec::new();
    
    for name in &self.active_strategies {
      if let Some(strategy) = self.strategies.get_mut(name) {
        let orders = strategy.get_orders()?;
        all_orders.extend(orders);
      }
    }
    
    Ok(all_orders)
  }
  
  // 특정 전략의 주문 가져오기
  pub fn get_orders_from_strategy(&mut self, name: &str) -> Result<Vec<Order>, TradingError> {
    let strategy = self.strategies.get_mut(name)
      .ok_or_else(|| TradingError::StrategyNotFound(format!("Strategy '{}' not found", name)))?;
    
    strategy.get_orders()
  }

  // 전략 상태 조회
  pub fn get_strategy_status(&self, name: &str) -> Result<(String, bool), TradingError> {
    let strategy = self.strategies.get(name)
      .ok_or_else(|| TradingError::StrategyNotFound(format!("Strategy '{}' not found", name)))?;
    Ok((name.to_string(), strategy.is_active()))
  }
  
  // 사용 가능한 전략 목록
  pub fn list_strategies(&self) -> Vec<(String, bool)> {
    self.strategies.iter()
      .map(|(name, strategy)| (name.clone(), strategy.is_active()))
      .collect()
  }
}