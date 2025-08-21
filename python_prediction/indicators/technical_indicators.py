"""
Technical Indicators Module
기술적 지표 계산 모듈
"""

import pandas as pd
import numpy as np
from typing import Optional, Tuple
import ta
from loguru import logger

class TechnicalIndicators:
    """Technical indicators calculator"""
    
    @staticmethod
    def add_moving_averages(df: pd.DataFrame, periods: list = [10, 20, 50, 200]) -> pd.DataFrame:
        """
        Add Simple Moving Averages (SMA) and Exponential Moving Averages (EMA)
        
        Args:
            df: DataFrame with OHLCV data
            periods: List of periods for moving averages
            
        Returns:
            DataFrame with added moving averages
        """
        for period in periods:
            df[f'SMA_{period}'] = ta.trend.sma_indicator(df['close'], window=period)
            df[f'EMA_{period}'] = ta.trend.ema_indicator(df['close'], window=period)
            
        logger.info(f"Added moving averages for periods: {periods}")
        return df
    
    @staticmethod
    def add_macd(
        df: pd.DataFrame,
        fast: int = 12,
        slow: int = 26,
        signal: int = 9
    ) -> pd.DataFrame:
        """
        Add MACD (Moving Average Convergence Divergence)
        
        Args:
            df: DataFrame with OHLCV data
            fast: Fast EMA period
            slow: Slow EMA period
            signal: Signal line EMA period
            
        Returns:
            DataFrame with MACD indicators
        """
        macd = ta.trend.MACD(df['close'], window_slow=slow, window_fast=fast, window_sign=signal)
        df['MACD'] = macd.macd()
        df['MACD_signal'] = macd.macd_signal()
        df['MACD_histogram'] = macd.macd_diff()
        
        logger.info("Added MACD indicators")
        return df
    
    @staticmethod
    def add_rsi(df: pd.DataFrame, period: int = 14) -> pd.DataFrame:
        """
        Add RSI (Relative Strength Index)
        
        Args:
            df: DataFrame with OHLCV data
            period: RSI period
            
        Returns:
            DataFrame with RSI
        """
        df['RSI'] = ta.momentum.RSIIndicator(df['close'], window=period).rsi()
        
        logger.info(f"Added RSI with period {period}")
        return df
    
    @staticmethod
    def add_stoch_rsi(
        df: pd.DataFrame,
        period: int = 14,
        smooth_k: int = 3,
        smooth_d: int = 3
    ) -> pd.DataFrame:
        """
        Add Stochastic RSI
        
        Args:
            df: DataFrame with OHLCV data
            period: RSI period
            smooth_k: K smoothing period
            smooth_d: D smoothing period
            
        Returns:
            DataFrame with Stochastic RSI
        """
        stoch_rsi = ta.momentum.StochRSIIndicator(
            df['close'],
            window=period,
            smooth1=smooth_k,
            smooth2=smooth_d
        )
        df['StochRSI_K'] = stoch_rsi.stochrsi_k() * 100
        df['StochRSI_D'] = stoch_rsi.stochrsi_d() * 100
        
        logger.info(f"Added Stochastic RSI")
        return df
    
    @staticmethod
    def add_bollinger_bands(
        df: pd.DataFrame,
        period: int = 20,
        std_dev: float = 2.0
    ) -> pd.DataFrame:
        """
        Add Bollinger Bands
        
        Args:
            df: DataFrame with OHLCV data
            period: Moving average period
            std_dev: Standard deviation multiplier
            
        Returns:
            DataFrame with Bollinger Bands
        """
        bb = ta.volatility.BollingerBands(
            close=df['close'],
            window=period,
            window_dev=std_dev
        )
        df['BB_upper'] = bb.bollinger_hband()
        df['BB_middle'] = bb.bollinger_mavg()
        df['BB_lower'] = bb.bollinger_lband()
        df['BB_width'] = bb.bollinger_wband()
        df['BB_percent'] = bb.bollinger_pband()
        
        logger.info(f"Added Bollinger Bands")
        return df
    
    @staticmethod
    def add_vwap(df: pd.DataFrame) -> pd.DataFrame:
        """
        Add VWAP (Volume Weighted Average Price)
        
        Args:
            df: DataFrame with OHLCV data
            
        Returns:
            DataFrame with VWAP
        """
        df['VWAP'] = ta.volume.VolumeWeightedAveragePrice(
            high=df['high'],
            low=df['low'],
            close=df['close'],
            volume=df['volume']
        ).volume_weighted_average_price()
        
        logger.info("Added VWAP")
        return df
    
    @staticmethod
    def add_parabolic_sar(
        df: pd.DataFrame,
        step: float = 0.02,
        max_step: float = 0.2
    ) -> pd.DataFrame:
        """
        Add Parabolic SAR
        
        Args:
            df: DataFrame with OHLCV data
            step: Acceleration factor step
            max_step: Maximum acceleration factor
            
        Returns:
            DataFrame with Parabolic SAR
        """
        psar = ta.trend.PSARIndicator(
            high=df['high'],
            low=df['low'],
            close=df['close'],
            step=step,
            max_step=max_step
        )
        df['PSAR'] = psar.psar()
        df['PSAR_up'] = psar.psar_up()
        df['PSAR_down'] = psar.psar_down()
        df['PSAR_up_indicator'] = psar.psar_up_indicator()
        df['PSAR_down_indicator'] = psar.psar_down_indicator()
        
        logger.info("Added Parabolic SAR")
        return df
    
    @staticmethod
    def add_fibonacci_levels(
        df: pd.DataFrame,
        lookback: int = 100
    ) -> pd.DataFrame:
        """
        Add Fibonacci retracement levels
        
        Args:
            df: DataFrame with OHLCV data
            lookback: Number of periods to look back for high/low
            
        Returns:
            DataFrame with Fibonacci levels
        """
        if len(df) < lookback:
            lookback = len(df)
            
        high = df['high'].rolling(lookback).max().iloc[-1]
        low = df['low'].rolling(lookback).min().iloc[-1]
        diff = high - low
        
        # Fibonacci levels
        levels = {
            'fib_0': high,
            'fib_236': high - diff * 0.236,
            'fib_382': high - diff * 0.382,
            'fib_500': high - diff * 0.500,
            'fib_618': high - diff * 0.618,
            'fib_786': high - diff * 0.786,
            'fib_1000': low
        }
        
        for name, value in levels.items():
            df[name] = value
            
        logger.info(f"Added Fibonacci levels with lookback {lookback}")
        return df
    
    @staticmethod
    def add_atr(df: pd.DataFrame, period: int = 14) -> pd.DataFrame:
        """
        Add ATR (Average True Range)
        
        Args:
            df: DataFrame with OHLCV data
            period: ATR period
            
        Returns:
            DataFrame with ATR
        """
        df['ATR'] = ta.volatility.AverageTrueRange(
            high=df['high'],
            low=df['low'],
            close=df['close'],
            window=period
        ).average_true_range()
        
        logger.info(f"Added ATR with period {period}")
        return df
    
    @staticmethod
    def add_volume_indicators(df: pd.DataFrame) -> pd.DataFrame:
        """
        Add volume-based indicators
        
        Args:
            df: DataFrame with OHLCV data
            
        Returns:
            DataFrame with volume indicators
        """
        # On Balance Volume
        df['OBV'] = ta.volume.OnBalanceVolumeIndicator(
            close=df['close'],
            volume=df['volume']
        ).on_balance_volume()
        
        # Volume SMA
        df['Volume_SMA'] = df['volume'].rolling(window=20).mean()
        
        # Money Flow Index
        df['MFI'] = ta.volume.MFIIndicator(
            high=df['high'],
            low=df['low'],
            close=df['close'],
            volume=df['volume'],
            window=14
        ).money_flow_index()
        
        logger.info("Added volume indicators")
        return df
    
    @staticmethod
    def add_all_indicators(df: pd.DataFrame) -> pd.DataFrame:
        """
        Add all technical indicators
        
        Args:
            df: DataFrame with OHLCV data
            
        Returns:
            DataFrame with all indicators
        """
        df = TechnicalIndicators.add_moving_averages(df)
        df = TechnicalIndicators.add_macd(df)
        df = TechnicalIndicators.add_rsi(df)
        df = TechnicalIndicators.add_stoch_rsi(df)
        df = TechnicalIndicators.add_bollinger_bands(df)
        df = TechnicalIndicators.add_vwap(df)
        df = TechnicalIndicators.add_parabolic_sar(df)
        df = TechnicalIndicators.add_atr(df)
        df = TechnicalIndicators.add_volume_indicators(df)
        df = TechnicalIndicators.add_fibonacci_levels(df)
        
        logger.info("Added all technical indicators")
        return df
    
    @staticmethod
    def generate_signals(df: pd.DataFrame) -> pd.DataFrame:
        """
        Generate trading signals based on indicators
        
        Args:
            df: DataFrame with indicators
            
        Returns:
            DataFrame with trading signals
        """
        # Initialize signal column
        df['signal'] = 0
        
        # Moving Average Crossover
        df['ma_signal'] = np.where(df['EMA_10'] > df['EMA_30'], 1, -1)
        
        # MACD Signal
        df['macd_signal'] = np.where(df['MACD'] > df['MACD_signal'], 1, -1)
        
        # RSI Signal
        df['rsi_signal'] = np.where(df['RSI'] < 30, 1, np.where(df['RSI'] > 70, -1, 0))
        
        # Bollinger Bands Signal
        df['bb_signal'] = np.where(
            df['close'] < df['BB_lower'], 1,
            np.where(df['close'] > df['BB_upper'], -1, 0)
        )
        
        # Stochastic RSI Signal
        df['stoch_signal'] = np.where(
            (df['StochRSI_K'] < 20) & (df['StochRSI_K'] > df['StochRSI_D']), 1,
            np.where((df['StochRSI_K'] > 80) & (df['StochRSI_K'] < df['StochRSI_D']), -1, 0)
        )
        
        # Combine signals (simple voting)
        df['combined_signal'] = (
            df['ma_signal'] + 
            df['macd_signal'] + 
            df['rsi_signal'] + 
            df['bb_signal'] + 
            df['stoch_signal']
        ) / 5
        
        # Final signal
        df['signal'] = np.where(df['combined_signal'] > 0.3, 1, 
                                np.where(df['combined_signal'] < -0.3, -1, 0))
        
        logger.info("Generated trading signals")
        return df

# Example usage
if __name__ == "__main__":
    # Create sample data
    dates = pd.date_range('2024-01-01', periods=100, freq='1H')
    df = pd.DataFrame({
        'open': np.random.randn(100).cumsum() + 100,
        'high': np.random.randn(100).cumsum() + 101,
        'low': np.random.randn(100).cumsum() + 99,
        'close': np.random.randn(100).cumsum() + 100,
        'volume': np.random.randint(1000, 10000, 100)
    }, index=dates)
    
    # Add indicators
    df = TechnicalIndicators.add_all_indicators(df)
    df = TechnicalIndicators.generate_signals(df)
    
    print(df.tail())