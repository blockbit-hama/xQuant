use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Duration, Utc};

use crate::backtest::data_provider::{BacktestDataProvider, CsvDataProvider, MultiCsvDataProvider};
use crate::backtest::engine::BacktestEngine;
use crate::backtest::result::BacktestResult;
use crate::error::TradingError;
use crate::strategies::Strategy;

/// 백테스트 시나리오
pub struct BacktestScenario {
    name: String,
    description: String,
    data_files: Vec<PathBuf>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    initial_balance: HashMap<String, f64>,
    fee_rate: f64,
    slippage: f64,
    strategies: Vec<Box<dyn Strategy>>,
}

impl BacktestScenario {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        BacktestScenario {
            name: name.into(),
            description: description.into(),
            data_files: Vec::new(),
            start_time,
            end_time,
            initial_balance: HashMap::new(),
            fee_rate: 0.001, // 0.1% 기본 수수료
            slippage: 0.0005, // 0.05% 기본 슬리피지
            strategies: Vec::new(),
        }
    }

    /// 데이터 파일 추가
    pub fn add_data_file(&mut self, file_path: PathBuf) -> &mut Self {
        self.data_files.push(file_path);
        self
    }

    /// 초기 잔고 설정
    pub fn set_initial_balance(&mut self, currency: impl Into<String>, amount: f64) -> &mut Self {
        self.initial_balance.insert(currency.into(), amount);
        self
    }

    /// 수수료율 설정
    pub fn set_fee_rate(&mut self, fee_rate: f64) -> &mut Self {
        self.fee_rate = fee_rate;
        self
    }

    /// 슬리피지 설정
    pub fn set_slippage(&mut self, slippage: f64) -> &mut Self {
        self.slippage = slippage;
        self
    }

    /// 전략 추가
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) -> &mut Self {
        self.strategies.push(strategy);
        self
    }

    /// 시나리오 실행
    pub async fn run(&mut self) -> Result<BacktestResult, TradingError> {
        // 데이터 제공자 생성
        let data_provider: Box<dyn BacktestDataProvider> = if self.data_files.len() == 1 {
            Box::new(CsvDataProvider::new(&self.data_files[0])?)
        } else {
            Box::new(MultiCsvDataProvider::new(self.data_files.clone()))
        };

        // 백테스트 엔진 생성
        let mut engine = BacktestEngine::new(
            data_provider,
            self.start_time,
            self.end_time,
            self.initial_balance.clone(),
            self.fee_rate,
            self.slippage,
        );

        // 전략 추가
        for strategy in self.strategies.drain(..) {
            engine.add_strategy(strategy);
        }

        // 백테스트 실행
        engine.run().await
    }
}

/// 시나리오 빌더 패턴
pub struct BacktestScenarioBuilder {
    name: String,
    description: String,
    data_files: Vec<PathBuf>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    initial_balance: HashMap<String, f64>,
    fee_rate: f64,
    slippage: f64,
    strategies: Vec<Box<dyn Strategy>>,
}

impl BacktestScenarioBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        BacktestScenarioBuilder {
            name: name.into(),
            description: String::new(),
            data_files: Vec::new(),
            start_time: None,
            end_time: None,
            initial_balance: HashMap::new(),
            fee_rate: 0.001,
            slippage: 0.0005,
            strategies: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn data_file(mut self, file_path: PathBuf) -> Self {
        self.data_files.push(file_path);
        self
    }

    pub fn period(mut self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        self
    }

    pub fn last_days(mut self, days: i64) -> Self {
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(days);
        self.start_time = Some(start_time);
        self.end_time = Some(end_time);
        self
    }

    pub fn initial_balance(mut self, currency: impl Into<String>, amount: f64) -> Self {
        self.initial_balance.insert(currency.into(), amount);
        self
    }

    pub fn fee_rate(mut self, fee_rate: f64) -> Self {
        self.fee_rate = fee_rate;
        self
    }

    pub fn slippage(mut self, slippage: f64) -> Self {
        self.slippage = slippage;
        self
    }

    pub fn strategy(mut self, strategy: Box<dyn Strategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn build(self) -> Result<BacktestScenario, TradingError> {
        let start_time = self.start_time.ok_or_else(||
            TradingError::InvalidParameter("Start time not set".to_string())
        )?;

        let end_time = self.end_time.ok_or_else(||
            TradingError::InvalidParameter("End time not set".to_string())
        )?;

        if self.data_files.is_empty() {
            return Err(TradingError::InvalidParameter("No data files specified".to_string()));
        }

        if self.initial_balance.is_empty() {
            return Err(TradingError::InvalidParameter("Initial balance not set".to_string()));
        }

        if self.strategies.is_empty() {
            return Err(TradingError::InvalidParameter("No strategies specified".to_string()));
        }

        let mut scenario = BacktestScenario::new(
            self.name,
            self.description,
            start_time,
            end_time,
        );

        for file in self.data_files {
            scenario.add_data_file(file);
        }

        for (currency, amount) in self.initial_balance {
            scenario.set_initial_balance(currency, amount);
        }

        scenario.set_fee_rate(self.fee_rate);
        scenario.set_slippage(self.slippage);

        for strategy in self.strategies {
            scenario.add_strategy(strategy);
        }

        Ok(scenario)
    }
}