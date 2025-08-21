## 실행 튜토리얼 (로컬 개발용)

이 문서는 xQuant 모노레포의 각 구성요소를 로컬에서 실행하는 방법과 필수 환경설정을 안내합니다.

### 사전 준비물
- Rust 1.74+ (rustup 권장)
- Node.js 18+ (Next.js 개발용)
- Python 3.10+ (FastAPI 예측 서비스)
- Git

---

## 1) 설정 파일과 환경변수

### Config.json (루트)
- 기본 설정은 `Config.json`에 있으며, 다음 환경변수로 오버라이드 가능합니다.
  - `EXCHANGE_API_KEY` / `EXCHANGE_API_SECRET`: 바이낸스 선물 API 키/시크릿
  - `EXCHANGE_BASE_URL`: 기본값 `https://fapi.binance.com`
  - `USE_MOCK`: `true`(기본) → 모의거래소 사용, `false` → 실거래소 사용
  - `API_TOKEN`: (향후) Bearer 토큰 보호용

### 선물 기본 설정(Config.futures)
- `symbols`: ["BTCUSDT", ...]
- `leverage`: 예) 20
- `isolated`: 예) false
- `hedge`: 예) false

환경변수 예시(macOS zsh/bash):
```bash
export EXCHANGE_API_KEY="..."
export EXCHANGE_API_SECRET="..."
export EXCHANGE_BASE_URL="https://fapi.binance.com"
export USE_MOCK=false
export API_TOKEN="dev-token"
```

---

## 2) Rust 트레이딩 엔진 + HTTP API 실행

### 의존성 설치 및 빌드
```bash
# 포맷/체크/빌드
cargo fmt
cargo check
cargo build
```

### 모의 모드(기본, Axum 단일)
```bash
# USE_MOCK=true이면 모의 거래소로 실행됩니다.
cargo run
```
- HTTP API
  - Axum: `http://127.0.0.1:4000/health`, `/strategies`, `/strategies/ta`

### 실거래 모드(바이낸스 USDT-M Futures)
```bash
export USE_MOCK=false
export EXCHANGE_API_KEY="..."
export EXCHANGE_API_SECRET="..."
export EXCHANGE_BASE_URL="https://fapi.binance.com"

cargo run
```
- 시작 시 서버시간 동기화 → 선물 설정(포지션모드/마진모드/레버리지) 적용 시도
- 주의: 실제 주문이 전송될 수 있으니 소액/테스트 환경에서 먼저 검증하세요

### 백테스트 실행
```bash
cargo run -- backtest       # 기본 VWAP 예제
cargo test backtest_tests   # 테스트 실행
```

---

## 3) Python 예측 서비스(FastAPI)

### 설치 및 실행
```bash
cd python_prediction
python -m venv .venv && source .venv/bin/activate
pip install -r requirements.txt
python -m python_prediction.api.server
```
- 기본 헬스 체크: `http://127.0.0.1:8000/health`
- 시그널/백테스트: `/signals`, `/backtest`

필요시 `.env` 또는 `python_prediction/config/config.py`에서 API 호스트/포트, 바이낸스 테스트넷 여부 등을 조정하세요.

---

## 4) Next.js 프론트엔드(대시보드)

현재 리포에는 최소 스캐폴딩만 예정입니다. 일반적인 실행 절차는 다음과 같습니다.
```bash
cd quant_front
npm install
npm run dev
```
- 브라우저: `http://localhost:3000`
- 프론트가 Axum(`:4000`)과 Python FastAPI(`:8000`)를 호출하도록 구성하세요(CORS 허용됨).

노드 의존성(`node_modules`)은 Git에 포함하지 마세요. `.gitignore`에 이미 추가되어 있습니다.

---

## 5) 인증/보안(개발용 가이드)
- Warp: Bearer 토큰 스캐폴딩이 있으며, 필요한 엔드포인트에 적용 확장 예정
- Axum: `tower-http` CORS로 로컬 개발 허용. 운영에서는 허용 오리진을 제한하세요
- 비밀정보는 반드시 환경변수로 주입하고, VCS에 커밋하지 마세요

---

## 6) 트러블슈팅
- `-1021`/timestamp 오류: 거래소 시간 동기화 실패 → 자동 재시도, 필요 시 네트워크/시스템 시간 확인
- 429 레이트리밋: 자동 백오프. API 호출 빈도를 낮추거나 키/계정 제한 확인
- 빌드 실패: `cargo clean && cargo build` 재시도, Rust toolchain 업데이트
- GitHub push 실패(대용량): `node_modules`는 절대 커밋하지 않기, LFS 고려

---

## 7) 단일 HTTP 프레임워크
현재 프로젝트는 **Axum 단일 프레임워크**로 정리되어 있습니다. 모든 HTTP 호출은 `:4000` 포트를 사용합니다.
