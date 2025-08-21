use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::RwLock;

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderStatus};
use crate::models::trade::Trade;
use crate::core::strategy_manager::StrategyManager;
use crate::exchange::traits::Exchange;
use crate::exchange::mocks::MockExchange;
use crate::strategies::Strategy;
use super::result::BacktestResult;
use super::data_provider::HistoricalDataProvider;

/// 백테스트 엔진 - 전략 백테스팅을 위한 코어 컴포넌트
pub struct BacktestEngine {
    name: String,
    description: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    market_data: HashMap<String, Vec<MarketData>>,
    strategy_manager: StrategyManager,
    exchange: MockExchange,
    initial_balance: HashMap<String, f64>,
    fee_rate: f64,
    slippage: f64,
    data_provider: Option<HistoricalDataProvider>,
}

impl BacktestEngine {
    /// 새로운 백테스트 엔진 생성
    pub fn new(
        name: String,
        description: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        initial_balance: HashMap<String, f64>,
        fee_rate: f64,
        slippage: f64,
    ) -> Self {
        // 모의 거래소 생성 및 초기 잔고 설정
        let mut exchange_config = crate::config::Config::default();
        exchange_config.exchange.initial_balance = initial_balance.clone();
        exchange_config.exchange.fee_rate = fee_rate;
        exchange_config.exchange.slippage = slippage;
        
        let exchange = MockExchange::new(exchange_config);
        
        BacktestEngine {
            name,
            description,
            start_time,
            end_time,
            market_data: HashMap::new(),
            strategy_manager: StrategyManager::new(),
            exchange,
            initial_balance,
            fee_rate,
            slippage,
            data_provider: None,
        }
    }
    
    /// 시장 데이터 직접 추가
    pub fn add_market_data(&mut self, symbol: &str, data: Vec<MarketData>) {
        self.market_data.insert(symbol.to_string(), data);
    }
    
    /// 데이터 제공자 설정
    pub fn set_data_provider(&mut self, provider: HistoricalDataProvider) {
        self.data_provider = Some(provider);
    }
    
    /// 전략 추가
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) -> Result<(), TradingError> {
        self.strategy_manager.add_strategy(strategy)
    }
    
    /// 백테스트 실행
    pub async fn run(&mut self) -> Result<BacktestResult, TradingError> {
        // 데이터 로드 확인
        if self.market_data.is_empty() {
            if let Some(provider) = &self.data_provider {
                // 데이터 제공자를 통해 시장 데이터 로드
                for symbol in provider.available_symbols() {
                    let data = provider.load_data(
                        &symbol,
                        self.start_time,
                        self.end_time,
                    ).await?;
                    
                    self.market_data.insert(symbol, data);
                }
            } else {
                return Err(TradingError::InsufficientData);
            }
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
            if let Some(data) = self.get_market_data(&symbol, time)? {
                // 모든 전략 업데이트
                self.strategy_manager.update_all(&data)?;
                
                // 주문 생성 및 처리
                let orders = self.strategy_manager.get_all_orders()?;
                for order in orders {
                    self.process_order(order, time)?;
                }
                
                // 미결제 주문 처리
                self.process_pending_orders(time)?;
                
                // 거래소 상태 갱신
                self.exchange.update_market_data(&data);
            }
        }
        
        // 최종 결과 생성
        let final_balance = self.exchange.get_balances()?;
        let final_value = self.exchange.get_portfolio_value()?;
        let trades = self.exchange.get_trades()?;
        let fee_paid = trades.iter().map(|t| t.fee).sum();
        
        let profit = final_value - initial_value;
        let profit_percentage = if initial_value > 0.0 {
            (profit / initial_value) * 100.0
        } else {
            0.0
        };
        
        Ok(BacktestResult {
            name: self.name.clone(),
            description: self.description.clone(),
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
            symbols: self.market_data.keys().cloned().collect(),
        })
    }
    
    // 특정 시간의 시장 데이터 가져오기
    fn get_market_data(&self, symbol: &str, time: DateTime<Utc>) -> Result<Option<MarketData>, TradingError> {
        if let Some(data_series) = self.market_data.get(symbol) {
            // 정확한 시간 또는 가장 가까운 이전 데이터 찾기
            let matching_data = data_series.iter()
              .filter(|data| data.timestamp <= time)
              .max_by_key(|data| data.timestamp)
              .cloned();
            
            Ok(matching_data)
        } else {
            Ok(None)
        }
    }
    
    // 주문 처리
    fn process_order(&mut self, order: Order, time: DateTime<Utc>) -> Result<(), TradingError> {
        self.exchange.place_order(order)
    }
    
    // 미결제 주문 처리
    fn process_pending_orders(&mut self, time: DateTime<Utc>) -> Result<(), TradingError> {
        self.exchange.process_pending_orders(time)
    }
}