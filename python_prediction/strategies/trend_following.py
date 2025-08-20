"""
Trend Following Strategy
추세 추종 전략 구현
"""

import pandas as pd
import numpy as np
from typing import Dict, Optional, Tuple
from loguru import logger
from ..indicators.technical_indicators import TechnicalIndicators

class TrendFollowingStrategy:
    """Trend following trading strategy"""
    
    def __init__(
        self,
        ma_short: int = 10,
        ma_long: int = 30,
        atr_multiplier: float = 2.0,
        risk_per_trade: float = 0.02
    ):
        """
        Initialize trend following strategy
        
        Args:
            ma_short: Short moving average period
            ma_long: Long moving average period
            atr_multiplier: ATR multiplier for stop loss
            risk_per_trade: Risk per trade as fraction of capital
        """
        self.ma_short = ma_short
        self.ma_long = ma_long
        self.atr_multiplier = atr_multiplier
        self.risk_per_trade = risk_per_trade
        self.indicators = TechnicalIndicators()
        
    def generate_signals(self, data: pd.DataFrame) -> pd.DataFrame:
        """
        Generate trading signals based on trend following strategy
        
        Args:
            data: DataFrame with OHLCV data
            
        Returns:
            DataFrame with signals
        """
        # Add indicators
        data = self._add_indicators(data)
        
        # Initialize signals
        data['signal'] = 0
        data['position'] = 0
        data['stop_loss'] = 0
        data['take_profit'] = 0
        
        # Generate entry signals
        data = self._generate_entry_signals(data)
        
        # Generate exit signals
        data = self._generate_exit_signals(data)
        
        # Add position sizing
        data = self._calculate_position_size(data)
        
        logger.info(f"Generated trend following signals for {len(data)} bars")
        return data
    
    def _add_indicators(self, data: pd.DataFrame) -> pd.DataFrame:
        """Add required indicators"""
        # Moving averages
        data[f'MA_{self.ma_short}'] = data['close'].rolling(self.ma_short).mean()
        data[f'MA_{self.ma_long}'] = data['close'].rolling(self.ma_long).mean()
        
        # ATR for stop loss
        data = self.indicators.add_atr(data, period=14)
        
        # MACD for trend confirmation
        data = self.indicators.add_macd(data)
        
        # RSI for momentum
        data = self.indicators.add_rsi(data)
        
        # Bollinger Bands for volatility
        data = self.indicators.add_bollinger_bands(data)
        
        return data
    
    def _generate_entry_signals(self, data: pd.DataFrame) -> pd.DataFrame:
        """Generate entry signals"""
        # Long entry conditions
        long_condition = (
            (data[f'MA_{self.ma_short}'] > data[f'MA_{self.ma_long}']) &  # MA crossover
            (data[f'MA_{self.ma_short}'].shift(1) <= data[f'MA_{self.ma_long}'].shift(1)) &  # Just crossed
            (data['MACD'] > data['MACD_signal']) &  # MACD confirmation
            (data['RSI'] > 50) & (data['RSI'] < 70)  # RSI not overbought
        )
        
        # Short entry conditions
        short_condition = (
            (data[f'MA_{self.ma_short}'] < data[f'MA_{self.ma_long}']) &  # MA crossunder
            (data[f'MA_{self.ma_short}'].shift(1) >= data[f'MA_{self.ma_long}'].shift(1)) &  # Just crossed
            (data['MACD'] < data['MACD_signal']) &  # MACD confirmation
            (data['RSI'] < 50) & (data['RSI'] > 30)  # RSI not oversold
        )
        
        data.loc[long_condition, 'signal'] = 1
        data.loc[short_condition, 'signal'] = -1
        
        # Calculate stop loss and take profit levels
        data.loc[long_condition, 'stop_loss'] = data['close'] - (data['ATR'] * self.atr_multiplier)
        data.loc[long_condition, 'take_profit'] = data['close'] + (data['ATR'] * self.atr_multiplier * 2)
        
        data.loc[short_condition, 'stop_loss'] = data['close'] + (data['ATR'] * self.atr_multiplier)
        data.loc[short_condition, 'take_profit'] = data['close'] - (data['ATR'] * self.atr_multiplier * 2)
        
        return data
    
    def _generate_exit_signals(self, data: pd.DataFrame) -> pd.DataFrame:
        """Generate exit signals"""
        # Track position
        position = 0
        stop_loss = 0
        take_profit = 0
        
        for i in range(len(data)):
            if data.iloc[i]['signal'] != 0:
                # New position
                position = data.iloc[i]['signal']
                stop_loss = data.iloc[i]['stop_loss']
                take_profit = data.iloc[i]['take_profit']
            elif position != 0:
                # Check exit conditions
                if position == 1:  # Long position
                    if (data.iloc[i]['close'] <= stop_loss or 
                        data.iloc[i]['close'] >= take_profit or
                        data.iloc[i][f'MA_{self.ma_short}'] < data.iloc[i][f'MA_{self.ma_long}']):
                        data.iloc[i, data.columns.get_loc('signal')] = 0
                        position = 0
                elif position == -1:  # Short position
                    if (data.iloc[i]['close'] >= stop_loss or 
                        data.iloc[i]['close'] <= take_profit or
                        data.iloc[i][f'MA_{self.ma_short}'] > data.iloc[i][f'MA_{self.ma_long}']):
                        data.iloc[i, data.columns.get_loc('signal')] = 0
                        position = 0
            
            data.iloc[i, data.columns.get_loc('position')] = position
        
        return data
    
    def _calculate_position_size(self, data: pd.DataFrame) -> pd.DataFrame:
        """Calculate position size based on risk management"""
        data['position_size'] = 0
        
        for i in range(len(data)):
            if data.iloc[i]['signal'] != 0 and data.iloc[i]['ATR'] > 0:
                # Calculate position size based on ATR
                risk_amount = self.risk_per_trade  # As fraction of capital
                stop_distance = data.iloc[i]['ATR'] * self.atr_multiplier
                
                if stop_distance > 0:
                    position_size = risk_amount / (stop_distance / data.iloc[i]['close'])
                    data.iloc[i, data.columns.get_loc('position_size')] = min(position_size, 1.0)
        
        return data

