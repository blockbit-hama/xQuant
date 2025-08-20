# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

xQuant is a Rust-based automated trading system that implements various order execution strategies with technical analysis capabilities. The system uses async runtime with Tokio, and is designed with a modular architecture.

## Build and Development Commands

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Run specific test module
cargo test vwap_tests
cargo test iceberg_tests
cargo test trailing_stop_tests
cargo test backtest_tests

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code (when analyzer/optimizer modules are fixed)
cargo clippy

# Run the main application
cargo run

# Run in backtest mode
cargo run -- backtest
```

## Known Issues to Fix

- Missing modules: `src/backtest/analyzer.rs` and `src/backtest/optimizer.rs` are referenced but don't exist
- Duplicate module name `performance` in backtest module

## High-Level Architecture

### Core Trading System
- **models/**: Core data structures (Order, Trade, MarketData, Position)
- **exchange/**: Exchange interface abstractions with mock implementation for testing
- **order_core/**: Order lifecycle management (creation, validation, repository)
- **market_data/**: Real-time market data streaming via WebSocket/FIX protocols

### Trading Strategies (`core/` and `strategies/`)
- **VWAP Splitter**: Volume-weighted average price order execution
- **Iceberg Manager**: Hide large orders by exposing only small portions
- **Trailing Stop**: Dynamic stop-loss that follows market movements
- **TWAP Splitter**: Time-weighted average price execution
- **Technical Strategy**: TA-based trading with indicators

### Technical Analysis Components
- **indicators/**: Technical indicators (moving averages, oscillators, trend, volume)
- **signals/**: Trading signal generation and analysis
- **trading_bots/**: Automated bots using different strategies (MA crossover, RSI, MACD, multi-indicator)

### Support Systems
- **backtest/**: Historical data testing with performance analysis
- **api/**: REST API server using Warp framework
- **utils/**: Common utilities (time, math, logging)

## Key Design Patterns

1. **Strategy Pattern**: All trading strategies implement a common `Strategy` trait
2. **Repository Pattern**: Order storage abstraction with in-memory implementation
3. **Manager Pattern**: Centralized management for orders, strategies, and market data
4. **Mock Testing**: MockExchange for testing without real exchanges

## Configuration

The system uses a `config.json` file with server, exchange, and logging settings. Mock exchange is enabled by default for development.

## API Server

REST API runs on Warp framework (default: 127.0.0.1:3030). Routes are defined in `api/routes.rs` with handlers in `api/handlers.rs`.

## Testing Approach

- Unit tests for individual components
- Integration tests for trading simulations
- Mock exchange for isolated testing
- CSV data provider for backtest scenarios