"""
Python Prediction System for xQuant Trading Bot
알고리즘 트레이딩을 위한 예측 및 분석 시스템
"""

__version__ = "0.1.0"
__author__ = "HAMA"

# Import main components
from .data_collection.binance_client import BinanceDataCollector
from .indicators.technical_indicators import TechnicalIndicators
from .backtest.backtester import Backtester
from .strategies.trend_following import (
    TrendFollowingStrategy,
    MeanReversionStrategy,
    MACDStochRSIStrategy,
    BollingerBandsStrategy
)
from .config.config import config

__all__ = [
    'BinanceDataCollector',
    'TechnicalIndicators', 
    'Backtester',
    'TrendFollowingStrategy',
    'MeanReversionStrategy',
    'MACDStochRSIStrategy',
    'BollingerBandsStrategy',
    'config'
]