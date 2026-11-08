'use client';

import { AppleFilled, ArrowDownOutlined, WindowsFilled } from '@ant-design/icons';

const downloads = {
  windows: 'https://github.com/BintangDanuarta/simulasi-skd-binpoin/releases/download/v0.1.0-beta.1/simulasi-skd-binpoin-0.1.0-beta.1-x86_64-pc-windows-msvc.zip',
  mac: 'https://github.com/BintangDanuarta/simulasi-skd-binpoin/releases/download/v0.1.0-beta.1/simulasi-skd-binpoin-0.1.0-beta.1-aarch64-apple-darwin.tar.gz',
};

// Beta Mac build is compiled natively by GitHub Actions on Apple Silicon runners.

export default function Home() {
  return (
    <main className="page-shell">
      <header className="topbar">
        <a className="brand" href="#top" aria-label="Binpoin, ke beranda">
          <span className="brand-mark">b.</span>
          <span className="brand-name">binpoin</span>
        </a>
        <span className="top-note">SIMULASI SKD · BETA 01</span>
      </header>

      <section className="hero" id="top">
        <div className="hero-copy">
          <p className="eyebrow"><span className="live-dot" /> VERSI BETA PERTAMA</p>
          <h1>Persiapan SKD,<br /><em>mulai dari sini.</em></h1>
          <p className="intro">Pelajari materi, latihan soal, lalu ukur kesiapanmu lewat simulasi SKD. Unduh beta Simulasi SKD by Binpoin dan coba langsung di komputermu.</p>
          <div className="product-line">
            <span className="product-rule" />
            <span>Dibuat untuk belajar mandiri</span>
          </div>
        </div>

        <aside className="download-panel" aria-labelledby="download-title">
          <p className="eyebrow">PILIH PERANGKAT</p>
          <h2 id="download-title">Unduh aplikasinya</h2>
          <p className="panel-copy">Paket beta gratis. Aplikasi berjalan di browser bawaan dan menyimpan riwayat latihan secara lokal.</p>

          <a className="download-option download-primary" href={downloads.windows} download>
            <span className="platform-icon"><WindowsFilled /></span>
            <span className="platform-copy"><b>Windows x64</b><small>ZIP · sekitar 2 MB</small></span>
            <ArrowDownOutlined className="download-arrow" />
          </a>

          <a className="download-option download-secondary" href={downloads.mac} download>
            <span className="platform-icon"><AppleFilled /></span>
            <span className="platform-copy"><b>Mac · Apple Silicon</b><small>TAR.GZ · build native macOS</small></span>
            <ArrowDownOutlined className="download-arrow" />
          </a>
          <p className="mac-note">Paket ini untuk Mac dengan Apple Silicon (seri M). Mac Intel belum tersedia pada beta ini.</p>
          <div className="panel-foot">Beta 01 <span>·</span> Gratis untuk dicoba</div>
        </aside>
      </section>

      <section className="details" aria-label="Informasi aplikasi">
        <div className="detail-main">
          <p className="eyebrow">APA YANG ADA DI DALAM</p>
          <p className="detail-copy">Bank soal, materi belajar, dan simulasi CAT dalam satu aplikasi ringan. Dirancang untuk membantu latihan yang lebih terarah—bukan menjanjikan kelulusan.</p>
        </div>
        <div className="detail-note">
          <span className="check-mark">01</span>
          <p><b>Masih beta.</b> Beberapa hal mungkin belum sempurna. Masukanmu membantu kami menentukan apa yang perlu diperbaiki.</p>
        </div>
      </section>

      <footer className="footer">
        <span>© {new Date().getFullYear()} Binpoin</span>
        <span>Belajar terarah. Hadapi SKD dengan tenang.</span>
      </footer>
    </main>
  );
}