class MeanReversionStrategy:
    """Mean reversion (counter-trend) trading strategy"""
    
    def __init__(
        self,
        bb_period: int = 20,
        bb_std: float = 2.0,
        rsi_period: int = 14,
        rsi_oversold: float = 30,
        rsi_overbought: float = 70
    ):
        """
        Initialize mean reversion strategy
        
        Args:
            bb_period: Bollinger Bands period
            bb_std: Bollinger Bands standard deviation
            rsi_period: RSI period
            rsi_oversold: RSI oversold level
            rsi_overbought: RSI overbought level
        """
        self.bb_period = bb_period
        self.bb_std = bb_std
        self.rsi_period = rsi_period
        self.rsi_oversold = rsi_oversold
        self.rsi_overbought = rsi_overbought
        self.indicators = TechnicalIndicators()
        
    def generate_signals(self, data: pd.DataFrame) -> pd.DataFrame:
        """
        Generate trading signals based on mean reversion strategy
        
        Args:
            data: DataFrame with OHLCV data
            
        Returns:
            DataFrame with signals
        """
        # Add indicators
        data = self.indicators.add_bollinger_bands(data, self.bb_period, self.bb_std)
        data = self.indicators.add_rsi(data, self.rsi_period)
        data = self.indicators.add_stoch_rsi(data)
        
        # Initialize signals
        data['signal'] = 0
        
        # Long signals (buy at oversold conditions)
        long_condition = (
            (data['close'] < data['BB_lower']) &  # Price below lower band
            (data['RSI'] < self.rsi_oversold) &  # RSI oversold
            (data['StochRSI_K'] < 20) &  # Stoch RSI oversold
            (data['StochRSI_K'] > data['StochRSI_D'])  # Stoch RSI turning up
        )
        
        # Short signals (sell at overbought conditions)
        short_condition = (
            (data['close'] > data['BB_upper']) &  # Price above upper band
            (data['RSI'] > self.rsi_overbought) &  # RSI overbought
            (data['StochRSI_K'] > 80) &  # Stoch RSI overbought
            (data['StochRSI_K'] < data['StochRSI_D'])  # Stoch RSI turning down
        )
        
        # Exit signals (return to mean)
        exit_long = (
            (data['close'] > data['BB_middle']) |  # Price above middle band
            (data['RSI'] > 50)  # RSI neutral
        )
        
        exit_short = (
            (data['close'] < data['BB_middle']) |  # Price below middle band
            (data['RSI'] < 50)  # RSI neutral
        )
        
        # Apply signals
        data.loc[long_condition, 'signal'] = 1
        data.loc[short_condition, 'signal'] = -1
        
        # Track positions and apply exit signals
        position = 0
        for i in range(len(data)):
            if data.iloc[i]['signal'] != 0:
                position = data.iloc[i]['signal']
            elif position == 1 and exit_long.iloc[i]:
                data.iloc[i, data.columns.get_loc('signal')] = 0
                position = 0
            elif position == -1 and exit_short.iloc[i]:
                data.iloc[i, data.columns.get_loc('signal')] = 0
                position = 0
        
        logger.info(f"Generated mean reversion signals for {len(data)} bars")
        return data

