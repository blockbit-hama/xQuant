import { AXUM_BASE, PY_BASE } from './config';

export async function getHealth() {
  const res = await fetch(`${AXUM_BASE}/health`, { cache: 'no-store' });
  if (!res.ok) throw new Error('health failed');
  return res.json();
}

export async function listStrategies() {
  const res = await fetch(`${AXUM_BASE}/strategies`, { cache: 'no-store' });
  if (!res.ok) throw new Error('list strategies failed');
  return res.json() as Promise<[string, boolean][]>;
}

export async function toggleStrategy(name: string, active: boolean) {
  const res = await fetch(`${AXUM_BASE}/strategies/${encodeURIComponent(name)}/toggle`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ active })
  });
  if (!res.ok) throw new Error('toggle failed');
  return res.json();
}

export async function deleteStrategy(name: string) {
  const res = await fetch(`${AXUM_BASE}/strategies/${encodeURIComponent(name)}`, { method: 'DELETE' });
  if (!res.ok) throw new Error('delete failed');
  return res.json();
}

export async function createTaStrategy(payload: { symbol: string; strategy_type: string; params: Record<string, unknown> }) {
  const res = await fetch(`${AXUM_BASE}/strategies/ta`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });
  if (!res.ok) throw new Error('create ta failed');
  return res.json();
}

export async function createVwapStrategy(payload: { symbol: string; side: string; quantity: number; window: number; participation?: number }) {
  const res = await fetch(`${AXUM_BASE}/strategies/vwap`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
  if (!res.ok) throw new Error('create vwap failed');
  return res.json();
}

export async function createTwapStrategy(payload: { symbol: string; side: string; quantity: number; window: number }) {
  const res = await fetch(`${AXUM_BASE}/strategies/twap`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
  if (!res.ok) throw new Error('create twap failed');
  return res.json();
}

export async function createIcebergStrategy(payload: { symbol: string; side: string; total_qty: number; visible_qty: number; price: number }) {
  const res = await fetch(`${AXUM_BASE}/strategies/iceberg`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
  if (!res.ok) throw new Error('create iceberg failed');
  return res.json();
}

export async function createTrailingStrategy(payload: { symbol: string; side: string; qty: number; callback: number; activation?: number }) {
  const res = await fetch(`${AXUM_BASE}/strategies/trailing`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
  if (!res.ok) throw new Error('create trailing failed');
  return res.json();
}
