use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use chrono::{DateTime, Utc, Duration};

use crate::backtest::data_provider::{BacktestDataProvider, CsvDataProvider};
use crate::backtest::result::BacktestResult;
use crate::backtest::scenario::BacktestScenario;
use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};
use crate::models::trade::Trade;
use crate::strategies::Strategy;

/// 백테스트 엔진
pub struct BacktestEngine {
    data_provider: Box<dyn BacktestDataProvider>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    initial_balance: HashMap<String, f64>,
    current_balance: HashMap<String, f64>,
    orders: HashMap<OrderId, Order>,
    trades: Vec<Trade>,
    market_data: HashMap<String, Vec<MarketData>>,
    strategies: Vec<Box<dyn Strategy>>,
    fee_rate: f64,
    slippage: f64,
    order_id_counter: u64,
}

impl BacktestEngine {
    pub fn new(
        data_provider: Box<dyn BacktestDataProvider>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        initial_balance: HashMap<String, f64>,
        fee_rate: f64,
        slippage: f64,
    ) -> Self {
        BacktestEngine {
            data_provider,
            start_time,
            end_time,
            initial_balance: initial_balance.clone(),
            current_balance: initial_balance,
            orders: HashMap::new(),
            trades: Vec::new(),
            market_data: HashMap::new(),
            strategies: Vec::new(),
            fee_rate,
            slippage,
            order_id_counter: 0,
        }
    }

    /// CSV 파일에서 백테스트 엔진 생성
    pub fn from_csv(
        csv_path: impl AsRef<Path>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        symbol: &str,
        initial_balance: HashMap<String, f64>,
        fee_rate: f64,
        slippage: f64,
    ) -> Result<Self, TradingError> {
        let data_provider = CsvDataProvider::new(csv_path)?;

        Ok(BacktestEngine {
            data_provider: Box::new(data_provider),
            start_time,
            end_time,
            initial_balance: initial_balance.clone(),
            current_balance: initial_balance,
            orders: HashMap::new(),
            trades: Vec::new(),
            market_data: HashMap::new(),
            strategies: Vec::new(),
            fee_rate,
            slippage,
            order_id_counter: 0,
        })
    }

