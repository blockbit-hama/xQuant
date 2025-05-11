/**
* filename : utils
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use crate::error::TradingError;
use crate::models::market_data::MarketData;
use super::Indicator;

// 단일 가격 데이터를 사용하여 여러 지표 업데이트
pub fn update_indicators(
  indicators: &mut [Box<dyn Indicator>],
  price: f64,
  volume: Option<f64>
) -> Result<(), TradingError> {
  for indicator in indicators.iter_mut() {
    indicator.update(price, volume)?;
  }
  
  Ok(())
}

// MarketData를 사용하여 여러 지표 업데이트
pub fn update_indicators_with_market_data(
  indicators: &mut [Box<dyn Indicator>],
  market_data: &MarketData
) -> Result<(), TradingError> {
  update_indicators(indicators, market_data.close_price, Some(market_data.volume))
}

// 지표 초기화
pub fn reset_indicators(indicators: &mut [Box<dyn Indicator>]) {
  for indicator in indicators.iter_mut() {
    indicator.reset();
  }
}