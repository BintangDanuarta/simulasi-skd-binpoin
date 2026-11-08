import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Ikut Uji Coba Beta — Simulasi SKD by Binpoin',
  description: 'Coba versi beta Simulasi SKD by Binpoin dan bantu kami membuat persiapan CPNS lebih terarah.',
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="id"><body>{children}</body></html>;
}
