# xQuant Architecture

알고리즘 트레이딩 시스템의 통합 아키텍처 문서

## 시스템 개요

xQuant는 Rust 기반 트레이딩 봇과 Python 기반 예측 시스템으로 구성된 하이브리드 알고리즘 트레이딩 플랫폼입니다.

```
┌─────────────────┐    REST API    ┌─────────────────┐
│   Rust Bot      │ ◄─────────────► │ Python System   │
│                 │                 │                 │
│ • 주문 실행     │                 │ • 데이터 수집   │
│ • 포지션 관리   │                 │ • 기술적 분석   │
│ • 리스크 관리   │                 │ • 전략 백테스트 │
│ • 실시간 처리   │                 │ • 예측/ML       │
└─────────────────┘                 └─────────────────┘
         │                                   │
         │                                   │
         ▼                                   ▼
┌─────────────────┐                 ┌─────────────────┐
│ Binance API     │                 │ Historical Data │
│ • 실시간 주문   │                 │ • 백테스팅      │
│ • 마켓 데이터   │                 │ • 전략 최적화   │
└─────────────────┘                 └─────────────────┘
```

## Rust 트레이딩 봇 구조

### Core Components

```rust
src/
├── main.rs                     // 메인 실행 파일
├── lib.rs                      // 라이브러리 루트
├── prediction_client.rs        // Python API 클라이언트
│
├── models/                     // 데이터 모델
│   ├── order.rs               // 주문 모델
│   ├── trade.rs               // 거래 모델
│   ├── market_data.rs         // 시장 데이터
│   └── position.rs            // 포지션 모델
│
├── exchange/                   // 거래소 인터페이스
│   ├── traits.rs              // Exchange 트레이트
│   └── mocks.rs               // 모의 거래소
│
├── order_core/                 // 주문 관리
│   ├── manager.rs             // 주문 생명주기
│   ├── repository.rs          // 주문 저장소
│   └── validator.rs           // 주문 검증
│
├── core/                       // 전략 엔진
│   ├── vwap_splitter.rs       // VWAP 분할
│   ├── iceberg_manager.rs     // Iceberg 주문
│   ├── trailing_stop_manager.rs // Trailing Stop
│   ├── twap_splitter.rs       // TWAP 분할
│   └── risk_manager.rs        // 리스크 관리
│
├── strategies/                 // 전략 구현
│   ├── vwap.rs                // VWAP 전략
│   ├── iceberg.rs             // Iceberg 전략
│   ├── trailing_stop.rs       // Trailing Stop 전략
│   └── combined.rs            // 복합 전략
│
├── trading_bots/              // 자동화 봇
│   ├── base_bot.rs            // 봇 인터페이스
│   ├── ma_crossover_bot.rs    // 이평선 교차
│   ├── rsi_bot.rs             // RSI 봇
│   └── multi_indicator_bot.rs // 다지표 봇
│
├── indicators/                 // 기술적 지표
│   ├── moving_averages.rs     // 이동평균
│   ├── oscillators.rs         // 오실레이터
│   ├── trend.rs               // 추세 지표
│   └── volume.rs              // 거래량 지표
│
└── api/                       // REST API 서버
    ├── routes.rs              // API 라우트
    └── handlers.rs            // API 핸들러
```

### 주요 특징

- **비동기 처리**: Tokio 기반 고성능 비동기 처리
- **타입 안전성**: Rust의 강력한 타입 시스템으로 런타임 오류 방지
- **메모리 안전성**: 메모리 누수와 데이터 레이스 방지
- **모듈화**: 재사용 가능한 컴포넌트 설계

## Python 예측 시스템 구조

### Directory Structure

```python
python_prediction/
├── main.py                     // 메인 실행 스크립트
├── requirements.txt            // 의존성 패키지
│
├── config/
│   └── config.py              // 시스템 설정
│
├── data_collection/
│   └── binance_client.py      // 바이낸스 데이터 수집
│
├── indicators/
│   └── technical_indicators.py // 기술적 지표 계산
│
├── strategies/
│   └── trend_following.py     // 트레이딩 전략들
│
├── backtest/
│   └── backtester.py          // 백테스팅 엔진
│
├── api/
│   └── server.py              // FastAPI 서버
│
├── models/                     // 데이터 모델
├── utils/                      // 유틸리티 함수
└── notebooks/                  // Jupyter 노트북
```

