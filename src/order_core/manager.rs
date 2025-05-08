use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Mutex};
use uuid::Uuid;

use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::order::{Order, OrderId, OrderStatus, OrderType, OrderSide};
use crate::order_core::repository::OrderRepository;
use crate::order_core::validator::OrderValidator;

/// 주문 관리자 - 주문 생명주기 관리
pub struct OrderManager {
    exchange: Arc<RwLock<dyn Exchange>>,
    repository: Arc<RwLock<dyn OrderRepository>>,
    validators: Vec<Box<dyn OrderValidator>>,
    status_channels: HashMap<String, broadcast::Sender<OrderStatus>>,
}

impl OrderManager {
    pub fn new(
        exchange: Arc<RwLock<dyn Exchange>>,
        repository: Arc<RwLock<dyn OrderRepository>>,
    ) -> Self {
        OrderManager {
            exchange,
            repository,
            validators: Vec::new(),
            status_channels: HashMap::new(),
        }
    }

    /// 주문 검증기 추가
    pub fn add_validator(&mut self, validator: Box<dyn OrderValidator>) {
        self.validators.push(validator);
    }

    /// 주문 생성 및 제출
    pub async fn create_order(&self, mut order: Order) -> Result<OrderId, TradingError> {
        // 주문 검증
        for validator in &self.validators {
            validator.validate(&order)?;
        }

        // 클라이언트 ID 설정 (없을 경우)
        if order.client_order_id.is_none() {
            order.client_order_id = Some(Uuid::new_v4().to_string());
        }

        // 주문 저장소에 임시 저장
        {
            let mut repo = self.repository.write().await;
            repo.save(&order).await?;
        }

        // 주문 제출
        let order_id = {
            let mut exchange = self.exchange.write().await;
            exchange.submit_order(order.clone()).await?
        };

        // 주문 ID 업데이트
        {
            let mut repo = self.repository.write().await;
            let mut updated_order = order.clone();
            updated_order.id = order_id.clone();
            repo.update(&updated_order).await?;
        }

        Ok(order_id)
    }

    /// 주문 취소
    pub async fn cancel_order(&self, order_id: &OrderId) -> Result<(), TradingError> {
        // 주문 존재 여부 확인
        {
            let repo = self.repository.read().await;
            if repo.find_by_id(order_id).await?.is_none() {
                return Err(TradingError::OrderNotFound(order_id.clone()));
            }
        }

        // 주문 취소 요청
        {
            let mut exchange = self.exchange.write().await;
            exchange.cancel_order(order_id).await?;
        }

        // 주문 상태 업데이트
        {
            let mut repo = self.repository.write().await;
            if let Some(mut order) = repo.find_by_id(order_id).await? {
                // 상태 변경
                let status = OrderStatus::Cancelled;

                // 상태 채널 알림
                if let Some(client_id) = order.client_order_id.as_ref() {
                    if let Some(sender) = self.status_channels.get(client_id) {
                        let _ = sender.send(status.clone());
                    }
                }

                // 주문 업데이트
                order.id = order_id.clone();
                repo.update(&order).await?;
            }
        }

        Ok(())
    }

    /// 주문 수정
    pub async fn modify_order(&self, order_id: &OrderId, new_params: Order) -> Result<OrderId, TradingError> {
        // 주문 존재 여부 확인
        let original_order = {
            let repo = self.repository.read().await;
            if let Some(order) = repo.find_by_id(order_id).await? {
                order
            } else {
                return Err(TradingError::OrderNotFound(order_id.clone()));
            }
        };

        // 주문 수정 요청
        let new_order_id = {
            let mut exchange = self.exchange.write().await;
            exchange.modify_order(order_id, new_params.clone()).await?
        };

        // 주문 상태 업데이트
        {
            let mut repo = self.repository.write().await;

            // 원래 주문 취소 상태로 변경
            let mut cancelled_order = original_order.clone();
            let status = OrderStatus::Cancelled;

            // 상태 채널 알림
            if let Some(client_id) = cancelled_order.client_order_id.as_ref() {
                if let Some(sender) = self.status_channels.get(client_id) {
                    let _ = sender.send(status.clone());
                }
            }

            // 새 주문 저장
            let mut new_order = new_params.clone();
            new_order.id = new_order_id.clone();
            repo.save(&new_order).await?;
        }

        Ok(new_order_id)
    }

