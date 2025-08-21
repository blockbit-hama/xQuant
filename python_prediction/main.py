#!/usr/bin/env python3
"""
Main entry point for Python Prediction System
Python 예측 시스템 실행 스크립트
"""

import asyncio
import argparse
import sys
from pathlib import Path
from loguru import logger

# Add the prediction system to path
sys.path.append(str(Path(__file__).parent))

from api.server import start_server
from data_collection.binance_client import BinanceDataCollector
from indicators.technical_indicators import TechnicalIndicators
from strategies.trend_following import TrendFollowingStrategy, MeanReversionStrategy
from backtest.backtester import Backtester
from config.config import config

async def test_data_collection():
    """Test data collection functionality"""
    logger.info("Testing data collection...")
    try:
        collector = BinanceDataCollector()
        
        # Test ticker fetch
        ticker = collector.fetch_ticker("BTC/USDT")
        logger.info(f"Current BTC price: {ticker['last']}")
        
        # Test OHLCV fetch
        df = collector.fetch_ohlcv("BTC/USDT", "1h", limit=10)
        logger.info(f"Fetched {len(df)} candles")
        
        return True
    except Exception as e:
        logger.error(f"Data collection test failed: {e}")
        return False

async def test_indicators():
    """Test technical indicators"""
    logger.info("Testing technical indicators...")
    try:
        import pandas as pd
        import numpy as np
        
        # Create sample data
        dates = pd.date_range('2024-01-01', periods=100, freq='1H')
        df = pd.DataFrame({
            'open': np.random.randn(100).cumsum() + 100,
            'high': np.random.randn(100).cumsum() + 101,
            'low': np.random.randn(100).cumsum() + 99,
            'close': np.random.randn(100).cumsum() + 100,
            'volume': np.random.randint(1000, 10000, 100)
        }, index=dates)
        
        # Test indicators
        indicators = TechnicalIndicators()
        df = indicators.add_all_indicators(df)
        
        logger.info(f"Added indicators: {[col for col in df.columns if col not in ['open', 'high', 'low', 'close', 'volume']]}")
        return True
    except Exception as e:
        logger.error(f"Indicators test failed: {e}")
        return False

async def test_strategies():
    """Test trading strategies"""
    logger.info("Testing trading strategies...")
    try:
        import pandas as pd
        import numpy as np
        
        # Create sample data
        dates = pd.date_range('2024-01-01', periods=500, freq='1H')
        np.random.seed(42)
        df = pd.DataFrame({
            'open': 100 + np.random.randn(500).cumsum(),
            'high': 101 + np.random.randn(500).cumsum(),
            'low': 99 + np.random.randn(500).cumsum(),
            'close': 100 + np.random.randn(500).cumsum(),
            'volume': np.random.randint(1000, 10000, 500)
        }, index=dates)
        
        # Test strategies
        trend_strategy = TrendFollowingStrategy()
        signals = trend_strategy.generate_signals(df.copy())
        trend_signals = (signals['signal'] != 0).sum()
        
        reversion_strategy = MeanReversionStrategy()
        signals = reversion_strategy.generate_signals(df.copy())
        reversion_signals = (signals['signal'] != 0).sum()
        
        logger.info(f"Trend following signals: {trend_signals}")
        logger.info(f"Mean reversion signals: {reversion_signals}")
        return True
    except Exception as e:
        logger.error(f"Strategies test failed: {e}")
        return False

