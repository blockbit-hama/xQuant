"""
Backtesting Module
백테스팅 엔진 구현
"""

import pandas as pd
import numpy as np
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime
from loguru import logger

@dataclass
class BacktestResult:
    """Backtest result container"""
    initial_capital: float
    final_capital: float
    total_return: float
    total_return_pct: float
    sharpe_ratio: float
    max_drawdown: float
    win_rate: float
    total_trades: int
    winning_trades: int
    losing_trades: int
    avg_win: float
    avg_loss: float
    profit_factor: float
    trades: pd.DataFrame
    equity_curve: pd.Series
    
    def __str__(self):
        return f"""
        ===== Backtest Results =====
        Initial Capital: ${self.initial_capital:,.2f}
        Final Capital: ${self.final_capital:,.2f}
        Total Return: ${self.total_return:,.2f} ({self.total_return_pct:.2f}%)
        
        Sharpe Ratio: {self.sharpe_ratio:.2f}
        Max Drawdown: {self.max_drawdown:.2f}%
        
        Total Trades: {self.total_trades}
        Win Rate: {self.win_rate:.2f}%
        Winning Trades: {self.winning_trades}
        Losing Trades: {self.losing_trades}
        
        Avg Win: ${self.avg_win:.2f}
        Avg Loss: ${self.avg_loss:.2f}
        Profit Factor: {self.profit_factor:.2f}
        ===========================
        """

class Backtester:
    """Backtesting engine for trading strategies"""
    
    def __init__(
        self,
        initial_capital: float = 10000,
        commission: float = 0.001,
        slippage: float = 0.0005,
        leverage: float = 1.0
    ):
        """
        Initialize backtester
        
        Args:
            initial_capital: Starting capital
            commission: Trading commission (as fraction)
            slippage: Slippage (as fraction)
            leverage: Leverage multiplier
        """
        self.initial_capital = initial_capital
        self.commission = commission
        self.slippage = slippage
        self.leverage = leverage
        self.reset()
        
    def reset(self):
        """Reset backtester state"""
        self.capital = self.initial_capital
        self.position = 0
        self.avg_entry_price = 0
        self.trades = []
        self.equity_curve = []
        
    def run(
        self,
        data: pd.DataFrame,
        strategy_func,
        **strategy_params
    ) -> BacktestResult:
        """
        Run backtest on historical data
        
        Args:
            data: DataFrame with OHLCV data and signals
            strategy_func: Strategy function that generates signals
            strategy_params: Parameters for strategy function
            
        Returns:
            BacktestResult object
        """
        self.reset()
        
        # Generate signals using strategy
        data = strategy_func(data, **strategy_params)
        
        # Ensure we have required columns
        if 'signal' not in data.columns:
            raise ValueError("Strategy must generate 'signal' column")
        
        # Run backtest
        for idx, row in data.iterrows():
            signal = row['signal']
            price = row['close']
            
            # Process signal
            if signal == 1:  # Buy signal
                self._open_long(price, idx)
            elif signal == -1:  # Sell signal
                self._open_short(price, idx)
            elif signal == 0 and self.position != 0:  # Close position
                self._close_position(price, idx)
            
            # Record equity
            current_equity = self._calculate_equity(price)
            self.equity_curve.append({
                'timestamp': idx,
                'equity': current_equity,
                'price': price
            })
        
        # Close any remaining position
        if self.position != 0:
            self._close_position(data.iloc[-1]['close'], data.index[-1])
        
        # Calculate results
        return self._calculate_results()
    
    def _open_long(self, price: float, timestamp):
        """Open long position"""
        if self.position <= 0:  # Not in long position
            # Close short if exists
            if self.position < 0:
                self._close_position(price, timestamp)
            
            # Calculate position size with leverage
            position_size = (self.capital * self.leverage) / price
            cost = position_size * price * (1 + self.commission + self.slippage)
            
            if cost <= self.capital:
                self.position = position_size
                self.avg_entry_price = price * (1 + self.slippage)
                self.capital -= cost - (position_size * price)  # Deduct commission
                
                logger.debug(f"Opened long: {position_size:.4f} @ ${price:.2f}")
    
    def _open_short(self, price: float, timestamp):
        """Open short position"""
        if self.position >= 0:  # Not in short position
            # Close long if exists
            if self.position > 0:
                self._close_position(price, timestamp)
            
            # Calculate position size with leverage
            position_size = (self.capital * self.leverage) / price
            cost = position_size * price * (self.commission + self.slippage)
            
            if cost <= self.capital:
                self.position = -position_size
                self.avg_entry_price = price * (1 - self.slippage)
                self.capital -= cost  # Deduct commission
                
                logger.debug(f"Opened short: {position_size:.4f} @ ${price:.2f}")
    
    def _close_position(self, price: float, timestamp):
        """Close current position"""
        if self.position == 0:
            return
        
        if self.position > 0:  # Close long
            exit_price = price * (1 - self.slippage)
            pnl = self.position * (exit_price - self.avg_entry_price)
            proceeds = self.position * exit_price * (1 - self.commission)
        else:  # Close short
            exit_price = price * (1 + self.slippage)
            pnl = -self.position * (self.avg_entry_price - exit_price)
            proceeds = -self.position * self.avg_entry_price - abs(self.position) * exit_price * self.commission
        
        self.capital += proceeds
        
        # Record trade
        self.trades.append({
            'entry_time': None,  # Would need to track this
            'exit_time': timestamp,
            'entry_price': self.avg_entry_price,
            'exit_price': exit_price,
            'position_size': abs(self.position),
            'side': 'long' if self.position > 0 else 'short',
            'pnl': pnl,
            'pnl_pct': (pnl / (abs(self.position) * self.avg_entry_price)) * 100
        })
        
        logger.debug(f"Closed {'long' if self.position > 0 else 'short'}: PnL ${pnl:.2f}")
        
        self.position = 0
        self.avg_entry_price = 0
    
    def _calculate_equity(self, current_price: float) -> float:
        """Calculate current equity"""
        if self.position == 0:
            return self.capital
        elif self.position > 0:
            unrealized_pnl = self.position * (current_price - self.avg_entry_price)
        else:
            unrealized_pnl = -self.position * (self.avg_entry_price - current_price)
        
        return self.capital + unrealized_pnl
    
    def _calculate_results(self) -> BacktestResult:
        """Calculate backtest results"""
        trades_df = pd.DataFrame(self.trades) if self.trades else pd.DataFrame()
        equity_df = pd.DataFrame(self.equity_curve)
        
        if not equity_df.empty:
            equity_series = equity_df.set_index('timestamp')['equity']
            final_capital = equity_series.iloc[-1]
        else:
            equity_series = pd.Series([self.initial_capital])
            final_capital = self.initial_capital
        
        total_return = final_capital - self.initial_capital
        total_return_pct = (total_return / self.initial_capital) * 100
        
        # Calculate metrics
        if not trades_df.empty:
            winning_trades = trades_df[trades_df['pnl'] > 0]
            losing_trades = trades_df[trades_df['pnl'] < 0]
            
            total_trades = len(trades_df)
            num_winning = len(winning_trades)
            num_losing = len(losing_trades)
            win_rate = (num_winning / total_trades * 100) if total_trades > 0 else 0
            
            avg_win = winning_trades['pnl'].mean() if not winning_trades.empty else 0
            avg_loss = abs(losing_trades['pnl'].mean()) if not losing_trades.empty else 0
            
            total_wins = winning_trades['pnl'].sum() if not winning_trades.empty else 0
            total_losses = abs(losing_trades['pnl'].sum()) if not losing_trades.empty else 0
            profit_factor = total_wins / total_losses if total_losses > 0 else float('inf')
        else:
            total_trades = 0
            num_winning = 0
            num_losing = 0
            win_rate = 0
            avg_win = 0
            avg_loss = 0
            profit_factor = 0
        
        # Calculate Sharpe ratio
        if len(equity_series) > 1:
            returns = equity_series.pct_change().dropna()
            sharpe_ratio = (returns.mean() / returns.std() * np.sqrt(252)) if returns.std() > 0 else 0
        else:
            sharpe_ratio = 0
        
        # Calculate max drawdown
        cummax = equity_series.cummax()
        drawdown = (equity_series - cummax) / cummax * 100
        max_drawdown = abs(drawdown.min())
        
        return BacktestResult(
            initial_capital=self.initial_capital,
            final_capital=final_capital,
            total_return=total_return,
            total_return_pct=total_return_pct,
            sharpe_ratio=sharpe_ratio,
            max_drawdown=max_drawdown,
            win_rate=win_rate,
            total_trades=total_trades,
            winning_trades=num_winning,
            losing_trades=num_losing,
            avg_win=avg_win,
            avg_loss=avg_loss,
            profit_factor=profit_factor,
            trades=trades_df,
            equity_curve=equity_series
        )

