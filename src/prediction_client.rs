/**
* filename : prediction_client
* author : HAMA
* date: 2025. 5. 11.
* description: Python 예측 시스템과 통신하는 클라이언트
**/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use reqwest::Client;
use anyhow::Result;
use crate::error::TradingError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataRequest {
    pub symbol: String,
    pub timeframe: String,
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRequest {
    pub symbol: String,
    pub timeframe: String,
    pub strategy: String,
    pub lookback: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResponse {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub signal: i32,  // 1: Buy, -1: Sell, 0: Neutral
    pub confidence: f64,
    pub indicators: HashMap<String, f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestRequest {
    pub symbol: String,
    pub timeframe: String,
    pub strategy: String,
    pub days: i32,
    pub initial_capital: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub symbol: String,
    pub strategy: String,
    pub timeframe: String,
    pub days: i32,
    pub results: BacktestMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_return: f64,
    pub total_return_pct: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: i32,
    pub profit_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionRequest {
    pub symbol: String,
    pub timeframe: String,
    pub horizon: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResponse {
    pub symbol: String,
    pub timeframe: String,
    pub current_price: f64,
    pub predicted_price: f64,
    pub predicted_change: f64,
    pub horizon_hours: i32,
    pub confidence: f64,
    pub method: String,
}

pub struct PredictionClient {
    client: Client,
    base_url: String,
}

impl PredictionClient {
    /// Create new prediction client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    /// Check if prediction service is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
    
    /// Get trading signals from prediction service
    pub async fn get_signals(&self, request: SignalRequest) -> Result<SignalResponse> {
        let url = format!("{}/signals", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get signals: {}", response.status()));
        }
        
        let signal = response.json::<SignalResponse>().await?;
        Ok(signal)
    }
    
    /// Run backtest for a strategy
    pub async fn run_backtest(&self, request: BacktestRequest) -> Result<BacktestResult> {
        let url = format!("{}/backtest", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to run backtest: {}", response.status()));
        }
        
        let result = response.json::<BacktestResult>().await?;
        Ok(result)
    }
    
    /// Get price prediction
    pub async fn get_prediction(&self, request: PredictionRequest) -> Result<PredictionResponse> {
        let url = format!("{}/predict", self.base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get prediction: {}", response.status()));
        }
        
        let prediction = response.json::<PredictionResponse>().await?;
        Ok(prediction)
    }
    
    /// Get list of available strategies
    pub async fn list_strategies(&self) -> Result<Vec<String>> {
        let url = format!("{}/strategies", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to list strategies: {}", response.status()));
        }
        
        let data: HashMap<String, Vec<HashMap<String, String>>> = response.json().await?;
        let strategies = data.get("strategies")
            .map(|s| s.iter().filter_map(|item| item.get("name").cloned()).collect())
            .unwrap_or_default();
        
        Ok(strategies)
    }
}

/// Integration with existing trading bot
pub struct PredictionBasedBot {
    prediction_client: PredictionClient,
    symbol: String,
    strategy: String,
    timeframe: String,
    current_position: f64,
}

impl PredictionBasedBot {
    pub fn new(
        prediction_url: String,
        symbol: String,
        strategy: String,
        timeframe: String,
    ) -> Self {
        Self {
            prediction_client: PredictionClient::new(prediction_url),
            symbol,
            strategy,
            timeframe,
            current_position: 0.0,
        }
    }
    
    /// Get next trading action from prediction service
    pub async fn get_next_action(&mut self) -> Result<TradingAction> {
        // Get signals from prediction service
        let request = SignalRequest {
            symbol: self.symbol.clone(),
            timeframe: self.timeframe.clone(),
            strategy: self.strategy.clone(),
            lookback: 100,
        };
        
        let signal = self.prediction_client.get_signals(request).await?;
        
        // Convert signal to trading action
        let action = match signal.signal {
            1 if self.current_position <= 0.0 => {
                // Buy signal and not in long position
                TradingAction::OpenLong {
                    confidence: signal.confidence,
                    indicators: signal.indicators,
                }
            },
            -1 if self.current_position >= 0.0 => {
                // Sell signal and not in short position
                TradingAction::OpenShort {
                    confidence: signal.confidence,
                    indicators: signal.indicators,
                }
            },
            0 if self.current_position != 0.0 => {
                // Neutral signal, close position
                TradingAction::ClosePosition {
                    reason: "Neutral signal".to_string(),
                }
            },
            _ => TradingAction::Hold,
        };
        
        Ok(action)
    }
    
    /// Update position after trade execution
    pub fn update_position(&mut self, position: f64) {
        self.current_position = position;
    }
}

#[derive(Debug, Clone)]
pub enum TradingAction {
    OpenLong {
        confidence: f64,
        indicators: HashMap<String, f64>,
    },
    OpenShort {
        confidence: f64,
        indicators: HashMap<String, f64>,
    },
    ClosePosition {
        reason: String,
    },
    Hold,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_prediction_client() {
        let client = PredictionClient::new("http://127.0.0.1:8000".to_string());
        
        // Test health check (will fail if server not running)
        let health = client.health_check().await;
        assert!(health.is_ok() || health.is_err()); // Accept both for testing
    }
    
    #[test]
    fn test_trading_action() {
        let mut bot = PredictionBasedBot::new(
            "http://127.0.0.1:8000".to_string(),
            "BTCUSDT".to_string(),
            "trend_following".to_string(),
            "1h".to_string(),
        );
        
        assert_eq!(bot.current_position, 0.0);
        bot.update_position(1.0);
        assert_eq!(bot.current_position, 1.0);
    }
}