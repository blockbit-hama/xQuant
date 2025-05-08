use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Mutex};
use tokio::task::JoinHandle;

use crate::models::market_data::MarketData;
use crate::error::TradingError;

/// 시장 데이터 스트림 처리기
pub struct MarketDataStream {
    channels: HashMap<String, broadcast::Sender<MarketData>>,
    latest_data: HashMap<String, MarketData>,
    buffer_size: usize,
    aggregation_tasks: HashMap<String, JoinHandle<()>>,
}

impl MarketDataStream {
    pub fn new(buffer_size: usize) -> Self {
        MarketDataStream {
            channels: HashMap::new(),
            latest_data: HashMap::new(),
            buffer_size,
            aggregation_tasks: HashMap::new(),
        }
    }

    /// 심볼 채널 생성 또는 가져오기
    pub fn get_or_create_channel(&mut self, symbol: &str) -> broadcast::Sender<MarketData> {
        if let Some(sender) = self.channels.get(symbol) {
            sender.clone()
        } else {
            let (sender, _) = broadcast::channel(self.buffer_size);
            self.channels.insert(symbol.to_string(), sender.clone());
            sender
        }
    }

    /// 데이터 추가 및 브로드캐스트
    pub fn publish(&mut self, data: MarketData) -> Result<(), TradingError> {
        let symbol = data.symbol.clone();

        // 최신 데이터 업데이트
        self.latest_data.insert(symbol.clone(), data.clone());

        // 채널에 데이터 전송
        if let Some(sender) = self.channels.get(&symbol) {
            let _ = sender.send(data);
            Ok(())
        } else {
            Err(TradingError::ChannelNotFound(symbol))
        }
    }

    /// 최신 시장 데이터 조회
    pub fn get_latest_data(&self, symbol: &str) -> Option<MarketData> {
        self.latest_data.get(symbol).cloned()
    }

    /// 데이터 수신기 얻기
    pub fn get_receiver(&self, symbol: &str) -> Result<broadcast::Receiver<MarketData>, TradingError> {
        if let Some(sender) = self.channels.get(symbol) {
            Ok(sender.subscribe())
        } else {
            Err(TradingError::ChannelNotFound(symbol.to_string()))
        }
    }

    /// 집계 작업 시작 (예: 1분봉 생성)
    pub fn start_aggregation(&mut self, symbol: &str, interval: u64) -> Result<(), TradingError> {
        if self.aggregation_tasks.contains_key(symbol) {
            return Err(TradingError::AlreadyRunning(format!("Aggregation for {} already running", symbol)));
        }

        let receiver = self.get_receiver(symbol)?;
        let task = tokio::spawn(async move {
            // 집계 로직 구현 (생략)
        });

        self.aggregation_tasks.insert(symbol.to_string(), task);
        Ok(())
    }

    /// 집계 작업 중지
    pub fn stop_aggregation(&mut self, symbol: &str) -> Result<(), TradingError> {
        if let Some(task) = self.aggregation_tasks.remove(symbol) {
            task.abort();
            Ok(())
        } else {
            Err(TradingError::TaskNotFound(format!("Aggregation task for {} not found", symbol)))
        }
    }
}