### 주요 기능

1. **데이터 수집**
   - Binance 선물 API 연동
   - 실시간/역사적 OHLCV 데이터
   - 호가창, 거래량, 펀딩비율

2. **기술적 분석**
   - 이동평균 (SMA, EMA)
   - MACD, RSI, Stochastic RSI
   - 볼린저 밴드, VWAP
   - 파라볼릭 SAR, 피보나치

3. **트레이딩 전략**
   - 추세 추종 전략
   - 평균 회귀 전략  
   - MACD & StochRSI 전략
   - 볼린저 밴드 전략

4. **백테스팅**
   - 성과 측정 (수익률, 샤프비율, MDD)
   - 거래 통계 (승률, 수익/손실비)
   - 시각화 및 분석

## 통신 인터페이스

### REST API Endpoints

```
GET  /health                    // 시스템 상태 확인
POST /market-data               // 시장 데이터 조회
POST /signals                   // 트레이딩 신호 생성
POST /backtest                  // 백테스트 실행
POST /predict                   // 가격 예측
GET  /strategies                // 전략 목록
GET  /indicators                // 지표 목록
```

### 데이터 플로우

1. **신호 생성 플로우**
   ```
   Rust Bot → Python API → Data Collection → 
   Technical Analysis → Strategy → Signal → Rust Bot
   ```

2. **백테스트 플로우**
   ```
   Request → Historical Data → Strategy Test → 
   Performance Analysis → Results
   ```

## 사용 사례

### 1. 기본 트레이딩 봇 실행

```bash
# Python 예측 시스템 시작
cd python_prediction
python main.py server

# Rust 트레이딩 봇 실행
cargo run
```

### 2. 전략 백테스트

```bash
# Python에서 백테스트 실행
python main.py backtest --symbol BTC/USDT --strategy trend_following --days 30

# 또는 API를 통해
curl -X POST "http://localhost:8000/backtest" \
     -H "Content-Type: application/json" \
     -d '{"symbol":"BTC/USDT","strategy":"trend_following","days":30}'
```

### 3. 실시간 신호 받기

```rust
use xquant::prediction_client::{PredictionClient, SignalRequest};

let client = PredictionClient::new("http://localhost:8000".to_string());
let request = SignalRequest {
    symbol: "BTC/USDT".to_string(),
    strategy: "trend_following".to_string(),
    timeframe: "1h".to_string(),
    lookback: 100,
};

let signal = client.get_signals(request).await?;
println!("Signal: {}, Confidence: {:.2}", signal.signal, signal.confidence);
```

## 확장성 고려사항

### 성능 최적화

- **Rust**: 제로 코스트 추상화, 메모리 효율성
- **Python**: 벡터화 연산 (NumPy, Pandas), 비동기 처리
- **캐싱**: Redis를 통한 지표 및 신호 캐싱
- **데이터베이스**: PostgreSQL/MongoDB for 거래 기록

### 확장 가능성

- **다중 거래소**: CCXT 라이브러리로 다양한 거래소 지원
- **머신러닝**: TensorFlow/PyTorch 모델 통합
- **실시간 스트리밍**: WebSocket 기반 실시간 데이터
- **분산 처리**: 여러 인스턴스 병렬 실행

### 모니터링 및 로깅

- **Rust**: tracing, log crates
- **Python**: loguru, structured logging
- **메트릭**: Prometheus + Grafana
- **알림**: Slack/Discord 봇 통합

## 보안 및 안전성

### API 키 관리
- 환경 변수로 민감 정보 관리
- 테스트넷/메인넷 분리
- 권한 최소화 원칙

### 리스크 관리
- 포지션 크기 제한
- 최대 손실 한도
- 연속 손실 방지 로직
- 긴급 정지 기능

### 테스팅
- 단위 테스트
- 통합 테스트
- 백테스트 검증
- 모의 거래 환경

이 아키텍처는 유연성, 확장성, 안전성을 고려하여 설계되었으며, 실제 트레이딩 환경에서의 요구사항을 충족할 수 있도록 구성되었습니다.