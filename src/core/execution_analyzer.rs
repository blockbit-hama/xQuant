/**
* filename : execution_analyzer
* author : HAMA
* date: 2025. 5. 8.
* description: 
**/

use std::collections::HashMap;

use crate::models::market_data::MarketData;
use crate::models::order::{Order, OrderSide};
use crate::models::trade::Trade;

/// 실행 성능 분석기
pub struct ExecutionAnalyzer {
  /// 거래 내역
  trades: Vec<Trade>,
  /// 시장 데이터 내역
  market_data: Vec<MarketData>,
  /// 분석 대상 심볼
  symbol: String,
  /// 거래량 가중 평균 가격
  vwap: f64,
  /// 시간 가중 평균 가격
  twap: f64,
  /// 슬리피지 (%)
  slippage: f64,
  /// 시장 영향 (%)
  impact: f64,
}

impl ExecutionAnalyzer {
  /// 새 실행 분석기 생성
  pub fn new(symbol: impl Into<String>) -> Self {
    ExecutionAnalyzer {
      trades: Vec::new(),
      market_data: Vec::new(),
      symbol: symbol.into(),
      vwap: 0.0,
      twap: 0.0,
      slippage: 0.0,
      impact: 0.0,
    }
  }
  
  /// 분석을 위한 거래 추가
  pub fn add_trade(&mut self, trade: Trade) {
    if trade.symbol == self.symbol {
      self.trades.push(trade);
      self.calculate_metrics();
    }
  }
  
  /// 분석을 위한 시장 데이터 추가
  pub fn add_market_data(&mut self, data: MarketData) {
    if data.symbol == self.symbol {
      self.market_data.push(data);
      self.calculate_metrics();
    }
  }
  
  /// 실행 지표 계산
  pub fn calculate_metrics(&mut self) {
    if self.trades.is_empty() || self.market_data.is_empty() {
      return;
    }
    
    self.calculate_vwap();
    self.calculate_twap();
    self.calculate_slippage();
    self.calculate_market_impact();
  }
  
  /// VWAP (거래량 가중 평균 가격) 계산
  fn calculate_vwap(&mut self) {
    let mut total_value = 0.0;
    let mut total_volume = 0.0;
    
    for trade in &self.trades {
      let value = trade.price * trade.quantity;
      total_value += value;
      total_volume += trade.quantity;
    }
    
    if total_volume > 0.0 {
      self.vwap = total_value / total_volume;
    }
  }
  
  /// TWAP (시간 가중 평균 가격) 계산
  fn calculate_twap(&mut self) {
    if self.market_data.is_empty() {
      return;
    }
    
    let mut total = 0.0;
    
    for data in &self.market_data {
      total += data.close;
    }
    
    self.twap = total / self.market_data.len() as f64;
  }
  
  /// VWAP 대비 슬리피지 계산
  fn calculate_slippage(&mut self) {
    if self.vwap == 0.0 || self.trades.is_empty() {
      return;
    }
    
    let mut total_slippage = 0.0;
    let mut total_volume = 0.0;
    
    for trade in &self.trades {
      let slippage = match trade.side {
        OrderSide::Buy => (trade.price - self.vwap) / self.vwap * 100.0,
        OrderSide::Sell => (self.vwap - trade.price) / self.vwap * 100.0,
      };
      
      total_slippage += slippage * trade.quantity;
      total_volume += trade.quantity;
    }
    
    if total_volume > 0.0 {
      self.slippage = total_slippage / total_volume;
    }
  }
  
  /// 시장 영향 계산
  fn calculate_market_impact(&mut self) {
    if self.trades.is_empty() || self.market_data.len() < 2 {
      return;
    }
    
    // 거래 전후 가격 가져오기
    let start_price = self.market_data.first().unwrap().close;
    let end_price = self.market_data.last().unwrap().close;
    
    // 총 거래량 계산
    let mut total_volume = 0.0;
    let mut is_buy = false;
    
    for trade in &self.trades {
      total_volume += trade.quantity;
      is_buy = trade.side == OrderSide::Buy;
    }
    
    // 가격 변화율 계산
    let price_change = (end_price - start_price) / start_price * 100.0;
    
    // 영향은 거래 방향으로 가격이 움직였으면 양수
    self.impact = if is_buy {
      price_change
    } else {
      -price_change
    };
  }
  
  /// 분석 보고서 생성
  pub fn get_report(&self) -> HashMap<String, f64> {
    let mut report = HashMap::new();
    
    report.insert("vwap".to_string(), self.vwap);
    report.insert("twap".to_string(), self.twap);
    report.insert("slippage".to_string(), self.slippage);
    report.insert("market_impact".to_string(), self.impact);
    
    if self.trades.is_empty() {
      return report;
    }
    
    // 추가 지표 계산
    let mut total_value = 0.0;
    let mut total_quantity = 0.0;
    
    for trade in &self.trades {
      total_value += trade.price * trade.quantity;
      total_quantity += trade.quantity;
    }
    
    let avg_price = if total_quantity > 0.0 {
      total_value / total_quantity
    } else {
      0.0
    };
    
    report.insert("average_price".to_string(), avg_price);
    report.insert("total_quantity".to_string(), total_quantity);
    report.insert("total_value".to_string(), total_value);
    report.insert("trade_count".to_string(), self.trades.len() as f64);
    
    report
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::order::OrderId;
  
  #[test]
  fn test_execution_analyzer() {
    let mut analyzer = ExecutionAnalyzer::new("BTCUSDT");
    
    // 시장 데이터 추가
    let md1 = MarketData {
      symbol: "BTCUSDT".to_string(),
      timestamp: 1000,
      open: 50000.0,
      high: 50100.0,
      low: 49900.0,
      close: 50000.0,
      volume: 10.0,
    };
    
    let md2 = MarketData {
      symbol: "BTCUSDT".to_string(),
      timestamp: 2000,
      open: 50000.0,
      high: 50200.0,
      low: 49950.0,
      close: 50100.0,
      volume: 15.0,
    };
    
    analyzer.add_market_data(md1);
    analyzer.add_market_data(md2);
    
    // 거래 내역 추가
    let trade1 = Trade {
      id: "trade1".to_string(),
      symbol: "BTCUSDT".to_string(),
      price: 50050.0,
      quantity: 0.5,
      timestamp: 1500,
      order_id: OrderId("order1".to_string()),
      side: OrderSide::Buy,
    };
    
    let trade2 = Trade {
      id: "trade2".to_string(),
      symbol: "BTCUSDT".to_string(),
      price: 50080.0,
      quantity: 0.3,
      timestamp: 1800,
      order_id: OrderId("order2".to_string()),
      side: OrderSide::Buy,
    };
    
    analyzer.add_trade(trade1);
    analyzer.add_trade(trade2);
    
    // 보고서 생성
    let report = analyzer.get_report();
    
    // 보고서 검증
    assert!(report.contains_key("vwap"));
    assert!(report.contains_key("twap"));
    assert!(report.contains_key("slippage"));
    assert!(report.contains_key("market_impact"));
    assert!(report.contains_key("average_price"));
    assert!(report.contains_key("total_quantity"));
    
    assert!(report["vwap"] > 0.0);
    assert_eq!(report["trade_count"], 2.0);
    assert_eq!(report["total_quantity"], 0.8);
  }
}