## xQuant 실행 로드맵 (만들어진 것 / 만들어야 할 것)

### 프로젝트 목표
- Rust 실행 트레이딩 봇 + Python 예측/백테스트 REST API 연동으로, 실거래와 모의거래(목) 모두 “빨리 돌아가는” 안정 경로 확보
- Mock 우선(즉시 실행) + 실연동(WS/바이낸스) 공존 구조 유지

---

## 서브이슈별 정리

### 1) Python 예측 API
- 코드 구현
  - ✅ 만들어진 것: FastAPI 서버(`/`, `/health`, `/market-data`, `/signals`, `/backtest`, `/predict`, `/strategies`, `/indicators`), WebSocket `ws/{symbol}`. 파일: `python_prediction/api/server.py`
  - ✅ 만들어진 것: 수집기 `ccxt.binance`(선물, Testnet 지원) OHLCV/틱커/오더북/펀딩레이트. 파일: `python_prediction/data_collection/binance_client.py`
  - ✅ 만들어진 것: 지표 모듈(`ta` 기반), 전략 4종(추세/역추세/MACD+StochRSI/볼린저), 간단 백테스터. 파일: `python_prediction/indicators/technical_indicators.py`, `python_prediction/strategies/trend_following.py`, `python_prediction/backtest/backtester.py`
  - 필수: 입력 검증/에러 메시지 표준화(서버 레이어), 예외 시 4xx/5xx 구분 강화
  - 있으면 좋은 것: 레이트리밋/토큰 키(헤더) 지원, 응답 스키마 OpenAPI 문서 보강
- 환경/기타
  - ✅ 만들어진 것: 설정 로더(`.env`) 및 기본값(Testnet). 파일: `python_prediction/config/config.py`
  - 필수: 가상환경 및 의존성 설치
    - `python -m venv .venv && source .venv/bin/activate`
    - `pip install -r python_prediction/requirements.txt`
  - 필수(환경): macOS `ta-lib` 설치 필요 시 `brew install ta-lib`
  - 있으면 좋은 것: 실행 스크립트 `scripts/run_py_api.sh`, Makefile 타깃(`api`, `test`, `backtest`)

### 2) Rust 트레이딩 서버(목+실 공존)
- 코드 구현
  - ✅ 만들어진 것: Mock 거래소(주문/체결/잔고/히스토리 시뮬). 파일: `src/exchange/mocks.rs`
  - ✅ 만들어진 것: 시장데이터 스트림/채널, WS Provider 스켈레톤. 파일: `src/market_data/{stream.rs,provider.rs,websocket.rs}`
  - ⚠️ 부분: 라우트 ↔ 핸들러 미정합. 파일: `src/api/{routes.rs,handlers.rs}`
  - ⚠️ 부분: `CombinedStrategy`가 미정의 타입(`OrderType::Algorithm`) 참조, 실행 전략 생성자 시그니처 불일치. 파일: `src/strategies/combined.rs`, `src/models/order.rs`, `src/strategies/{twap.rs,vwap.rs,iceberg.rs}`
  - 필수: 라우트 대응 핸들러 스텁 추가(200/JSON 또는 501)
  - 필수: `CombinedStrategy` 주문 변환 로직을 `with_twap_params`/`with_vwap_params`/`with_iceberg_qty` 사용으로 정정
  - 필수: `main.rs`에서 `dyn Exchange` 임포트, WS 연결 실패 시 비차단 폴백(Mock만 동작) 처리
  - 있으면 좋은 것: `MockMarketDataProvider` 추가(WS 없이도 스트림 퍼블리시)
- 환경/기타
  - 필수: `cargo check`/`cargo run` (서버 3030) 동작 확인
  - 있으면 좋은 것: `RUST_LOG=info` 기본, 실행 스크립트 `scripts/run_rust.sh`

