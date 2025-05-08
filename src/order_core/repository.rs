use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::error::TradingError;
use crate::models::order::{Order, OrderId, OrderStatus};

/// 주문 저장소 인터페이스
#[async_trait]
pub trait OrderRepository: Send + Sync {
    /// 주문 저장
    async fn save(&mut self, order: &Order) -> Result<(), TradingError>;

    /// 주문 업데이트
    async fn update(&mut self, order: &Order) -> Result<(), TradingError>;

    /// ID로 주문 찾기
    async fn find_by_id(&self, order_id: &OrderId) -> Result<Option<Order>, TradingError>;

    /// 클라이언트 ID로 주문 찾기
    async fn find_by_client_id(&self, client_id: &str) -> Result<Option<Order>, TradingError>;

    /// 상태별 주문 찾기
    async fn find_by_status(&self, statuses: &[OrderStatus]) -> Result<Vec<Order>, TradingError>;

    /// 심볼별 주문 찾기
    async fn find_by_symbol(&self, symbol: &str) -> Result<Vec<Order>, TradingError>;

    /// 모든 주문 가져오기
    async fn find_all(&self) -> Result<Vec<Order>, TradingError>;

    /// 주문 삭제
    async fn delete(&mut self, order_id: &OrderId) -> Result<(), TradingError>;
}

/// 메모리 기반 주문 저장소 구현
pub struct InMemoryOrderRepository {
    orders: HashMap<String, Order>,  // OrderId.0 -> Order
    client_id_index: HashMap<String, String>,  // client_order_id -> OrderId.0
}

impl InMemoryOrderRepository {
    pub fn new() -> Self {
        InMemoryOrderRepository {
            orders: HashMap::new(),
            client_id_index: HashMap::new(),
        }
    }
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn save(&mut self, order: &Order) -> Result<(), TradingError> {
        // ID 인덱스 저장
        let id_key = order.id.0.clone();

        // 클라이언트 ID 인덱스 저장
        if let Some(client_id) = &order.client_order_id {
            self.client_id_index.insert(client_id.clone(), id_key.clone());
        }

        // 주문 저장
        self.orders.insert(id_key, order.clone());

        Ok(())
    }

    async fn update(&mut self, order: &Order) -> Result<(), TradingError> {
        let id_key = order.id.0.clone();

        if !self.orders.contains_key(&id_key) {
            return Err(TradingError::OrderNotFound(order.id.clone()));
        }

        // 주문 업데이트
        self.orders.insert(id_key.clone(), order.clone());

        // 클라이언트 ID 인덱스 업데이트
        if let Some(client_id) = &order.client_order_id {
            self.client_id_index.insert(client_id.clone(), id_key);
        }

        Ok(())
    }

    async fn find_by_id(&self, order_id: &OrderId) -> Result<Option<Order>, TradingError> {
        Ok(self.orders.get(&order_id.0).cloned())
    }

    async fn find_by_client_id(&self, client_id: &str) -> Result<Option<Order>, TradingError> {
        if let Some(id_key) = self.client_id_index.get(client_id) {
            return Ok(self.orders.get(id_key).cloned());
        }

        Ok(None)
    }

    async fn find_by_status(&self, statuses: &[OrderStatus]) -> Result<Vec<Order>, TradingError> {
        let filtered: Vec<Order> = self.orders.values()
            .filter(|o| statuses.contains(&OrderStatus::New))  // 이 부분은 실제 주문의 상태를 확인해야 함
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn find_by_symbol(&self, symbol: &str) -> Result<Vec<Order>, TradingError> {
        let filtered: Vec<Order> = self.orders.values()
            .filter(|o| o.symbol == symbol)
            .cloned()
            .collect();

        Ok(filtered)
    }

    async fn find_all(&self) -> Result<Vec<Order>, TradingError> {
        let all: Vec<Order> = self.orders.values().cloned().collect();
        Ok(all)
    }

    async fn delete(&mut self, order_id: &OrderId) -> Result<(), TradingError> {
        let id_key = &order_id.0;

        if let Some(order) = self.orders.remove(id_key) {
            if let Some(client_id) = order.client_order_id {
                self.client_id_index.remove(&client_id);
            }
        }

        Ok(())
    }
}