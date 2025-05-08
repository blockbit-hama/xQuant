use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::market_data::provider::MarketDataProvider;
use crate::market_data::stream::MarketDataStream;
use crate::models::market_data::MarketData;
use crate::error::TradingError;

/// WebSocket 기반 시장 데이터 제공자
pub struct WebSocketProvider {
    url: String,
    stream: Arc<RwLock<MarketDataStream>>,
    subscriptions: HashMap<String, String>,  // 심볼 -> 구독 ID
    connected: bool,
    ws_task: Option<JoinHandle<()>>,
    reconnect_interval: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionRequest {
    method: String,
    params: Vec<String>,
    id: u64,
}

impl WebSocketProvider {
    pub fn new(url: impl Into<String>, stream: Arc<RwLock<MarketDataStream>>) -> Self {
        WebSocketProvider {
            url: url.into(),
            stream,
            subscriptions: HashMap::new(),
            connected: false,
            ws_task: None,
            reconnect_interval: Duration::from_secs(5),
        }
    }

    async fn start_websocket(&mut self) -> Result<(), TradingError> {
        let url = self.url.clone();
        let stream_clone = self.stream.clone();
        let subscriptions_clone = self.subscriptions.clone();

        let ws_task = tokio::spawn(async move {
            loop {
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        let (mut write, mut read) = ws_stream.split();

                        // 기존 구독 재설정
                        for (symbol, _) in &subscriptions_clone {
                            let sub_msg = SubscriptionRequest {
                                method: "SUBSCRIBE".to_string(),
                                params: vec![format!("{}@ticker", symbol.to_lowercase())],
                                id: rand::random::<u64>(),
                            };

                            let msg = serde_json::to_string(&sub_msg).unwrap();
                            let _ = write.send(Message::Text(msg)).await;
                        }

                        // 메시지 처리 루프
                        while let Some(msg_result) = read.next().await {
                            match msg_result {
                                Ok(msg) => {
                                    if let Message::Text(text) = msg {
                                        // 메시지 파싱 및 처리 (예시 - 실제 구현은 거래소별 포맷에 맞게 조정 필요)
                                        if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                            if let Some(data) = parse_market_data(json) {
                                                let mut stream = stream_clone.write().await;
                                                let _ = stream.publish(data);
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    log::error!("WebSocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to connect to WebSocket: {}", e);
                    }
                }

                // 재연결 대기
                tokio::time::sleep(Duration::from_secs(5)).await;
                log::info!("Attempting to reconnect WebSocket...");
            }
        });

        self.ws_task = Some(ws_task);
        self.connected = true;
        Ok(())
    }
}

#[async_trait]
impl MarketDataProvider for WebSocketProvider {
    async fn subscribe(&mut self, symbol: &str) -> Result<(), TradingError> {
        if !self.connected {
            return Err(TradingError::NotConnected);
        }

        if self.subscriptions.contains_key(symbol) {
            return Ok(());
        }

        // 구독 ID 생성
        let sub_id = format!("sub_{}", rand::random::<u64>());

        // 실제 WebSocket 구독 메시지 전송은 start_websocket 내에서 처리
        self.subscriptions.insert(symbol.to_string(), sub_id.clone());

        Ok(())
    }

    async fn unsubscribe(&mut self, symbol: &str) -> Result<(), TradingError> {
        if !self.connected {
            return Err(TradingError::NotConnected);
        }

        if let Some(sub_id) = self.subscriptions.remove(symbol) {
            // 실제 구독 해제 메시지는 start_websocket 내에서 처리
        }

        Ok(())
    }

    fn get_receiver(&self, symbol: &str) -> Result<broadcast::Receiver<MarketData>, TradingError> {
        if !self.subscriptions.contains_key(symbol) {
            return Err(TradingError::NotSubscribed(symbol.to_string()));
        }

        let stream = match self.stream.try_read() {
            Ok(guard) => guard,
            Err(_) => return Err(TradingError::LockError),
        };

        stream.get_receiver(symbol)
    }

    async fn get_current_data(&self, symbol: &str) -> Result<MarketData, TradingError> {
        let stream = self.stream.read().await;

        if let Some(data) = stream.get_latest_data(symbol) {
            Ok(data)
        } else {
            Err(TradingError::DataNotFound(symbol.to_string()))
        }
    }

    async fn is_connected(&self) -> bool {
        self.connected
    }

    async fn connect(&mut self) -> Result<(), TradingError> {
        if self.connected {
            return Ok(());
        }

        self.start_websocket().await
    }

    async fn disconnect(&mut self) -> Result<(), TradingError> {
        if !self.connected {
            return Ok(());
        }

        if let Some(task) = self.ws_task.take() {
            task.abort();
        }

        self.connected = false;
        self.subscriptions.clear();

        Ok(())
    }
}

// WebSocket 메시지를 MarketData로 파싱하는 함수 (거래소별 포맷에 맞게 구현 필요)
fn parse_market_data(json: Value) -> Option<MarketData> {
    // 예: 바이낸스 형식의 메시지 파싱
    if let (Some(symbol), Some(close), Some(high), Some(low), Some(open), Some(volume), Some(time)) = (
        json.get("s").and_then(Value::as_str),
        json.get("c").and_then(Value::as_str).and_then(|s| s.parse::<f64>().ok()),
        json.get("h").and_then(Value::as_str).and_then(|s| s.parse::<f64>().ok()),
        json.get("l").and_then(Value::as_str).and_then(|s| s.parse::<f64>().ok()),
        json.get("o").and_then(Value::as_str).and_then(|s| s.parse::<f64>().ok()),
        json.get("v").and_then(Value::as_str).and_then(|s| s.parse::<f64>().ok()),
        json.get("E").and_then(Value::as_i64),
    ) {
        Some(MarketData {
            symbol: symbol.to_string(),
            timestamp: time,
            open,
            high,
            low,
            close,
            volume,
        })
    } else {
        None
    }
}