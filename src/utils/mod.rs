//! 시간 관련 유틸리티
//!
//! 시간 변환, 포맷팅, 계산 함수 제공

pub mod logging;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

/// 타임스탬프(밀리초)를 DateTime<Utc>로 변환
pub fn timestamp_to_datetime(timestamp_ms: i64) -> DateTime<Utc> {
  let secs = timestamp_ms / 1000;
  let nsecs = ((timestamp_ms % 1000) * 1_000_000) as u32;
  let naive = NaiveDateTime::from_timestamp_opt(secs, nsecs).unwrap_or_default();
  Utc.from_utc_datetime(&naive)
}

/// DateTime<Utc>를 타임스탬프(밀리초)로 변환
pub fn datetime_to_timestamp(dt: DateTime<Utc>) -> i64 {
  dt.timestamp() * 1000 + dt.timestamp_subsec_millis() as i64
}

/// 현재 시간을 타임스탬프(밀리초)로 반환
pub fn current_timestamp_ms() -> i64 {
  datetime_to_timestamp(Utc::now())
}

/// 타임스탬프(밀리초)를 포맷팅된 문자열로 변환
pub fn format_timestamp(timestamp_ms: i64, format: &str) -> String {
  let dt = timestamp_to_datetime(timestamp_ms);
  dt.format(format).to_string()
}

/// 시간 간격 계산 (초 단위)
pub fn time_diff_seconds(start_ts: i64, end_ts: i64) -> f64 {
  (end_ts - start_ts) as f64 / 1000.0
}

/// 시간 간격에 균등 분할점 계산
pub fn calculate_time_slices(start_ts: i64, end_ts: i64, num_slices: usize) -> Vec<i64> {
  if num_slices == 0 {
    return Vec::new();
  }
  
  let interval = (end_ts - start_ts) as f64 / num_slices as f64;
  let mut result = Vec::with_capacity(num_slices);
  
  for i in 0..num_slices {
    let point = start_ts + (interval * i as f64) as i64;
    result.push(point);
  }
  
  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Duration;
  
  #[test]
  fn test_timestamp_conversion() {
    let now = Utc::now();
    let ts = datetime_to_timestamp(now);
    let dt = timestamp_to_datetime(ts);
    
    // 밀리초 변환으로 인한 약간의 손실 허용 (1초 이내)
    let diff = (now - dt).num_milliseconds().abs();
    assert!(diff < 1000);
  }
  
  #[test]
  fn test_time_slices() {
    let start = 1000;
    let end = 11000;
    let slices = calculate_time_slices(start, end, 5);
    
    assert_eq!(slices.len(), 5);
    assert_eq!(slices[0], 1000);
    assert_eq!(slices[4], 9000);
  }
}