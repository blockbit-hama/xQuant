"use client";
import { useEffect, useState } from 'react';

async function fetchJSON(url: string) {
  const r = await fetch(url, { cache: 'no-store' });
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}
async function postJSON(url: string, body: any) {
  const r = await fetch(url, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}
async function del(url: string) {
  const r = await fetch(url, { method: 'DELETE' });
  if (!r.ok) throw new Error(await r.text());
  return r.json();
}

export default function OrdersPage() {
  const base = process.env.NEXT_PUBLIC_AXUM_BASE || 'http://localhost:4000';
  const [orders, setOrders] = useState<any[]>([]);
  const [form, setForm] = useState({ symbol: 'BTCUSDT', side: 'buy', type: 'market', qty: 0.001, price: 0 });

  useEffect(() => {
    let alive = true;
    async function load() {
      try { const json = await fetchJSON(`${base}/orders`); if (alive) setOrders(json); } catch {}
    }
    load(); const id = setInterval(load, 2000);
    return () => { alive = false; clearInterval(id); };
  }, [base]);

  useEffect(() => {
    // WebSocket live updates
    let alive = true;
    let ws: WebSocket | null = null;
    try {
      const wsBase = base.replace(/^http/, 'ws');
      ws = new WebSocket(`${wsBase}/ws/orders`);
      ws.onmessage = (evt) => {
        if (!alive) return;
        try {
          const list = JSON.parse(evt.data as string);
          if (Array.isArray(list)) setOrders(list);
        } catch {}
      };
    } catch {}
    return () => { alive = false; if (ws) { try { ws.close(); } catch {} } };
  }, [base]);

  return (
    <main className="p-6 space-y-6 max-w-4xl">
      <h1 className="text-2xl font-semibold">Orders</h1>

      <section className="space-y-3">
        <h2 className="text-lg font-medium">Create Order</h2>
        <div className="grid grid-cols-5 gap-3">
          <input className="rounded bg-gray-900 border border-gray-800 p-2" value={form.symbol} onChange={e=>setForm({...form, symbol:e.target.value})} />
          <select className="rounded bg-gray-900 border border-gray-800 p-2" value={form.side} onChange={e=>setForm({...form, side:e.target.value})}>
            <option value="buy">Buy</option>
            <option value="sell">Sell</option>
          </select>
          <select className="rounded bg-gray-900 border border-gray-800 p-2" value={form.type} onChange={e=>setForm({...form, type:e.target.value})}>
            <option value="market">Market</option>
            <option value="limit">Limit</option>
          </select>
          <input type="number" className="rounded bg-gray-900 border border-gray-800 p-2" value={form.qty} onChange={e=>setForm({...form, qty:Number(e.target.value)})} />
          <input type="number" className="rounded bg-gray-900 border border-gray-800 p-2" value={form.price} onChange={e=>setForm({...form, price:Number(e.target.value)})} />
        </div>
        <button className="rounded bg-green-600 px-4 py-2" onClick={() => postJSON(`${base}/orders`, { symbol: form.symbol, side: form.side, order_type: form.type, quantity: form.qty, price: form.price })}>Submit</button>
      </section>

      <section>
        <h2 className="text-lg font-medium mb-2">Open Orders</h2>
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left border-b border-gray-800">
              <th className="py-2">ID</th>
              <th className="py-2">Symbol</th>
              <th className="py-2">Side</th>
              <th className="py-2">Type</th>
              <th className="py-2">Qty</th>
              <th className="py-2">Price</th>
              <th className="py-2">Actions</th>
            </tr>
          </thead>
          <tbody>
            {orders.map((o:any) => (
              <tr key={o.id?.[0] || o.client_order_id} className="border-b border-gray-800">
                <td className="py-2">{o.id?.[0]}</td>
                <td className="py-2">{o.symbol}</td>
                <td className="py-2">{o.side}</td>
                <td className="py-2">{o.order_type}</td>
                <td className="py-2">{o.quantity}</td>
                <td className="py-2">{o.price}</td>
                <td className="py-2">
                  <button className="rounded bg-red-600 px-3 py-1" onClick={() => del(`${base}/orders/${encodeURIComponent(o.id?.[0])}`)}>Cancel</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </main>
  );
}