class MACDStochRSIStrategy:
    """MACD and Stochastic RSI combined strategy"""
    
    def __init__(self):
        """Initialize MACD & StochRSI strategy"""
        self.indicators = TechnicalIndicators()
        
    def generate_signals(self, data: pd.DataFrame) -> pd.DataFrame:
        """
        Generate trading signals based on MACD and StochRSI
        
        Args:
            data: DataFrame with OHLCV data
            
        Returns:
            DataFrame with signals
        """
        # Add indicators
        data = self.indicators.add_macd(data)
        data = self.indicators.add_stoch_rsi(data)
        data = self.indicators.add_moving_averages(data, [50, 200])
        
        # Initialize signals
        data['signal'] = 0
        
        # Long signals
        long_condition = (
            (data['MACD'] > data['MACD_signal']) &  # MACD bullish
            (data['MACD'].shift(1) <= data['MACD_signal'].shift(1)) &  # MACD crossover
            (data['StochRSI_K'] > data['StochRSI_D']) &  # StochRSI bullish
            (data['StochRSI_K'] < 80) &  # Not overbought
            (data['close'] > data['SMA_50'])  # Above medium-term trend
        )
        
        # Short signals
        short_condition = (
            (data['MACD'] < data['MACD_signal']) &  # MACD bearish
            (data['MACD'].shift(1) >= data['MACD_signal'].shift(1)) &  # MACD crossunder
            (data['StochRSI_K'] < data['StochRSI_D']) &  # StochRSI bearish
            (data['StochRSI_K'] > 20) &  # Not oversold
            (data['close'] < data['SMA_50'])  # Below medium-term trend
        )
        
        # Exit signals
        exit_long = (
            (data['MACD'] < data['MACD_signal']) |  # MACD turns bearish
            (data['StochRSI_K'] > 80)  # Overbought
        )
        
        exit_short = (
            (data['MACD'] > data['MACD_signal']) |  # MACD turns bullish
            (data['StochRSI_K'] < 20)  # Oversold
        )
        
        # Apply signals with position tracking
        position = 0
        for i in range(len(data)):
            if long_condition.iloc[i] and position <= 0:
                data.iloc[i, data.columns.get_loc('signal')] = 1
                position = 1
            elif short_condition.iloc[i] and position >= 0:
                data.iloc[i, data.columns.get_loc('signal')] = -1
                position = -1
            elif position == 1 and exit_long.iloc[i]:
                data.iloc[i, data.columns.get_loc('signal')] = 0
                position = 0
            elif position == -1 and exit_short.iloc[i]:
                data.iloc[i, data.columns.get_loc('signal')] = 0
                position = 0
        
        logger.info(f"Generated MACD & StochRSI signals for {len(data)} bars")
        return data

