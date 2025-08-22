import { deleteStrategy, listStrategies, toggleStrategy } from '@/src/lib/api';

async function fetchData() {
  const items = await listStrategies();
  return items;
}

export default async function StrategiesPage() {
  const items = await fetchData();
  return (
    <main className="p-6 space-y-6">
      <h1 className="text-2xl font-semibold">Strategies</h1>
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
              <td className="py-2">{name}</td>
              <td className="py-2">{String(active)}</td>
              <td className="py-2 space-x-2">
                <form action={async () => { 'use server'; await toggleStrategy(name, !active) }}>
                  <button className="rounded bg-blue-600 px-3 py-1">{active ? 'Disable' : 'Enable'}</button>
                </form>
                <form action={async () => { 'use server'; await deleteStrategy(name) }}>
                  <button className="rounded bg-red-600 px-3 py-1">Delete</button>
                </form>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </main>
  );
}
