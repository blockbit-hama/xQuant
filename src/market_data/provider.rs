use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};

use crate::models::market_data::MarketData;
use crate::error::TradingError;

/// 시장 데이터 제공자 인터페이스
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// 시장 데이터 구독 시작
    async fn subscribe(&mut self, symbol: &str) -> Result<(), TradingError>;

    /// 시장 데이터 구독 해제
    async fn unsubscribe(&mut self, symbol: &str) -> Result<(), TradingError>;

    /// 시장 데이터 수신 채널 얻기
    fn get_receiver(&self, symbol: &str) -> Result<broadcast::Receiver<MarketData>, TradingError>;

    /// 현재 시장 데이터 조회
    async fn get_current_data(&self, symbol: &str) -> Result<MarketData, TradingError>;

    /// 제공자 연결 상태 확인
    async fn is_connected(&self) -> bool;

    /// 연결 시작
    async fn connect(&mut self) -> Result<(), TradingError>;

    /// 연결 종료
    async fn disconnect(&mut self) -> Result<(), TradingError>;
}

/// 시장 데이터 관리자
pub struct MarketDataManager {
    providers: Vec<Arc<RwLock<dyn MarketDataProvider>>>,
    active_symbols: Vec<String>,
}

impl MarketDataManager {
    pub fn new() -> Self {
        MarketDataManager {
            providers: Vec::new(),
            active_symbols: Vec::new(),
        }
    }

    /// 데이터 제공자 추가
    pub fn add_provider(&mut self, provider: Arc<RwLock<dyn MarketDataProvider>>) {
        self.providers.push(provider);
    }

    /// 모든 제공자에 심볼 구독
    pub async fn subscribe_all(&mut self, symbol: &str) -> Result<(), TradingError> {
        if !self.active_symbols.contains(&symbol.to_string()) {
            self.active_symbols.push(symbol.to_string());
        }

        for provider in &self.providers {
            let mut provider = provider.write().await;
            let _ = provider.subscribe(symbol).await;
        }

        Ok(())
    }

    /// 모든 제공자에서 구독 해제
    pub async fn unsubscribe_all(&mut self, symbol: &str) -> Result<(), TradingError> {
        self.active_symbols.retain(|s| s != symbol);

        for provider in &self.providers {
            let mut provider = provider.write().await;
            let _ = provider.unsubscribe(symbol).await;
        }

        Ok(())
    }

    /// 모든 제공자 연결 시작
    pub async fn connect_all(&mut self) -> Result<(), TradingError> {
        for provider in &self.providers {
            let mut provider = provider.write().await;
            let _ = provider.connect().await;
        }

        // 기존 구독 심볼 다시 구독
        let symbols = self.active_symbols.clone();
        for symbol in &symbols {
            self.subscribe_all(symbol).await?;
        }

        Ok(())
    }

    /// 모든 제공자 연결 종료
    pub async fn disconnect_all(&mut self) -> Result<(), TradingError> {
        for provider in &self.providers {
            let mut provider = provider.write().await;
            let _ = provider.disconnect().await;
        }

        Ok(())
    }

    /// 시장 데이터 수신기 얻기 (첫 번째 가용 제공자 사용)
    pub async fn get_receiver(&self, symbol: &str) -> Result<broadcast::Receiver<MarketData>, TradingError> {
        for provider in &self.providers {
            let provider = provider.read().await;
            if provider.is_connected().await {
                return provider.get_receiver(symbol);
            }
        }

        Err(TradingError::NoAvailableProvider)
    }

    /// 현재 시장 데이터 얻기 (첫 번째 가용 제공자 사용)
    pub async fn get_current_data(&self, symbol: &str) -> Result<MarketData, TradingError> {
        for provider in &self.providers {
            let provider = provider.read().await;
            if provider.is_connected().await {
                return provider.get_current_data(symbol).await;
            }
        }

        Err(TradingError::NoAvailableProvider)
    }
}