"""
Configuration module for Python Prediction System
"""

import os
from dataclasses import dataclass
from typing import Optional
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

@dataclass
class BinanceConfig:
    """Binance API configuration"""
    api_key: str = os.getenv("BINANCE_API_KEY", "")
    api_secret: str = os.getenv("BINANCE_API_SECRET", "")
    testnet: bool = os.getenv("BINANCE_TESTNET", "true").lower() == "true"
    base_url: str = "https://testnet.binancefuture.com" if testnet else "https://fapi.binance.com"

@dataclass
class TradingConfig:
    """Trading configuration"""
    symbol: str = "BTCUSDT"
    interval: str = "1m"  # 1m, 5m, 15m, 1h, 4h, 1d
    leverage: int = 10
    position_size: float = 0.01  # Position size as fraction of capital
    stop_loss: float = 0.02  # 2% stop loss
    take_profit: float = 0.03  # 3% take profit
    
@dataclass
class IndicatorConfig:
    """Technical indicator configuration"""
    # Moving Averages
    sma_short: int = 10
    sma_long: int = 30
    ema_short: int = 12
    ema_long: int = 26
    
    # MACD
    macd_fast: int = 12
    macd_slow: int = 26
    macd_signal: int = 9
    
    # RSI
    rsi_period: int = 14
    rsi_oversold: float = 30
    rsi_overbought: float = 70
    
    # Stochastic RSI
    stoch_rsi_period: int = 14
    stoch_rsi_smooth_k: int = 3
    stoch_rsi_smooth_d: int = 3
    
    # Bollinger Bands
    bb_period: int = 20
    bb_std: float = 2.0
    
    # VWAP
    vwap_period: int = 14

@dataclass
class BacktestConfig:
    """Backtesting configuration"""
    initial_capital: float = 10000.0
    commission: float = 0.001  # 0.1%
    slippage: float = 0.0005  # 0.05%
    start_date: Optional[str] = None
    end_date: Optional[str] = None

@dataclass
class APIConfig:
    """API server configuration"""
    host: str = "127.0.0.1"
    port: int = 8000
    rust_bot_url: str = "http://127.0.0.1:3030"
    
class Config:
    """Main configuration class"""
    def __init__(self):
        self.binance = BinanceConfig()
        self.trading = TradingConfig()
        self.indicators = IndicatorConfig()
        self.backtest = BacktestConfig()
        self.api = APIConfig()
        
    def validate(self) -> bool:
        """Validate configuration"""
        if not self.binance.testnet and (not self.binance.api_key or not self.binance.api_secret):
            raise ValueError("Binance API credentials required for mainnet")
        return True

# Global config instance
config = Config()