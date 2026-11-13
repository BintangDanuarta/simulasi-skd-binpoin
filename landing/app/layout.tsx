import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Simulasi SKD by Binpoin | Latihan SKD di Browser',
  description: 'Belajar materi, latihan TWK, TIU, dan TKP, lalu ikuti simulasi SKD melalui browser. Masuk atau buat akun untuk menyimpan progres dan riwayat latihan.',
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
