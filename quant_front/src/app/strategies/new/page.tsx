import { createTaStrategy } from '@/src/lib/api';

export default function NewStrategyPage() {
  async function create(formData: FormData) {
    'use server';
    const symbol = String(formData.get('symbol') || 'BTCUSDT');
    const kind = String(formData.get('kind') || 'ma_crossover');
    const params: Record<string, unknown> = {};
    if (kind === 'ma_crossover') {
      params.fast_period = Number(formData.get('fast') || 12);
      params.slow_period = Number(formData.get('slow') || 26);
    } else if (kind === 'rsi') {
      params.period = Number(formData.get('period') || 14);
      params.oversold = Number(formData.get('oversold') || 30);
      params.overbought = Number(formData.get('overbought') || 70);
    }
    await createTaStrategy({ symbol, strategy_type: kind, params });
  }

  return (
    <main className="p-6 space-y-6 max-w-xl">
      <h1 className="text-2xl font-semibold">New Strategy</h1>
      <form action={create} className="space-y-4">
        <div>
          <label className="block text-sm mb-1">Symbol</label>
          <input name="symbol" defaultValue="BTCUSDT" className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
        </div>
        <div>
          <label className="block text-sm mb-1">Type</label>
          <select name="kind" className="w-full rounded bg-gray-900 border border-gray-800 p-2">
            <option value="ma_crossover">MA Crossover</option>
            <option value="rsi">RSI</option>
          </select>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-sm mb-1">Fast</label>
            <input name="fast" type="number" defaultValue={12} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Slow</label>
            <input name="slow" type="number" defaultValue={26} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
        </div>
        <div className="grid grid-cols-3 gap-3">
          <div>
            <label className="block text-sm mb-1">RSI Period</label>
            <input name="period" type="number" defaultValue={14} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Oversold</label>
            <input name="oversold" type="number" defaultValue={30} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Overbought</label>
            <input name="overbought" type="number" defaultValue={70} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
        </div>
        <button className="rounded bg-green-600 px-4 py-2">Create</button>
      </form>
    </main>
  );
}
