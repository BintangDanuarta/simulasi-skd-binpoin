SIMULASI SKD BY BINPOIN — BETA 01 — PETUNJUK MAC

Paket ini berisi SOURCE CODE, bukan aplikasi Mac siap klik. Build dijalankan di Mac Anda.
Pilih instruksi sesuai jenis Mac:

APPLE SILICON (M1/M2/M3/M4):
1. Buka Terminal, masuk ke folder hasil ekstrak ZIP:
   cd ~/Downloads/cpns-app
2. Pastikan Xcode Command Line Tools dan Rust tersedia.
3. Jalankan:
   rustup target add aarch64-apple-darwin
   ./build-release.sh aarch64-apple-darwin
4. Jalankan aplikasi:
   cd release/simulasi-skd-binpoin-0.1.0-beta.1-aarch64-apple-darwin
   ./simulasi-skd-binpoin

MAC INTEL:
Ganti target di atas dengan x86_64-apple-darwin pada kedua perintah build.
Folder hasil build bernama simulasi-skd-binpoin-0.1.0-beta.1-x86_64-apple-darwin.

Browser bawaan akan dibuka. Aplikasi ini merupakan server lokal ringan yang memakai browser,
bukan jendela native. Riwayat latihan tersimpan lokal di komputer Anda.