class BollingerBandsStrategy:
    """Bollinger Bands breakout strategy"""
    
    def __init__(
        self,
        bb_period: int = 20,
        bb_std: float = 2.0,
        volume_threshold: float = 1.5
    ):
        """
        Initialize Bollinger Bands strategy
        
        Args:
            bb_period: Bollinger Bands period
            bb_std: Standard deviation multiplier
            volume_threshold: Volume spike threshold
        """
        self.bb_period = bb_period
        self.bb_std = bb_std
        self.volume_threshold = volume_threshold
        self.indicators = TechnicalIndicators()
        
    def generate_signals(self, data: pd.DataFrame) -> pd.DataFrame:
        """
        Generate trading signals based on Bollinger Bands
        
        Args:
            data: DataFrame with OHLCV data
            
        Returns:
            DataFrame with signals
        """
        # Add indicators
        data = self.indicators.add_bollinger_bands(data, self.bb_period, self.bb_std)
        data = self.indicators.add_volume_indicators(data)
        data = self.indicators.add_rsi(data)
        
        # Initialize signals
        data['signal'] = 0
        
        # Calculate volume spike
        data['volume_spike'] = data['volume'] > (data['Volume_SMA'] * self.volume_threshold)
        
        # Breakout strategy
        long_breakout = (
            (data['close'] > data['BB_upper']) &  # Break above upper band
            (data['close'].shift(1) <= data['BB_upper'].shift(1)) &  # Just broke
            (data['volume_spike']) &  # Volume confirmation
            (data['RSI'] > 50) & (data['RSI'] < 80)  # Momentum confirmation
        )
        
        short_breakout = (
            (data['close'] < data['BB_lower']) &  # Break below lower band
            (data['close'].shift(1) >= data['BB_lower'].shift(1)) &  # Just broke
            (data['volume_spike']) &  # Volume confirmation
            (data['RSI'] < 50) & (data['RSI'] > 20)  # Momentum confirmation
        )
        
        # Mean reversion strategy (opposite of breakout)
        long_reversion = (
            (data['close'] < data['BB_lower']) &  # Touch lower band
            (data['BB_percent'] < 0) &  # Below lower band
            (data['RSI'] < 30)  # Oversold
        )
        
        short_reversion = (
            (data['close'] > data['BB_upper']) &  # Touch upper band
            (data['BB_percent'] > 1) &  # Above upper band
            (data['RSI'] > 70)  # Overbought
        )
        
        # Combine strategies (breakout preferred)
        data.loc[long_breakout | long_reversion, 'signal'] = 1
        data.loc[short_breakout | short_reversion, 'signal'] = -1
        
        # Exit when price returns to middle band
        position = 0
        for i in range(len(data)):
            if data.iloc[i]['signal'] != 0:
                position = data.iloc[i]['signal']
            elif position != 0:
                # Exit conditions
                if position == 1 and data.iloc[i]['close'] >= data.iloc[i]['BB_middle']:
                    data.iloc[i, data.columns.get_loc('signal')] = 0
                    position = 0
                elif position == -1 and data.iloc[i]['close'] <= data.iloc[i]['BB_middle']:
                    data.iloc[i, data.columns.get_loc('signal')] = 0
                    position = 0
        
        logger.info(f"Generated Bollinger Bands signals for {len(data)} bars")
        return data

# Example usage
if __name__ == "__main__":
    # Create sample data
    dates = pd.date_range('2024-01-01', periods=500, freq='1H')
    np.random.seed(42)
    
    data = pd.DataFrame({
        'open': 100 + np.random.randn(500).cumsum(),
        'high': 101 + np.random.randn(500).cumsum(),
        'low': 99 + np.random.randn(500).cumsum(),
        'close': 100 + np.random.randn(500).cumsum(),
        'volume': np.random.randint(1000, 10000, 500)
    }, index=dates)
    
    # Test strategies
    trend_strategy = TrendFollowingStrategy()
    signals = trend_strategy.generate_signals(data.copy())
    print(f"Trend Following - Signals generated: {(signals['signal'] != 0).sum()}")
    
    reversion_strategy = MeanReversionStrategy()
    signals = reversion_strategy.generate_signals(data.copy())
    print(f"Mean Reversion - Signals generated: {(signals['signal'] != 0).sum()}")
    
    macd_stoch_strategy = MACDStochRSIStrategy()
    signals = macd_stoch_strategy.generate_signals(data.copy())
    print(f"MACD & StochRSI - Signals generated: {(signals['signal'] != 0).sum()}")
    
    bb_strategy = BollingerBandsStrategy()
    signals = bb_strategy.generate_signals(data.copy())
    print(f"Bollinger Bands - Signals generated: {(signals['signal'] != 0).sum()}")