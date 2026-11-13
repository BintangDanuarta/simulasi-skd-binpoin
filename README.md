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

## Deploy web ke `skd.binpoin.my.id/app`

Deployment produksi menggunakan Docker Compose dan PostgreSQL di server Linux, di belakang Nginx/HTTPS yang sudah melayani domain `skd.binpoin.my.id`.

1. Salin `.env.web.example` menjadi `.env` di server. Isi password PostgreSQL panjang dengan karakter alfanumerik dan gunakan nilai yang sama untuk `POSTGRES_PASSWORD` serta `POSTGRES_PASSWORD_URLENCODED`. Jangan commit `.env`.
2. Tambahkan konfigurasi dari [`deploy/nginx-app.conf`](deploy/nginx-app.conf) ke server block HTTPS Nginx untuk domain tersebut. Uji dengan `nginx -t` lalu reload Nginx.
3. Jalankan `./scripts/web-up.sh` dari root repository di server.

Aplikasi akan tersedia di `https://skd.binpoin.my.id/app/` dan pendaftaran akun di `/app/register`. Backend hanya diekspos ke loopback `127.0.0.1:8787`; database tetap privat. Backup Docker volume `postgres_data` secara berkala.

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

## Distribusi beta

Paket rilis dibangun terpisah melalui GitHub Actions. File hasil build dan paket distribusi lokal tidak disimpan sebagai source di repo (`release/` diabaikan).

## Lisensi

Kode aplikasi dilisensikan di bawah MIT; lihat [LICENSE](LICENSE).

Bank soal dan materi pada `data/` didistribusikan terpisah di bawah izin yang dimiliki Binpoin. MIT License yang mencakup kode **tidak otomatis melisensikan** konten pihak ketiga. File sumber PDF, dump ekstraksi, dan manifest sumber tidak disertakan di repository publik.

## Catatan

Versi desktop macOS membuka aplikasi dalam jendela WebView native, bukan browser eksternal. Backend tetap merupakan layanan lokal yang hanya bind ke loopback. Riwayat latihan disimpan pada komputer pengguna.
