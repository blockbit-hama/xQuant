"use client";
import { useEffect, useState } from 'react';

export default function MarketPage() {
  const [symbol, setSymbol] = useState('BTCUSDT');
  const [data, setData] = useState<any>(null);
  const [series, setSeries] = useState<{ t: number; p: number }[]>([]);
  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';

  useEffect(() => {
    let alive = true;
    async function load() {
      try {
        const res = await fetch(`${base}/market/${encodeURIComponent(symbol)}`, { cache: 'no-store' });
        const json = await res.json();
        if (alive) {
          setData(json);
          const p = Number(json.close || json.price);
          const t = Number(json.timestamp || Date.now());
          if (p) setSeries(s => [...s.slice(-180), { t, p }]);
        }
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
      <div className="h-64 border border-gray-800 rounded p-2 bg-gray-900">
        <Spark data={series} />
      </div>
      <pre className="text-sm whitespace-pre-wrap break-all border border-gray-800 rounded p-3 bg-gray-900">{JSON.stringify(data, null, 2)}</pre>
    </main>
  );
}

function Spark({ data }: { data: { t: number; p: number }[] }) {
  if (data.length < 2) return <div className="text-sm text-gray-400">Waiting data...</div>;
  const width = 800, height = 200, pad = 10;
  const xs = data.map((d, i) => i);
  const ys = data.map(d => d.p);
  const minY = Math.min(...ys), maxY = Math.max(...ys);
  const path = xs.map((x, i) => {
    const px = pad + (x / (xs.length - 1)) * (width - pad * 2);
    const py = pad + (1 - (ys[i] - minY) / Math.max(1e-9, (maxY - minY))) * (height - pad * 2);
    return `${i === 0 ? 'M' : 'L'}${px},${py}`;
  }).join(' ');
  const stroke = ys[ys.length - 1] >= ys[0] ? '#4ade80' : '#f87171';
  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="w-full h-full">
      <path d={path} fill="none" stroke={stroke} strokeWidth={2} />
    </svg>
  );
}
