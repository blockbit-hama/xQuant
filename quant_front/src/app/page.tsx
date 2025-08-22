export default function Page() {
  return (
    <main className="p-6 space-y-6">
      <h1 className="text-2xl font-semibold">xQuant</h1>
      <div className="grid gap-4 md:grid-cols-2">
        <Health />
        <Strategies />
      </div>
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
