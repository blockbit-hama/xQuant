import './globals.css';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'xQuant Dashboard',
  description: 'Trading control panel',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className="min-h-screen antialiased">{children}</body>
    </html>
  );
}
