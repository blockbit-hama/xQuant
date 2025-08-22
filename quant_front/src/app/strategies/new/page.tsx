import { createTaStrategy, createVwapStrategy, createTwapStrategy, createIcebergStrategy, createTrailingStrategy } from '@/src/lib/api';

export default function NewStrategyPage() {
  async function create(formData: FormData) {
    'use server';
    const symbol = String(formData.get('symbol') || 'BTCUSDT');
    const kind = String(formData.get('kind') || 'ma_crossover');
    if (kind === 'ma_crossover' || kind === 'rsi') {
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
      return;
    }
    if (kind === 'vwap') {
      await createVwapStrategy({ symbol, side: String(formData.get('side')||'buy'), quantity: Number(formData.get('qty')||0.1), window: Number(formData.get('window')||60000), participation: Number(formData.get('participation')||0.1) });
      return;
    }
    if (kind === 'twap') {
      await createTwapStrategy({ symbol, side: String(formData.get('side')||'buy'), quantity: Number(formData.get('qty')||0.1), window: Number(formData.get('window')||60000) });
      return;
    }
    if (kind === 'iceberg') {
      await createIcebergStrategy({ symbol, side: String(formData.get('side')||'buy'), total_qty: Number(formData.get('qty')||1), visible_qty: Number(formData.get('visible')||0.1), price: Number(formData.get('price')||0) });
      return;
    }
    if (kind === 'trailing') {
      await createTrailingStrategy({ symbol, side: String(formData.get('side')||'sell'), qty: Number(formData.get('qty')||1), callback: Number(formData.get('callback')||1), activation: Number(formData.get('activation')||0) });
      return;
    }
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
            <option value="vwap">VWAP</option>
            <option value="twap">TWAP</option>
            <option value="iceberg">Iceberg</option>
            <option value="trailing">Trailing Stop</option>
          </select>
        </div>
        <div className="grid grid-cols-3 gap-3">
          <div>
            <label className="block text-sm mb-1">Side</label>
            <select name="side" className="w-full rounded bg-gray-900 border border-gray-800 p-2">
              <option value="buy">Buy</option>
              <option value="sell">Sell</option>
            </select>
          </div>
          <div>
            <label className="block text-sm mb-1">Qty</label>
            <input name="qty" type="number" defaultValue={0.1} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Window(ms)</label>
            <input name="window" type="number" defaultValue={60000} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Participation</label>
            <input name="participation" type="number" step="0.01" defaultValue={0.1} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Visible</label>
            <input name="visible" type="number" step="0.01" defaultValue={0.1} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Price</label>
            <input name="price" type="number" step="0.01" defaultValue={0} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Callback(%)</label>
            <input name="callback" type="number" step="0.1" defaultValue={1} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
          <div>
            <label className="block text-sm mb-1">Activation</label>
            <input name="activation" type="number" step="0.01" defaultValue={0} className="w-full rounded bg-gray-900 border border-gray-800 p-2" />
          </div>
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
