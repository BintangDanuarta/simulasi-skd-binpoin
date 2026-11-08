# DESIGN.md — Simulasi SKD CPNS

> Sumber identitas visual & UX. Setiap keputusan besar harus bisa dijelaskan dalam satu baris (aturan antislop R-31).

## Arah visual (dipilih pengguna)
Gaya **bootcamp belajar modern**: terang, energik, fokus ke progres belajar (streak, checklist modul, roadmap), dengan aksen hangat.
Alasan: produk adalah alat latihan; nuansa bootcamp memberi rasa maju selangkah demi selangkah, bukan sekadar bank soal.

## Palet (2 warna inti + 1 aksen + warna state)
- `--bg: #FAF6EF` krem hangat — latar seperti kertas kerja, bukan abu dingin.
- `--card: #FFFFFF`
- `--ink: #29241C` tinta cokelat tua — kontras baca yang hangat.
- `--muted: #6F675A`
- `--line: #E7DFD1`
- `--accent: #0154d0` biru binpoin — aksen utama tombol, badge, dan fokus belajar.
- State: `success #2F7D4F`, `danger #B33A34`, `warn #A9701D`.
Alasan: palet hangat membedakan produk dari default biru-ungu AI dan mendukung suasana belajar yang serius tapi manusiawi.

## Radius (hierarki, bukan seragam)
- 8px: input, tombol kecil, chip.
- 12px: kartu, panel, baris materi.
- 16px: CTA utama saja.
Alasan: radius dipakai sebagai penanda hierarki, bukan dekorasi merata (R-11).

## Bayangan & elevasi
Hanya dua elemen memakai shadow: **header saat sticky** dan **modal**.
Alasan: elevasi menandai lapisan yang mengambang di atas konten; kartu lainnya duduk rata di latar (R-12).

## Motif identitas
- **Kotak centang** pada modul materi dan roadmap bootcamp (satu motif diulang).
- **Garis bawah aksen 2px** pada nav/tab aktif.
Alasan: satu motif yang diulang memberi kesan sistem, bukan dekorasi acak.

## Ikon & dekorasi
- Tanpa emoji dekoratif di heading, navigasi, dan tombol.
- Ikon memakai SVG garis sederhana: centang, play, panah navigasi soal.
- Tidak ada gradient dekoratif, glassmorphism, glow, atau background grid (R-06, R-07, R-10, R-13).

## Motion
- Dial MOTION: **1** — hanya transisi hover/focus ±120ms. Tanpa loop, tanpa pulse (R-19).
- Timer ujian berubah warna + bobot saat sisa < 5 menit (bukan berkedip).
- `prefers-reduced-motion` dihormati.

## Tipografi
- System stack tanpa unduhan: `ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif`.
- Skala: 13–15px teks; 16px teks soal; 18–20px judul seksi; 28–32px H1.

## Konten & kejujuran
- Statistik diambil dari `/api/meta` (data asli), tidak ada angka rekaan (R-17).
- Video YouTube dikurasi dan diverifikasi via oEmbed; bukan placeholder.
- Sumber resmi hanya yang terverifikasi dapat diakses.
- Empty state selalu menyebut penyebab + satu tindakan lanjutan (R-27).

## Mode antislop
`antislop active: gaya bootcamp belajar modern (session override).`