    /// 주문 상태 조회
    pub async fn get_order_status(&self, order_id: &OrderId) -> Result<OrderStatus, TradingError> {
        // 거래소에서 최신 상태 확인
        let exchange_status = {
            let exchange = self.exchange.read().await;
            exchange.get_order_status(order_id).await?
        };

        // 주문 저장소 업데이트
        {
            let mut repo = self.repository.write().await;
            if let Some(mut order) = repo.find_by_id(order_id).await? {
                // 상태 채널 알림
                if let Some(client_id) = order.client_order_id.as_ref() {
                    if let Some(sender) = self.status_channels.get(client_id) {
                        let _ = sender.send(exchange_status.clone());
                    }
                }
            }
        }

        Ok(exchange_status)
    }

    /// 미체결 주문 조회
    pub async fn get_open_orders(&self) -> Result<Vec<Order>, TradingError> {
        let exchange = self.exchange.read().await;
        exchange.get_open_orders().await
    }

    /// 주문 상태 변경 알림 구독
    pub fn subscribe_to_status_updates(&mut self, client_order_id: &str) -> broadcast::Receiver<OrderStatus> {
        if let Some(sender) = self.status_channels.get(client_order_id) {
            sender.subscribe()
        } else {
            let (sender, receiver) = broadcast::channel(100);
            self.status_channels.insert(client_order_id.to_string(), sender);
            receiver
        }
    }

    /// 주문 상태 감시 시작
    pub async fn start_order_monitoring(&self) -> Result<(), TradingError> {
        let exchange_clone = self.exchange.clone();
        let repository_clone = self.repository.clone();
        let status_channels_clone = self.status_channels.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                // 미체결 주문 가져오기
                let repo = repository_clone.read().await;
                let open_orders = match repo.find_by_status(&[OrderStatus::New, OrderStatus::PartiallyFilled]).await {
                    Ok(orders) => orders,
                    Err(_) => continue,
                };

                // 각 주문 상태 확인
                for order in open_orders {
                    let exchange = exchange_clone.read().await;
                    if let Ok(status) = exchange.get_order_status(&order.id).await {
                        drop(exchange);

                        // 상태가 변경된 경우 업데이트
                        let mut repo = repository_clone.write().await;
                        let mut updated_order = order.clone();
                        updated_order.id = order.id.clone();

                        // 상태 채널 알림
                        if let Some(client_id) = order.client_order_id.as_ref() {
                            if let Some(sender) = status_channels_clone.get(client_id) {
                                let _ = sender.send(status.clone());
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchange::mocks::MockExchange;
    use crate::order_core::repository::InMemoryOrderRepository;

    #[tokio::test]
    async fn test_order_lifecycle() {
        // 테스트 환경 설정
        let config = crate::config::Config::default();
        let exchange = Arc::new(RwLock::new(MockExchange::new(config)));
        let repository = Arc::new(RwLock::new(InMemoryOrderRepository::new()));
        let manager = OrderManager::new(exchange, repository);

        // 주문 생성
        let order = Order::new(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Limit,
            0.1,
            50000.0,
        );

        let order_id = manager.create_order(order).await.unwrap();

        // 주문 상태 확인
        let status = manager.get_order_status(&order_id).await.unwrap();
        assert!(status == OrderStatus::PartiallyFilled || status == OrderStatus::New);

        // 주문 취소
        manager.cancel_order(&order_id).await.unwrap();

        // 취소 확인
        let status = manager.get_order_status(&order_id).await.unwrap();
        assert_eq!(status, OrderStatus::Cancelled);
    }
}