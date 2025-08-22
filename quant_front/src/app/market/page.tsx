"use client";
import { useEffect, useState } from 'react';

export default function MarketPage() {
  const [symbol, setSymbol] = useState('BTCUSDT');
  const [data, setData] = useState<any>(null);
  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';

  useEffect(() => {
    let alive = true;
    async function load() {
      try {
        const res = await fetch(`${base}/market/${encodeURIComponent(symbol)}`, { cache: 'no-store' });
        const json = await res.json();
        if (alive) setData(json);
      } catch {}
    }
    load();
    const id = setInterval(load, 2000);
    return () => { alive = false; clearInterval(id); };
  }, [symbol, base]);

  return (
    <main className="p-6 space-y-6 max-w-3xl">
      <h1 className="text-2xl font-semibold">Market</h1>
      <div className="flex gap-2 items-center">
        <input className="rounded bg-gray-900 border border-gray-800 p-2" value={symbol} onChange={e=>setSymbol(e.target.value)} />
      </div>
      <pre className="text-sm whitespace-pre-wrap break-all border border-gray-800 rounded p-3 bg-gray-900">{JSON.stringify(data, null, 2)}</pre>
    </main>
  );
}
