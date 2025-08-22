"use client";
import { useState } from 'react';
import { PY_BASE } from '@/src/lib/config';

export default function BacktestPage() {
  const [symbol, setSymbol] = useState('BTC/USDT');
  const [timeframe, setTimeframe] = useState('1h');
  const [strategy, setStrategy] = useState('trend_following');
  const [days, setDays] = useState(30);
  const [initial, setInitial] = useState(10000);
  const [result, setResult] = useState<any>(null);

  async function submit() {
    const res = await fetch(`${PY_BASE}/backtest`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ symbol, timeframe, strategy, days, initial_capital: initial }) });
    const json = await res.json();
    setResult(json);
  }

  return (
    <main className="p-6 space-y-6 max-w-3xl">
      <h1 className="text-2xl font-semibold">Backtest (Python)</h1>
      <div className="grid grid-cols-2 gap-3">
        <input className="rounded bg-gray-900 border border-gray-800 p-2" value={symbol} onChange={e=>setSymbol(e.target.value)} />
        <input className="rounded bg-gray-900 border border-gray-800 p-2" value={timeframe} onChange={e=>setTimeframe(e.target.value)} />
        <select className="rounded bg-gray-900 border border-gray-800 p-2" value={strategy} onChange={e=>setStrategy(e.target.value)}>
          <option value="trend_following">trend_following</option>
          <option value="mean_reversion">mean_reversion</option>
          <option value="macd_stochrsi">macd_stochrsi</option>
          <option value="bollinger_bands">bollinger_bands</option>
        </select>
        <input type="number" className="rounded bg-gray-900 border border-gray-800 p-2" value={days} onChange={e=>setDays(Number(e.target.value))} />
        <input type="number" className="rounded bg-gray-900 border border-gray-800 p-2" value={initial} onChange={e=>setInitial(Number(e.target.value))} />
      </div>
      <button className="rounded bg-green-600 px-4 py-2" onClick={submit}>Run</button>
      <pre className="text-sm whitespace-pre-wrap break-all border border-gray-800 rounded p-3 bg-gray-900">{result ? JSON.stringify(result, null, 2) : 'No data'}</pre>
    </main>
  );
}
