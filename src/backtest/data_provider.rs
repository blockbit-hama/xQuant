use std::path::PathBuf;
use chrono::{DateTime, Utc};

use crate::error::TradingError;
use crate::models::market_data::MarketData;

pub trait HistoricalDataProvider {
    fn available_symbols(&self) -> Vec<String>;
    fn load_data(
        &self,
        symbol: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<MarketData>, TradingError>;
}

pub struct CsvDataProvider {
    path: PathBuf,
    delimiter: u8,
}

impl CsvDataProvider {
    pub fn new(path: PathBuf, delimiter: char) -> Result<Self, TradingError> {
        Ok(Self { path, delimiter: delimiter as u8 })
    }
}

impl HistoricalDataProvider for CsvDataProvider {
    fn available_symbols(&self) -> Vec<String> {
        // Single-file CSV provider: infer symbol from filename
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| vec![s.to_string()])
            .unwrap_or_default()
    }

    fn load_data(
        &self,
        _symbol: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<MarketData>, TradingError> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.delimiter)
            .from_path(&self.path)
            .map_err(|e| TradingError::IoError(e.into()))?;

        let mut result = Vec::new();
        for rec in rdr.deserialize() {
            let row: CsvRow = rec.map_err(|e| TradingError::ParseError(e.to_string()))?;
            if row.timestamp >= start_time.timestamp_millis() && row.timestamp <= end_time.timestamp_millis() {
                result.push(MarketData {
                    symbol: row.symbol,
                    timestamp: row.timestamp,
                    open: row.open,
                    high: row.high,
                    low: row.low,
                    close: row.close,
                    volume: row.volume,
                });
            }
        }
        Ok(result)
    }
}

#[derive(serde::Deserialize)]
struct CsvRow {
    symbol: String,
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}