async def test_backtesting():
    """Test backtesting functionality"""
    logger.info("Testing backtesting...")
    try:
        import pandas as pd
        import numpy as np
        
        # Create sample data
        dates = pd.date_range('2024-01-01', periods=1000, freq='1H')
        np.random.seed(42)
        data = pd.DataFrame({
            'open': 100 + np.random.randn(1000).cumsum(),
            'high': 101 + np.random.randn(1000).cumsum(),
            'low': 99 + np.random.randn(1000).cumsum(),
            'close': 100 + np.random.randn(1000).cumsum(),
            'volume': np.random.randint(1000, 10000, 1000)
        }, index=dates)
        
        # Simple strategy function
        def simple_strategy(df):
            df['MA_10'] = df['close'].rolling(10).mean()
            df['MA_30'] = df['close'].rolling(30).mean()
            df['signal'] = 0
            df.loc[df['MA_10'] > df['MA_30'], 'signal'] = 1
            df.loc[df['MA_10'] < df['MA_30'], 'signal'] = -1
            return df
        
        # Run backtest
        backtester = Backtester(initial_capital=10000)
        result = backtester.run(data, simple_strategy)
        
        logger.info(f"Backtest results: Return {result.total_return_pct:.2f}%, Trades: {result.total_trades}")
        return True
    except Exception as e:
        logger.error(f"Backtesting test failed: {e}")
        return False

async def run_tests():
    """Run all tests"""
    logger.info("Starting comprehensive tests...")
    
    tests = [
        ("Data Collection", test_data_collection),
        ("Technical Indicators", test_indicators),
        ("Trading Strategies", test_strategies),
        ("Backtesting", test_backtesting),
    ]
    
    results = []
    for test_name, test_func in tests:
        try:
            result = await test_func()
            results.append((test_name, result))
            status = "✅ PASS" if result else "❌ FAIL"
            logger.info(f"{test_name}: {status}")
        except Exception as e:
            results.append((test_name, False))
            logger.error(f"{test_name}: ❌ FAIL - {e}")
    
    # Summary
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    logger.info(f"\nTest Summary: {passed}/{total} passed")
    
    if passed == total:
        logger.info("🎉 All tests passed!")
    else:
        logger.warning("⚠️  Some tests failed. Check logs for details.")
    
    return passed == total

def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="xQuant Python Prediction System")
    parser.add_argument(
        "command", 
        choices=["server", "test", "collect", "backtest"],
        help="Command to run"
    )
    parser.add_argument("--symbol", default="BTC/USDT", help="Trading symbol")
    parser.add_argument("--strategy", default="trend_following", help="Trading strategy")
    parser.add_argument("--days", type=int, default=30, help="Days of data")
    
    args = parser.parse_args()
    
    # Configure logging
    logger.add(
        "logs/prediction_system.log",
        rotation="10 MB",
        retention="10 days",
        level="INFO"
    )
    
    if args.command == "server":
        logger.info("Starting prediction API server...")
        start_server()
    
    elif args.command == "test":
        asyncio.run(run_tests())
    
    elif args.command == "collect":
        asyncio.run(collect_data(args.symbol, args.days))
    
    elif args.command == "backtest":
        asyncio.run(run_backtest(args.symbol, args.strategy, args.days))

async def collect_data(symbol: str, days: int):
    """Collect and save historical data"""
    logger.info(f"Collecting {days} days of data for {symbol}...")
    
    collector = BinanceDataCollector()
    df = collector.fetch_historical_data(symbol, "1h", days)
    
    if not df.empty:
        filename = f"{symbol.replace('/', '_')}_{days}d.csv"
        collector.save_to_csv(df, filename)
        logger.info(f"Data saved to data/{filename}")
    else:
        logger.error("No data collected")

async def run_backtest(symbol: str, strategy: str, days: int):
    """Run standalone backtest"""
    logger.info(f"Running backtest: {strategy} on {symbol} for {days} days")
    
    collector = BinanceDataCollector()
    df = collector.fetch_historical_data(symbol, "1h", days)
    
    if df.empty:
        logger.error("No data available for backtest")
        return
    
    # Select strategy
    if strategy == "trend_following":
        strat = TrendFollowingStrategy()
    elif strategy == "mean_reversion":
        strat = MeanReversionStrategy()
    else:
        logger.error(f"Unknown strategy: {strategy}")
        return
    
    # Run backtest
    backtester = Backtester(initial_capital=10000)
    result = backtester.run(df, strat.generate_signals)
    
    logger.info(f"Backtest completed:\n{result}")

if __name__ == "__main__":
    main()