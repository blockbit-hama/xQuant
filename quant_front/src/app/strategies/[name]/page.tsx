import { AXUM_BASE } from '@/src/lib/config';
import StrategyLiveInfo from '@/src/components/StrategyLiveInfo';

async function getInfo(name: string) {
  const res = await fetch(`${AXUM_BASE}/strategies/${encodeURIComponent(name)}`, { cache: 'no-store' });
  if (!res.ok) throw new Error('not found');
  return res.json() as Promise<{ name: string; description: string; active: boolean }>;
}

export default async function StrategyDetail({ params }: { params: { name: string } }) {
  const info = await getInfo(params.name);

  async function toggle() {
    'use server';
    await fetch(`${AXUM_BASE}/strategies/${encodeURIComponent(params.name)}/toggle`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ active: !info.active })
    });
  }
  async function remove() {
    'use server';
    await fetch(`${AXUM_BASE}/strategies/${encodeURIComponent(params.name)}`, { method: 'DELETE' });
  }

  return (
    <main className="p-6 space-y-6 max-w-3xl">
      <h1 className="text-2xl font-semibold">Strategy: {info.name}</h1>
      <div className="rounded border border-gray-800 bg-gray-900 p-4 space-y-2">
        <div><span className="text-gray-400">Description:</span> {info.description}</div>
        <div><span className="text-gray-400">Active:</span> {String(info.active)}</div>
      </div>
      <div className="flex gap-2">
        <form action={toggle}><button className="rounded bg-blue-600 px-4 py-2">{info.active ? 'Disable' : 'Enable'}</button></form>
        <form action={remove}><button className="rounded bg-red-600 px-4 py-2">Delete</button></form>
      </div>
      <StrategyLiveInfo name={info.name} />
    </main>
  );
}
