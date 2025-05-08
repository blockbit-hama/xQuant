# 트레이딩 시스템 API 문서

이 문서는 자동 매매 시스템의 REST API 엔드포인트에 대한 설명입니다.

## 기본 URL

모든 엔드포인트는 다음 기본 URL을 기준으로 합니다:

```
http://localhost:3030
```

## 상태 확인

### GET /health

API 서버 동작 여부를 확인합니다.

**응답:**

```json
{
  "status": "up",
  "timestamp": "2025-05-08T12:34:56.789Z"
}
```

## 주문 관리

### POST /orders

새 주문을 생성합니다.

**요청 본문:**

```json
{
  "symbol": "BTCUSDT",
  "side": "Buy",
  "order_type": "Market",
  "quantity": 0.1,
  "price": 50000,
  "stop_price": null,
  "time_in_force": "GTC",
  "client_order_id": "client_order_123"
}
```

**응답:**

```json
{
  "order_id": "mock-1234",
  "status": "success"
}
```

### GET /orders

모든 미체결 주문을 조회합니다.

**응답:**

```json
[
  {
    "id": "mock-1234",
    "symbol": "BTCUSDT",
    "side": "Buy",
    "order_type": "Limit",
    "quantity": 0.1,
    "price": 50000,
    "stop_price": null,
    "time_in_force": "GTC",
    "created_at": 1715171436789,
    "client_order_id": "client_order_123"
  }
]
```

### DELETE /orders/{order_id}

주문을 취소합니다.

**매개변수:**

- `order_id`: 취소할 주문의 ID

**응답:**

```json
{
  "order_id": "mock-1234",
  "status": "cancelled"
}
```

## VWAP 주문

### POST /vwap

새 VWAP(Volume-Weighted Average Price) 주문을 생성합니다.

**요청 본문:**

```json
{
  "symbol": "BTCUSDT",
  "side": "Buy",
  "quantity": 1.0,
  "execution_interval": 3600000,
  "target_percentage": 10
}
```

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "started"
}
```

### GET /vwap/{vwap_id}

VWAP 주문 실행 상태를 조회합니다.

**매개변수:**

- `vwap_id`: VWAP 주문의 ID

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "is_active": true,
  "executed_quantity": 0.5,
  "total_quantity": 1.0,
  "progress_percentage": 50.0
}
```

### DELETE /vwap/{vwap_id}

VWAP 주문 실행을 취소합니다.

**매개변수:**

- `vwap_id`: 취소할 VWAP 주문의 ID

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "cancelled"
}
```

## Iceberg 주문

### POST /iceberg

새 Iceberg 주문을 생성합니다.

**요청 본문:**

```json
{
  "symbol": "BTCUSDT",
  "side": "Buy",
  "total_quantity": 10.0,
  "limit_price": 50000,
  "display_quantity": 1.0
}
```

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "started"
}
```

### GET /iceberg/{iceberg_id}

Iceberg 주문 실행 상태를 조회합니다.

**매개변수:**

- `iceberg_id`: Iceberg 주문의 ID

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "is_active": true,
  "executed_quantity": 5.0,
  "total_quantity": 10.0,
  "progress_percentage": 50.0
}
```

### DELETE /iceberg/{iceberg_id}

Iceberg 주문 실행을 취소합니다.

**매개변수:**

- `iceberg_id`: 취소할 Iceberg 주문의 ID

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "cancelled"
}
```

## Trailing Stop 주문

### POST /trailing

새 Trailing Stop 주문을 생성합니다.

**요청 본문:**

```json
{
  "symbol": "BTCUSDT",
  "side": "Sell",
  "quantity": 0.5,
  "trailing_delta": 2.0,
  "activation_price": 53000
}
```

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "started"
}
```

### GET /trailing/{trailing_id}

Trailing Stop 주문 상태를 조회합니다.

**매개변수:**

- `trailing_id`: Trailing Stop 주문의 ID

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "is_active": true,
  "executed": false,
  "trigger_price": 51940,
  "quantity": 0.5
}
```

### DELETE /trailing/{trailing_id}

Trailing Stop 주문 실행을 취소합니다.

