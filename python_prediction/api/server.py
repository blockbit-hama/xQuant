"""
FastAPI Server for Python Prediction System
Rust 트레이딩 봇과 통신하는 API 서버
"""

from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Dict, List, Optional
import pandas as pd
from datetime import datetime
from loguru import logger
import uvicorn

from ..data_collection.binance_client import BinanceDataCollector
from ..indicators.technical_indicators import TechnicalIndicators
from ..strategies.trend_following import (
    TrendFollowingStrategy,
    MeanReversionStrategy,
    MACDStochRSIStrategy,
    BollingerBandsStrategy
)
from ..backtest.backtester import Backtester
from ..config.config import config

# Initialize FastAPI app
app = FastAPI(
    title="xQuant Prediction API",
    description="Python prediction system for xQuant trading bot",
    version="0.1.0"
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Initialize components
data_collector = BinanceDataCollector()
indicators = TechnicalIndicators()

# Request/Response Models
class MarketDataRequest(BaseModel):
    symbol: str = "BTC/USDT"
    timeframe: str = "1m"
    limit: int = 100

class SignalRequest(BaseModel):
    symbol: str = "BTC/USDT"
    timeframe: str = "1m"
    strategy: str = "trend_following"
    lookback: int = 100

class BacktestRequest(BaseModel):
    symbol: str = "BTC/USDT"
    timeframe: str = "1h"
    strategy: str = "trend_following"
    days: int = 30
    initial_capital: float = 10000

class PredictionRequest(BaseModel):
    symbol: str = "BTC/USDT"
    timeframe: str = "1h"
    horizon: int = 24  # Hours to predict

class SignalResponse(BaseModel):
    symbol: str
    timestamp: datetime
    signal: int  # 1: Buy, -1: Sell, 0: Neutral
    confidence: float
    indicators: Dict
    metadata: Dict

# API Endpoints
@app.get("/")
async def root():
    """Health check endpoint"""
    return {
        "status": "online",
        "service": "xQuant Prediction API",
        "version": "0.1.0"
    }

@app.get("/health")
async def health_check():
    """Detailed health check"""
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
        "config": {
            "binance_testnet": config.binance.testnet,
            "api_host": config.api.host,
            "api_port": config.api.port
        }
    }

@app.post("/market-data")
async def get_market_data(request: MarketDataRequest):
    """Fetch current market data"""
    try:
        # Fetch OHLCV data
        df = data_collector.fetch_ohlcv(
            symbol=request.symbol,
            timeframe=request.timeframe,
            limit=request.limit
        )
        
        # Add indicators
        df = indicators.add_all_indicators(df)
        
        # Convert to dict for JSON response
        data = df.tail(1).to_dict('records')[0] if not df.empty else {}
        
        return {
            "symbol": request.symbol,
            "timeframe": request.timeframe,
            "timestamp": datetime.now().isoformat(),
            "data": data
        }
    except Exception as e:
        logger.error(f"Error fetching market data: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/signals")
async def get_signals(request: SignalRequest):
    """Generate trading signals"""
    try:
        # Fetch data
        df = data_collector.fetch_ohlcv(
            symbol=request.symbol,
            timeframe=request.timeframe,
            limit=request.lookback
        )
        
        # Select strategy
        if request.strategy == "trend_following":
            strategy = TrendFollowingStrategy()
        elif request.strategy == "mean_reversion":
            strategy = MeanReversionStrategy()
        elif request.strategy == "macd_stochrsi":
            strategy = MACDStochRSIStrategy()
        elif request.strategy == "bollinger_bands":
            strategy = BollingerBandsStrategy()
        else:
            raise ValueError(f"Unknown strategy: {request.strategy}")
        
        # Generate signals
        df = strategy.generate_signals(df)
        
        # Get latest signal
        latest = df.iloc[-1]
        signal = int(latest.get('signal', 0))
        
        # Calculate confidence based on indicator alignment
        confidence = 0.5  # Base confidence
        if 'RSI' in latest:
            if (signal == 1 and latest['RSI'] < 40) or (signal == -1 and latest['RSI'] > 60):
                confidence += 0.2
        if 'MACD' in latest and 'MACD_signal' in latest:
            if (signal == 1 and latest['MACD'] > latest['MACD_signal']) or \
               (signal == -1 and latest['MACD'] < latest['MACD_signal']):
                confidence += 0.2
        
        # Prepare response
        response = SignalResponse(
            symbol=request.symbol,
            timestamp=datetime.now(),
            signal=signal,
            confidence=min(confidence, 1.0),
            indicators={
                'close': float(latest['close']),
                'RSI': float(latest.get('RSI', 0)),
                'MACD': float(latest.get('MACD', 0)),
                'BB_percent': float(latest.get('BB_percent', 0))
            },
            metadata={
                'strategy': request.strategy,
                'timeframe': request.timeframe,
                'lookback': request.lookback
            }
        )
        
        return response
        
    except Exception as e:
        logger.error(f"Error generating signals: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/backtest")
