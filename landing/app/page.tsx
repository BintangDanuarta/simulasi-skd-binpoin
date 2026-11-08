'use client';

import React, { useState } from 'react';
import Image from 'next/image';
import { cn } from '@/lib/utils';
import {
  WindowsFilled,
  AppleFilled,
  ArrowDownOutlined,
  CheckCircleFilled,
  CompassOutlined,
  SafetyCertificateOutlined,
  ThunderboltOutlined,
  BookOutlined,
} from '@ant-design/icons';

const DOWNLOAD_LINKS = {
  windows: '/downloads/simulasi-skd-binpoin-0.1.0-beta.1-x86_64-pc-windows-gnu.zip',
  macArm: '/downloads/simulasi-skd-binpoin-0.1.0-beta.1-aarch64-apple-darwin.tar.gz',
  macIntel: '/downloads/simulasi-skd-binpoin-0.1.0-beta.1-x86_64-apple-darwin.tar.gz',
  linux: '/downloads/simulasi-skd-binpoin-0.1.0-beta.1-x86_64-unknown-linux-gnu.tar.gz',
};

const SCREENSHOT_TABS = [
  {
    id: 'beranda',
    label: 'Beranda & Roadmap',
    tag: 'Dashboard Belajar',
    title: 'Pantau Progres Belajar Secara Visual',
    desc: 'Roadmap belajar 3 tahap: Pelajari Modul, Kuis Pemahaman, dan Simulasi Penuh. Dilengkapi statistik real-time 2.056 soal per kategori.',
    src: '/screenshots/beranda.png',
  },
  {
    id: 'materi',
    label: '55 Modul Materi',
    tag: 'Kurikulum SKD',
    title: 'Materi Terstruktur Tanpa Terkunci',
    desc: 'Rangkuman lengkap TWK, TIU, dan TKP yang siap dipelajari kapan saja. Setiap modul dilengkapi tip ujian dan latihan mandiri.',
    src: '/screenshots/materi.png',
  },
  {
    id: 'simulasi',
    label: 'Simulasi CAT BKN',
    tag: 'Standar Ujian Resmi',
    title: 'Ujian Ber-timer Persis Sistem CAT Asli',
    desc: 'Format 110 soal dengan timer 100 menit, navigasi nomor cepat, penanda ragu-ragu, dan pembobotan skor resmi BKN.',
    src: '/screenshots/simulasi.png',
  },
  {
    id: 'hasil',
    label: 'Hasil & Pembahasan',
    tag: 'Evaluasi & Solusi',
    title: 'Analisis Passing Grade & Kunci Jawaban',
    desc: 'Langsung ketahui status kelulusan passing grade TWK (65), TIU (80), TKP (166), disertai pembahasan komprehensif setiap nomor.',
    src: '/screenshots/hasil.png',
  },
];

