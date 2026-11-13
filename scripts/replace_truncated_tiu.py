#!/usr/bin/env python3
"""Replace only high-confidence incomplete TIU stems with self-contained items."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BANK = ROOT / "data" / "questions.json"

PROVENANCE = (
    "Soal pengganti orisinal; angka/kunci dihitung dari informasi dalam soal, "
    "atau kesimpulan diturunkan langsung dari premis yang dicantumkan."
)

REPLACEMENTS = {
    6899: {
        "sub": "Numerik",
        "teks": "Sebuah sepeda motor menempuh 56 km dengan 8 liter bensin. Jika konsumsi bensinnya tetap, berapa jarak yang dapat ditempuh dengan 14 liter?",
        "pilihan": ["84 km", "91 km", "98 km", "105 km", "112 km"],
        "kunci": 2,
        "pembahasan": "Kendaraan menempuh 56 ÷ 8 = 7 km per liter. Dengan 14 liter, jaraknya 14 × 7 = 98 km.",
    },
    6900: {
        "sub": "Numerik",
        "teks": "Seorang pedagang membeli 50 kg mangga seharga total Rp400.000,00. Semua mangga terjual dengan harga Rp9.000,00 per kg. Berapa keuntungan kotor pedagang itu?",
        "pilihan": ["Rp30.000,00", "Rp40.000,00", "Rp50.000,00", "Rp60.000,00", "Rp70.000,00"],
        "kunci": 2,
        "pembahasan": "Hasil penjualan = 50 × Rp9.000,00 = Rp450.000,00. Keuntungan kotor = Rp450.000,00 − Rp400.000,00 = Rp50.000,00.",
    },
    7028: {
        "sub": "Logika",
        "teks": "Premis 1: Jika sistem arsip aktif, maka setiap berkas baru dicadangkan. Premis 2: Sistem arsip aktif. Kesimpulan yang sah adalah …",
        "pilihan": [
            "Tidak ada berkas baru yang dicadangkan.",
            "Setiap berkas baru dicadangkan.",
            "Sistem arsip tidak aktif.",
            "Pencadangan hanya berlaku untuk berkas lama.",
            "Tidak dapat disimpulkan apakah sistem arsip aktif.",
        ],
        "kunci": 1,
        "pembahasan": "Misalkan P = sistem arsip aktif dan Q = setiap berkas baru dicadangkan. Dari P → Q serta P, dengan modus ponens diperoleh Q.",
    },
    7306: {
        "sub": "Analitis",
        "teks": "Lima peserta, Nia, Oki, Putra, Raka, dan Sari, berbaris dari depan ke belakang. Oki tepat di depan Putra, Raka tepat di belakang Putra, Sari berada paling belakang, dan Nia berada di depan Oki. Siapa yang menempati posisi ketiga?",
        "pilihan": ["Nia", "Oki", "Putra", "Raka", "Sari"],
        "kunci": 2,
        "pembahasan": "Oki–Putra–Raka harus berurutan. Sari berada di posisi kelima. Karena Nia berada di depan Oki dan hanya ada lima peserta, urutannya Nia, Oki, Putra, Raka, Sari. Posisi ketiga ditempati Putra.",
    },
    7307: {
        "sub": "Analitis",
        "teks": "Empat peserta, A, B, C, dan D, mendapat urutan presentasi yang berbeda. B tampil tepat setelah A. D tampil sebelum C. A tampil sebelum D. Siapa yang tampil pada urutan kedua?",
        "pilihan": ["A", "B", "C", "D", "Tidak dapat ditentukan"],
        "kunci": 1,
        "pembahasan": "A harus sebelum D, D sebelum C, dan B tepat setelah A. Urutan A–B–D–C memenuhi seluruh syarat. Urutan lain tidak dapat menempatkan B tepat setelah A sambil mempertahankan D sebelum C. Jadi urutan kedua adalah B.",
    },
    7601: {
        "sub": "Analitis",
        "teks": "Enam peserta, Bimo, Kia, Shaka, Dhuha, Adifa, dan Nata, diurutkan dari usia tertua ke termuda. Bimo lebih tua daripada Shaka. Nata lebih muda daripada Adifa. Kia paling muda, tepat di bawah Dhuha. Jika Adifa berada di urutan ketiga, siapa yang berada di urutan pertama dan kedua?",
        "pilihan": ["Bimo dan Shaka", "Shaka dan Bimo", "Bimo dan Nata", "Dhuha dan Kia", "Nata dan Shaka"],
        "kunci": 0,
        "pembahasan": "Dhuha berada tepat di atas Kia, sehingga keduanya menempati urutan 5 dan 6. Adifa di urutan 3 dan Nata lebih muda daripada Adifa, maka Nata di urutan 4. Tersisa Bimo dan Shaka untuk urutan 1 dan 2. Karena Bimo lebih tua daripada Shaka, urutannya Bimo lalu Shaka.",
    },
    7698: {
        "sub": "Logika",
        "teks": "Semua dokter yang memakai jas putih membawa stetoskop. Tidak ada orang yang membawa stetoskop yang bekerja di Puskesmas Sehat. Kesimpulan manakah yang pasti benar?",
        "pilihan": [
            "Semua pegawai Puskesmas Sehat adalah dokter.",
            "Tidak ada dokter berjas putih yang bekerja di Puskesmas Sehat.",
            "Semua dokter yang tidak memakai jas putih bekerja di Puskesmas Sehat.",
            "Ada pegawai Puskesmas Sehat yang membawa stetoskop.",
            "Tidak ada dokter yang bekerja di Puskesmas Sehat.",
        ],
        "kunci": 1,
        "pembahasan": "Dokter berjas putih selalu membawa stetoskop. Orang yang membawa stetoskop tidak bekerja di Puskesmas Sehat. Karena itu, dokter berjas putih tidak bekerja di Puskesmas Sehat. Premis tidak cukup untuk menyimpulkan tentang dokter tanpa jas putih.",
    },
    1750: {
        "sub": "Logika",
        "teks": "Premis 1: Jika sebuah bilangan habis dibagi 4, maka bilangan itu genap. Premis 2: Bilangan n habis dibagi 4. Kesimpulan yang pasti benar adalah …",
        "pilihan": ["n genap", "n ganjil", "n bilangan prima", "n lebih besar dari 4", "Tidak dapat ditentukan apakah n genap"],
        "kunci": 0,
        "pembahasan": "Dari premis jika habis dibagi 4 maka genap, serta fakta bahwa n habis dibagi 4, dengan modus ponens dapat disimpulkan bahwa n genap.",
    },
    1753: {
        "sub": "Numerik",
        "teks": "Diketahui p = 3 dan q = 1. Jika x = p × 64 dan y = p × q + q³, berapakah nilai x − y?",
        "pilihan": ["188", "190", "192", "194", "196"],
        "kunci": 0,
        "pembahasan": "x = 3 × 64 = 192. y = 3 × 1 + 1³ = 4. Maka x − y = 192 − 4 = 188, yaitu opsi A.",
    },
    2095: {
        "sub": "Logika",
        "teks": "Ingkaran dari pernyataan “Semua dokter memakai tanda pengenal” adalah …",
        "pilihan": [
            "Tidak ada dokter yang memakai tanda pengenal.",
            "Semua orang yang memakai tanda pengenal adalah dokter.",
            "Ada dokter yang tidak memakai tanda pengenal.",
            "Ada orang yang bukan dokter dan memakai tanda pengenal.",
            "Semua dokter tidak memakai tanda pengenal.",
        ],
        "kunci": 2,
        "pembahasan": "Negasi dari “semua A adalah B” adalah “ada A yang bukan B”. Jadi ingkarannya: ada dokter yang tidak memakai tanda pengenal.",
    },
    2106: {
        "sub": "Logika",
        "teks": "Pernyataan bersyaratnya adalah: “Jika hujan turun, maka lapangan basah.” Manakah invers dari pernyataan tersebut?",
        "pilihan": [
            "Jika hujan turun, maka lapangan tidak basah.",
            "Jika hujan tidak turun, maka lapangan tidak basah.",
            "Jika lapangan tidak basah, maka hujan tidak turun.",
            "Jika lapangan basah, maka hujan turun.",
            "Hujan turun jika dan hanya jika lapangan basah.",
        ],
        "kunci": 1,
        "pembahasan": "Untuk implikasi P → Q, inversnya adalah ¬P → ¬Q. Dengan P = hujan turun dan Q = lapangan basah, inversnya: jika hujan tidak turun, maka lapangan tidak basah.",
    },
    2107: {
        "sub": "Logika",
        "teks": "Ingkaran dari pernyataan “Ada bilangan prima yang genap” adalah …",
        "pilihan": [
            "Ada bilangan prima yang tidak genap.",
            "Semua bilangan prima genap.",
            "Tidak ada bilangan prima yang genap.",
            "Semua bilangan genap adalah bilangan prima.",
            "Ada bilangan genap yang bukan bilangan prima.",
        ],
        "kunci": 2,
        "pembahasan": "Negasi dari pernyataan eksistensial “ada A yang B” adalah “tidak ada A yang B”. Jadi tidak ada bilangan prima yang genap.",
    },
    6984: {
        "sub": "Numerik",
        "teks": "Diketahui x = 1/16 dan y = 16%. Manakah hubungan antara x dan y yang benar?",
        "pilihan": ["x = y", "x < y", "x > y", "x = 2y", "Tidak dapat ditentukan"],
        "kunci": 1,
        "pembahasan": "x = 1/16 = 0,0625 atau 6,25%. Sementara y = 16% = 0,16. Jadi x < y.",
    },
    7214: {
        "sub": "Numerik",
        "teks": "Hitung nilai 237² − 236².",
        "pilihan": ["471", "472", "473", "474", "475"],
        "kunci": 2,
        "pembahasan": "Gunakan selisih kuadrat: 237² − 236² = (237 − 236)(237 + 236) = 1 × 473 = 473.",
    },
    7305: {
        "sub": "Logika",
        "teks": "Sebuah pabrik akan memilih produk untuk satu hari. Produk A dan B tidak boleh diproduksi pada hari yang sama. Produk C hanya boleh diproduksi jika produk B juga diproduksi. Jika produk A diproduksi, pasangan produk lain manakah yang mustahil diproduksi bersamaan dengannya?",
        "pilihan": ["B dan C", "C dan D", "D dan E", "C dan E", "D dan F"],
        "kunci": 0,
        "pembahasan": "A dan B tidak boleh diproduksi bersama, sehingga pasangan yang memuat B mustahil jika A diproduksi. C mensyaratkan B, sehingga B dan C juga tidak dapat dipilih. Pilihan A adalah pasangan yang pasti tidak mungkin.",
    },
    7510: {
        "sub": "Numerik",
        "teks": "Diketahui a = 2023² + 2022² dan b = (2023 + 2022)². Manakah hubungan yang benar?",
        "pilihan": ["a < b", "a > b", "a = b", "a = 2b", "Tidak dapat ditentukan"],
        "kunci": 0,
        "pembahasan": "b = 2023² + 2(2023)(2022) + 2022². Karena suku tambahan 2(2023)(2022) positif, b > a. Jadi a < b.",
    },
    7607: {
        "sub": "Numerik",
        "teks": "Jika a = 4 dan b = 3, berapakah nilai a³ + 3a²b + 3ab² + b³?",
        "pilihan": ["240", "300", "343", "350", "400"],
        "kunci": 2,
        "pembahasan": "Bentuk a³ + 3a²b + 3ab² + b³ sama dengan (a+b)³. Jadi (4+3)³ = 7³ = 343.",
    },
    7706: {
        "sub": "Numerik",
        "teks": "Diketahui a = 697² + 97² dan b = (697 + 97)². Manakah hubungan yang benar antara a dan b?",
        "pilihan": ["a < b", "a > b", "a = b", "a = 2b", "Tidak dapat ditentukan"],
        "kunci": 0,
        "pembahasan": "b = 697² + 2(697)(97) + 97². Suku tambahan 2(697)(97) positif, sehingga b > a dan a < b.",
    },
    7308: {
        "sub": "Numerik",
        "teks": "Perhatikan deret 4, 7, 10, 13, 17, 19. Satu angka tidak mengikuti pola yang sama dengan angka lainnya. Angka manakah yang menyimpang?",
        "pilihan": ["7", "10", "13", "17", "19"],
        "kunci": 3,
        "pembahasan": "Deret bertambah 3 setiap langkah: 4, 7, 10, 13, 16, 19. Angka 17 seharusnya 16, sehingga 17 adalah angka yang menyimpang.",
    },
    7507: {
        "sub": "Deret Angka",
        "teks": "Perhatikan urutan huruf B, E, H, K, N, Q, U, W. Jika jarak setiap huruf seharusnya tetap, huruf manakah yang menyimpang?",
        "pilihan": ["E", "H", "K", "U", "W"],
        "kunci": 3,
        "pembahasan": "Nomor huruf bertambah 3: B(2), E(5), H(8), K(11), N(14), Q(17), T(20), W(23). Pada urutan tersebut U(21) seharusnya T(20), sehingga U menyimpang.",
    },
    7612: {
        "sub": "Numerik",
        "teks": "Dina berangkat dengan sepeda motor pukul 06.25. Jarak ke sekolah 18 km dan ia melaju dengan kecepatan tetap 36 km/jam. Pukul berapa Dina tiba?",
        "pilihan": ["06.45", "06.50", "06.55", "07.00", "07.05"],
        "kunci": 2,
        "pembahasan": "Waktu tempuh = 18 ÷ 36 jam = 0,5 jam = 30 menit. Pukul 06.25 + 30 menit = 06.55.",
    },
    7711: {
        "sub": "Numerik",
        "teks": "Nara menabung Rp8.400.000,00 dengan bunga tunggal 6% per tahun. Jika uang disimpan selama 10 bulan tanpa setoran atau penarikan, berapa saldo akhir Nara?",
        "pilihan": ["Rp8.750.000,00", "Rp8.800.000,00", "Rp8.820.000,00", "Rp8.840.000,00", "Rp8.900.000,00"],
        "kunci": 2,
        "pembahasan": "Bunga = Rp8.400.000,00 × 6% × 10/12 = Rp420.000,00. Saldo akhir = Rp8.400.000,00 + Rp420.000,00 = Rp8.820.000,00.",
    },
}


def main() -> None:
    bank = json.loads(BANK.read_text(encoding="utf-8"))
    by_id = {question["id"]: question for question in bank}
    if len(by_id) != len(bank):
        raise ValueError("Bank has duplicate IDs; refusing ambiguous edits")
    missing = set(REPLACEMENTS) - by_id.keys()
    if missing:
        raise ValueError(f"Replacement IDs missing from bank: {sorted(missing)}")

    before_count = len(bank)
    old = {}
    for qid, replacement in REPLACEMENTS.items():
        question = by_id[qid]
        if question.get("kat") != "TIU":
            raise ValueError(f"Question {qid} is no longer TIU")
        old[qid] = {"teks": question["teks"], "sub": question["sub"]}
        question.update(replacement)
        question["skor_pilihan"] = None
        question["sumber"] = PROVENANCE

    # The repository's reviewed TIU taxonomy keeps this valid percentage item
    # in the Numerik practice category even though its stem completes via options.
    by_id[7111]["sub"] = "Numerik"

    if len(bank) != before_count:
        raise AssertionError("Replacement changed bank row count")
    for qid in REPLACEMENTS:
        q = by_id[qid]
        assert len(q["pilihan"]) == 5
        assert 0 <= q["kunci"] < len(q["pilihan"])
        assert len({option.casefold().strip() for option in q["pilihan"]}) == 5
        assert q["pembahasan"].strip()
    # Independently verify all generated replacement arithmetic and logic keys.
    checks = {
        6899: "98 km", 6900: "Rp50.000,00", 7028: "Setiap berkas baru dicadangkan.",
        7306: "Putra", 7307: "B", 7308: "17", 7507: "U", 7601: "Bimo dan Shaka",
        7612: "06.55", 7698: "Tidak ada dokter berjas putih yang bekerja di Puskesmas Sehat.",
        7711: "Rp8.820.000,00", 1750: "n genap", 1753: "188",
    }
    for qid, expected_choice in checks.items():
        q = by_id[qid]
        if q["pilihan"][q["kunci"]] != expected_choice:
            raise ValueError(f"Key check failed for {qid}: expected {expected_choice!r}, got {q['pilihan'][q['kunci']]!r}")

    BANK.write_text(json.dumps(bank, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"Replaced {len(REPLACEMENTS)} incomplete or OCR-damaged TIU items; bank still has {len(bank)} items.")
    for qid, previous in old.items():
        print(f"#{qid}: {previous['sub']} -> {by_id[qid]['sub']} | old stem: {previous['teks']}")


if __name__ == "__main__":
    main()