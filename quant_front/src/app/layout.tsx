import './globals.css';
import type { Metadata } from 'next';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'xQuant Dashboard',
  description: 'Trading control panel',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className="min-h-screen antialiased">
        <header className="p-4 border-b border-gray-800 flex gap-4">
          <Link className="underline" href="/">Home</Link>
          <Link className="underline" href="/strategies">Strategies</Link>
          <Link className="underline" href="/strategies/new">New Strategy</Link>
          <Link className="underline" href="/futures">Futures</Link>
        </header>
        {children}
      </body>
    </html>
  );
}