async def run_backtest(request: BacktestRequest):
    """Run backtest for a strategy"""
    try:
        # Fetch historical data
        df = data_collector.fetch_historical_data(
            symbol=request.symbol,
            timeframe=request.timeframe,
            days=request.days
        )
        
        if df.empty:
            raise ValueError("No historical data available")
        
        # Select strategy
        if request.strategy == "trend_following":
            strategy = TrendFollowingStrategy()
            strategy_func = strategy.generate_signals
        elif request.strategy == "mean_reversion":
            strategy = MeanReversionStrategy()
            strategy_func = strategy.generate_signals
        elif request.strategy == "macd_stochrsi":
            strategy = MACDStochRSIStrategy()
            strategy_func = strategy.generate_signals
        elif request.strategy == "bollinger_bands":
            strategy = BollingerBandsStrategy()
            strategy_func = strategy.generate_signals
        else:
            raise ValueError(f"Unknown strategy: {request.strategy}")
        
        # Run backtest
        backtester = Backtester(
            initial_capital=request.initial_capital,
            commission=config.backtest.commission,
            slippage=config.backtest.slippage
        )
        
        result = backtester.run(df, strategy_func)
        
        # Prepare response
        return {
            "symbol": request.symbol,
            "strategy": request.strategy,
            "timeframe": request.timeframe,
            "days": request.days,
            "results": {
                "initial_capital": result.initial_capital,
                "final_capital": result.final_capital,
                "total_return": result.total_return,
                "total_return_pct": result.total_return_pct,
                "sharpe_ratio": result.sharpe_ratio,
                "max_drawdown": result.max_drawdown,
                "win_rate": result.win_rate,
                "total_trades": result.total_trades,
                "profit_factor": result.profit_factor
            }
        }
        
    except Exception as e:
        logger.error(f"Error running backtest: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/predict")
async def predict_price(request: PredictionRequest):
    """Predict future price movements (placeholder for ML model)"""
    try:
        # Fetch data
        df = data_collector.fetch_ohlcv(
            symbol=request.symbol,
            timeframe=request.timeframe,
            limit=500
        )
        
        # Add indicators
        df = indicators.add_all_indicators(df)
        
        # Simple prediction based on trend (placeholder for ML model)
        current_price = float(df.iloc[-1]['close'])
        sma_50 = float(df.iloc[-1].get('SMA_50', current_price))
        sma_200 = float(df.iloc[-1].get('SMA_200', current_price))
        
        # Trend-based prediction
        if sma_50 > sma_200:
            # Uptrend
            predicted_change = 0.02  # 2% up
        elif sma_50 < sma_200:
            # Downtrend
            predicted_change = -0.02  # 2% down
        else:
            # Sideways
            predicted_change = 0
        
        predicted_price = current_price * (1 + predicted_change)
        
        return {
            "symbol": request.symbol,
            "timeframe": request.timeframe,
            "current_price": current_price,
            "predicted_price": predicted_price,
            "predicted_change": predicted_change,
            "horizon_hours": request.horizon,
            "confidence": 0.6,  # Placeholder confidence
            "method": "trend_analysis"  # Will be "ml_model" when implemented
        }
        
    except Exception as e:
        logger.error(f"Error predicting price: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/strategies")
async def list_strategies():
    """List available trading strategies"""
    return {
        "strategies": [
            {
                "name": "trend_following",
                "description": "Follow market trends using moving averages and momentum",
                "indicators": ["MA", "MACD", "RSI", "ATR"]
            },
            {
                "name": "mean_reversion",
                "description": "Trade reversals at extreme price levels",
                "indicators": ["Bollinger Bands", "RSI", "Stochastic RSI"]
            },
            {
                "name": "macd_stochrsi",
                "description": "Combined MACD and Stochastic RSI signals",
                "indicators": ["MACD", "Stochastic RSI", "MA"]
            },
            {
                "name": "bollinger_bands",
                "description": "Trade breakouts and reversals using Bollinger Bands",
                "indicators": ["Bollinger Bands", "Volume", "RSI"]
            }
        ]
    }

@app.get("/indicators")
async def list_indicators():
    """List available technical indicators"""
    return {
        "indicators": [
            "SMA", "EMA", "MACD", "RSI", "Stochastic RSI",
            "Bollinger Bands", "VWAP", "Parabolic SAR",
            "ATR", "OBV", "MFI", "Fibonacci Levels"
        ]
    }

# WebSocket endpoint for real-time data (placeholder)
@app.websocket("/ws/{symbol}")
async def websocket_endpoint(websocket, symbol: str):
    """WebSocket for real-time market data streaming"""
    await websocket.accept()
    try:
        # This would stream real-time data
        await websocket.send_json({
            "type": "connected",
            "symbol": symbol,
            "timestamp": datetime.now().isoformat()
        })
        
        # Placeholder for real-time streaming
        # In production, this would connect to Binance WebSocket
        
    except Exception as e:
        logger.error(f"WebSocket error: {e}")
    finally:
        await websocket.close()

def start_server():
    """Start the FastAPI server"""
    logger.info(f"Starting API server on {config.api.host}:{config.api.port}")
    uvicorn.run(
        app,
        host=config.api.host,
        port=config.api.port,
        reload=False
    )

if __name__ == "__main__":
    start_server()