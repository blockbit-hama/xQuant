use async_trait::async_trait;

use crate::error::TradingError;
use crate::models::order::Order;

/// 주문 검증기 인터페이스
pub trait OrderValidator: Send + Sync {
    /// 주문 검증
    fn validate(&self, order: &Order) -> Result<(), TradingError>;
}

/// 기본 주문 검증기
pub struct BasicOrderValidator {
    min_order_size: f64,
    max_order_size: f64,
}

impl BasicOrderValidator {
    pub fn new(min_order_size: f64, max_order_size: f64) -> Self {
        BasicOrderValidator {
            min_order_size,
            max_order_size,
        }
    }
}

impl OrderValidator for BasicOrderValidator {
    fn validate(&self, order: &Order) -> Result<(), TradingError> {
        // 주문 수량 검증
        if order.quantity <= 0.0 {
            return Err(TradingError::InvalidParameter("Order quantity must be positive".to_string()));
        }

        if order.quantity < self.min_order_size {
            return Err(TradingError::InvalidParameter(
                format!("Order quantity too small, minimum: {}", self.min_order_size)
            ));
        }

        if order.quantity > self.max_order_size {
            return Err(TradingError::InvalidParameter(
                format!("Order quantity too large, maximum: {}", self.max_order_size)
            ));
        }

        // 가격 검증
        if order.price < 0.0 {
            return Err(TradingError::InvalidParameter("Price cannot be negative".to_string()));
        }

        Ok(())
    }
}

/// 리스크 기반 주문 검증기
pub struct RiskOrderValidator {
    max_position_size: f64,
    max_notional_value: f64,
}

impl RiskOrderValidator {
    pub fn new(max_position_size: f64, max_notional_value: f64) -> Self {
        RiskOrderValidator {
            max_position_size,
            max_notional_value,
        }
    }
}

impl OrderValidator for RiskOrderValidator {
    fn validate(&self, order: &Order) -> Result<(), TradingError> {
        // 포지션 크기 제한 검증
        if order.quantity > self.max_position_size {
            return Err(TradingError::RiskLimitExceeded(
                format!("Order exceeds maximum position size: {}", self.max_position_size)
            ));
        }

        // 명목 가치 제한 검증
        let notional_value = order.quantity * order.price;
        if notional_value > self.max_notional_value {
            return Err(TradingError::RiskLimitExceeded(
                format!("Order exceeds maximum notional value: {}", self.max_notional_value)
            ));
        }

        Ok(())
    }
}