# Example strategy function
def simple_ma_strategy(data: pd.DataFrame, short_window: int = 10, long_window: int = 30) -> pd.DataFrame:
    """
    Simple moving average crossover strategy
    
    Args:
        data: DataFrame with OHLCV data
        short_window: Short MA period
        long_window: Long MA period
        
    Returns:
        DataFrame with signals
    """
    data['MA_short'] = data['close'].rolling(window=short_window).mean()
    data['MA_long'] = data['close'].rolling(window=long_window).mean()
    
    # Generate signals
    data['signal'] = 0
    data.loc[data['MA_short'] > data['MA_long'], 'signal'] = 1
    data.loc[data['MA_short'] < data['MA_long'], 'signal'] = -1
    
    return data

# Example usage
if __name__ == "__main__":
    # Create sample data
    dates = pd.date_range('2024-01-01', periods=1000, freq='1H')
    np.random.seed(42)
    prices = 100 + np.random.randn(1000).cumsum()
    
    data = pd.DataFrame({
        'open': prices + np.random.randn(1000) * 0.5,
        'high': prices + abs(np.random.randn(1000)),
        'low': prices - abs(np.random.randn(1000)),
        'close': prices,
        'volume': np.random.randint(1000, 10000, 1000)
    }, index=dates)
    
    # Run backtest
    backtester = Backtester(
        initial_capital=10000,
        commission=0.001,
        slippage=0.0005,
        leverage=2.0
    )
    
    result = backtester.run(data, simple_ma_strategy, short_window=10, long_window=30)
    print(result)