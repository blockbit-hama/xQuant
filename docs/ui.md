## Frontend UI (Next.js) 설계 및 할 일 총정리

이 문서는 `quant_front/` 대시보드의 현재 상태, 목표 기능, 기술 스택/구조, API 연동, 운영 관점과 함께 앞으로 추가해야 할 항목을 정리합니다.

### 1) 개요
- 목적: 트레이딩 엔진(Rust, Axum)과 예측 서비스(Python, FastAPI)를 통합 모니터링/제어하는 대시보드 제공
- 대상 사용자: 트레이더/운영자/개발자
- API 의존성: Axum(`http://127.0.0.1:4000`), FastAPI(`http://127.0.0.1:8000`)

---

### 2) 현재 제공 중인 최소 UI
- 페이지: `src/app/page.tsx`
  - Health 위젯: Axum `/health` 호출 결과 표시
  - Strategies 목록: Axum `/strategies` 호출 결과 표시
- 스타일: Tailwind 초기 설정(`globals.css`, `tailwind.config.js`)

---

### 3) 주요 화면 설계(목표)
- 대시보드(메인)
  - 시스템 상태(엔진/예측 API/시간 동기화/심볼별 최신가)
  - 알림/경보(리스크, 레이트리밋, 시간 드리프트 등)
- 전략 관리
  - 전략 목록/상태/토글/삭제
  - 전략 생성 폼: Technical(MA, RSI, MACD, Multi), VWAP, TWAP, Iceberg, Trailing Stop
  - 전략 상세: 파라미터/최근 주문/성과 요약
- 주문/포지션
  - 오픈 포지션, 오더북(간략), 최근 체결, 주문 내역
  - 주문 생성/취소(테스트/드라이런 보호)
- 선물 설정
  - 레버리지/마진 모드/포지션 모드 조회/변경
  - 배치 적용(여러 심볼)
- 마켓 모니터링
  - 심볼별 가격/변동률/거래량(차트 + 테이블)
  - 실시간 업데이트(WebSocket/폴링)
- 시그널/백테스트(Python 연동)
  - 시그널 생성 요청/결과 시각화(지표 오버레이)
  - 백테스트 실행/결과(수익률/샤프/드로우다운/거래 수 등) + 차트
- 시스템/설정
  - API Base URL/토큰 관리(.env 기반)
  - 테마(다크/라이트), 언어(i18n) 선택

---

### 4) UI 기능 상세 목록
- 공통
  - 글로벌 에러/로딩 핸들링, 빈 상태 처리, 토스트 알림
  - 환경변수 설정 `NEXT_PUBLIC_AXUM_BASE`, `NEXT_PUBLIC_PY_BASE`
  - API 토큰(Bearer) 헤더 주입(옵션)
- 대시보드
  - /health 종합 결과, 최근 로그 요약(향후), 심볼 스냅샷
- 전략 관리
  - 목록: GET `/strategies`
  - 생성: POST `/strategies/ta` (추가: VWAP/TWAP/Iceberg/TrailingStop 전용 폼)
  - 상태/토글/삭제(추가: Axum 라우트 확장 후 연동)
- 선물 설정
  - POST `/futures/settings`(Warp에서 Axum으로 마이그레이션 필요) → Axum 라우트 추가 후 UI 연동
- 마켓 모니터링
  - 가격/체결 데이터 폴링 또는 WebSocket 구독(추가: Axum 또는 전용 WS 게이트웨이 도입)
- 시그널/백테스트
  - FastAPI `/signals`, `/backtest` 호출, 결과 차트 렌더링
- 인증/권한(선택)
  - 로그인 폼(로컬 토큰 보관), 민감 기능 보호

---

### 5) 기술 스택/구조 제안
- Next.js(App Router) + Tailwind
- 데이터 페칭: React Query 또는 SWR 권장(캐시/재시도/에러 관리)
- 차트: TradingView Lightweight Charts 또는 Recharts(echarts도 가능)
- 상태 관리: 최소화(서버 페칭 우선), 폼 상태는 React Hook Form 권장
- 디렉터리 구조(예시)
```
quant_front/
  src/
    app/
      (routes)
      dashboard/
      strategies/
      futures/
      market/
      signals/
      backtest/
      settings/
    components/
      ui/
      charts/
      tables/
      forms/
    lib/
      api.ts (axios/fetch 래퍼)
      config.ts (BASE_URL 등)
      ws.ts (웹소켓 유틸)
```

