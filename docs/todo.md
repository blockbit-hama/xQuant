## xQuant 작업 계획 (책 목차 매핑 + TODO)

### 목표
- Rust 트레이딩 봇 + Python 예측 API를 연동해 자동매매/백테스트가 가능한 하이브리드 시스템 완성
- 아래 책 목차의 흐름에 맞춰 모듈/기능을 정리하고, 내일부터 실행할 구체적 체크리스트 제공

---

## Day 1 실행 체크리스트 (빌드 그린 최우선)
- [ ] API 라우트-핸들러 정합성 복구 (`src/api/routes.rs` ↔ `src/api/handlers.rs`)
  - [ ] 다음 핸들러 구현 추가 또는 라우트 정리: `create_order`, `get_orders`, `cancel_order`, `create_vwap_order`, `get_vwap_status`, `cancel_vwap_order`, `create_iceberg_order`, `get_iceberg_status`, `cancel_iceberg_order`, `create_trailing_stop`, `get_trailing_stop_status`, `cancel_trailing_stop`, `get_market_data`
- [ ] `CombinedStrategy` 정정 (`src/strategies/combined.rs`)
  - [ ] TWAP/VWAP/Iceberg 생성자 인자 시그니처에 맞게 수정
  - [ ] 존재하지 않는 `OrderType::Algorithm` 제거, 대신 `with_twap_params`/`with_vwap_params`/`with_iceberg_qty` 사용
  - [ ] 존재하지 않는 `order.meta` 접근 제거
- [ ] 백테스트 모듈 정리 (`src/backtest/mod.rs`)
  - [ ] `analyzer.rs`, `optimizer.rs` 파일 추가(스텁) 또는 `mod.rs`에서 노출 제거 중 택1
- [ ] `close_price` → `close` 필드 사용 확인 (봇/핸들러/인디케이터 전역)
- [ ] `TradingError` 변형 사용처 확인 (인디케이터에서 `InsufficientData`, `MissingData`, `CalculationError` 등)
- [ ] 설정에 예측 API URL 추가 및 사용 연결
  - [ ] `Config`에 `prediction_api.base_url` 추가, `PredictionClient` 주입 위치 연결 (`main.rs` 또는 전략/봇 구성부)
- [ ] 빌드/테스트
  - [ ] `cargo check`
  - [ ] `cargo test`

---

## 책 목차 ↔ 현재 구성 매핑 및 TODO

### 01~04장: 개요/선물/기술이론/지표
- **현황**: 지표/시그널/봇/전략 골격 존재
- **코드**: `indicators/*`, `signals/*`, `trading_bots/*`
- **TODO**
  - [ ] 각 지표에 간단 사용 예/공식 문서화 (`docs/project-overview.md` 또는 신규 `docs/indicators.md`)
  - [ ] `signals/position_sizing.rs`, `signals/signal_analyzer.rs` 검토 및 단위테스트 보강

### 05~06장: 바이낸스/화면
- **현황**: Python 쪽 데이터 수집기 존재(`python_prediction/data_collection/binance_client.py` 참고), Rust는 `market_data/*` Mock/FIX/WebSocket 골격
- **TODO**
  - [ ] Python 수집기 키/엔드포인트 설정 `.env`/`config` 정리
  - [ ] Rust 실시간 스트림 Mock로 우선 연동, 실거래소 커넥터는 후순위 이슈로 분리

### 07~11장: 개발환경/판다스/수집/지표/API
- **현황**: FastAPI 서버 가동 코드 있음 (`python_prediction/api/server.py`)
- **TODO**
  - [ ] Python `requirements.txt` 최신화(버전 고정), 서버 실행 스크립트 작성(`make` 또는 `scripts/run_py_api.sh`)
  - [ ] 서버 `/signals`, `/backtest`, `/predict` 스키마를 Rust `prediction_client.rs`와 명확히 동기화 (필드/타입)
  - [ ] CORS/보안(키, 레이트리밋) 기본 설정

### 12장: 전략 및 백테스트
- **현황**: Rust `backtest/engine.rs`, `scenario.rs`, `result.rs`, `performance.rs` 존재
- **TODO**
  - [ ] `backtest/analyzer.rs`, `backtest/optimizer.rs` 스텁/구현
  - [ ] CSV 데이터 규격 정의(`timestamp, open, high, low, close, volume, symbol`)
  - [ ] `CsvDataProvider` 확장: 헤더/구분자 옵션, 멀티심볼 지원
  - [ ] 백테스트 결과 요약/지표 확정(샤프, MDD, Profit Factor 등)

