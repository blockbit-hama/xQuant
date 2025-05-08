use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::Rng;
use uuid::Uuid;

use crate::config::Config;
use crate::error::TradingError;
use crate::exchange::traits::Exchange;
use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderId, OrderSide, OrderStatus, OrderType};
use crate::models::trade::Trade;

/// A mock implementation of the Exchange trait for testing and development
pub struct MockExchange {
    config: Config,
    orders: HashMap<OrderId, (Order, OrderStatus)>,
    market_data: HashMap<String, Vec<MarketData>>,
    balances: HashMap<String, f64>,
    trades: HashMap<String, Vec<Trade>>,
    order_id_counter: u64,
}

impl MockExchange {
    pub fn new(config: Config) -> Self {
        let mut exchange = Self {
            config,
            orders: HashMap::new(),
            market_data: HashMap::new(),
            balances: HashMap::new(),
            trades: HashMap::new(),
            order_id_counter: 0,
        };

        // Initialize with some test data
        exchange.initialize_test_data();
        exchange
    }

    fn initialize_test_data(&mut self) {
        // Add some initial balances
        self.balances.insert("BTC".to_string(), 10.0);
        self.balances.insert("ETH".to_string(), 100.0);
        self.balances.insert("USDT".to_string(), 50000.0);

        // Create some mock market data for BTC/USDT
        let symbol = "BTCUSDT".to_string();
        let now = Utc::now();
        let mut market_data = Vec::new();
        let mut last_price = 50000.0;
        let mut volume = 0.0;

        for i in 0..1000 {
            let timestamp = now - chrono::Duration::minutes(i);
            let price_change = (rand::thread_rng().gen_range(-200.0..200.0)) / 100.0;
            last_price = f64::max(f64::min(last_price * (1.0 + price_change), 100000.0), 10000.0);
            volume += rand::thread_rng().gen_range(0.1..10.0);

            market_data.push(MarketData {
                symbol: symbol.clone(),
                timestamp: timestamp.timestamp_millis(),
                open: last_price * (1.0 - 0.001),
                high: last_price * (1.0 + 0.002),
                low: last_price * (1.0 - 0.002),
                close: last_price,
                volume,
            });
        }

        self.market_data.insert(symbol, market_data);

        // Create similar data for ETH/USDT
        let symbol = "ETHUSDT".to_string();
        let mut market_data = Vec::new();
        let mut last_price = 3000.0;
        let mut volume = 0.0;

        for i in 0..1000 {
            let timestamp = now - chrono::Duration::minutes(i);
            let price_change = (rand::thread_rng().gen_range(-200.0..200.0)) / 100.0;
            last_price = f64::max(f64::min(last_price * (1.0 + price_change), 100000.0), 10000.0);
            volume += rand::thread_rng().gen_range(1.0..20.0);

            market_data.push(MarketData {
                symbol: symbol.clone(),
                timestamp: timestamp.timestamp_millis(),
                open: last_price * (1.0 - 0.001),
                high: last_price * (1.0 + 0.002),
                low: last_price * (1.0 - 0.002),
                close: last_price,
                volume,
            });
        }

        self.market_data.insert(symbol, market_data);
    }

    fn generate_order_id(&mut self) -> OrderId {
        self.order_id_counter += 1;
        OrderId(format!("mock-{}", self.order_id_counter))
    }

    // Simulates execution of an order
    fn process_order(&mut self, order: &Order) -> Result<(), TradingError> {
        // Simple simulation - immediate fill for market orders, partial fills for limit orders
        let symbol = &order.symbol;
        let latest_market_data = self.get_latest_market_data(symbol)?;

        match order.order_type {
            OrderType::Market => {
                // Create a trade for the market order
                let trade = Trade {
                    id: Uuid::new_v4().to_string(),
                    symbol: symbol.clone(),
                    price: latest_market_data.close,
                    quantity: order.quantity,
                    timestamp: Utc::now().timestamp_millis(),
                    order_id: order.id.clone(),
                    side: order.side.clone(),
                };

                // Update balances based on the trade
                self.update_balances(&trade)?;

                // Store the trade
                self.trades
                    .entry(symbol.clone())
                    .or_insert_with(Vec::new)
                    .push(trade);
            },
            OrderType::Limit => {
                // For simplicity, partially fill limit orders
                if (order.side == OrderSide::Buy && order.price >= latest_market_data.close) ||
                    (order.side == OrderSide::Sell && order.price <= latest_market_data.close) {
                    // Partial fill (50%)
                    let filled_quantity = order.quantity * 0.5;
                    let trade = Trade {
                        id: Uuid::new_v4().to_string(),
                        symbol: symbol.clone(),
                        price: if order.side == OrderSide::Buy {
                            order.price
                        } else {
                            latest_market_data.close
                        },
                        quantity: filled_quantity,
                        timestamp: Utc::now().timestamp_millis(),
                        order_id: order.id.clone(),
                        side: order.side.clone(),
                    };

                    // Update balances
                    self.update_balances(&trade)?;

                    // Store the trade
                    self.trades
                        .entry(symbol.clone())
                        .or_insert_with(Vec::new)
                        .push(trade);
                }
            },
            // Handle other order types as needed
            _ => {},
        }

        Ok(())
    }

