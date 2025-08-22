"use client";
import { useEffect, useMemo, useState } from 'react';

function guessSymbolFromName(name: string): string {
  const parts = name.split(/[-_]/);
  if (parts.length >= 2) {
    const candidate = parts[parts.length - 1];
    if (/^[A-Z0-9]+$/.test(candidate)) return candidate;
  }
  return '';
}

export default function StrategyLiveInfo({ name }: { name: string }) {
  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';
  const [symbol, setSymbol] = useState<string>(guessSymbolFromName(name));
  const [orders, setOrders] = useState<any[]>([]);
  const [positions, setPositions] = useState<any[]>([]);

  useEffect(() => {
    let alive = true;
    const wsBase = base.replace(/^http/, 'ws');
    const wsO = new WebSocket(`${wsBase}/ws/orders`);
    const wsP = new WebSocket(`${wsBase}/ws/positions`);
    wsO.onmessage = (evt) => {
      if (!alive) return;
      try { const list = JSON.parse(evt.data as string); if (Array.isArray(list)) setOrders(list); } catch {}
    };
    wsP.onmessage = (evt) => {
      if (!alive) return;
      try { const list = JSON.parse(evt.data as string); if (Array.isArray(list)) setPositions(list); } catch {}
    };
    return () => { alive = false; try { wsO.close(); wsP.close(); } catch {} };
  }, [base]);

  const filteredOrders = useMemo(() => {
    if (!symbol) return orders;
    return orders.filter((o: any) => String(o.symbol || '').toUpperCase() === symbol.toUpperCase());
  }, [orders, symbol]);
  const filteredPositions = useMemo(() => {
    if (!symbol) return positions;
    return positions.filter((p: any) => String(p.symbol || '').toUpperCase() === symbol.toUpperCase());
  }, [positions, symbol]);

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <span className="text-sm text-gray-400">Symbol filter</span>
        <input className="rounded bg-gray-900 border border-gray-800 p-2" value={symbol} onChange={e=>setSymbol(e.target.value)} placeholder="e.g., BTCUSDT" />
      </div>
      <div>
        <h3 className="font-medium mb-2">Open Orders</h3>
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left border-b border-gray-800">
              <th className="py-2">ID</th><th className="py-2">Symbol</th><th className="py-2">Side</th><th className="py-2">Type</th><th className="py-2">Qty</th><th className="py-2">Price</th>
            </tr>
          </thead>
          <tbody>
            {filteredOrders.map((o:any) => (
              <tr key={o.id?.[0] || o.client_order_id} className="border-b border-gray-800">
                <td className="py-2">{o.id?.[0]}</td>
                <td className="py-2">{o.symbol}</td>
                <td className="py-2">{o.side}</td>
                <td className="py-2">{o.order_type}</td>
                <td className="py-2">{o.quantity}</td>
                <td className="py-2">{o.price}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div>
        <h3 className="font-medium mb-2">Positions</h3>
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left border-b border-gray-800">
              <th className="py-2">Symbol</th><th className="py-2">Qty</th><th className="py-2">Entry</th><th className="py-2">Price</th><th className="py-2">UPnL</th>
            </tr>
          </thead>
          <tbody>
            {filteredPositions.map((p:any) => (
              <tr key={p.symbol} className="border-b border-gray-800">
                <td className="py-2">{p.symbol}</td>
                <td className="py-2">{p.quantity}</td>
                <td className="py-2">{p.entry_price}</td>
                <td className="py-2">{p.current_price}</td>
                <td className="py-2">{p.unrealized_pnl}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
