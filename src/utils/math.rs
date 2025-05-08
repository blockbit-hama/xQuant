//! 수학 관련 유틸리티
//!
//! 통계, 금융 계산, 가격 분석 함수 제공

/// 최소값 계산
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
  if a < b { a } else { b }
}

/// 최대값 계산
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
  if a > b { a } else { b }
}

/// 값을 범위 내로 제한
pub fn clamp<T: PartialOrd>(value: T, min_value: T, max_value: T) -> T {
  max(min_value, min(max_value, value))
}

/// 수량 단위 반올림 (거래소 요구사항에 맞춰)
pub fn round_quantity(quantity: f64, step_size: f64) -> f64 {
  (quantity / step_size).floor() * step_size
}

/// 가격 단위 반올림 (거래소 요구사항에 맞춰)
pub fn round_price(price: f64, tick_size: f64) -> f64 {
  (price / tick_size).floor() * tick_size
}

/// 평균 계산
pub fn average(values: &[f64]) -> Option<f64> {
  if values.is_empty() {
    return None;
  }
  
  Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// 표준 편차 계산
pub fn standard_deviation(values: &[f64]) -> Option<f64> {
  if values.is_empty() {
    return None;
  }
  
  let avg = average(values)?;
  let variance = values.iter()
    .map(|value| {
      let diff = avg - *value;
      diff * diff
    })
    .sum::<f64>() / values.len() as f64;
  
  Some(variance.sqrt())
}

/// VWAP (거래량 가중 평균 가격) 계산
pub fn calculate_vwap(prices: &[f64], volumes: &[f64]) -> Option<f64> {
  if prices.len() != volumes.len() || prices.is_empty() {
    return None;
  }
  
  let total_volume: f64 = volumes.iter().sum();
  if total_volume == 0.0 {
    return None;
  }
  
  let sum_pv: f64 = prices.iter()
    .zip(volumes.iter())
    .map(|(p, v)| p * v)
    .sum();
  
  Some(sum_pv / total_volume)
}

/// TWAP (시간 가중 평균 가격) 계산
pub fn calculate_twap(prices: &[f64]) -> Option<f64> {
  average(prices)
}

/// 수익률 계산 (백분율)
pub fn calculate_return(entry_price: f64, exit_price: f64) -> f64 {
  (exit_price - entry_price) / entry_price * 100.0
}

/// 샤프 비율 계산 (간단 구현)
pub fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> Option<f64> {
  if returns.len() < 2 {
    return None;
  }
  
  let avg_return = average(returns)?;
  let std_dev = standard_deviation(returns)?;
  
  if std_dev == 0.0 {
    return None;
  }
  
  Some((avg_return - risk_free_rate) / std_dev)
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_min_max() {
    assert_eq!(min(5, 10), 5);
    assert_eq!(max(5, 10), 10);
    assert_eq!(min(-5.5, 3.3), -5.5);
    assert_eq!(max(-5.5, 3.3), 3.3);
  }
  
  #[test]
  fn test_clamp() {
    assert_eq!(clamp(5, 0, 10), 5);
    assert_eq!(clamp(-5, 0, 10), 0);
    assert_eq!(clamp(15, 0, 10), 10);
  }
  
  #[test]
  fn test_round_quantity_price() {
    assert_eq!(round_quantity(1.23456, 0.01), 1.23);
    assert_eq!(round_quantity(1.23456, 0.001), 1.234);
    assert_eq!(round_price(50123.45, 0.1), 50123.4);
    assert_eq!(round_price(50123.45, 10.0), 50120.0);
  }
  
  #[test]
  fn test_vwap() {
    let prices = vec![100.0, 101.0, 102.0, 103.0];
    let volumes = vec![10.0, 20.0, 15.0, 5.0];
    
    let vwap = calculate_vwap(&prices, &volumes).unwrap();
    // (100*10 + 101*20 + 102*15 + 103*5) / (10+20+15+5) = 5050 / 50 = 101.0
    assert!((vwap - 101.0).abs() < 0.001);
  }
  
  #[test]
  fn test_sharpe_ratio() {
    let returns = vec![0.01, 0.02, 0.015, -0.01, 0.03];
    let risk_free = 0.005;
    
    let sharpe = calculate_sharpe_ratio(&returns, risk_free).unwrap();
    assert!(sharpe > 0.0);
  }
}