    fn update_balances(&mut self, trade: &Trade) -> Result<(), TradingError> {
        // Extract the base and quote currencies from the symbol
        // Assuming symbols are in the format BTCUSDT, ETHUSDT, etc.
        let base_asset = &trade.symbol[0..3];
        let quote_asset = &trade.symbol[3..];

        match trade.side {
            OrderSide::Buy => {
                // Increase base asset, decrease quote asset
                *self.balances.entry(base_asset.to_string()).or_insert(0.0) += trade.quantity;
                *self.balances.entry(quote_asset.to_string()).or_insert(0.0) -= trade.quantity * trade.price;
            },
            OrderSide::Sell => {
                // Decrease base asset, increase quote asset
                *self.balances.entry(base_asset.to_string()).or_insert(0.0) -= trade.quantity;
                *self.balances.entry(quote_asset.to_string()).or_insert(0.0) += trade.quantity * trade.price;
            },
        }

        Ok(())
    }

    fn get_latest_market_data(&self, symbol: &str) -> Result<MarketData, TradingError> {
        if let Some(data) = self.market_data.get(symbol) {
            if let Some(latest) = data.first() {
                return Ok(latest.clone());
            }
        }

        Err(TradingError::DataNotFound(format!("No market data for {}", symbol)))
    }
}

#[async_trait]
impl Exchange for MockExchange {
    async fn submit_order(&mut self, mut order: Order) -> Result<OrderId, TradingError> {
        let order_id = self.generate_order_id();
        order.id = order_id.clone();

        // Process the order (execution simulation)
        self.process_order(&order)?;

        // Store the order with its status
        let status = if order.order_type == OrderType::Market {
            OrderStatus::Filled
        } else {
            OrderStatus::PartiallyFilled
        };

        self.orders.insert(order_id.clone(), (order, status));

        Ok(order_id)
    }

    async fn cancel_order(&mut self, order_id: &OrderId) -> Result<(), TradingError> {
        if let Some((_, status)) = self.orders.get_mut(order_id) {
            *status = OrderStatus::Cancelled;
            Ok(())
        } else {
            Err(TradingError::OrderNotFound(order_id.clone()))
        }
    }

    async fn modify_order(
        &mut self,
        order_id: &OrderId,
        mut new_order: Order,
    ) -> Result<OrderId, TradingError> {
        if self.orders.contains_key(order_id) {
            // Cancel the old order
            self.cancel_order(order_id).await?;

            // Create a new order with the updated parameters
            new_order.id = self.generate_order_id();
            self.submit_order(new_order).await
        } else {
            Err(TradingError::OrderNotFound(order_id.clone()))
        }
    }

    async fn get_order_status(&self, order_id: &OrderId) -> Result<OrderStatus, TradingError> {
        if let Some((_, status)) = self.orders.get(order_id) {
            Ok(status.clone())
        } else {
            Err(TradingError::OrderNotFound(order_id.clone()))
        }
    }

    async fn get_open_orders(&self) -> Result<Vec<Order>, TradingError> {
        let open_orders = self
            .orders
            .iter()
            .filter(|(_, (_, status))| {
                *status == OrderStatus::New || *status == OrderStatus::PartiallyFilled
            })
            .map(|(_, (order, _))| order.clone())
            .collect();

        Ok(open_orders)
    }

    async fn get_recent_trades(
        &self,
        symbol: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Trade>, TradingError> {
        let limit = limit.unwrap_or(100);

        if let Some(trades) = self.trades.get(symbol) {
            let mut recent_trades = trades.clone();
            recent_trades.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            Ok(recent_trades.into_iter().take(limit).collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_market_data(&self, symbol: &str) -> Result<MarketData, TradingError> {
        self.get_latest_market_data(symbol)
    }

    async fn get_historical_data(
        &self,
        symbol: &str,
        _interval: &str,
        start_time: i64,
        end_time: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<MarketData>, TradingError> {
        let end_time = end_time.unwrap_or(Utc::now().timestamp_millis());
        let limit = limit.unwrap_or(1000);

        if let Some(data) = self.market_data.get(symbol) {
            let filtered_data = data
                .iter()
                .filter(|d| d.timestamp >= start_time && d.timestamp <= end_time)
                .take(limit)
                .cloned()
                .collect();

            Ok(filtered_data)
        } else {
            Err(TradingError::DataNotFound(format!("No data for {}", symbol)))
        }
    }

    async fn get_balance(&self, asset: &str) -> Result<f64, TradingError> {
        if let Some(balance) = self.balances.get(asset) {
            Ok(*balance)
        } else {
            Ok(0.0) // Asset not found, return zero balance
        }
    }
}