# 자동 매매 시스템

러스트 기반의 고성능 자동 매매 시스템으로 다양한 주문 실행 전략을 지원합니다.

## 주요 기능

- **VWAP 기반 주문 분할기**: 거래량 가중 평균 가격 전략을 활용한 대량 주문 실행
- **Iceberg 주문 매니저**: 대량 주문을 작은 부분만 노출하여 숨기는 전략
- **Trailing Stop 주문 관리기**: 시장 움직임에 따라 자동 조정되는 동적 손절매 주문
- **TWAP 주문 분할기**: 시간 가중 평균 가격 기반 주문 실행 전략
- **백테스팅 시스템**: 과거 데이터로 전략을 테스트하고 성능 분석
- **리스크 관리**: 위험 한도 및 통제 기능
- **실시간 시장 데이터 스트림**: WebSocket/FIX 기반 시장 데이터 처리
- **모의 거래소**: 실제 거래 없이 전략 테스트 가능

```bash
trading-system/
├── Cargo.toml                  # 프로젝트 및 의존성 정보
├── README.md                   # 프로젝트 설명 및 사용법
├── API.md                      # API 문서
├── src/
│   ├── main.rs                 # 메인 실행 파일
│   ├── config.rs               # 설정 관리
│   ├── error.rs                # 오류 타입 정의
│   │
│   ├── models/                 # 공용 창고(common): 데이터 모델
│   │   ├── mod.rs
│   │   ├── order.rs            # 주문 모델
│   │   ├── trade.rs            # 거래 모델
│   │   ├── market_data.rs      # 시장 데이터 모델
│   │   └── position.rs         # 포지션 모델
│   │
│   ├── exchange/               # 배달부(exchange): 거래소 인터페이스
│   │   ├── mod.rs
│   │   ├── traits.rs           # 거래소 트레이트 정의
│   │   └── mock.rs             # 모의 거래소 구현
│   │
│   ├── market_data/            # 시장 방송국(market_data): 시장 데이터 처리
│   │   ├── mod.rs
│   │   ├── provider.rs         # 데이터 제공자 인터페이스
│   │   ├── stream.rs           # 데이터 스트림 처리
│   │   ├── websocket.rs        # WebSocket 기반 제공자
│   │   └── fix.rs              # FIX 프로토콜 기반 제공자
│   │
│   ├── order_core/             # 주문 교무실(order_core): 주문 관리
│   │   ├── mod.rs
│   │   ├── manager.rs          # 주문 생명주기 관리
│   │   ├── repository.rs       # 주문 저장소
│   │   └── validator.rs        # 주문 유효성 검증
│   │
│   ├── core/                   # 작전 방(strategies): 주문 전략 코어
│   │   ├── mod.rs
│   │   ├── vwap_splitter.rs    # VWAP 주문 분할기
│   │   ├── iceberg  # Iceberg 주문 관리기
│   │   ├── trailing_stop # Trailing Stop 관리기
│   │   ├── twap    # TWAP 주문 분할기
│   │   ├── risk_manager.rs     # 리스크 관리
│   │   └── execution_analyzer.rs # 실행 성능 분석
│   │
│   ├── strategies/             # 전략 인터페이스 및 구현
│   │   ├── mod.rs              # 전략 인터페이스
│   │   ├── vwap.rs             # VWAP 전략
│   │   ├── iceberg.rs          # Iceberg 전략
│   │   ├── trailing_stop.rs    # Trailing Stop 전략
│   │   ├── twap.rs             # TWAP 전략
│   │   └── combined.rs         # 복합 전략
│   │
│   ├── backtest/               # 연습장(backtest): 백테스팅 시스템
│   │   ├── mod.rs
│   │   ├── engine.rs           # 백테스트 엔진
│   │   ├── data_provider.rs    # 백테스트 데이터 제공자
│   │   ├── result.rs           # 백테스트 결과 분석
│   │   └── scenario.rs         # 백테스트 시나리오 관리
│   │
│   ├── api/                    # API 서버
│   │   ├── mod.rs
│   │   ├── routes.rs           # API 라우트 정의
│   │   └── handlers.rs         # API 핸들러 구현
│   │
│   └── utils/                  # 유틸리티 기능
│       ├── mod.rs
│       ├── time.rs             # 시간 관련 유틸리티
│       ├── math.rs             # 수학 연산 유틸리티
│       └── logging.rs          # 로깅 설정
│
└── tests/                      # 테스트 코드
    ├── integration_tests.rs
    ├── vwap_tests.rs
    ├── iceberg_tests.rs
    └── trailing_stop_tests.rs
```

