"use client";
import Link from 'next/link';
import { useEffect, useState } from 'react';

export default function StrategiesLive() {
  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';
  const [items, setItems] = useState<[string, boolean][]>([]);

  useEffect(() => {
    let alive = true;
    let ws: WebSocket | null = null;
    try {
      const wsBase = base.replace(/^http/, 'ws');
      ws = new WebSocket(`${wsBase}/ws/strategies`);
      ws.onmessage = (evt) => {
        if (!alive) return;
        try {
          const list = JSON.parse(evt.data as string);
          if (Array.isArray(list)) setItems(list);
        } catch {}
      };
    } catch {}
    return () => { alive = false; if (ws) { try { ws.close(); } catch {} } };
  }, [base]);

  async function toggle(name: string, active: boolean) {
    await fetch(`${base}/strategies/${encodeURIComponent(name)}/toggle`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ active: !active }) });
  }
  async function remove(name: string) {
    await fetch(`${base}/strategies/${encodeURIComponent(name)}`, { method: 'DELETE' });
  }

  return (
    <div className="space-y-3">
      <h2 className="text-lg font-medium">Live Strategies</h2>
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left border-b border-gray-800">
            <th className="py-2">Name</th>
            <th className="py-2">Active</th>
            <th className="py-2">Actions</th>
          </tr>
        </thead>
        <tbody>
          {items.map(([name, active]) => (
            <tr key={name} className="border-b border-gray-800">
              <td className="py-2"><Link className="underline" href={`/strategies/${encodeURIComponent(name)}`}>{name}</Link></td>
              <td className="py-2">{String(active)}</td>
              <td className="py-2 space-x-2">
                <button className="rounded bg-blue-600 px-3 py-1" onClick={() => toggle(name, active)}>{active ? 'Disable' : 'Enable'}</button>
                <button className="rounded bg-red-600 px-3 py-1" onClick={() => remove(name)}>Delete</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