    /// 전략 추가
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }
    
    
    /// 백테스트 실행
    pub async fn run(&mut self) -> Result<BacktestResult, TradingError> {
        // 데이터 로드
        self.load_data().await?;
        
        // 심볼별 시장 데이터
        let symbols: Vec<String> = self.market_data.keys().cloned().collect();
        
        // 시간 기준으로 정렬된 데이터 인덱스 만들기
        let mut timeline: Vec<(DateTime<Utc>, String)> = Vec::new();
        
        for symbol in &symbols {
            if let Some(data_series) = self.market_data.get(symbol) {
                for data in data_series {
                    let timestamp = DateTime::<Utc>::from_timestamp_millis(data.timestamp).unwrap();
                    timeline.push((timestamp, symbol.clone()));
                }
            }
        }
        
        // 시간순 정렬
        timeline.sort_by(|a, b| a.0.cmp(&b.0));
        
        // 필터링된 타임라인 (시작-종료 시간 내)
        let filtered_timeline: Vec<_> = timeline
          .into_iter()
          .filter(|(time, _)| *time >= self.start_time && *time <= self.end_time)
          .collect();
        
        // 시간에 따라 시뮬레이션 실행
        let mut current_time = self.start_time;
        
        for (time, symbol) in filtered_timeline {
            current_time = time;
            
            // 현재 시장 데이터 가져오기
            let current_data = self.get_current_market_data(&symbol, time)?;
            
            // 전략별로 순차 처리
            for i in 0..self.strategies.len() {
                // 해당 전략에 대한 가변 참조 얻기
                let strategy = &mut self.strategies[i];
                
                // 전략 업데이트
                strategy.update(current_data.clone())?;
                
                // 신규 주문 가져오기
                let new_orders = strategy.get_orders()?;
                
                // 주문을 모두 처리한 후 루프를 빠져나감
                for order in new_orders {
                    self.process_order(order, time)?;
                }
            }
            
            // 미결제 주문 처리
            self.process_pending_orders(time)?;
        }
        
        // 백테스트 결과 생성
        self.generate_result(current_time)
    }

    /// 데이터 로드
    async fn load_data(&mut self) -> Result<(), TradingError> {
        self.market_data = self.data_provider.load_data().await?;
        Ok(())
    }

    /// 현재 시장 데이터 얻기
    fn get_current_market_data(&self, symbol: &str, time: DateTime<Utc>) -> Result<MarketData, TradingError> {
        if let Some(data_series) = self.market_data.get(symbol) {
            // 시간에 가장 가까운 데이터 찾기
            let timestamp = time.timestamp_millis();

            for data in data_series {
                if data.timestamp <= timestamp {
                    return Ok(data.clone());
                }
            }

            return Err(TradingError::DataNotFound(format!("No data for {} at {}", symbol, time)));
        }

        Err(TradingError::DataNotFound(format!("No data series for {}", symbol)))
    }

    /// 주문 처리
    fn process_order(&mut self, mut order: Order, time: DateTime<Utc>) -> Result<(), TradingError> {
        // 주문 ID 생성
        let order_id = self.generate_order_id();
        order.id = order_id.clone();

        // 시장 데이터 얻기
        let market_data = self.get_current_market_data(&order.symbol, time)?;

        // 잔고 확인
        match order.side {
            OrderSide::Buy => {
                // 기본 구조는 BTCUSDT와 같은 형태 가정
                let quote_currency = &order.symbol[3..];
                let required_balance = order.quantity * order.price * (1.0 + self.fee_rate);

                if let Some(balance) = self.current_balance.get(quote_currency) {
                    if *balance < required_balance {
                        return Err(TradingError::InsufficientBalance);
                    }
                } else {
                    return Err(TradingError::InsufficientBalance);
                }
            },
            OrderSide::Sell => {
                let base_currency = &order.symbol[0..3];

                if let Some(balance) = self.current_balance.get(base_currency) {
                    if *balance < order.quantity {
                        return Err(TradingError::InsufficientBalance);
                    }
                } else {
                    return Err(TradingError::InsufficientBalance);
                }
            }
        }

        // 주문 처리
        match order.order_type {
            OrderType::Market => {
                // 시장가 주문 즉시 체결
                let executed_price = match order.side {
                    OrderSide::Buy => market_data.close * (1.0 + self.slippage),
                    OrderSide::Sell => market_data.close * (1.0 - self.slippage),
                };

                self.execute_trade(&order, executed_price, order.quantity, time)?;
            },
            OrderType::Limit => {
                // 지정가 주문은 가격 조건 확인 후 체결
                let can_execute = match order.side {
                    OrderSide::Buy => market_data.low <= order.price,
                    OrderSide::Sell => market_data.high >= order.price,
                };

                if can_execute {
                    self.execute_trade(&order, order.price, order.quantity, time)?;
                } else {
                    // 미체결 주문 저장
                    self.orders.insert(order_id, order);
                }
            },
            // 기타 주문 유형 처리 (실제 구현 필요)
            _ => {
                self.orders.insert(order_id, order);
            }
        }

        Ok(())
    }

    /// 미결제 주문 처리
    fn process_pending_orders(&mut self, time: DateTime<Utc>) -> Result<(), TradingError> {
        let order_ids: Vec<OrderId> = self.orders.keys().cloned().collect();

        for order_id in order_ids {
            if let Some(order) = self.orders.get(&order_id) {
                let market_data = self.get_current_market_data(&order.symbol, time)?;

                match order.order_type {
                    OrderType::Limit => {
                        let can_execute = match order.side {
                            OrderSide::Buy => market_data.low <= order.price,
                            OrderSide::Sell => market_data.high >= order.price,
                        };

                        if can_execute {
                            let order = self.orders.remove(&order_id).unwrap();
                            self.execute_trade(&order, order.price, order.quantity, time)?;
                        }
                    },
                    // 기타 주문 유형 처리
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// 거래 실행
    fn execute_trade(&mut self, order: &Order, price: f64, quantity: f64, time: DateTime<Utc>) -> Result<(), TradingError> {
        // 잔고 업데이트
        let base_currency = &order.symbol[0..3];
        let quote_currency = &order.symbol[3..];

        match order.side {
            OrderSide::Buy => {
                // 기준 통화 증가, 견적 통화 감소
                *self.current_balance.entry(base_currency.to_string()).or_insert(0.0) += quantity;
                *self.current_balance.entry(quote_currency.to_string()).or_insert(0.0) -= quantity * price * (1.0 + self.fee_rate);
            },
            OrderSide::Sell => {
                // 기준 통화 감소, 견적 통화 증가
                *self.current_balance.entry(base_currency.to_string()).or_insert(0.0) -= quantity;
                *self.current_balance.entry(quote_currency.to_string()).or_insert(0.0) += quantity * price * (1.0 - self.fee_rate);
            }
        }

        // 거래 기록 추가
        let trade = Trade {
            id: format!("trade-{}", self.trades.len() + 1),
            symbol: order.symbol.clone(),
            price,
            quantity,
            timestamp: time.timestamp_millis(),
            order_id: order.id.clone(),
            side: order.side.clone(),
        };

        self.trades.push(trade);

        Ok(())
    }

    /// 주문 ID 생성
    fn generate_order_id(&mut self) -> OrderId {
        self.order_id_counter += 1;
        OrderId(format!("backtest-{}", self.order_id_counter))
    }

    /// 백테스트 결과 생성
    fn generate_result(&self, end_time: DateTime<Utc>) -> Result<BacktestResult, TradingError> {
        // 초기 포트폴리오 가치 계산
        let mut initial_value = 0.0;
        let mut final_value = 0.0;

        // 마지막 시장 데이터로 가치 계산
        for (currency, balance) in &self.initial_balance {
            if currency == "USDT" {
                initial_value += balance;
            } else {
                // 가능한 경우 시장 데이터로 가치 변환
                let symbol = format!("{}USDT", currency);

                if let Ok(market_data) = self.get_current_market_data(&symbol, end_time) {
                    initial_value += balance * market_data.close;
                }
            }
        }

        // 최종 포트폴리오 가치 계산
        for (currency, balance) in &self.current_balance {
            if currency == "USDT" {
                final_value += balance;
            } else {
                // 가능한 경우 시장 데이터로 가치 변환
                let symbol = format!("{}USDT", currency);

                if let Ok(market_data) = self.get_current_market_data(&symbol, end_time) {
                    final_value += balance * market_data.close;
                }
            }
        }

        // 수익률 계산
        let profit = final_value - initial_value;
        let profit_percentage = (profit / initial_value) * 100.0;

        // 결과 생성
        Ok(BacktestResult {
            start_time: self.start_time,
            end_time,
            initial_balance: self.initial_balance.clone(),
            final_balance: self.current_balance.clone(),
            initial_value,
            final_value,
            profit,
            profit_percentage,
            trades: self.trades.clone(),
            fee_paid: self.calculate_total_fees(),
        })
    }

    /// 총 수수료 계산
    fn calculate_total_fees(&self) -> f64 {
        self.trades.iter()
            .map(|trade| trade.price * trade.quantity * self.fee_rate)
            .sum()
    }
}