## 시작하기

### 준비 사항

- Rust (1.68 이상)
- Cargo
- 호환 가능한 거래소 API (또는 내장된 모의 거래소 사용)

### 설치

1. 저장소 복제:

```bash
git clone https://github.com/yourusername/trading-system.git
cd trading-system
```

2. 프로젝트 빌드:

```bash
cargo build --release
```

3. `config.json` 파일 생성 (설정 섹션 참조)

### 시스템 실행

```bash
# 라이브 트레이딩 모드
./target/release/trading-system

# 백테스트 모드
./target/release/trading-system backtest
```

## 설정

루트 디렉토리에 다음 구조의 `config.json` 파일을 생성하세요:

```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 3030
  },
  "exchange": {
    "name": "Mock", 
    "api_key": null,
    "api_secret": null,
    "base_url": null,
    "use_mock": true
  },
  "logging": {
    "level": "info",
    "file_path": null
  }
}
```

실제 거래소를 사용하려면 `use_mock`을 `false`로 설정하고 필요한 API 인증 정보를 제공하세요.

## 사용 예제

### VWAP 주문 생성

```rust
use xQuant::core::vwap_splitter::VwapSplitter;
use xQuant::models::order::OrderSide;

// VwapSplitter 초기화
let vwap = VwapSplitter::new(
    exchange,
    "BTCUSDT",
    OrderSide::Buy,
    1.0,           // 총 수량
    3600000,       // 실행 간격 (1시간: 밀리초)
    Some(10.0),    // 목표 비율(%)
);

// 실행 시작
vwap.start().await?;
```

### 백테스트 실행

```rust
use xQuant::backtest::scenario::BacktestScenarioBuilder;
use xQuant::strategies::vwap::VwapStrategy;
use xQuant::models::order::OrderSide;

// 백테스트 시나리오 생성
let mut scenario = BacktestScenarioBuilder::new("VWAP 전략 테스트")
    .description("BTCUSDT에 대한 VWAP 기반 매수 전략 테스트")
    .data_file("./data/BTCUSDT-1m.csv".into())
    .last_days(30)  // 최근 30일
    .initial_balance("USDT", 10000.0)
    .fee_rate(0.001)  // 0.1% 수수료
    .strategy(Box::new(VwapStrategy::new(
        "BTCUSDT",
        OrderSide::Buy,
        1.0,  // 1 BTC 매수
        86400000,  // 24시간(밀리초) 동안 실행
        100,  // 100개 캔들의 VWAP 윈도우
    )))
    .build()?;

// 백테스트 실행
let result = scenario.run().await?;
println!("\n{}", result.summary());
```

## API 문서

시스템은 다양한 트레이딩 전략과 상호작용하기 위한 REST API를 제공합니다. 자세한 내용은 [API.md](API.md)를 참조하세요.

## 아키텍처

시스템은 모듈식 아키텍처로 설계되었습니다:

- **core**: 트레이딩 전략 구현
- **exchange**: 거래소와 상호작용하는 인터페이스
- **models**: 주문, 거래, 시장 데이터 등의 자료 구조
- **order_core**: 주문 생명주기 관리
- **market_data**: 시장 데이터 처리 및 스트리밍
- **backtest**: 전략 백테스팅 및 성능 분석
- **strategies**: 다양한 트레이딩 전략
- **api**: 클라이언트 상호작용을 위한 REST API
- **utils**: 로깅, 수학, 시간 유틸리티

## 테스트

테스트 실행:

```bash
cargo test
```

트레이딩 시뮬레이션 통합 테스트:

```bash
cargo test --features "integration"
```

백테스트 실행:

```bash
cargo run -- backtest
```

## 라이선스

이 프로젝트는 MIT 라이선스로 제공됩니다 - 자세한 내용은 LICENSE 파일을 참조하세요.