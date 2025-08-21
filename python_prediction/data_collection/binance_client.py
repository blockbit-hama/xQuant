"""
Binance Data Collection Module
바이낸스 선물 데이터 수집 모듈
"""

import pandas as pd
import numpy as np
from typing import List, Dict, Optional, Tuple
from datetime import datetime, timedelta
import ccxt
import time
from loguru import logger
from ..config.config import config

class BinanceDataCollector:
    """Binance futures data collector"""
    
    def __init__(self):
        """Initialize Binance client"""
        self.exchange = ccxt.binance({
            'apiKey': config.binance.api_key,
            'secret': config.binance.api_secret,
            'enableRateLimit': True,
            'options': {
                'defaultType': 'future',  # 선물 거래
                'testnet': config.binance.testnet
            }
        })
        
        if config.binance.testnet:
            self.exchange.set_sandbox_mode(True)
            
        logger.info(f"Binance client initialized (testnet: {config.binance.testnet})")
    
    def fetch_ohlcv(
        self, 
        symbol: str = "BTC/USDT",
        timeframe: str = "1m",
        limit: int = 500,
        since: Optional[int] = None
    ) -> pd.DataFrame:
        """
        Fetch OHLCV data from Binance
        
        Args:
            symbol: Trading pair symbol
            timeframe: Candle timeframe (1m, 5m, 15m, 1h, 4h, 1d)
            limit: Number of candles to fetch
            since: Start timestamp in milliseconds
            
        Returns:
            DataFrame with OHLCV data
        """
        try:
            ohlcv = self.exchange.fetch_ohlcv(
                symbol=symbol,
                timeframe=timeframe,
                limit=limit,
                since=since
            )
            
            df = pd.DataFrame(
                ohlcv,
                columns=['timestamp', 'open', 'high', 'low', 'close', 'volume']
            )
            df['timestamp'] = pd.to_datetime(df['timestamp'], unit='ms')
            df.set_index('timestamp', inplace=True)
            
            logger.info(f"Fetched {len(df)} candles for {symbol} {timeframe}")
            return df
            
        except Exception as e:
            logger.error(f"Error fetching OHLCV data: {e}")
            raise
    
    def fetch_historical_data(
        self,
        symbol: str = "BTC/USDT",
        timeframe: str = "1m",
        days: int = 30
    ) -> pd.DataFrame:
        """
        Fetch historical data for specified number of days
        
        Args:
            symbol: Trading pair symbol
            timeframe: Candle timeframe
            days: Number of days to fetch
            
        Returns:
            DataFrame with historical OHLCV data
        """
        all_data = []
        
        # Calculate time range
        end_time = datetime.now()
        start_time = end_time - timedelta(days=days)
        
        # Convert to milliseconds
        since = int(start_time.timestamp() * 1000)
        
        # Fetch data in chunks
        while since < int(end_time.timestamp() * 1000):
            try:
                df = self.fetch_ohlcv(
                    symbol=symbol,
                    timeframe=timeframe,
                    limit=1000,
                    since=since
                )
                
                if df.empty:
                    break
                    
                all_data.append(df)
                since = int(df.index[-1].timestamp() * 1000) + 1
                
                # Rate limiting
                time.sleep(0.1)
                
            except Exception as e:
                logger.error(f"Error fetching historical data chunk: {e}")
                break
        
        if all_data:
            result = pd.concat(all_data)
            result = result[~result.index.duplicated(keep='first')]
            result.sort_index(inplace=True)
            
            logger.info(f"Fetched {len(result)} total candles for {symbol}")
            return result
        else:
            return pd.DataFrame()
    
    def fetch_ticker(self, symbol: str = "BTC/USDT") -> Dict:
        """
        Fetch current ticker data
        
        Args:
            symbol: Trading pair symbol
            
        Returns:
            Dictionary with ticker data
        """
        try:
            ticker = self.exchange.fetch_ticker(symbol)
            return {
                'symbol': ticker['symbol'],
                'last': ticker['last'],
                'bid': ticker['bid'],
                'ask': ticker['ask'],
                'volume': ticker['baseVolume'],
                'quote_volume': ticker['quoteVolume'],
                'percentage': ticker['percentage'],
                'timestamp': ticker['timestamp']
            }
        except Exception as e:
            logger.error(f"Error fetching ticker: {e}")
            raise
    
    def fetch_order_book(
        self, 
        symbol: str = "BTC/USDT",
        limit: int = 20
    ) -> Dict:
        """
        Fetch order book data
        
        Args:
            symbol: Trading pair symbol
            limit: Depth of order book
            
        Returns:
            Dictionary with order book data
        """
        try:
            order_book = self.exchange.fetch_order_book(symbol, limit)
            return {
                'bids': order_book['bids'][:limit],
                'asks': order_book['asks'][:limit],
                'timestamp': order_book['timestamp']
            }
        except Exception as e:
            logger.error(f"Error fetching order book: {e}")
            raise
    
    def fetch_funding_rate(self, symbol: str = "BTC/USDT") -> Dict:
        """
        Fetch funding rate for perpetual futures
        
        Args:
            symbol: Trading pair symbol
            
        Returns:
            Dictionary with funding rate data
        """
        try:
            funding = self.exchange.fetch_funding_rate(symbol)
            return {
                'symbol': funding['symbol'],
                'funding_rate': funding['fundingRate'],
                'funding_timestamp': funding['fundingDatetime'],
                'timestamp': funding['timestamp']
            }
        except Exception as e:
            logger.error(f"Error fetching funding rate: {e}")
            return {}
    
    def save_to_csv(self, df: pd.DataFrame, filename: str):
        """
        Save DataFrame to CSV file
        
        Args:
            df: DataFrame to save
            filename: Output filename
        """
        try:
            df.to_csv(f"data/{filename}")
            logger.info(f"Data saved to data/{filename}")
        except Exception as e:
            logger.error(f"Error saving to CSV: {e}")
            raise
    
    def stream_realtime_data(
        self,
        symbol: str = "BTC/USDT",
        callback=None
    ):
        """
        Stream real-time market data (placeholder for WebSocket implementation)
        
        Args:
            symbol: Trading pair symbol
            callback: Callback function for processing streaming data
        """
        logger.info(f"Starting real-time data stream for {symbol}")
        
        # This would be replaced with actual WebSocket implementation
        while True:
            try:
                ticker = self.fetch_ticker(symbol)
                if callback:
                    callback(ticker)
                time.sleep(1)  # Simulate real-time updates
                
            except KeyboardInterrupt:
                logger.info("Stopping real-time data stream")
                break
            except Exception as e:
                logger.error(f"Error in real-time stream: {e}")
                time.sleep(5)  # Wait before retrying

# Example usage
if __name__ == "__main__":
    collector = BinanceDataCollector()
    
    # Fetch recent data
    df = collector.fetch_ohlcv("BTC/USDT", "1h", limit=100)
    print(df.head())
    
    # Fetch ticker
    ticker = collector.fetch_ticker("BTC/USDT")
    print(f"Current BTC price: {ticker['last']}")