**매개변수:**

- `trailing_id`: 취소할 Trailing Stop 주문의 ID

**응답:**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "cancelled"
}
```

## 시장 데이터

### GET /market/{symbol}

특정 심볼의 현재 시장 데이터를 조회합니다.

**매개변수:**

- `symbol`: 거래 심볼 (예: "BTCUSDT")

**응답:**

```json
{
  "symbol": "BTCUSDT",
  "timestamp": 1715171436789,
  "open": 49800.0,
  "high": 50200.0,
  "low": 49700.0,
  "close": 50000.0,
  "volume": 1250.75
}
```

## 백테스트 API

### POST /backtest/run

백테스트를 실행합니다.

**요청 본문:**

```json
{
  "name": "VWAP 전략 테스트",
  "description": "BTCUSDT에 대한 VWAP 기반 매수 전략 테스트",
  "data_files": ["BTCUSDT-1m.csv"],
  "start_time": "2025-04-08T00:00:00Z",
  "end_time": "2025-05-08T00:00:00Z",
  "initial_balance": {
    "USDT": 10000.0
  },
  "fee_rate": 0.001,
  "slippage": 0.0005,
  "strategy": {
    "type": "vwap",
    "symbol": "BTCUSDT",
    "side": "Buy",
    "quantity": 1.0,
    "execution_interval": 86400000,
    "vwap_window": 100
  }
}
```

**응답:**

```json
{
  "id": "backtest-123456",
  "status": "completed",
  "result": {
    "start_time": "2025-04-08T00:00:00Z",
    "end_time": "2025-05-08T00:00:00Z",
    "initial_value": 10000.0,
    "final_value": 11500.0,
    "profit": 1500.0,
    "profit_percentage": 15.0,
    "trade_count": 10,
    "win_rate": 70.0,
    "sharpe_ratio": 1.5
  }
}
```

### GET /backtest/{backtest_id}

백테스트 결과를 조회합니다.

**매개변수:**

- `backtest_id`: 백테스트 ID

**응답:**

```json
{
  "id": "backtest-123456",
  "name": "VWAP 전략 테스트",
  "description": "BTCUSDT에 대한 VWAP 기반 매수 전략 테스트",
  "start_time": "2025-04-08T00:00:00Z",
  "end_time": "2025-05-08T00:00:00Z",
  "initial_balance": {
    "USDT": 10000.0
  },
  "final_balance": {
    "BTC": 0.2,
    "USDT": 5000.0
  },
  "initial_value": 10000.0,
  "final_value": 11500.0,
  "profit": 1500.0,
  "profit_percentage": 15.0,
  "trade_count": 10,
  "win_rate": 70.0,
  "sharpe_ratio": 1.5,
  "max_drawdown": 5.2,
  "trades": [
    {
      "id": "trade-1",
      "symbol": "BTCUSDT",
      "price": 49800.0,
      "quantity": 0.1,
      "timestamp": 1712534400000,
      "side": "Buy"
    }
  ]
}
```

## 오류 응답

모든 엔드포인트는 적절한 HTTP 상태 코드를 반환합니다:

- `200 OK`: 요청 성공
- `201 Created`: 리소스 생성 성공
- `400 Bad Request`: 잘못된 매개변수
- `404 Not Found`: 리소스를 찾을 수 없음
- `500 Internal Server Error`: 서버 오류

오류 응답은 다음 형식을 따릅니다:

```json
{
  "error": "오류 내용 설명"
}
```

## WebSocket API

시스템은 실시간 업데이트를 위한 WebSocket API도 제공합니다. 연결 URL:

```
ws://localhost:3030/ws
```

### 사용 가능한 채널

- `orderbook`: 실시간 호가창 업데이트
- `trades`: 실시간 거래 알림
- `executions`: 실시간 주문 실행 상태 업데이트

### 구독 메시지 형식

```json
{
  "action": "subscribe",
  "channel": "orderbook",
  "symbol": "BTCUSDT"
}
```

### 구독 해제 메시지 형식

```json
{
  "action": "unsubscribe",
  "channel": "orderbook",
  "symbol": "BTCUSDT"
}
```