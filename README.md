# Simulasi SKD by Binpoin

Aplikasi latihan Seleksi Kompetensi Dasar (SKD) CPNS dengan bank soal, materi belajar, latihan topik, dan simulasi CAT.

## Fitur

- Latihan soal TWK, TIU, TKP, dan campuran
- Simulasi ujian bertimer dan hasil per bagian
- Materi belajar terstruktur
- Riwayat latihan tersimpan lokal
- Backend Rust (Axum) dan antarmuka web
- Aplikasi desktop macOS berbasis Tauri dengan jendela WebView
- Landing page unduhan beta berbasis Next.js

## Jalankan aplikasi

Diperlukan Rust stable. Dari root repo:

```bash
cargo run --release
```

Aplikasi akan menampilkan alamat lokal pada terminal. Untuk build paket portabel platform saat ini, lihat `build-release.sh`.

## Landing page

Diperlukan Node.js 20 atau lebih baru. Dari folder `landing/`:

```bash
npm ci
npm run dev
```

Untuk build production:

```bash
npm run build
npm run start
```

## Aplikasi desktop macOS

Build lokal memerlukan Rust stable, Node.js 20+, Xcode Command Line Tools, serta Tauri CLI (diunduh melalui `npx` saat build):

```bash
./build-release.sh aarch64-apple-darwin
```

Hasil DMG ada di `release/`; buka DMG lalu seret aplikasi ke Applications. Jendela aplikasi memakai WebView macOS, sedangkan Axum tetap melayani API dan menyimpan riwayat lokal.

Paket yang dibuat tanpa sertifikat Developer ID dan notarization Apple masih dapat memunculkan peringatan Gatekeeper. Untuk rilis publik tanpa peringatan, konfigurasi Developer ID signing dan notarization beserta kredensial Apple di CI tetap diperlukan.

### Update aplikasi

Versi desktop macOS dan Windows memeriksa update saat dibuka dan menampilkan pilihan **Update sekarang** atau **Nanti**. Unduhan hanya dipasang setelah pengguna memilih Update sekarang, lalu aplikasi dimulai ulang. Update ditandatangani dengan Tauri updater signing key (terpisah dari Apple Developer ID). Fungsi ini berlaku untuk aplikasi desktop Tauri; build lama berbasis arsip bukan target updater. Windows didistribusikan sebagai installer NSIS per-user.

Untuk build rilis, set `TAURI_SIGNING_PRIVATE_KEY` dan `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Private key updater dibuat di `$HOME/.tauri/simulasi-skd-updater.key`; jangan commit atau membagikannya. Tambahkan nilainya sebagai GitHub Actions secrets dengan nama yang sama. Public key verifikasi sudah tertanam di `tauri.conf.json`. Release macOS perlu menyertakan file `.app.tar.gz` beserta `.sig` yang dihasilkan Tauri; feed `latest-darwin-*.json` diunduh dari GitHub Releases. Feed untuk auto-update diambil dari latest non-prerelease.

## Distribusi beta

Paket rilis dibangun terpisah melalui GitHub Actions. File hasil build dan paket distribusi lokal tidak disimpan sebagai source di repo (`release/` diabaikan).

## Lisensi

Kode aplikasi dilisensikan di bawah MIT; lihat [LICENSE](LICENSE).

Bank soal dan materi pada `data/` didistribusikan terpisah di bawah izin yang dimiliki Binpoin. MIT License yang mencakup kode **tidak otomatis melisensikan** konten pihak ketiga. File sumber PDF, dump ekstraksi, dan manifest sumber tidak disertakan di repository publik.

## Catatan

Versi desktop macOS membuka aplikasi dalam jendela WebView native, bukan browser eksternal. Backend tetap merupakan layanan lokal yang hanya bind ke loopback. Riwayat latihan disimpan pada komputer pengguna.