export default function Home() {
  const [activeTab, setActiveTab] = useState(SCREENSHOT_TABS[0].id);
  const currentTab = SCREENSHOT_TABS.find((t) => t.id === activeTab) || SCREENSHOT_TABS[0];

  return (
    <div className="min-h-screen flex flex-col bg-[#fbf9f5] text-[#181512]">
      {/* ================= HEADER ================= */}
      <header className="sticky top-0 z-50 bg-[#fbf9f5] border-b border-[#e6e0d4] shadow-xs">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between">
          <a href="#top" className="flex items-center gap-2.5 group">
            <div className="w-8 h-8 rounded-lg overflow-hidden border border-[#0154d0]/20 bg-white p-0.5 shadow-sm group-hover:border-[#0154d0] transition-colors">
              <Image
                src="/binpoin-logo.png"
                alt="Binpoin Logo"
                width={32}
                height={32}
                className="w-full h-full object-contain"
                unoptimized
              />
            </div>
            <div className="flex flex-col">
              <span className="font-bold text-sm tracking-tight text-[#181512]">
                Simulasi SKD
              </span>
              <span className="text-[10px] text-[#625c52] -mt-0.5 font-medium">
                by binpoin
              </span>
            </div>
            <span className="ml-1.5 px-2 py-0.5 rounded-full text-[10px] font-semibold tracking-wider bg-[#0154d0]/10 text-[#0154d0] border border-[#0154d0]/20">
              BETA 0.1
            </span>
          </a>

          <nav className="hidden md:flex items-center gap-7 text-xs font-semibold text-[#625c52]">
            <a href="#fitur" className="hover:text-[#0154d0] transition-colors min-h-[44px] inline-flex items-center">
              Fitur Utama
            </a>
            <a href="#preview" className="hover:text-[#0154d0] transition-colors min-h-[44px] inline-flex items-center">
              Tampilan Aplikasi
            </a>
            <a href="#unduh" className="hover:text-[#0154d0] transition-colors min-h-[44px] inline-flex items-center">
              Unduh (Win · Mac · Linux)
            </a>
            <a href="#faq" className="hover:text-[#0154d0] transition-colors min-h-[44px] inline-flex items-center">
              FAQ
            </a>
          </nav>

          <div className="flex items-center gap-3">
            <a
              href="#unduh"
              className="inline-flex items-center gap-2 px-3.5 py-2 rounded-lg text-xs font-semibold bg-[#0154d0] text-white hover:bg-[#013b94] transition-all shadow-sm shadow-[#0154d0]/25 active:translate-y-0.5 min-h-[36px]"
            >
              <ArrowDownOutlined className="text-[11px]" />
              <span>Unduh Gratis</span>
            </a>
          </div>
        </div>
      </header>

      {/* ================= HERO SECTION ================= */}
      <section id="top" className="relative pt-12 pb-14 md:pt-16 md:pb-20 hero-pattern overflow-hidden">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 text-center">
          <div className="inline-flex items-center gap-2 px-3.5 py-1 rounded-full bg-[#0154d0]/10 border border-[#0154d0]/25 text-[#0154d0] text-xs font-semibold tracking-wide mb-6">
            <span className="w-2 h-2 rounded-full bg-[#0154d0] animate-pulse" />
            <span>APLIKASI DESKTOP LATIHAN CPNS · 100% OFFLINE</span>
          </div>

          <h1 className="text-3xl sm:text-5xl md:text-6xl font-black tracking-tight text-[#181512] leading-[1.08] mb-5 text-balance">
            Belajar terarah.{' '}
            <span className="text-[#0154d0] inline-block">Hadapi SKD</span> dengan tenang.
          </h1>

          <p className="text-base sm:text-lg text-[#625c52] max-w-2xl mx-auto leading-relaxed mb-8 text-pretty">
            TWK, TIU, dan TKP dalam satu aplikasi desktop yang cepat, ringan (2 MB),
            dan berjalan langsung di komputermu tanpa login atau koneksi internet.
            Tersedia untuk <b>Windows</b>, <b>Mac (Apple Silicon & Intel)</b>, dan <b>Linux</b>.
          </p>

          {/* DUAL CTA: WINDOWS & MAC SIDE-BY-SIDE */}
          <div className="flex flex-col sm:flex-row items-center justify-center gap-3.5 mb-5">
            <a
              href={DOWNLOAD_LINKS.windows}
              download
              className="w-full sm:w-auto inline-flex items-center justify-center gap-2.5 px-6 py-3.5 rounded-xl bg-[#0154d0] text-white text-sm font-bold shadow-md shadow-[#0154d0]/25 hover:bg-[#013b94] transition-all active:translate-y-0.5 min-h-[48px]"
            >
              <WindowsFilled className="text-base" />
              <span>Unduh untuk Windows (.zip · 2.1 MB)</span>
            </a>

            <a
              href={DOWNLOAD_LINKS.macArm}
              download
              className="w-full sm:w-auto inline-flex items-center justify-center gap-2.5 px-6 py-3.5 rounded-xl bg-white border-2 border-[#0154d0] text-[#0154d0] hover:bg-[#0154d0]/5 text-sm font-bold transition-all shadow-sm active:translate-y-0.5 min-h-[48px]"
            >
              <AppleFilled className="text-base text-[#181512]" />
              <span>Unduh untuk Mac (.tar.gz · 1.4 MB)</span>
            </a>
          </div>

          {/* Secondary platform options & preview link */}
          <div className="flex flex-wrap items-center justify-center gap-x-4 gap-y-2 text-xs text-[#625c52] mb-10">
            <span>Pilihan lain:</span>
            <a href={DOWNLOAD_LINKS.macIntel} download className="hover:text-[#0154d0] underline underline-offset-2">
              Mac Intel (.tar.gz)
            </a>
            <span>·</span>
            <a href={DOWNLOAD_LINKS.linux} download className="hover:text-[#0154d0] underline underline-offset-2">
              Linux x86_64 (.tar.gz)
            </a>
            <span>·</span>
            <a href="#preview" className="font-semibold text-[#0154d0] hover:underline underline-offset-2">
              Lihat Tampilan Aplikasi ↓
            </a>
          </div>

          {/* Micro-trust indicators */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 sm:gap-6 pt-6 border-t border-[#e6e0d4]/80 text-left sm:text-center">
            <div className="p-2">
              <div className="font-extrabold text-xl sm:text-2xl text-[#181512]">2.056</div>
              <div className="text-xs text-[#625c52] font-medium">Soal Terverifikasi</div>
            </div>
            <div className="p-2">
              <div className="font-extrabold text-xl sm:text-2xl text-[#181512]">55</div>
              <div className="text-xs text-[#625c52] font-medium">Modul Materi Lengkap</div>
            </div>
            <div className="p-2">
              <div className="font-extrabold text-xl sm:text-2xl text-[#181512]">100 Menit</div>
              <div className="text-xs text-[#625c52] font-medium">Timer Standar BKN</div>
            </div>
            <div className="p-2">
              <div className="font-extrabold text-xl sm:text-2xl text-[#0154d0]">100% Offline</div>
              <div className="text-xs text-[#625c52] font-medium">Privasi & Riwayat Lokal</div>
            </div>
          </div>
        </div>
      </section>

      {/* ================= APP SHOWCASE SECTION (SCREENSHOTS) ================= */}
      <section id="preview" className="py-14 md:py-20 bg-white border-y border-[#e6e0d4]">
        <div className="max-w-6xl mx-auto px-4 sm:px-6">
          <div className="max-w-2xl mx-auto text-center mb-8">
            <span className="text-xs font-bold tracking-widest text-[#0154d0] uppercase">
              TAMPILAN NYATA
            </span>
            <h2 className="text-2xl sm:text-4xl font-extrabold tracking-tight text-[#181512] mt-2 mb-3">
              Antarmuka Bersih. Fokus Pada Latihan.
            </h2>
            <p className="text-sm sm:text-base text-[#625c52]">
              Didesain khusus untuk belajar mandiri tanpa gangguan iklan, pop-up, atau antarmuka yang rumit.
            </p>
          </div>

          {/* Screenshot navigation tabs */}
          <div className="flex items-center justify-start sm:justify-center gap-2 overflow-x-auto pb-3 mb-6 scrollbar-none">
            {SCREENSHOT_TABS.map((tab) => {
              const active = activeTab === tab.id;
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    'relative px-4 py-2.5 rounded-xl text-xs sm:text-sm font-semibold transition-all whitespace-nowrap cursor-pointer min-h-[44px] inline-flex items-center justify-center',
                    active
                      ? 'bg-[#0154d0] text-white shadow-sm shadow-[#0154d0]/25'
                      : 'bg-[#fbf9f5] text-[#625c52] hover:text-[#181512] border border-[#e6e0d4]'
                  )}
                >
                  {tab.label}
                </button>
              );
            })}
          </div>

          {/* Window Mockup Frame */}
          <div className="mockup-window rounded-2xl bg-white border border-[#e6e0d4] overflow-hidden">
            {/* Window title bar */}
            <div className="bg-[#f5f1e8] border-b border-[#e6e0d4] px-4 py-2.5 flex items-center justify-between gap-3">
              <div className="flex items-center gap-2 flex-none">
                <span className="w-3 h-3 rounded-full bg-[#ff5f56] inline-block border border-[#e0443e]" />
                <span className="w-3 h-3 rounded-full bg-[#ffbd2e] inline-block border border-[#dea123]" />
                <span className="w-3 h-3 rounded-full bg-[#27c93f] inline-block border border-[#1aab29]" />
              </div>

              {/* Browser address bar */}
              <div className="bg-white/80 border border-[#e6e0d4] rounded-md px-3 py-1 text-[11px] font-mono text-[#625c52] max-w-sm w-full text-center truncate">
                http://localhost:8787 · {currentTab.label}
              </div>

              <div className="px-2.5 py-0.5 rounded-full bg-[#0154d0]/10 text-[11px] font-bold text-[#0154d0] whitespace-nowrap flex-none">
                {currentTab.tag}
              </div>
            </div>

            {/* Window Content / Screenshot */}
            <div className="relative bg-[#fbf9f5] p-2 sm:p-4 flex flex-col justify-center">
              <div className="rounded-xl overflow-hidden border border-[#e6e0d4] shadow-sm bg-white">
                <Image
                  key={currentTab.id}
                  src={currentTab.src}
                  alt={currentTab.title}
                  width={2560}
                  height={1600}
                  className="w-full h-auto object-cover transition-opacity duration-200"
                  priority
                  unoptimized
                />
              </div>
            </div>

            {/* Contextual caption below screenshot */}
            <div className="bg-[#fbf9f5] border-t border-[#e6e0d4] px-6 py-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
              <div>
                <h3 className="font-bold text-sm text-[#181512]">{currentTab.title}</h3>
                <p className="text-xs text-[#625c52] mt-0.5">{currentTab.desc}</p>
              </div>
              <a
                href="#unduh"
                className="text-xs font-bold text-[#0154d0] hover:text-[#013b94] transition-colors whitespace-nowrap min-h-[32px] inline-flex items-center"
              >
                Coba Sekarang →
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* ================= FEATURES SECTION ================= */}
      <section id="fitur" className="py-14 md:py-20">
        <div className="max-w-6xl mx-auto px-4 sm:px-6">
          <div className="max-w-2xl mx-auto text-center mb-12">
            <span className="text-xs font-bold tracking-widest text-[#0154d0] uppercase">
              FITUR UTAMA
            </span>
            <h2 className="text-2xl sm:text-4xl font-extrabold tracking-tight text-[#181512] mt-2 mb-3">
              Semua Kebutuhan SKD dalam Satu Paket
            </h2>
            <p className="text-sm sm:text-base text-[#625c52]">
              Tidak perlu langganan bulanan mahal atau bergantung pada koneksi internet yang sering lambat.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            {/* Card 1 */}
            <div className="p-6 sm:p-7 rounded-2xl bg-white border border-[#e6e0d4] hover:border-[#0154d0]/40 transition-all shadow-sm">
              <div className="w-10 h-10 rounded-xl bg-[#0154d0]/10 text-[#0154d0] flex items-center justify-center text-lg mb-4">
                <BookOutlined />
              </div>
              <h3 className="text-base sm:text-lg font-bold text-[#181512] mb-2">
                2.056 Bank Soal Terkurasi
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed mb-4">
                Terdiri atas 769 soal TWK, 406 soal TIU, 621 soal TKP, serta 260 soal campuran.
                Setiap butir dilengkapi kunci jawaban resmi dan pembahasan penalaran yang logis.
              </p>
              <div className="flex flex-wrap gap-2 text-[11px] font-semibold text-[#0154d0]">
                <span className="px-2.5 py-1 rounded-md bg-[#0154d0]/8 border border-[#0154d0]/20">
                  769 TWK
                </span>
                <span className="px-2.5 py-1 rounded-md bg-[#0154d0]/8 border border-[#0154d0]/20">
                  406 TIU
                </span>
                <span className="px-2.5 py-1 rounded-md bg-[#0154d0]/8 border border-[#0154d0]/20">
                  621 TKP
                </span>
              </div>
            </div>

            {/* Card 2 */}
            <div className="p-6 sm:p-7 rounded-2xl bg-white border border-[#e6e0d4] hover:border-[#0154d0]/40 transition-all shadow-sm">
              <div className="w-10 h-10 rounded-xl bg-[#0154d0]/10 text-[#0154d0] flex items-center justify-center text-lg mb-4">
                <CompassOutlined />
              </div>
              <h3 className="text-base sm:text-lg font-bold text-[#181512] mb-2">
                Simulasi Standar Resmi BKN
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed mb-4">
                Meniru alur CAT BKN: 110 soal dalam 100 menit. Ambang batas passing grade
                dihitung otomatis (TWK min 65, TIU min 80, TKP min 166) dengan tombol ragu-ragu.
              </p>
              <div className="flex items-center gap-2 text-xs text-[#28744c] font-semibold">
                <CheckCircleFilled />
                <span>Timer akurat + penanda ragu-ragu + rekap nilai otomatis</span>
              </div>
            </div>

            {/* Card 3 */}
            <div className="p-6 sm:p-7 rounded-2xl bg-white border border-[#e6e0d4] hover:border-[#0154d0]/40 transition-all shadow-sm">
              <div className="w-10 h-10 rounded-xl bg-[#0154d0]/10 text-[#0154d0] flex items-center justify-center text-lg mb-4">
                <ThunderboltOutlined />
              </div>
              <h3 className="text-base sm:text-lg font-bold text-[#181512] mb-2">
                55 Modul Materi & Kuis Mandiri
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed mb-4">
                Pelajari pilar negara, integritas, silogisme, hingga tes karakteristik pribadi per topik.
                Langsung uji pemahaman dengan kuis topik sebelum melangkah ke simulasi penuh.
              </p>
              <div className="flex items-center gap-2 text-xs text-[#28744c] font-semibold">
                <CheckCircleFilled />
                <span>Bebas pilih topik tanpa terikat level atau urutan terkunci</span>
              </div>
            </div>

            {/* Card 4 */}
            <div className="p-6 sm:p-7 rounded-2xl bg-white border border-[#e6e0d4] hover:border-[#0154d0]/40 transition-all shadow-sm">
              <div className="w-10 h-10 rounded-xl bg-[#0154d0]/10 text-[#0154d0] flex items-center justify-center text-lg mb-4">
                <SafetyCertificateOutlined />
              </div>
              <h3 className="text-base sm:text-lg font-bold text-[#181512] mb-2">
                100% Offline, Bebas Iklan & Privat
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed mb-4">
                Aplikasi dibangun dengan Rust (Axum) yang sangat ringan dan efisien.
                Seluruh bank soal dan riwayat tersimpan lokal di komputer Anda tanpa tracking.
              </p>
              <div className="flex items-center gap-2 text-xs text-[#28744c] font-semibold">
                <CheckCircleFilled />
                <span>Tanpa akun, tanpa bayar langganan, tanpa koneksi internet</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ================= DOWNLOAD SECTION ================= */}
      <section id="unduh" className="py-14 md:py-20 bg-white border-t border-[#e6e0d4]">
        <div className="max-w-6xl mx-auto px-4 sm:px-6">
          <div className="max-w-2xl mx-auto text-center mb-10">
            <span className="text-xs font-bold tracking-widest text-[#0154d0] uppercase">
              UNDUH PAKET APLIKASI
            </span>
            <h2 className="text-2xl sm:text-4xl font-extrabold tracking-tight text-[#181512] mt-2 mb-3">
              Pilih Paket Sesuai Perangkatmu
            </h2>
            <p className="text-sm sm:text-base text-[#625c52]">
              Unduh paket rilis resmi v0.1.0-beta.1. Langsung ekstrak dan gunakan tanpa proses instalasi rumit.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
            {/* WINDOWS CARD */}
            <div className="p-6 rounded-2xl bg-[#fbf9f5] border-2 border-[#0154d0] relative flex flex-col justify-between shadow-md shadow-[#0154d0]/10">
              <div className="absolute -top-3 left-6 px-2.5 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-wider bg-[#0154d0] text-white">
                Windows
              </div>
              <div>
                <div className="flex items-center justify-between mb-4 mt-2">
                  <div className="flex items-center gap-2.5">
                    <WindowsFilled className="text-2xl text-[#0154d0]" />
                    <span className="font-extrabold text-base text-[#181512]">Windows 10/11</span>
                  </div>
                  <span className="text-xs font-semibold px-2 py-0.5 rounded bg-white border border-[#e6e0d4] text-[#625c52]">
                    ~2.1 MB
                  </span>
                </div>
                <p className="text-xs text-[#625c52] leading-relaxed mb-6">
                  Untuk Windows 64-bit. Berisi binary executable (<code>.exe</code>) berikon Binpoin dan peluncur otomatis <code>Simulasi SKD by Binpoin.bat</code>.
                </p>
              </div>

              <a
                href={DOWNLOAD_LINKS.windows}
                download
                className="w-full inline-flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-[#0154d0] hover:bg-[#013b94] text-white text-xs font-bold transition-all shadow-sm active:translate-y-0.5 min-h-[44px]"
              >
                <ArrowDownOutlined />
                <span>Unduh Windows (.ZIP)</span>
              </a>
            </div>

            {/* MACINTOSH CARD (APPLE SILICON & INTEL) */}
            <div className="p-6 rounded-2xl bg-[#fbf9f5] border-2 border-[#0154d0] relative flex flex-col justify-between shadow-md shadow-[#0154d0]/10">
              <div className="absolute -top-3 left-6 px-2.5 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-wider bg-[#0154d0] text-white">
                macOS
              </div>
              <div>
                <div className="flex items-center justify-between mb-4 mt-2">
                  <div className="flex items-center gap-2.5">
                    <AppleFilled className="text-2xl text-[#0154d0]" />
                    <span className="font-extrabold text-base text-[#181512]">Mac (macOS 11+)</span>
                  </div>
                  <span className="text-xs font-semibold px-2 py-0.5 rounded bg-white border border-[#e6e0d4] text-[#625c52]">
                    ~1.4 MB
                  </span>
                </div>
                <p className="text-xs text-[#625c52] leading-relaxed mb-4">
                  Binary native portabel untuk macOS. Cukup ekstrak dan jalankan binary aplikasi.
                </p>
              </div>

              <div className="flex flex-col gap-2">
                <a
                  href={DOWNLOAD_LINKS.macArm}
                  download
                  className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-[#0154d0] hover:bg-[#013b94] text-white text-xs font-bold transition-all shadow-sm active:translate-y-0.5 min-h-[40px]"
                >
                  <ArrowDownOutlined />
                  <span>Apple Silicon (M1/M2/M3/M4)</span>
                </a>

                <a
                  href={DOWNLOAD_LINKS.macIntel}
                  download
                  className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-white border border-[#e6e0d4] hover:border-[#0154d0] hover:text-[#0154d0] text-[#181512] text-xs font-semibold transition-all active:translate-y-0.5 min-h-[40px]"
                >
                  <ArrowDownOutlined />
                  <span>Mac Intel (.TAR.GZ)</span>
                </a>
              </div>
            </div>

            {/* LINUX CARD */}
            <div className="p-6 rounded-2xl bg-white border border-[#e6e0d4] flex flex-col justify-between hover:border-[#0154d0]/40 transition-all shadow-sm">
              <div>
                <div className="flex items-center justify-between mb-4 mt-2">
                  <div className="flex items-center gap-2.5">
                    <span className="text-xl">🐧</span>
                    <span className="font-extrabold text-base text-[#181512]">Linux x86_64</span>
                  </div>
                  <span className="text-xs font-semibold px-2 py-0.5 rounded bg-[#fbf9f5] border border-[#e6e0d4] text-[#625c52]">
                    ~1.5 MB
                  </span>
                </div>
                <p className="text-xs text-[#625c52] leading-relaxed mb-6">
                  Binary portabel ELF x86_64 untuk Ubuntu, Debian, Fedora, Arch, dan distro Linux modern lainnya.
                </p>
              </div>

              <a
                href={DOWNLOAD_LINKS.linux}
                download
                className="w-full inline-flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-white border border-[#e6e0d4] hover:border-[#0154d0] hover:text-[#0154d0] text-[#181512] text-xs font-bold transition-all active:translate-y-0.5 min-h-[44px]"
              >
                <ArrowDownOutlined />
                <span>Unduh Linux (.TAR.GZ)</span>
              </a>
            </div>
          </div>

          {/* Quick instructions for Windows & Mac */}
          <div className="mt-8 p-5 rounded-xl bg-[#fbf9f5] border border-[#e6e0d4] text-xs text-[#625c52] grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <b className="text-[#181512] block mb-2 font-semibold">
                ⊞ Cara Menjalankan di Windows:
              </b>
              <ol className="list-decimal pl-4 space-y-1">
                <li>Unduh file paket Windows (.ZIP).</li>
                <li>Ekstrak folder hasil unduhan ke komputer Anda.</li>
                <li>Jalankan file <code>Simulasi SKD by Binpoin.bat</code> atau <code>.exe</code>.</li>
                <li>Browser akan otomatis membuka <code>http://localhost:8787</code>.</li>
              </ol>
            </div>

            <div>
              <b className="text-[#181512] block mb-2 font-semibold">
                 Cara Menjalankan di Mac / Linux:
              </b>
              <ol className="list-decimal pl-4 space-y-1">
                <li>Unduh paket (.TAR.GZ) sesuai arsitektur (Apple Silicon / Intel).</li>
                <li>Ekstrak file tar.gz (cukup dobel klik di Finder).</li>
                <li>Buka terminal di folder tersebut lalu jalankan <code>./simulasi-skd-binpoin</code>.</li>
                <li>Buka <code>http://localhost:8787</code> di Safari atau Chrome.</li>
              </ol>
            </div>
          </div>
        </div>
      </section>

      {/* ================= FAQ SECTION ================= */}
      <section id="faq" className="py-14 md:py-18">
        <div className="max-w-3xl mx-auto px-4 sm:px-6">
          <div className="text-center mb-8">
            <h2 className="text-2xl font-extrabold tracking-tight text-[#181512]">
              Pertanyaan yang Sering Diajukan
            </h2>
          </div>

          <div className="space-y-3.5">
            <div className="p-5 rounded-xl bg-white border border-[#e6e0d4]">
              <h3 className="font-bold text-sm text-[#181512] mb-1.5">
                Apakah aplikasi ini benar-benar gratis?
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed">
                Ya, versi beta Simulasi SKD by Binpoin ini sepenuhnya gratis untuk diunduh dan digunakan untuk persiapan belajar mandiri.
              </p>
            </div>

            <div className="p-5 rounded-xl bg-white border border-[#e6e0d4]">
              <h3 className="font-bold text-sm text-[#181512] mb-1.5">
                Mengapa dibuat sebagai aplikasi desktop bukan website online?
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed">
                Supaya Anda bisa belajar kapan saja tanpa terganggu koneksi internet atau lag server saat ribuan orang mengakses bersamaan. Selain itu, riwayat nilai dan jawaban Anda tersimpan aman 100% di komputer lokal Anda.
              </p>
            </div>

            <div className="p-5 rounded-xl bg-white border border-[#e6e0d4]">
              <h3 className="font-bold text-sm text-[#181512] mb-1.5">
                Apakah aplikasi ini menjamin kelulusan ujian CPNS?
              </h3>
              <p className="text-xs sm:text-sm text-[#625c52] leading-relaxed">
                Tidak ada jaminan kelulusan. Aplikasi ini adalah sarana latihan mandiri untuk membiasakan diri dengan format soal, kecepatan manajemen waktu 100 menit, dan standar penilaian resmi BKN.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* ================= FOOTER ================= */}
      <footer className="border-t border-[#e6e0d4] bg-white py-8">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-[#625c52]">
          <div className="flex items-center gap-2">
            <Image
              src="/binpoin-logo.png"
              alt="Binpoin Logo"
              width={20}
              height={20}
              className="w-5 h-5 object-contain"
              unoptimized
            />
            <span className="font-semibold text-[#181512]">Binpoin</span>
            <span>— Belajar yang punya arah.</span>
          </div>

          <div className="flex items-center gap-6">
            <a
              href="https://binpoin.my.id"
              target="_blank"
              rel="noreferrer"
              className="hover:text-[#0154d0] transition-colors min-h-[44px] inline-flex items-center"
            >
              Situs Utama binpoin.my.id ↗
            </a>
            <a href="#unduh" className="hover:text-[#0154d0] transition-colors min-h-[44px] inline-flex items-center">
              Unduh Aplikasi
            </a>
          </div>

          <div>© {new Date().getFullYear()} Binpoin. Hak cipta dilindungi.</div>
        </div>
      </footer>
    </div>
  );
}
