use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::models::trade::Trade;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BacktestResult {
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
}

impl BacktestResult {
    /// 거래 수 계산
    pub fn trade_count(&self) -> usize {
        self.trades.len()
    }

    /// 승률 계산
    pub fn win_rate(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }

        let mut winning_trades = 0;
        let mut losing_trades = 0;

        // 각 거래별 이익/손실 계산
        for trade in &self.trades {
            let profit = self.calculate_trade_profit(trade);

            if profit > 0.0 {
                winning_trades += 1;
            } else if profit < 0.0 {
                losing_trades += 1;
            }
        }

        let total_trades = winning_trades + losing_trades;
        if total_trades == 0 {
            return 0.0;
        }

        (winning_trades as f64 / total_trades as f64) * 100.0
    }

    /// 최대 이익 거래 계산
    pub fn max_profit_trade(&self) -> Option<(&Trade, f64)> {
        if self.trades.is_empty() {
            return None;
        }

        let mut max_profit = 0.0;
        let mut max_profit_trade = None;

        for trade in &self.trades {
            let profit = self.calculate_trade_profit(trade);

            if profit > max_profit {
                max_profit = profit;
                max_profit_trade = Some((trade, profit));
            }
        }

        max_profit_trade
    }

    /// 최대 손실 거래 계산
    pub fn max_loss_trade(&self) -> Option<(&Trade, f64)> {
        if self.trades.is_empty() {
            return None;
        }

        let mut max_loss = 0.0;
        let mut max_loss_trade = None;

        for trade in &self.trades {
            let profit = self.calculate_trade_profit(trade);

            if profit < max_loss {
                max_loss = profit;
                max_loss_trade = Some((trade, profit));
            }
        }

        max_loss_trade
    }

    /// 거래당 평균 이익/손실 계산
    pub fn average_profit_per_trade(&self) -> f64 {
        if self.trades.is_empty() {
            return 0.0;
        }

        let total_profit: f64 = self.trades.iter()
            .map(|trade| self.calculate_trade_profit(trade))
            .sum();

        total_profit / self.trades.len() as f64
    }

    /// 샤프 비율 계산 (간단한 구현)
    pub fn sharpe_ratio(&self) -> f64 {
        if self.trades.len() < 2 {
            return 0.0;
        }

        // 일별 수익률 계산 (거래 기반)
        let profits: Vec<f64> = self.trades.iter()
            .map(|trade| self.calculate_trade_profit(trade))
            .collect();

        // 평균 수익률
        let mean = profits.iter().sum::<f64>() / profits.len() as f64;

        // 표준 편차
        let variance = profits.iter()
            .map(|profit| (profit - mean).powi(2))
            .sum::<f64>() / (profits.len() as f64 - 1.0);

        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        // 무위험 이자율 (예: 0%)
        let risk_free_rate = 0.0;

        // 샤프 비율
        (mean - risk_free_rate) / std_dev
    }

    /// 개별 거래 이익/손실 계산 (단순화된 구현)
    fn calculate_trade_profit(&self, trade: &Trade) -> f64 {
        // 실제로는 이전 거래와 현재 잔고를 고려해야 함
        // 여기서는 단순 구현
        match trade.side {
            crate::models::order::OrderSide::Buy => 0.0, // 매수는 포지션 진입으로 간주
            crate::models::order::OrderSide::Sell => trade.price * trade.quantity, // 매도는 이익 실현으로 간주
        }
    }

    /// 결과 요약 문자열 생성
    pub fn summary(&self) -> String {
        format!(
            "백테스트 결과 요약:\n\
             기간: {} ~ {}\n\
             초기 가치: {:.2} USDT\n\
             최종 가치: {:.2} USDT\n\
             이익: {:.2} USDT ({:.2}%)\n\
             거래 횟수: {}\n\
             승률: {:.2}%\n\
             수수료 총액: {:.2} USDT\n\
             샤프 비율: {:.2}",
            self.start_time.format("%Y-%m-%d %H:%M:%S"),
            self.end_time.format("%Y-%m-%d %H:%M:%S"),
            self.initial_value,
            self.final_value,
            self.profit,
            self.profit_percentage,
            self.trade_count(),
            self.win_rate(),
            self.fee_paid,
            self.sharpe_ratio()
        )
    }
}