### 13장: 트레이딩 봇 구현
- **현황**: `trading_bots/*`, `strategies/technical.rs`, `core/strategy_manager.rs` 구성
- **TODO**
  - [ ] `StrategyManager` API 노출/제어 라우트 안정화(활성/비활성 토글, 상태 조회)
  - [ ] 예측 기반 봇(`PredictionBasedBot`) 실제 주문 연계 샘플 플로우 문서화
  - [ ] 실패/폴백 전략: 예측 API 장애 시 로컬 TA 전략으로 자동 전환

### 14장: 고도화
- **TODO**
  - [ ] 포트폴리오 최적화(최소분산/최대샤프) 실 구현 (`backtest/optimizer.rs`)
  - [ ] 그리드서치/베이지안 파라미터 튜닝(오프라인)
  - [ ] 전략 조합(예: RSI 신호 + TWAP 실행) 성능 실험 템플릿

---

## Rust 코어 TODO (파일 기준)
- `src/api/routes.rs`
  - [ ] 미구현 핸들러들에 맞춰 라우트 재정의 또는 핸들러 구현 보강
- `src/api/handlers.rs`
  - [ ] 주문/전략/인디케이터 관련 핸들러 완성 및 응답 스키마 정리
- `src/strategies/combined.rs`
  - [ ] 실행 전략 생성자 인자 수정, `OrderType::Algorithm` 제거, 주문 변환 로직 정리
- `src/backtest/*`
  - [ ] `analyzer.rs`, `optimizer.rs` 추가 또는 `mod.rs`에서 제외 결정
  - [ ] `CsvDataProvider` 멀티파일/심볼 지원
- `src/config.rs`
  - [ ] `prediction_api.base_url` 추가, 기본값/환경변수 연동
- `src/prediction_client.rs`
  - [ ] `Config`에서 URL 주입받도록 리팩토링, 타임아웃/리트라이 정책
- 테스트
  - [ ] 전략/봇/인디케이터 단위테스트 보강, 통합테스트 추가(`tests/`)

## Python 예측 시스템 TODO
- API (`python_prediction/api/server.py`)
  - [ ] 모델 출력 스키마를 Rust와 동기화(필드명/타입/시간대)
  - [ ] 예외/에러메시지 표준화
- 데이터/지표/전략
  - [ ] 수집기 안정화(리밸런싱, 재시도, 레이트리밋)
  - [ ] 지표 계산 검증(단위테스트), 전략별 시그널 정의 명확화
- 백테스터
  - [ ] 수수료/슬리피지/체결규칙 옵션화, 리포트 지표 확정
- 운영
  - [ ] `.env`/설정 스키마 정리, 실행 스크립트 추가

## 통합/연동 TODO
- [ ] API 계약(엔드포인트, JSON 스키마) 명세서 `docs/api-contract.md` 작성
- [ ] 신뢰도(confidence) 가중치 기반 주문 크기/위험관리 규칙 합의
- [ ] 폴백/서킷브레이커: 예측 API 오류시 로컬 전략으로 전환

## 보안/운영
- [ ] 시크릿(.env) git-ignore, 키/토큰은 설정/런타임 주입
- [ ] 로깅/모니터링 구조 확정(Rust `env_logger`, Python `loguru`), 요청/응답 샘플 로깅 규칙
- [ ] 설정 검증 및 런타임 self-check(`/health` 고도화)

## 문서/예제
- [ ] README에 빠른 시작(파이썬 API 실행 → 러스트 봇 실행) 시나리오 추가
- [ ] Postman/HTTPie 예제, 간단 백테스트 실행 가이드

---

## 참고: 오늘 반영된 변경(검토/마무리)
- [x] `Strategy`에 `is_active`/`set_active` 추가 → `StrategyManager`와 연동
- [x] `core::strategy_manager` 모듈 공개(`pub mod`)
- [x] `indicators`/`signals`/`trading_bots` 공개
- [x] `backtest/data_provider.rs` 구현(CSV 로더)
- [x] `close_price` → `close`로 전역 수정 (핸들러/봇/유틸)
- [x] `TradingError`에 `InvalidStrategy`, `DuplicateStrategy`, `StrategyNotFound`, `MissingData`, `InsufficientData`, `CalculationError` 추가
- [x] `reqwest`/`chrono(serde)` 설정
- [x] `/health` 라우트 간단 응답으로 임시 교체 (핸들러 구축 전)

검토 포인트: 위 변경으로 빌드 에러가 줄었으나, 남은 라우트/핸들러/CombinedStrategy 수정이 필요합니다. 내일은 Day 1 체크리스트부터 처리해 “빌드 그린”을 먼저 달성하세요.
