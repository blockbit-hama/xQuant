use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Duration, Utc};

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use crate::strategies::Strategy;
use super::engine::BacktestEngine;
use super::result::BacktestResult;
use super::data_provider::{HistoricalDataProvider, CsvDataProvider};

/// 백테스트 시나리오 - 백테스트를 실행하기 위한 모든 설정 및 매개변수 포함
pub struct BacktestScenario {
    name: String,
    description: String,
    engine: BacktestEngine,
}

impl BacktestScenario {
    /// 시나리오 생성자
    pub fn new(
        name: String,
        description: String,
        engine: BacktestEngine,
    ) -> Self {
        BacktestScenario {
            name,
            description,
            engine,
        }
    }
    
    /// 시나리오 이름 가져오기
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 시나리오 설명 가져오기
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// 백테스트 실행
    pub async fn run(&mut self) -> Result<BacktestResult, TradingError> {
        self.engine.run().await
    }
}

/// 백테스트 시나리오 빌더 - 쉽게 시나리오 구성
pub struct BacktestScenarioBuilder {
    name: String,
    description: String,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    data_file: Option<PathBuf>,
    symbols: Vec<String>,
    initial_balance: HashMap<String, f64>,
    fee_rate: f64,
    slippage: f64,
    strategies: Vec<Box<dyn Strategy>>,
    csv_delimiter: char,
}

impl BacktestScenarioBuilder {
    /// 새 빌더 생성
    pub fn new(name: impl Into<String>) -> Self {
        BacktestScenarioBuilder {
            name: name.into(),
            description: String::new(),
            start_time: None,
            end_time: None,
            data_file: None,
            symbols: Vec::new(),
            initial_balance: HashMap::new(),
            fee_rate: 0.001, // 기본 수수료율 0.1%
            slippage: 0.0005, // 기본 슬리피지 0.05%
            strategies: Vec::new(),
            csv_delimiter: ',',
        }
    }
    
    /// 시나리오 설명 설정
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    
    /// 기간 설정
    pub fn period(mut self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        self
    }
    
    /// 최근 N일 기간 설정
    pub fn last_days(self, days: i64) -> Self {
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(days);
        self.period(start_time, end_time)
    }
    
    /// 데이터 파일 설정
    pub fn data_file(mut self, path: PathBuf) -> Self {
        self.data_file = Some(path);
        self
    }
    
    /// 심볼 추가
    pub fn symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbols.push(symbol.into());
        self
    }
    
    /// 초기 잔고 설정
    pub fn initial_balance(mut self, asset: impl Into<String>, amount: f64) -> Self {
        self.initial_balance.insert(asset.into(), amount);
        self
    }
    
    /// 수수료율 설정
    pub fn fee_rate(mut self, fee_rate: f64) -> Self {
        self.fee_rate = fee_rate;
        self
    }
    
    /// 슬리피지 설정
    pub fn slippage(mut self, slippage: f64) -> Self {
        self.slippage = slippage;
        self
    }
    
    /// 전략 추가
    pub fn strategy(mut self, strategy: Box<dyn Strategy>) -> Self {
        self.strategies.push(strategy);
        self
    }
    
    /// CSV 구분자 설정
    pub fn csv_delimiter(mut self, delimiter: char) -> Self {
        self.csv_delimiter = delimiter;
        self
    }
    
    /// 시나리오 빌드
    pub fn build(self) -> Result<BacktestScenario, TradingError> {
        // 필수 파라미터 검증
        let start_time = self.start_time
          .ok_or_else(|| TradingError::InvalidParameter("시작 시간이 설정되지 않았습니다".into()))?;
        
        let end_time = self.end_time
          .ok_or_else(|| TradingError::InvalidParameter("종료 시간이 설정되지 않았습니다".into()))?;
        
        if start_time >= end_time {
            return Err(TradingError::InvalidParameter("시작 시간은 종료 시간보다 이전이어야 합니다".into()));
        }
        
        if self.strategies.is_empty() {
            return Err(TradingError::InvalidParameter("최소 하나의 전략이 필요합니다".into()));
        }
        
        // 백테스트 엔진 생성
        let mut engine = BacktestEngine::new(
            self.name.clone(),
            self.description.clone(),
            start_time,
            end_time,
            self.initial_balance.clone(),
            self.fee_rate,
            self.slippage,
        );
        
        // 데이터 제공자 설정
        if let Some(data_file) = self.data_file {
            let provider = CsvDataProvider::new(
                data_file,
                self.csv_delimiter,
            )?;
            
            engine.set_data_provider(provider);
        } else if !self.symbols.is_empty() {
            // 심볼만 지정된 경우 기본 데이터 제공자 필요
            return Err(TradingError::InvalidParameter("데이터 파일 또는 데이터 제공자가 필요합니다".into()));
        }
        
        // 전략 추가
        for strategy in self.strategies {
            engine.add_strategy(strategy)?;
        }
        
        // 시나리오 생성
        let scenario = BacktestScenario::new(
            self.name,
            self.description,
            engine,
        );
        
        Ok(scenario)
    }
}