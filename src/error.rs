/**
* filename : lib
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/

use thiserror::Error;

use crate::models::order::OrderId;

#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Order not found: {0}")]
    OrderNotFound(OrderId),

    #[error("Data not found: {0}")]
    DataNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Already running: {0}")]
    AlreadyRunning(String),

    #[error("Exchange error: {0}")]
    ExchangeError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Not subscribed to {0}")]
    NotSubscribed(String),

    #[error("Lock error")]
    LockError,

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("No available provider")]
    NoAvailableProvider,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}