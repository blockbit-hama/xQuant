## xQuant 기능 리스트

- **프로젝트 개요**: Rust 기반 자동 매매/백테스트 엔진. 비동기(Tokio), Warp REST API, 모듈형 아키텍처.
- **주요 모듈**: `strategies`, `core`, `exchange`, `market_data`, `order_core`, `backtest`, `indicators`, `signals`, `trading_bots`, `api`, `utils`.

### 핵심 기능
- **전략 엔진**
  - **공통 인터페이스**: `strategies::Strategy` 트레이트
    - `update(market_data)`, `get_orders()`, `name()`, `description()`, `is_active()`, `set_active(bool)`
  - **실행 최적화 전략**
    - `VwapStrategy`(VWAP), `TwapStrategy`(TWAP), `IcebergStrategy`, `TrailingStopStrategy`
  - **신호/지표 전략**
    - `TechnicalStrategy` (MA Crossover, RSI, MACD, 다중 지표)
  - **복합 전략**
    - `CombinedStrategy` (TA 신호 + 실행전략 조합: RSI+TWAP, MACD+VWAP, MA Cross+Iceberg)
- **전략 매니저**
  - `core::strategy_manager::StrategyManager`
  - 전략 등록/삭제, 활성/비활성 토글, 일괄 업데이트, 주문 수집
- **주문 코어**
  - `order_core::{manager, repository, validator}`
  - In-memory 저장소, 상태 모니터링 스텁, Validator 구조
- **거래소 추상화 / 모의 거래소**
  - `exchange::traits::Exchange` 인터페이스
  - `exchange::mocks::MockExchange` (개발/백테스트용)
- **시장 데이터**
  - WebSocket/FIX 스텁 (`market_data::{websocket, fix}`), 다중 공급자 관리 (`provider`), 스트림 버퍼(`stream`)
- **기술 지표/시그널**
  - `indicators::{moving_averages, oscillators, trend, volume}`
  - `signals::{signal_types, signal_analyzer, position_sizing}`
- **백테스트**
  - `backtest::{engine, scenario, result, performance, data_provider}`
  - CSV 로더, 기간/슬리피지/수수료 설정, 기본 성과지표(샤프, MDD 등)
- **API (Warp)**
  - 주문: `/orders` (POST/GET/DELETE)
  - 실행전략 스텁: `/vwap`, `/iceberg`, `/trailing` (현재 Not Implemented 응답)
  - 시장데이터: `/market/{symbol}` (GET)
  - 전략 관리: `/strategies` (GET 목록, GET 상태, DELETE 삭제, POST `/strategies/{name}/toggle` 토글)
  - TA 전략 생성: `/strategies/ta` (POST) — `TechnicalStrategy`, `CombinedStrategy` 인스턴스 생성/등록
  - 인디케이터 계산: `/indicators/{symbol}` (GET, 쿼리 파라미터)
- **예측 API 연동**
  - `prediction_client.rs` — 헬스체크, 샘플 시그널 요청
- **설정/로깅**
  - `Config.json` 로드, `env_logger` 초기화, `utils::logging` 헬퍼

### 현재 전략 모듈 사용 현황 (src/strategies)
- **`vwap.rs` / VwapStrategy**
  - 사용처: `main.rs` 백테스트 기본 시나리오, `CombinedStrategy::macd_vwap` 실행전략, 테스트(`tests/vwap_tests.rs`)
  - 상태: 사용 중 (삭제 불가)
- **`twap.rs` / TwapStrategy**
  - 사용처: `CombinedStrategy::rsi_twap` 실행전략
  - 상태: 간접 사용 중 (삭제 불가)
- **`trailing_stop.rs` / TrailingStopStrategy**
  - 사용처: 테스트(`tests/trailing_stop_tests.rs`), API 라우트 스텁 존재
  - 상태: 테스트에서 사용 (삭제 불가)
- **`iceberg.rs` / IcebergStrategy**
  - 사용처: 테스트(`tests/iceberg_tests.rs`), `CombinedStrategy::ma_crossover_iceberg`
  - 상태: 테스트/간접 사용 (삭제 불가)
- **`technical.rs` / TechnicalStrategy**
  - 사용처: `main.rs` 라이브 트레이딩 초기화, `/strategies/ta` 생성, 백테스트 시나리오
  - 상태: 핵심 사용 (삭제 불가)
- **`combined.rs` / CombinedStrategy**
  - 사용처: `main.rs` 라이브 트레이딩 초기화, `/strategies/ta` 생성 분기
  - 상태: 핵심 사용 (삭제 불가)

> 결론: `src/strategies` 내 모든 모듈이 직접 또는 간접(복합전략/테스트)으로 사용 중입니다. 삭제 대상 없음.

### 실행/백테스트
- **빌드**: `cargo build --release`
- **체크/테스트**: `cargo check`, `cargo test`
- **실행**: `cargo run`
- **백테스트 실행**: `cargo run -- backtest [ma|rsi]`

### REST API 개요
- 건강 체크: `GET /health`
- 주문: `POST /orders`, `GET /orders`, `DELETE /orders/{id}`
- 실행전략 스텁: `POST/GET/DELETE /vwap|iceberg|trailing` (Not Implemented)
- 시장데이터: `GET /market/{symbol}`
- 전략
  - `GET /strategies` (목록), `GET /strategies/{name}` (상태), `DELETE /strategies/{name}` (삭제)
  - `POST /strategies/{name}/toggle` with `{ "active": bool }`
  - `POST /strategies/ta` — body 예시
    - `{ "symbol":"BTCUSDT", "strategy_type":"rsi", "params": {"period":14,"oversold":30,"overbought":70} }`
- 인디케이터: `GET /indicators/{symbol}` — 쿼리: `indicator_type`, `period`, `fast_period`, `slow_period`, `signal_period`, `overbought`, `oversold`, `limit`

### 지표/시그널 지원 목록
- **MA**: SMA, EMA, MA Crossover(EMA 기반)
- **오실레이터**: RSI(과매수/과매도 신호)
- **트렌드**: MACD(라인/시그널 교차 신호)
- **체결량/가격**: VWAP

### 알려진 이슈/해야 할 일
- `docs/todo.md`, `CLAUDE.md`의 미구현/중복 모듈 메모 확인 필요
  - `backtest/analyzer.rs`, `backtest/optimizer.rs` 없음(참조 제거 또는 구현 필요)
  - 백테스트 `performance` 중복 명명 이슈 점검
- API 중 `/vwap`, `/iceberg`, `/trailing`는 스텁 — 구현 필요
- 경고 다수(`cargo check/test`) — 사용하지 않는 import/변수 정리 가능
