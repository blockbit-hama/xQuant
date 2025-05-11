use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderStatus};
use crate::models::trade::Trade;
use crate::core::strategy_manager::StrategyManager;
use crate::exchange::traits::Exchange;
use crate::exchange::mock::MockExchange;
use super::result::BacktestResult;

pub struct BacktestEngine {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    market_data: HashMap<String, Vec<MarketData>>,
    strategies: StrategyManager,
    exchange: MockExchange,
    initial_balance: HashMap<String, f64>,
}

impl BacktestEngine {
    pub fn new(
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        initial_balance: HashMap<String, f64>,
    ) -> Self {
        let exchange = MockExchange::new(initial_balance.clone());
        
        BacktestEngine {
            start_time,
            end_time,
            market_data: HashMap::new(),
            strategies: StrategyManager::new(),
            exchange,
            initial_balance,
        }
    }
    
    // 시장 데이터 추가
    pub fn add_market_data(&mut self, symbol: &str, data: Vec<MarketData>) {
        self.market_data.insert(symbol.to_string(), data);
    }
    
    // 전략 추가
    pub fn add_strategy(&mut self, strategy: Box<dyn crate::strategies::traits::Strategy>) -> Result<(), TradingError> {
        self.strategies.add_strategy(strategy)
    }
    
    // 백테스트 실행
    pub fn run(&mut self) -> Result<BacktestResult, TradingError> {
        // 데이터 로드 확인
        if self.market_data.is_empty() {
            return Err(TradingError::InsufficientData);
        }
        
        // 시간 기준으로 정렬된 데이터 인덱스 만들기
        let mut timeline: Vec<(DateTime<Utc>, String)> = Vec::new();
        
        for (symbol, data_series) in &self.market_data {
            for data in data_series {
                timeline.push((data.timestamp, symbol.clone()));
            }
        }
        
        // 시간순 정렬
        timeline.sort_by(|a, b| a.0.cmp(&b.0));
        
        // 필터링된 타임라인 (시작-종료 시간 내)
        let filtered_timeline: Vec<_> = timeline
          .into_iter()
          .filter(|(time, _)| *time >= self.start_time && *time <= self.end_time)
          .collect();
        
        // 초기 포트폴리오 가치 계산
        let initial_value = self.exchange.get_portfolio_value()?;
        
        // 시간에 따라 시뮬레이션 실행
        let mut current_time = self.start_time;
        
        for (time, symbol) in filtered_timeline {
            current_time = time;
            
            // 현재 시장 데이터 가져오기
            if let Some(data) = self.get_market_data(&symbol, time) {
                // 전략 업데이트
                self.strategies.update_all(&data)?;
                
                // 주문 생성 및 처리
                let orders = self.strategies.get_all_orders()?;
                for order in orders {
                    self.exchange.place_order(order)?;
                }
                
                // 주문 실행 (모의 거래소 업데이트)
                self.exchange.update(time, &data)?;
            }
        }
        
        // 최종 결과 생성
        let final_balance = self.exchange.get_balances()?;
        let final_value = self.exchange.get_portfolio_value()?;
        let trades = self.exchange.get_trades()?;
        let fee_paid = trades.iter().map(|t| t.fee).sum();
        
        let profit = final_value - initial_value;
        let profit_percentage = (profit / initial_value) * 100.0;
        
        Ok(BacktestResult {
            start_time: self.start_time,
            end_time: current_time,
            initial_balance: self.initial_balance.clone(),
            final_balance,
            initial_value,
            final_value,
            profit,
            profit_percentage,
            trades,
            fee_paid,
        })
    }
    
    // 특정 시간의 시장 데이터 가져오기
    fn get_market_data(&self, symbol: &str, time: DateTime<Utc>) -> Option<MarketData> {
        if let Some(data_series) = self.market_data.get(symbol) {
            // 정확한 시간 또는 가장 가까운 이전 데이터 찾기
            data_series.iter()
              .filter(|data| data.timestamp <= time)
              .max_by_key(|data| data.timestamp)
              .cloned()
        } else {
            None
        }
    }
}