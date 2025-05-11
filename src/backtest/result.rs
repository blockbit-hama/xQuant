use std::collections::HashMap;
use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::models::trade::Trade;
use super::performance::PerformanceMetrics;

/// 백테스트 결과 - 백테스트 실행 결과를 저장하고 분석
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BacktestResult {
    pub name: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: HashMap<String, f64>,
    pub final_balance: HashMap<String, f64>,
    pub initial_value: f64,
    pub final_value: f64,
    pub profit: f64,
    pub profit_percentage: f64,
    pub trades: Vec<Trade>,
    pub fee_paid: f64,
    pub symbols: Vec<String>,
}

impl BacktestResult {
    /// 거래 횟수 반환
    pub fn trade_count(&self) -> usize {
        self.trades.len()
    }
    
    /// 승리 거래 수 반환
    pub fn winning_trades(&self) -> usize {
        self.trades.iter()
          .filter(|t| t.realized_pnl > 0.0)
          .count()
    }
    
    /// 패배 거래 수 반환
    pub fn losing_trades(&self) -> usize {
        self.trades.iter()
          .filter(|t| t.realized_pnl < 0.0)
          .count()
    }
    
    /// 승률 계산
    pub fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        
        (self.winning_trades() as f64 / self.trades.len() as f64) * 100.0
    }
    
    /// 평균 거래당 수익 계산
    pub fn average_profit_per_trade(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }
        
        let total_pnl: f64 = self.trades.iter()
          .map(|t| t.realized_pnl)
          .sum();
        
        total_pnl / self.trades.len() as f64
    }
    
    /// 최대 수익 거래 찾기
    pub fn max_profit_trade(&self) -> Option<(&Trade, f64)> {
        self.trades.iter()
          .filter(|t| t.realized_pnl > 0.0)
          .map(|t| (t, t.realized_pnl))
          .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    
    /// 최대 손실 거래 찾기
    pub fn max_loss_trade(&self) -> Option<(&Trade, f64)> {
        self.trades.iter()
          .filter(|t| t.realized_pnl < 0.0)
          .map(|t| (t, t.realized_pnl))
          .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
    
    /// 샤프 비율 계산
    pub fn sharpe_ratio(&self) -> f64 {
        PerformanceMetrics::calculate_sharpe_ratio(&self.trades, self.initial_value)
    }
    
    /// 최대 손실폭 계산
    pub fn max_drawdown(&self) -> f64 {
        PerformanceMetrics::calculate_max_drawdown(&self.trades, self.initial_value)
    }
    
    /// 수익 대 위험 비율 계산
    pub fn profit_factor(&self) -> f64 {
        PerformanceMetrics::calculate_profit_factor(&self.trades)
    }
    
    /// 연간 복합 수익률 계산
    pub fn car(&self) -> f64 {
        let days = (self.end_time - self.start_time).num_days() as f64;
        
        if days <= 0.0 || self.initial_value <= 0.0 {
            return 0.0;
        }
        
        let years = days / 365.0;
        ((self.final_value / self.initial_value).powf(1.0 / years)) - 1.0
    }
    
    /// 결과 요약 문자열 생성
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("===== 백테스트 결과: {} =====\n", self.name));
        summary.push_str(&format!("설명: {}\n", self.description));
        summary.push_str(&format!("기간: {} ~ {}\n", self.start_time, self.end_time));
        summary.push_str(&format!("심볼: {}\n", self.symbols.join(", ")));
        summary.push_str("\n");
        
        summary.push_str(&format!("초기 자산가치: ${:.2}\n", self.initial_value));
        summary.push_str(&format!("최종 자산가치: ${:.2}\n", self.final_value));
        summary.push_str(&format!("순이익: ${:.2} ({:.2}%)\n", self.profit, self.profit_percentage));
        summary.push_str(&format!("지불 수수료: ${:.2}\n", self.fee_paid));
        summary.push_str("\n");
        
        summary.push_str(&format!("총 거래 수: {}\n", self.trade_count()));
        summary.push_str(&format!("승리 거래: {}\n", self.winning_trades()));
        summary.push_str(&format!("패배 거래: {}\n", self.losing_trades()));
        summary.push_str(&format!("승률: {:.2}%\n", self.win_rate()));
        summary.push_str(&format!("평균 거래당 수익: ${:.2}\n", self.average_profit_per_trade()));
        summary.push_str("\n");
        
        summary.push_str(&format!("샤프 비율: {:.4}\n", self.sharpe_ratio()));
        summary.push_str(&format!("최대 손실폭: {:.2}%\n", self.max_drawdown() * 100.0));
        summary.push_str(&format!("수익/위험 비율: {:.2}\n", self.profit_factor()));
        summary.push_str(&format!("연간 복합 수익률: {:.2}%\n", self.car() * 100.0));
        
        summary
    }
}

impl fmt::Display for BacktestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}