---

### 6) API 연동 명세(현재/계획)
- Axum(현재)
  - GET `/health`: 시스템 상태 확인
  - GET `/strategies`: 활성 전략 나열
  - POST `/strategies/ta`: TA 전략 생성(MA/RSI 등)
- Axum(추가 필요)
  - 전략 상태/토글/삭제 엔드포인트(기존 Warp의 기능 대체)
  - 선물 설정: `/futures/position_mode`, `/futures/margin_mode`, `/futures/leverage`, `/futures/settings`
  - 시장 스냅샷/티커: `/market/:symbol`
  - WebSocket 게이트웨이: `/ws` (옵션)
- FastAPI
  - POST `/signals`, `/backtest`, GET `/health`

---

### 7) 데이터 갱신 전략
- 폴링: 1~5초 간격(간단), 백오프/가시성 고려
- WebSocket: 실시간 가격/체결/전략 시그널 스트림(엔진 측 WS 필요)
- SSE: 단방향 스트림 대체안

---

### 8) UX 가이드
- 에러: 사용자 친화적 메시지 + 재시도, 디버그용 상세 로그는 콘솔/개발 모드 제한
- 로딩: 스켈레톤/스피너, 영역별 비차단 로딩
- 빈 상태: 가이드 CTA(전략 생성/설정 이동)
- 접근성: 키보드 탐색/aria 라벨, 명도 대비 준수

---

### 9) 보안/환경
- .env(.local)로 베이스 URL/토큰 관리
- 민감 기능(주문/설정)은 토큰 보호(서버에서 검증)
- CORS: 개발은 허용, 운영은 허용 오리진 제한

---

### 10) 배포/운영
- 개발: `npm run dev`
- 빌드: `npm run build && npm start`
- 리버스 프록시(Nginx/Caddy)로 Axum/FASTAPI와 함께 노출(운영)
- 모니터링: 브라우저 콘솔/네트워크, 추후 Sentry 등 도입 고려

---

### 11) 추가되어야 할 것들(총정리)
- Axum 라우트 확장
  - [ ] 전략 토글/삭제/상세 조회
  - [ ] 선물 설정 일괄/단건 엔드포인트 이관(기존 Warp → Axum)
  - [ ] 시장 데이터 조회, (옵션) WebSocket 엔드포인트
- UI 기능
  - [ ] 전략 생성 폼(Technical 외 VWAP/TWAP/Iceberg/TrailingStop)
  - [ ] 전략 상세 카드(최근 주문/성과)
  - [ ] 오더/포지션 테이블 + 취소/정리 액션(드라이런 보호)
  - [ ] 선물 설정 페이지(레버리지/마진/포지션 모드)
  - [ ] 마켓 차트/티커(폴링→WS 전환)
  - [ ] 시그널/백테스트 차트 시각화(FastAPI 연동)
  - [ ] 알림/토스트/경보 배너
  - [ ] 국제화(i18n) 및 테마 전환
- 기술 기반
  - [ ] React Query/SWR 도입과 API 래퍼 구축
  - [ ] Chart 라이브러리 도입과 공통 컴포넌트화
  - [ ] 폼(React Hook Form) 및 유효성 검증
  - [ ] 환경설정 파일(.env) 템플릿 제공
- 운영/품질
  - [ ] E2E/컴포넌트 테스트(Playwright/Vitest)
  - [ ] 빌드 파이프라인/프리뷰 배포(Vercel/Cloudflare Pages 고려)
  - [ ] 린트/포맷팅 워크플로우(eslint/prettier 설정)
  - [ ] 접근성/성능 점검(Lighthouse 기준)

---

### 12) 빠른 시작(요약)
```bash
# Axum 서버
cargo run
# FastAPI (예측)
cd python_prediction && python -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt && python -m python_prediction.api.server
# Next.js
cd quant_front && npm install && npm run dev
```