### 3) Rust↔Python 연동(REST)
- 코드 구현
  - ✅ 만들어진 것: REST 클라이언트 및 Bot(예측 기반). 파일: `src/prediction_client.rs`
  - ⛔ 미구현: 설정 주입 경로(`Config`→`PredictionClient`) 및 부팅 시 헬스체크/샘플 호출 로그
  - 필수: `main.rs`에서 `PredictionClient` 생성(설정 주입) → `/signals` 1회 호출/로그
  - 있으면 좋은 것: 타임아웃/리트라이/서킷브레이커, 신뢰도 기반 포지션 사이징 샘플
- 환경/기타
  - 필수: `config.json` 또는 `.env`에 `prediction_api.base_url`(예: `http://127.0.0.1:8000`)
  - 있으면 좋은 것: 보안 토큰 헤더(서버/클라이언트 동시 적용)

### 4) 시장데이터 레이어
- 코드 구현
  - ✅ 만들어진 것: WS Provider 스켈레톤 및 스트림 채널
  - 필수: 구독 선행(심볼 채널 생성), 연결 실패 시 경고만 로그하고 계속 실행
  - 있으면 좋은 것: REST 폴링 백업, 간단 캔들 집계 태스크
- 환경/기타
  - 필수: 공개 WS 엔드포인트 접근 가능 네트워크
  - 있으면 좋은 것: 심볼 목록 설정화

### 5) 주문/전략 API 표면
- 코드 구현
  - ⛔ 미구현: 주문/스플릿/트레일링/시장데이터 핸들러 다수
  - 필수: 라우트에 대응하는 핸들러 스텁 일괄 추가(컴파일 우선)
  - 있으면 좋은 것: `StrategyManager` 활성/토글/상태 조회 라우트 연동 확인
- 환경/기타
  - 있으면 좋은 것: Postman/HTTPie 샘플 컬렉션

### 6) 백테스트
- 코드 구현
  - ✅ 만들어진 것: `engine.rs`/`scenario.rs`/`result.rs`/`performance.rs`
  - ⛔ 미구현/노출만: `backtest/analyzer.rs`, `backtest/optimizer.rs`
  - 필수: 스텁 파일 추가 또는 `mod.rs`에서 노출 제거
  - 있으면 좋은 것: CSV 예제 데이터/규격(`timestamp,open,high,low,close,volume,symbol`)과 리포트 지표 확정
- 환경/기타
  - 있으면 좋은 것: `data/` 샘플 데이터 동봉

### 7) 설정/비밀정보
- 코드 구현
  - ✅ 만들어진 것: Rust `Config`(서버/거래소/로깅)
  - ⛔ 미구현: `prediction_api.base_url` 필드/로드
  - 필수: `Config`에 `prediction_api.base_url` 추가(기본 `http://127.0.0.1:8000`) 및 `config.json` 로드
  - 있으면 좋은 것: `.env` 병합 로더, 민감정보 마스킹 로깅
- 환경/기타
  - 필수: Python `.env` (`BINANCE_TESTNET=true`, 선택: `BINANCE_API_KEY`, `BINANCE_API_SECRET`)
  - 있으면 좋은 것: GitHub Secrets/CI 템플릿

### 8) 테스트/운영
- 코드 구현
  - 필수: 최소 스모크
    - Rust: `cargo check`/`cargo test`
    - Python: `python python_prediction/main.py test`
    - e2e(있으면 좋은 것): 두 서버 기동 후 `/signals` 호출→Rust 로그 확인
- 환경/기타
  - 있으면 좋은 것: GitHub Actions(캐시 포함), pre-commit 훅

---

## 빠른 실행 가이드(로컬)
- Python API
  - `python -m venv .venv && source .venv/bin/activate`
  - `pip install -r python_prediction/requirements.txt`
  - `python python_prediction/main.py server` (기본 127.0.0.1:8000)
- Rust 서버
  - `cargo run` (기본 127.0.0.1:3030)

---

## 변경 이력 요약(최근)
- Rust: `Strategy`에 `is_active`/`set_active` 추가 및 `StrategyManager` 연동, `close_price`→`close` 정리, `TradingError` 변형 추가
- Python: FastAPI/전략/지표/백테스터 기본 골격 완성

