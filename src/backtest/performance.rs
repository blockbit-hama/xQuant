/**
* filename : performance
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::models::trade::Trade;

/// 성능 지표 계산 유틸리티
pub struct PerformanceMetrics;

impl PerformanceMetrics {
  /// 샤프 비율 계산
  pub fn calculate_sharpe_ratio(trades: &[Trade], initial_capital: f64) -> f64 {
    if trades.is_empty() {
      return 0.0;
    }
    
    // 일별 수익률 계산
    let daily_returns = Self::calculate_daily_returns(trades, initial_capital);
    
    if daily_returns.is_empty() {
      return 0.0;
    }
    
    // 평균과 표준편차 계산
    let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    
    let variance = daily_returns.iter()
      .map(|r| (r - mean_return).powi(2))
      .sum::<f64>() / daily_returns.len() as f64;
    
    let std_dev = variance.sqrt();
    
    if std_dev == 0.0 {
      return 0.0;
    }
    
    // 연간화된 샤프 비율
    // 252는 일반적인 연간 거래일 수
    let annualized_sharpe = (mean_return / std_dev) * (252.0_f64).sqrt();
    
    annualized_sharpe
  }
  
  /// 최대 손실폭 계산
  pub fn calculate_max_drawdown(trades: &[Trade], initial_capital: f64) -> f64 {
    if trades.is_empty() {
      return 0.0;
    }
    
    // 자산 가치 시계열 구성
    let mut equity_curve = vec![initial_capital];
    let mut current_equity = initial_capital;
    
    for trade in trades {
      current_equity += trade.realized_pnl;
      equity_curve.push(current_equity);
    }
    
    // 최대 손실폭 계산
    let mut max_drawdown = 0.0;
    let mut peak = equity_curve[0];
    
    for &equity in &equity_curve {
      if equity > peak {
        peak = equity;
      } else {
        let drawdown = (peak - equity) / peak;
        max_drawdown = max_drawdown.max(drawdown);
      }
    }
    
    max_drawdown
  }
  
  /// 수익 대 위험 비율 계산
  pub fn calculate_profit_factor(trades: &[Trade]) -> f64 {
    let gross_profit: f64 = trades.iter()
      .filter(|t| t.realized_pnl > 0.0)
      .map(|t| t.realized_pnl)
      .sum();
    
    let gross_loss: f64 = trades.iter()
      .filter(|t| t.realized_pnl < 0.0)
      .map(|t| t.realized_pnl.abs())
      .sum();
    
    if gross_loss == 0.0 {
      return if gross_profit > 0.0 { f64::INFINITY } else { 0.0 };
    }
    
    gross_profit / gross_loss
  }
  
  /// 일별 수익률 계산
  fn calculate_daily_returns(trades: &[Trade], initial_capital: f64) -> Vec<f64> {
    if trades.is_empty() {
      return Vec::new();
    }
    
    // 거래를 일자별로 그룹화
    let mut daily_pnl: HashMap<String, f64> = HashMap::new();
    
    for trade in trades {
      let date = trade.timestamp.format("%Y-%m-%d").to_string();
      *daily_pnl.entry(date).or_default() += trade.realized_pnl;
    }
    
    // 일별 수익률 계산
    let mut dates: Vec<String> = daily_pnl.keys().cloned().collect();
    dates.sort();
    
    let mut capital = initial_capital;
    let mut daily_returns = Vec::new();
    
    for date in dates {
      let day_pnl = daily_pnl[&date];
      let daily_return = day_pnl / capital;
      daily_returns.push(daily_return);
      capital += day_pnl;
    }
    
    daily_returns
  }
}