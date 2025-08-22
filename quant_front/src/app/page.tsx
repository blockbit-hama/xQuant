import './globals.css';
import Link from 'next/link';
import StrategiesLive from '@/src/components/StrategiesLive';

export default function Page() {
  return (
    <main className="p-6 space-y-6">
      <h1 className="text-2xl font-semibold">xQuant</h1>
      <nav className="space-x-4">
        <Link className="underline" href="/">Home</Link>
        <Link className="underline" href="/strategies">Strategies</Link>
        <Link className="underline" href="/strategies/new">New Strategy</Link>
      </nav>
      <div className="grid gap-4 md:grid-cols-2">
        <Health />
        <Strategies />
      </div>
      <StrategiesLive />
    </main>
  );
}

function Card({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="rounded-lg border border-gray-800 bg-gray-900 p-4">
      <h2 className="mb-2 text-lg font-medium">{title}</h2>
      {children}
    </div>
  );
}

async function Health() {
  const rust = await fetch('http://localhost:4000/health', { cache: 'no-store' }).then(r => r.json());
  return (
    <Card title="Health">
      <pre className="text-sm">{JSON.stringify(rust, null, 2)}</pre>
    </Card>
  );
}

async function Strategies() {
  const list = await fetch('http://localhost:4000/strategies', { cache: 'no-store' }).then(r => r.json());
  return (
    <Card title="Strategies">
      <pre className="text-sm">{JSON.stringify(list, null, 2)}</pre>
    </Card>
  );
}
