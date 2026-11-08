# Simulasi SKD by Binpoin

Aplikasi latihan Seleksi Kompetensi Dasar (SKD) CPNS dengan bank soal, materi belajar, latihan topik, dan simulasi CAT.

## Fitur

- Latihan soal TWK, TIU, TKP, dan campuran
- Simulasi ujian bertimer dan hasil per bagian
- Materi belajar terstruktur
- Riwayat latihan tersimpan lokal
- Backend Rust (Axum) dan antarmuka web
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

## Distribusi beta

Paket rilis dibangun terpisah melalui GitHub Actions. File hasil build dan paket distribusi lokal tidak disimpan sebagai source di repo (`release/` diabaikan).

## Lisensi

Kode aplikasi dilisensikan di bawah MIT; lihat [LICENSE](LICENSE).

Bank soal dan materi pada `data/` didistribusikan terpisah di bawah izin yang dimiliki Binpoin. MIT License yang mencakup kode **tidak otomatis melisensikan** konten pihak ketiga. File sumber PDF, dump ekstraksi, dan manifest sumber tidak disertakan di repository publik.

## Catatan

Aplikasi beta berjalan sebagai layanan lokal yang membuka browser bawaan, bukan jendela desktop native. Riwayat latihan disimpan pada komputer pengguna.
