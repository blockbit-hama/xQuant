"use client";
import { useEffect, useState } from 'react';

export default function PositionsPage() {
  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';
  const [rows, setRows] = useState<any[]>([]);

  useEffect(() => {
    let alive = true;
    const load = async () => {
      try { const r = await fetch(`${base}/positions`, { cache: 'no-store' }); const j = await r.json(); if (alive) setRows(j); } catch {}
    };
    load(); const id = setInterval(load, 2000);
    return () => { alive = false; clearInterval(id); };
  }, [base]);

  useEffect(() => {
    // WebSocket live updates
    let alive = true;
    let ws: WebSocket | null = null;
    try {
      const wsBase = base.replace(/^http/, 'ws');
      ws = new WebSocket(`${wsBase}/ws/positions`);
      ws.onmessage = (evt) => {
        if (!alive) return;
        try {
          const list = JSON.parse(evt.data as string);
          if (Array.isArray(list)) setRows(list);
        } catch {}
      };
    } catch {}
    return () => { alive = false; if (ws) { try { ws.close(); } catch {} } };
  }, [base]);

  return (
    <main className="p-6 space-y-6 max-w-3xl">
      <h1 className="text-2xl font-semibold">Positions</h1>
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left border-b border-gray-800">
            <th className="py-2">Symbol</th>
            <th className="py-2">Qty</th>
            <th className="py-2">Entry</th>
            <th className="py-2">Price</th>
            <th className="py-2">UPnL</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((p:any) => (
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
    </main>
  );
}
