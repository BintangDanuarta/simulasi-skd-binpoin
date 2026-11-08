import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Simulasi SKD by Binpoin — Aplikasi Latihan CAT CPNS Mandiri',
  description: 'Persiapan SKD CPNS mandiri: 2.056+ bank soal, 55 modul materi, simulasi CAT ber-timer standar BKN. Cepat, ringan (2 MB), dan 100% offline.',
  icons: {
    icon: '/favicon.ico',
    shortcut: '/favicon.ico',
    apple: '/binpoin-logo.png',
  },
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="id" className="scroll-smooth">
      <head>
        <link rel="icon" href="/favicon.ico" sizes="any" />
        <link rel="icon" type="image/png" href="/binpoin-logo.png" />
        <link rel="apple-touch-icon" href="/binpoin-logo.png" />
      </head>
      <body className="antialiased text-[#181512] bg-[#fbf9f5] selection:bg-[#0154d0]/15 selection:text-[#0154d0]">
        {children}
      </body>
    </html>
  );
}
