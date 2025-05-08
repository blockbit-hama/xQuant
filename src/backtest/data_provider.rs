use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, Read};
use async_trait::async_trait;
use csv::ReaderBuilder;
use chrono::{DateTime, Utc, NaiveDateTime};

use crate::error::TradingError;
use crate::models::market_data::MarketData;

/// 백테스트용 데이터 제공자 인터페이스
#[async_trait]
pub trait BacktestDataProvider: Send + Sync {
    /// 데이터 로드
    async fn load_data(&self) -> Result<HashMap<String, Vec<MarketData>>, TradingError>;
}

/// CSV 파일 기반 데이터 제공자
pub struct CsvDataProvider {
    file_path: PathBuf,
    symbol: String,
}

impl CsvDataProvider {
    pub fn new(file_path: impl AsRef<Path>) -> Result<Self, TradingError> {
        let file_path = file_path.as_ref().to_path_buf();

        // 파일 이름에서 심볼 추출 (예: BTCUSDT-1m.csv -> BTCUSDT)
        let file_name = file_path.file_name()
            .ok_or_else(|| TradingError::InvalidParameter("Invalid file path".to_string()))?
            .to_string_lossy();

        let symbol = file_name.split('-')
            .next()
            .ok_or_else(|| TradingError::InvalidParameter("Invalid file name format".to_string()))?
            .to_string();

        Ok(CsvDataProvider {
            file_path,
            symbol,
        })
    }

    /// CSV 파일에서 시장 데이터 읽기
    fn read_csv_file(&self) -> Result<Vec<MarketData>, TradingError> {
        let file = File::open(&self.file_path)
            .map_err(|e| TradingError::IoError(e))?;

        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut data_series = Vec::new();

        for result in csv_reader.records() {
            let record = result.map_err(|e| TradingError::ParseError(e.to_string()))?;

            if record.len() < 6 {
                return Err(TradingError::ParseError("Invalid CSV format".to_string()));
            }

            // CSV 형식: timestamp, open, high, low, close, volume
            let timestamp = record[0].parse::<i64>()
                .map_err(|e| TradingError::ParseError(format!("Invalid timestamp: {}", e)))?;

            let open = record[1].parse::<f64>()
                .map_err(|e| TradingError::ParseError(format!("Invalid open price: {}", e)))?;

            let high = record[2].parse::<f64>()
                .map_err(|e| TradingError::ParseError(format!("Invalid high price: {}", e)))?;

            let low = record[3].parse::<f64>()
                .map_err(|e| TradingError::ParseError(format!("Invalid low price: {}", e)))?;

            let close = record[4].parse::<f64>()
                .map_err(|e| TradingError::ParseError(format!("Invalid close price: {}", e)))?;

            let volume = record[5].parse::<f64>()
                .map_err(|e| TradingError::ParseError(format!("Invalid volume: {}", e)))?;

            let market_data = MarketData {
                symbol: self.symbol.clone(),
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            };

            data_series.push(market_data);
        }

        // 시간순으로 정렬
        data_series.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(data_series)
    }
}

#[async_trait]
impl BacktestDataProvider for CsvDataProvider {
    async fn load_data(&self) -> Result<HashMap<String, Vec<MarketData>>, TradingError> {
        let data_series = self.read_csv_file()?;
        let mut data_map = HashMap::new();
        data_map.insert(self.symbol.clone(), data_series);
        Ok(data_map)
    }
}

/// 다중 CSV 파일 데이터 제공자
pub struct MultiCsvDataProvider {
    file_paths: Vec<PathBuf>,
}

impl MultiCsvDataProvider {
    pub fn new(file_paths: Vec<PathBuf>) -> Self {
        MultiCsvDataProvider {
            file_paths,
        }
    }
}

#[async_trait]
impl BacktestDataProvider for MultiCsvDataProvider {
    async fn load_data(&self) -> Result<HashMap<String, Vec<MarketData>>, TradingError> {
        let mut data_map = HashMap::new();

        for file_path in &self.file_paths {
            let provider = CsvDataProvider::new(file_path)?;
            let symbol_data = provider.read_csv_file()?;

            data_map.insert(provider.symbol.clone(), symbol_data);
        }

        Ok(data_map)
    }
}