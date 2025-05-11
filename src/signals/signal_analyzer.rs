/**
* filename : signal_analyzer
* author : HAMA
* date: 2025. 5. 11.
* description: 
**/

use std::collections::HashMap;
use crate::indicators::IndicatorResult;
use super::signal_types::{SignalType, SignalWithMetadata};

pub struct SignalAnalyzer {
  indicator_weights: HashMap<String, f64>,
  conflicting_threshold: f64,
  min_confidence: f64,
}

impl SignalAnalyzer {
  pub fn new() -> Self {
    let mut weights = HashMap::new();
    
    // 기본 가중치 설정
    weights.insert("Golden Cross".to_string(), 0.7);
    weights.insert("Death Cross".to_string(), 0.7);
    weights.insert("RSI Overbought".to_string(), 0.6);
    weights.insert("RSI Oversold".to_string(), 0.6);
    weights.insert("MACD Bullish Crossover".to_string(), 0.8);
    weights.insert("MACD Bearish Crossover".to_string(), 0.8);
    weights.insert("MACD Above Zero".to_string(), 0.3);
    weights.insert("MACD Below Zero".to_string(), 0.3);
    weights.insert("Price Above VWAP".to_string(), 0.4);
    weights.insert("Price Below VWAP".to_string(), 0.4);
    
    SignalAnalyzer {
      indicator_weights: weights,
      conflicting_threshold: 0.3,
      min_confidence: 0.5,
    }
  }
  
  // 가중치 설정
  pub fn set_weight(&mut self, indicator_name: &str, weight: f64) {
    self.indicator_weights.insert(indicator_name.to_string(), weight);
  }
  
  // 여러 지표 결과에서 신호 분석
  pub fn analyze_indicator_results(&self, results: &[IndicatorResult]) -> Vec<SignalWithMetadata> {
    let mut all_signals = Vec::new();
    
    // 모든 지표 신호 수집
    for result in results {
      for signal in &result.signals {
        let weight = self.indicator_weights.get(&signal.name).unwrap_or(&0.5);
        
        // 가중치를 신뢰도로 사용
        let signal_metadata = SignalWithMetadata::from_indicator_signal(signal)
          .with_confidence(*weight);
        
        all_signals.push(signal_metadata);
      }
    }
    
    // 신뢰도가 최소 임계값보다 높은 신호만 필터링
    all_signals.retain(|signal| signal.confidence >= self.min_confidence);
    
    // 여기서 필요하면 신호 간의 충돌 해결 로직을 추가할 수 있음
    self.resolve_conflicting_signals(all_signals)
  }
  
  // 충돌하는 신호 해결
  fn resolve_conflicting_signals(&self, signals: Vec<SignalWithMetadata>) -> Vec<SignalWithMetadata> {
    if signals.is_empty() {
      return Vec::new();
    }
    
    // 매수/매도 신호별 강도 합계
    let mut buy_strength = 0.0;
    let mut sell_strength = 0.0;
    
    for signal in &signals {
      match signal.signal_type {
        SignalType::Buy | SignalType::StrongBuy | SignalType::ReduceShort =>
          buy_strength += signal.strength * signal.confidence,
        
        SignalType::Sell | SignalType::StrongSell | SignalType::ReduceLong =>
          sell_strength += signal.strength.abs() * signal.confidence,
        
        _ => {}
      }
    }
    
    // 신호 간의 충돌이 심한 경우
    if buy_strength > 0.0 && sell_strength > 0.0 {
      let net_strength = buy_strength - sell_strength;
      
      // 충돌이 임계값 이내면 모든 신호 반환
      if net_strength.abs() <= self.conflicting_threshold {
        return signals;
      }
      
      // 그렇지 않으면 우세한 방향의 신호만 반환
      if net_strength > 0.0 {
        return signals.into_iter()
          .filter(|s| matches!(s.signal_type,
                        SignalType::Buy | SignalType::StrongBuy | SignalType::ReduceShort))
          .collect();
      } else {
        return signals.into_iter()
          .filter(|s| matches!(s.signal_type,
                        SignalType::Sell | SignalType::StrongSell | SignalType::ReduceLong))
          .collect();
      }
    }
    
    // 충돌이 없으면 모든 신호 반환
    signals
  }
}