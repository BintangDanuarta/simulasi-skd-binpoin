'use client';

import Image from 'next/image';
import {
  ArrowRightOutlined,
  BookOutlined,
  CheckCircleOutlined,
  ClockCircleOutlined,
  DesktopOutlined,
  HistoryOutlined,
  SafetyCertificateOutlined,
} from '@ant-design/icons';

const APP_URL = 'https://skd.binpoin.my.id/app/';
const REGISTER_URL = 'https://skd.binpoin.my.id/app/register';

const featureItems = [
  {
    icon: <BookOutlined />,
    title: 'Belajar per topik',
    text: 'Buka materi TWK, TIU, dan TKP, lalu lanjutkan dengan latihan sesuai topik.',
  },
  {
    icon: <ClockCircleOutlined />,
    title: 'Latihan dengan timer',
    text: 'Coba simulasi SKD 110 soal dengan batas waktu 100 menit.',
  },
  {
    icon: <HistoryOutlined />,
    title: 'Progres tersimpan di akun',
    text: 'Masuk untuk melihat progres belajar, sesi ujian, dan riwayat latihan.',
  },
];

export default function Home() {
  return (
    <main className="min-h-screen bg-[#FAF6EF] text-[#29241C]">
      <header className="sticky top-0 z-20 border-b border-[#E7DFD1] bg-[#FAF6EF]">
        <div className="mx-auto flex min-h-16 max-w-6xl items-center justify-between gap-2 px-3 sm:gap-4 sm:px-8">
          <a href="#top" className="flex shrink-0 items-center gap-2 sm:gap-3" aria-label="Simulasi SKD by Binpoin, beranda">
            <Image src="/binpoin-logo.png" alt="" width={36} height={36} className="h-8 w-8 object-contain sm:h-9 sm:w-9" unoptimized />
            <span className="leading-tight">
              <strong className="block text-sm">Simulasi SKD</strong>
              <span className="text-xs text-[#6F675A]">by Binpoin</span>
            </span>
          </a>
          <nav className="hidden items-center gap-7 text-sm text-[#6F675A] sm:flex" aria-label="Navigasi utama">
            <a className="hover:text-[#0154d0]" href="#fitur">Fitur</a>
            <a className="hover:text-[#0154d0]" href="#cara-akses">Cara akses</a>
          </nav>
          <div className="flex items-center gap-3">
            <a className="hidden min-h-11 items-center px-2 text-sm font-semibold text-[#0154d0] hover:underline sm:inline-flex" href={APP_URL}>Masuk</a>
            <a className="inline-flex min-h-11 shrink-0 items-center justify-center gap-1.5 rounded-xl bg-[#0154d0] px-3 text-xs font-semibold text-white hover:bg-[#013b94] sm:gap-2 sm:px-4 sm:text-sm" href={REGISTER_URL}>
              <span className="sm:hidden">Daftar</span><span className="hidden sm:inline">Buat akun</span><ArrowRightOutlined aria-hidden="true" />
            </a>
          </div>
        </div>
      </header>

      <section id="top" className="mx-auto grid max-w-6xl gap-10 px-5 pb-16 pt-14 sm:px-8 sm:pb-20 sm:pt-20 md:grid-cols-[1.15fr_.85fr] md:items-center">
        <div>
          <p className="mb-4 text-sm font-semibold uppercase tracking-[.12em] text-[#0154d0]">Ruang latihan SKD CPNS</p>
          <h1 className="max-w-2xl text-4xl font-extrabold leading-[1.08] tracking-tight sm:text-5xl md:text-[56px]">
            Belajar terarah. Hadapi SKD dengan tenang.
          </h1>
          <p className="mt-6 max-w-xl text-base leading-7 text-[#6F675A] sm:text-lg">
            Pelajari materi, kerjakan latihan TWK, TIU, dan TKP, lalu ukur kesiapanmu lewat simulasi CAT. Aplikasi dibuka melalui browser di komputer atau ponsel.
          </p>
          <div className="mt-8 flex flex-col gap-3 sm:flex-row">
            <a className="inline-flex min-h-12 items-center justify-center gap-2 rounded-xl bg-[#0154d0] px-6 text-sm font-bold text-white hover:bg-[#013b94]" href={REGISTER_URL}>
              Daftar dan mulai belajar <ArrowRightOutlined aria-hidden="true" />
            </a>
            <a className="inline-flex min-h-12 items-center justify-center rounded-xl border border-[#D8CEBD] bg-white px-6 text-sm font-semibold text-[#29241C] hover:border-[#0154d0]" href={APP_URL}>
              Saya sudah punya akun
            </a>
          </div>
          <p className="mt-4 text-sm text-[#6F675A]">Perlu koneksi internet dan akun untuk menyimpan progres latihan.</p>
        </div>

        <div className="rounded-2xl border border-[#E7DFD1] bg-white p-6 sm:p-8">
          <div className="flex items-start gap-4">
            <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg bg-[#EAF1FC] text-lg text-[#0154d0]"><DesktopOutlined aria-hidden="true" /></span>
            <div>
              <h2 className="text-lg font-bold">Langsung dari browser</h2>
              <p className="mt-2 text-sm leading-6 text-[#6F675A]">Tidak perlu mengunduh installer Windows atau Mac. Buka situs, masuk atau daftar, lalu pilih materi latihan.</p>
            </div>
          </div>
          <div className="my-6 border-t border-[#E7DFD1]" />
          <ul className="space-y-3 text-sm text-[#514B41]">
            <li className="flex gap-3"><CheckCircleOutlined className="mt-1 text-[#2F7D4F]" />Materi dan latihan per topik</li>
            <li className="flex gap-3"><CheckCircleOutlined className="mt-1 text-[#2F7D4F]" />Simulasi ujian dengan timer</li>
            <li className="flex gap-3"><CheckCircleOutlined className="mt-1 text-[#2F7D4F]" />Riwayat dan progres terkait akunmu</li>
          </ul>
          <p className="mt-6 flex items-start gap-2 border-t border-[#E7DFD1] pt-5 text-xs leading-5 text-[#6F675A]">
            <SafetyCertificateOutlined className="mt-1 text-[#0154d0]" />
            Gunakan halaman masuk resmi di skd.binpoin.my.id/app. Jangan bagikan kata sandi akunmu.
          </p>
        </div>
      </section>

      <section id="fitur" className="border-y border-[#E7DFD1] bg-white py-14 sm:py-16">
        <div className="mx-auto max-w-6xl px-5 sm:px-8">
          <div className="max-w-2xl">
            <h2 className="text-2xl font-bold tracking-tight sm:text-3xl">Satu tempat untuk latihan</h2>
            <p className="mt-3 text-base leading-7 text-[#6F675A]">Pilih bagian yang ingin kamu pelajari. Progres latihan dan riwayat ujian tersedia setelah masuk.</p>
          </div>
          <div className="mt-8 grid gap-4 md:grid-cols-3">
            {featureItems.map((feature) => (
              <article className="rounded-xl border border-[#E7DFD1] p-5" key={feature.title}>
                <span className="flex h-10 w-10 items-center justify-center rounded-lg bg-[#EAF1FC] text-[#0154d0]">{feature.icon}</span>
                <h3 className="mt-4 font-bold">{feature.title}</h3>
                <p className="mt-2 text-sm leading-6 text-[#6F675A]">{feature.text}</p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section id="cara-akses" className="mx-auto max-w-6xl px-5 py-14 sm:px-8 sm:py-16">
        <div className="flex flex-col justify-between gap-6 rounded-2xl border border-[#E7DFD1] bg-white p-6 sm:flex-row sm:items-center sm:p-8">
          <div>
            <h2 className="text-2xl font-bold tracking-tight">Mulai dari browser</h2>
            <p className="mt-2 max-w-xl text-sm leading-6 text-[#6F675A]">Buat akun untuk menyimpan progres. Kalau sudah terdaftar, langsung masuk ke halaman latihan.</p>
          </div>
          <div className="flex shrink-0 flex-col gap-3 sm:flex-row">
            <a className="inline-flex min-h-11 items-center justify-center rounded-lg border border-[#D8CEBD] px-5 text-sm font-semibold hover:border-[#0154d0]" href={APP_URL}>Masuk</a>
            <a className="inline-flex min-h-11 items-center justify-center gap-2 rounded-lg bg-[#0154d0] px-5 text-sm font-semibold text-white hover:bg-[#013b94]" href={REGISTER_URL}>Buat akun <ArrowRightOutlined aria-hidden="true" /></a>
          </div>
        </div>
      </section>

      <footer className="border-t border-[#E7DFD1] bg-white py-6">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-3 px-5 text-xs text-[#6F675A] sm:flex-row sm:px-8">
          <span>© {new Date().getFullYear()} Binpoin</span>
          <span>Simulasi SKD untuk latihan mandiri.</span>
        </div>
      </footer>
    </main>
  );
}