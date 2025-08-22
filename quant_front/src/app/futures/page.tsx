"use client";
import { useState } from 'react';

async function postJSON(url: string, body: any) {
  const res = await fetch(url, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export default function FuturesPage() {
  const [symbol, setSymbol] = useState('BTCUSDT');
  const [leverage, setLeverage] = useState(20);
  const [isolated, setIsolated] = useState(false);
  const [hedge, setHedge] = useState(false);

  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';

  return (
    <main className="p-6 space-y-6 max-w-xl">
      <h1 className="text-2xl font-semibold">Futures Settings</h1>

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Position Mode</h2>
        <label className="flex items-center gap-2">
          <input type="checkbox" checked={hedge} onChange={e => setHedge(e.target.checked)} /> Hedge Mode
        </label>
        <button className="rounded bg-blue-600 px-4 py-2" onClick={() => postJSON(`${base}/futures/position_mode`, { hedge })}>Apply</button>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Margin Mode</h2>
        <div className="grid grid-cols-2 gap-3">
          <input className="rounded bg-gray-900 border border-gray-800 p-2" value={symbol} onChange={e=>setSymbol(e.target.value)} />
          <label className="flex items-center gap-2">
            <input type="checkbox" checked={isolated} onChange={e => setIsolated(e.target.checked)} /> Isolated
          </label>
        </div>
        <button className="rounded bg-blue-600 px-4 py-2" onClick={() => postJSON(`${base}/futures/margin_mode`, { symbol, isolated })}>Apply</button>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Leverage</h2>
        <div className="grid grid-cols-2 gap-3">
          <input className="rounded bg-gray-900 border border-gray-800 p-2" value={symbol} onChange={e=>setSymbol(e.target.value)} />
          <input type="number" className="rounded bg-gray-900 border border-gray-800 p-2" value={leverage} onChange={e=>setLeverage(Number(e.target.value))} />
        </div>
        <button className="rounded bg-blue-600 px-4 py-2" onClick={() => postJSON(`${base}/futures/leverage`, { symbol, leverage })}>Apply</button>
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Batch Apply</h2>
        <button className="rounded bg-green-600 px-4 py-2" onClick={() => postJSON(`${base}/futures/settings`, { position_mode: { hedge }, margins: [{ symbol, isolated }], leverages: [{ symbol, leverage }] })}>Apply All</button>
      </section>
    </main>
  );
}
