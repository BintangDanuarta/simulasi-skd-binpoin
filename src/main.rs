//! Simulasi SKD CPNS — aplikasi web latihan CPNS berbasis Rust (axum).
//! Bank soal: data/questions.json | Materi: data/materi.json | Riwayat: data/riwayat.json

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::TcpListener as StdTcpListener;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::ServeDir;

// ---------------------------------------------------------------------------
// Model data
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Kat {
    Twk,
    Tiu,
    Tkp,
    Campur,
}

impl Kat {
    fn as_str(&self) -> &'static str {
        match self {
            Kat::Twk => "TWK",
            Kat::Tiu => "TIU",
            Kat::Tkp => "TKP",
            Kat::Campur => "CAMPUR",
        }
    }
    fn from_str(s: &str) -> Option<Kat> {
        match s.to_uppercase().as_str() {
            "TWK" => Some(Kat::Twk),
            "TIU" => Some(Kat::Tiu),
            "TKP" => Some(Kat::Tkp),
            "CAMPUR" => Some(Kat::Campur),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Soal {
    pub id: u32,
    pub kat: Kat,
    pub sub: String,
    pub teks: String,
    pub pilihan: Vec<String>,
    pub kunci: u8, // indeks pilihan benar (0=A)
    #[serde(default)]
    pub skor_pilihan: Option<Vec<u8>>, // skor TKP per opsi (1-5)
    #[serde(default)]
    pub pembahasan: Option<String>,
    #[serde(default)]
    pub sumber: String,
}

/// Soal yang dikirim ke klien saat ujian (tanpa kunci).
#[derive(Clone, Serialize)]
pub struct SoalUjian {
    pub no: usize,
    pub kat: Kat,
    pub sub: String,
    pub teks: String,
    pub pilihan: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoRef {
    pub id: String,
    #[serde(default)]
    pub judul: String,
    #[serde(default)]
    pub topik: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Materi {
    pub id: String,
    pub kat: Kat,
    #[serde(default)]
    pub modul: String,
    #[serde(default)]
    pub sub: String,
    pub judul: String,
    #[serde(default)]
    pub ringkas: String,
    pub html: String,
    #[serde(default)]
    pub video: Vec<VideoRef>,
}

// ---------------------------------------------------------------------------
// Sesi ujian
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
pub struct UjianInfo {
    pub id: String,
    pub mode: String,
    pub mulai: u64,    // epoch detik
    pub deadline: u64, // epoch detik
    pub selesai: bool,
    pub soal: Vec<SoalUjian>,
    pub jawaban: HashMap<usize, u8>,
    pub ragu: HashSet<usize>,
}

pub struct UjianSession {
    pub id: String,
    pub mode: String,
    pub mulai: u64,
    pub deadline: u64,
    pub selesai: bool,
    pub soal: Vec<Soal>,
    pub jawaban: HashMap<usize, u8>,
    pub ragu: HashSet<usize>,
    pub hasil: Option<HasilUjian>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HasilBagian {
    pub kat: Kat,
    pub jumlah: usize,
    pub benar: usize,
    pub skor: u32,
    pub passing: u32,
    pub lulus: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HasilDetail {
    pub no: usize,
    pub kat: Kat,
    pub sub: String,
    pub teks: String,
    pub pilihan: Vec<String>,
    pub kunci: u8,
    pub jawaban: Option<u8>,
    pub benar: bool,
    pub skor: u32,
    pub pembahasan: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HasilUjian {
    pub waktu: String,
    pub mode: String,
    pub durasi_pakai: u64,
    pub skor_total: u32,
    pub lulus: bool,
    pub bagian: Vec<HasilBagian>,
    pub detail: Vec<HasilDetail>,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub soal: Vec<Soal>,
    pub materi: Vec<Materi>,
    pub video: serde_json::Value,
    pub ujian: Arc<Mutex<HashMap<String, UjianSession>>>,
    pub riwayat_path: PathBuf,
    pub riwayat: Arc<Mutex<Vec<HasilUjian>>>,
}

// ---------------------------------------------------------------------------
// Utilitas
// ---------------------------------------------------------------------------

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// PRNG sederhana xorshift + Fisher-Yates untuk mengacak pilihan soal.
fn shuffle<T>(mut v: Vec<T>, mut seed: u64) -> Vec<T> {
    let mut rng = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let n = v.len();
    for i in (1..n).rev() {
        let j = (rng() as usize) % (i + 1);
        v.swap(i, j);
    }
    v
}

fn today_iso() -> String {
    let secs = now();
    let days = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let (y, mo, da) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{da:02} {h:02}:{m:02}:{s:02}")
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ---------------------------------------------------------------------------
// Penilaian
// ---------------------------------------------------------------------------

const PASS_TWK: u32 = 65;
const PASS_TIU: u32 = 80;
const PASS_TKP: u32 = 166;

fn skor_soal(s: &Soal, jawaban: Option<u8>) -> u32 {
    match jawaban {
        None => 0,
        Some(j) => {
            if let Some(score) = s
                .skor_pilihan
                .as_ref()
                .and_then(|scores| scores.get(j as usize))
            {
                return u32::from(*score);
            }

            if j == s.kunci {
                5
            } else if s.kat == Kat::Tkp {
                1
            } else {
                0
            }
        }
    }
}

fn nilai_ujian(sesi: &UjianSession) -> HasilUjian {
    let mut per_kat: HashMap<Kat, (usize, usize, u32)> = HashMap::new();
    let mut detail = Vec::new();
    for (i, s) in sesi.soal.iter().enumerate() {
        let no = i + 1;
        let jw = sesi.jawaban.get(&no).copied();
        let sk = skor_soal(s, jw);
        let betul = jw == Some(s.kunci);
        let e = per_kat.entry(s.kat).or_insert((0, 0, 0));
        e.0 += 1;
        if betul {
            e.1 += 1;
        }
        e.2 += sk;
        detail.push(HasilDetail {
            no,
            kat: s.kat,
            sub: s.sub.clone(),
            teks: s.teks.clone(),
            pilihan: s.pilihan.clone(),
            kunci: s.kunci,
            jawaban: jw,
            benar: betul,
            skor: sk,
            pembahasan: s.pembahasan.clone(),
        });
    }
    let bagian: Vec<HasilBagian> = vec![Kat::Twk, Kat::Tiu, Kat::Tkp, Kat::Campur]
        .iter()
        .filter_map(|k| {
            per_kat.get(k).map(|&(n, b, sk)| {
                let pass = match k {
                    Kat::Twk => PASS_TWK,
                    Kat::Tiu => PASS_TIU,
                    Kat::Tkp => PASS_TKP,
                    Kat::Campur => 0,
                };
                HasilBagian {
                    kat: *k,
                    jumlah: n,
                    benar: b,
                    skor: sk,
                    passing: pass,
                    lulus: sk >= pass,
                }
            })
        })
        .collect();
    let bagian_berambang: Vec<&HasilBagian> =
        bagian.iter().filter(|b| b.kat != Kat::Campur).collect();
    let lulus = !bagian_berambang.is_empty() && bagian_berambang.iter().all(|b| b.lulus);
    let skor_total = bagian.iter().map(|b| b.skor).sum();
    HasilUjian {
        waktu: today_iso(),
        mode: sesi.mode.clone(),
        durasi_pakai: sesi.deadline.saturating_sub(sesi.mulai),
        skor_total,
        lulus,
        bagian,
        detail,
    }
}

// ---------------------------------------------------------------------------
// Handler API
// ---------------------------------------------------------------------------

async fn api_meta(State(st): State<AppState>) -> Json<serde_json::Value> {
    let mut per_kat: HashMap<&'static str, usize> = HashMap::new();
    let mut per_sub: HashMap<(Kat, String), usize> = HashMap::new();
    for s in &st.soal {
        *per_kat.entry(s.kat.as_str()).or_insert(0) += 1;
        *per_sub.entry((s.kat, s.sub.clone())).or_insert(0) += 1;
    }
    let mut subs: HashMap<&'static str, Vec<(String, usize)>> = HashMap::new();
    for ((k, sub), n) in per_sub {
        subs.entry(k.as_str()).or_default().push((sub, n));
    }
    for v in subs.values_mut() {
        v.sort_by(|a, b| b.1.cmp(&a.1));
    }
    Json(serde_json::json!({
        "total": st.soal.len(),
        "per_kat": per_kat,
        "subs": subs,
        "materi": st.materi.iter().map(|m| serde_json::json!({"id": m.id, "kat": m.kat.as_str(), "judul": m.judul, "modul": m.modul, "ringkas": m.ringkas})).collect::<Vec<_>>(),
    }))
}

async fn api_materi(State(st): State<AppState>) -> Json<Vec<Materi>> {
    Json(st.materi.clone())
}

async fn api_materi_satu(
    State(st): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Materi>, StatusCode> {
    st.materi
        .iter()
        .find(|m| m.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn api_video(State(st): State<AppState>) -> Json<serde_json::Value> {
    Json(st.video.clone())
}

#[derive(Deserialize)]
struct BankQuery {
    kat: Option<String>,
    sub: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
}

async fn api_bank(
    State(st): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<BankQuery>,
) -> Json<serde_json::Value> {
    let kat = q.kat.as_deref().and_then(Kat::from_str);
    let sub = q.sub.clone();
    let mut list: Vec<&Soal> = st
        .soal
        .iter()
        .filter(|s| kat.map_or(true, |k| s.kat == k))
        .filter(|s| sub.as_ref().map_or(true, |x| &s.sub == x))
        .collect();
    list.sort_by_key(|s| s.id);
    let offset = q.offset.unwrap_or(0);
    let limit = q.limit.unwrap_or(20).min(200);
    let total = list.len();
    let page: Vec<serde_json::Value> = list
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|s| {
            serde_json::json!({
                "id": s.id, "kat": s.kat.as_str(), "sub": s.sub,
                "teks": s.teks, "pilihan": s.pilihan,
                "kunci": s.kunci, "pembahasan": s.pembahasan,
                "skor_pilihan": s.skor_pilihan, "sumber": s.sumber,
            })
        })
        .collect();
    Json(serde_json::json!({"total": total, "offset": offset, "soal": page}))
}

fn pilih_soal_latihan(
    bank: &[Soal],
    kat: Kat,
    sub: &[String],
    jumlah: usize,
    seed: u64,
) -> Vec<Soal> {
    let subs: HashSet<&str> = sub.iter().map(String::as_str).collect();
    let mut questions: Vec<Soal> = bank
        .iter()
        .filter(|question| question.kat == kat && subs.contains(question.sub.as_str()))
        .cloned()
        .collect();
    questions = shuffle(questions, seed);
    questions.truncate(jumlah);
    questions
}

#[derive(Deserialize)]
struct LatihanStartBody {
    kat: String,
    sub: Vec<String>,
    jumlah: usize,
}

async fn api_latihan_start(
    State(st): State<AppState>,
    Json(body): Json<LatihanStartBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let Some(kat) = Kat::from_str(&body.kat) else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Pilih kategori TWK, TIU, atau TKP."})),
        ));
    };
    if kat == Kat::Campur {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Latihan mandiri tersedia untuk TWK, TIU, dan TKP."})),
        ));
    }
    if body.jumlah == 0 || body.jumlah > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Jumlah soal harus antara 1 dan 100."})),
        ));
    }
    let selected_subs: HashSet<&str> = body.sub.iter().map(String::as_str).collect();
    if selected_subs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Pilih setidaknya satu materi."})),
        ));
    }

    let available_count = st
        .soal
        .iter()
        .filter(|question| question.kat == kat && selected_subs.contains(question.sub.as_str()))
        .count();
    if available_count == 0 {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": "Tidak ada soal untuk pilihan materi tersebut."})),
        ));
    }

    let question_count = body.jumlah.min(available_count);
    let seed = now()
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(available_count as u64);
    let questions: Vec<serde_json::Value> =
        pilih_soal_latihan(&st.soal, kat, &body.sub, question_count, seed)
            .into_iter()
            .map(|question| {
                serde_json::json!({
                    "id": question.id,
                    "kat": question.kat.as_str(),
                    "sub": question.sub,
                    "teks": question.teks,
                    "pilihan": question.pilihan,
                    "kunci": question.kunci,
                    "pembahasan": question.pembahasan,
                    "skor_pilihan": question.skor_pilihan,
                })
            })
            .collect();

    Ok(Json(serde_json::json!({
        "kat": kat.as_str(),
        "diminta": body.jumlah,
        "jumlah": question_count,
        "total_tersedia": available_count,
        "soal": questions,
    })))
}

#[derive(Deserialize)]
struct StartQuery {
    mode: Option<String>,
    jumlah: Option<usize>,
}

const DURASI_SKD: u64 = 100 * 60; // 100 menit
const DURASI_PAKET: u64 = 30 * 60;

async fn api_ujian_start(
    State(st): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<StartQuery>,
) -> Response {
    let mode = q.mode.unwrap_or_else(|| "SKD".into());
    let now_ts = now();
    // hapus sesi lama (selesai/kedaluwarsa > 24 jam)
    {
        let mut map = st.ujian.lock().unwrap();
        map.retain(|_, s| s.selesai == false || now_ts - s.deadline < 86400);
        // hanya satu sesi aktif per pengguna (server tunggal)
        if let Some(aktif) = map.values().find(|s| !s.selesai) {
            let info = to_info(aktif);
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "ada_sesi_aktif", "sesi": info})),
            )
                .into_response();
        }
    }

    let (dikategori, durasi) = match mode.to_uppercase().as_str() {
        "TWK" => (vec![Kat::Twk], DURASI_PAKET),
        "TIU" => (vec![Kat::Tiu], DURASI_PAKET),
        "TKP" => (vec![Kat::Tkp], DURASI_PAKET),
        "CAMPUR" => (vec![Kat::Campur], DURASI_PAKET),
        _ => (vec![Kat::Twk, Kat::Tiu, Kat::Tkp], DURASI_SKD), // SKD penuh
    };

    let mut pilihan: Vec<Soal> = st
        .soal
        .iter()
        .filter(|s| dikategori.contains(&s.kat))
        .cloned()
        .collect();

    if pilihan.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "bank kosong"})),
        )
            .into_response();
    }

    let seed = now_ts ^ (now_ts << 21) ^ 0x9E37_79B9_7F4A_7C15;
    let mut terpilih: Vec<Soal> = Vec::new();
    if dikategori.len() == 3 {
        let mut by: HashMap<Kat, Vec<Soal>> = HashMap::new();
        for s in pilihan {
            by.entry(s.kat).or_default().push(s);
        }
        for (k, ambil) in [(Kat::Twk, 30usize), (Kat::Tiu, 35), (Kat::Tkp, 45)] {
            let mut v = by.remove(&k).unwrap_or_default();
            v = shuffle(v, seed.wrapping_add(k as u64 * 1_000_003));
            v.truncate(ambil);
            terpilih.extend(v);
        }
    } else {
        pilihan = shuffle(pilihan, seed);
        let n = q.jumlah.unwrap_or(25).min(pilihan.len());
        pilihan.truncate(n);
        terpilih = pilihan;
    }
    let id = format!("U{}", now_ts % 1_000_000);
    let sesi = UjianSession {
        id: id.clone(),
        mode: mode.to_uppercase(),
        mulai: now_ts,
        deadline: now_ts + durasi,
        selesai: false,
        soal: terpilih,
        jawaban: HashMap::new(),
        ragu: HashSet::new(),
        hasil: None,
    };
    let info = to_info(&sesi);
    st.ujian.lock().unwrap().insert(id, sesi);
    Json(serde_json::json!({"sesi": info})).into_response()
}

fn to_info(s: &UjianSession) -> UjianInfo {
    UjianInfo {
        id: s.id.clone(),
        mode: s.mode.clone(),
        mulai: s.mulai,
        deadline: s.deadline,
        selesai: s.selesai,
        soal: s
            .soal
            .iter()
            .enumerate()
            .map(|(i, x)| SoalUjian {
                no: i + 1,
                kat: x.kat,
                sub: x.sub.clone(),
                teks: x.teks.clone(),
                pilihan: x.pilihan.clone(),
            })
            .collect(),
        jawaban: s.jawaban.clone(),
        ragu: s.ragu.clone(),
    }
}

async fn api_ujian_aktif(State(st): State<AppState>) -> Json<serde_json::Value> {
    let map = st.ujian.lock().unwrap();
    if let Some(s) = map.values().find(|s| !s.selesai) {
        Json(serde_json::json!({"ada": true, "sesi": to_info(s)}))
    } else {
        Json(serde_json::json!({"ada": false}))
    }
}

#[derive(Deserialize)]
struct JawabBody {
    no: usize,
    pilihan: Option<u8>,
}

async fn api_ujian_jawab(
    State(st): State<AppState>,
    Path(id): Path<String>,
    Json(b): Json<JawabBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut map = st.ujian.lock().unwrap();
    let s = map.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if s.selesai {
        return Err(StatusCode::CONFLICT);
    }
    if b.no < 1 || b.no > s.soal.len() {
        return Err(StatusCode::BAD_REQUEST);
    }
    match b.pilihan {
        Some(p) if (p as usize) < s.soal[b.no - 1].pilihan.len() => {
            s.jawaban.insert(b.no, p);
        }
        _ => {
            s.jawaban.remove(&b.no);
        }
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct RaguBody {
    no: usize,
    ragu: bool,
}

async fn api_ujian_ragu(
    State(st): State<AppState>,
    Path(id): Path<String>,
    Json(b): Json<RaguBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut map = st.ujian.lock().unwrap();
    let s = map.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if s.selesai {
        return Err(StatusCode::CONFLICT);
    }
    if b.ragu {
        s.ragu.insert(b.no);
    } else {
        s.ragu.remove(&b.no);
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn api_ujian_kumpul(
    State(st): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<HasilUjian>, StatusCode> {
    let hasil = {
        let mut map = st.ujian.lock().unwrap();
        let s = map.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
        if s.selesai {
            return s.hasil.clone().map(Json).ok_or(StatusCode::NOT_FOUND);
        }
        let mut h = nilai_ujian(s);
        h.durasi_pakai = now().saturating_sub(s.mulai);
        s.selesai = true;
        s.hasil = Some(h.clone());
        h
    };
    let mut rw = st.riwayat.lock().unwrap();
    rw.insert(0, hasil.clone());
    rw.truncate(100);
    let _ = fs::write(
        &st.riwayat_path,
        serde_json::to_string_pretty(&*rw).unwrap_or_default(),
    );
    Ok(Json(hasil))
}

async fn api_ujian_batalkan(
    State(st): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut map = st.ujian.lock().unwrap();
    if let Some(s) = map.get_mut(&id) {
        if !s.selesai {
            s.selesai = true;
        }
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn api_riwayat(State(st): State<AppState>) -> Json<Vec<HasilUjian>> {
    Json(st.riwayat.lock().unwrap().clone())
}

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CPNS_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base =
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    base.map(|p| p.join("Binpoin/Simulasi SKD/data"))
        .unwrap_or_else(|| PathBuf::from("data"))
}

fn data_file(name: &str) -> PathBuf {
    let user = data_dir().join(name);
    if user.exists() {
        user
    } else {
        PathBuf::from("data").join(name)
    }
}

fn muat_soal() -> Vec<Soal> {
    let p = data_file("questions.json");
    match fs::read_to_string(&p) {
        Ok(txt) => serde_json::from_str(&txt).unwrap_or_default(),
        Err(_) => {
            eprintln!("[warn] data/questions.json tidak ditemukan — bank kosong");
            Vec::new()
        }
    }
}

fn muat_materi() -> Vec<Materi> {
    let p = data_file("materi.json");
    match fs::read_to_string(&p) {
        Ok(txt) => serde_json::from_str(&txt).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn muat_riwayat(p: &PathBuf) -> Vec<HasilUjian> {
    fs::read_to_string(p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn muat_video() -> serde_json::Value {
    let p = data_file("video.json");
    fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| serde_json::json!({ "TWK": [], "TIU": [], "TKP": [] }))
}

#[tokio::main]
async fn main() {
    let soal = muat_soal();
    let materi = muat_materi();
    let video = muat_video();
    let riwayat_path = data_dir().join("riwayat.json");
    if let Some(parent) = riwayat_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "[error] tidak bisa membuat folder data {}: {e}",
                parent.display()
            );
            std::process::exit(1);
        }
    }
    let riwayat = muat_riwayat(&riwayat_path);
    println!(
        "[cpns-simulasi] bank: {} soal | materi: {} | riwayat: {}",
        soal.len(),
        materi.len(),
        riwayat.len()
    );

    let state = AppState {
        soal,
        materi,
        video,
        ujian: Arc::new(Mutex::new(HashMap::new())),
        riwayat_path,
        riwayat: Arc::new(Mutex::new(riwayat)),
    };

    let app = Router::new()
        .route("/", get(hal_utama))
        .route("/api/meta", get(api_meta))
        .route("/api/materi", get(api_materi))
        .route("/api/materi/{id}", get(api_materi_satu))
        .route("/api/video", get(api_video))
        .route("/api/bank", get(api_bank))
        .route("/api/latihan/start", post(api_latihan_start))
        .route("/api/ujian/start", post(api_ujian_start))
        .route("/api/ujian/aktif", get(api_ujian_aktif))
        .route("/api/ujian/{id}/jawab", post(api_ujian_jawab))
        .route("/api/ujian/{id}/ragu", post(api_ujian_ragu))
        .route("/api/ujian/{id}/kumpul", post(api_ujian_kumpul))
        .route("/api/ujian/{id}/batalkan", post(api_ujian_batalkan))
        .route("/api/riwayat", get(api_riwayat))
        .fallback_service(ServeDir::new("static"))
        .with_state(state);

    let addr = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:0".to_string());
    let listener = if addr.ends_with(":0") {
        let std_listener = StdTcpListener::bind(&addr).expect("port lokal tidak tersedia");
        std_listener
            .set_nonblocking(true)
            .expect("gagal set nonblocking");
        tokio::net::TcpListener::from_std(std_listener).expect("gagal membuat listener")
    } else {
        tokio::net::TcpListener::bind(&addr)
            .await
            .expect("gagal bind alamat")
    };
    let actual_addr = listener.local_addr().expect("alamat listener");
    println!("[cpns-simulasi] mendengarkan di http://{actual_addr}");
    #[cfg(target_os = "windows")]
    let opener = std::process::Command::new("cmd")
        .args(["/C", "start", "", &format!("http://{actual_addr}")])
        .spawn();
    #[cfg(target_os = "macos")]
    let opener = std::process::Command::new("open")
        .arg(format!("http://{actual_addr}"))
        .spawn();
    #[cfg(target_os = "linux")]
    let opener = std::process::Command::new("xdg-open")
        .arg(format!("http://{actual_addr}"))
        .spawn();
    if opener.is_err() {
        eprintln!("Buka http://{actual_addr} di browser Anda");
    }
    axum::serve(listener, app).await.expect("server berhenti");
}

async fn hal_utama() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

#[cfg(test)]
mod tests {
    use super::{Kat, Soal, pilih_soal_latihan, skor_soal};
    use std::collections::HashSet;

    fn soal(kat: Kat, kunci: u8, skor_pilihan: Option<Vec<u8>>) -> Soal {
        Soal {
            id: 1,
            kat,
            sub: String::new(),
            teks: String::from("Soal tes"),
            pilihan: vec![String::from("A"), String::from("B"), String::from("C")],
            kunci,
            skor_pilihan,
            pembahasan: None,
            sumber: String::new(),
        }
    }

    #[test]
    fn skor_pilihan_dipakai_untuk_tkp_dan_kategori_lain() {
        let scores = vec![1, 3, 5];
        for kat in [Kat::Tkp, Kat::Twk, Kat::Tiu, Kat::Campur] {
            assert_eq!(skor_soal(&soal(kat, 2, Some(scores.clone())), Some(1)), 3);
        }
    }

    #[test]
    fn skor_biner_tetap_jika_tidak_ada_skor_pilihan() {
        assert_eq!(skor_soal(&soal(Kat::Twk, 1, None), Some(1)), 5);
        assert_eq!(skor_soal(&soal(Kat::Twk, 1, None), Some(0)), 0);
        assert_eq!(skor_soal(&soal(Kat::Tkp, 1, None), Some(0)), 1);
        assert_eq!(skor_soal(&soal(Kat::Tkp, 1, None), None), 0);
    }

    #[test]
    fn bank_soal_memiliki_teks_pilihan_dan_kunci_yang_valid() {
        let bank: Vec<Soal> = serde_json::from_str(include_str!("../data/questions.json"))
            .expect("bank soal harus JSON valid");
        let app_ids: Vec<String> =
            serde_json::from_str(include_str!("../reports/app_question_ids.json"))
                .expect("manifest ID bank aplikasi harus JSON valid");
        let curated_ids: Vec<String> =
            serde_json::from_str(include_str!("../reports/valid_question_ids.json"))
                .expect("manifest ID kandidat valid harus JSON valid");
        let mut ids = HashSet::new();

        for item in &bank {
            assert!(ids.insert(item.id), "ID soal duplikat: {}", item.id);
            assert!(
                !item.teks.trim().is_empty(),
                "teks kosong di soal {}",
                item.id
            );
            assert!(
                !item.pilihan.is_empty(),
                "pilihan kosong di soal {}",
                item.id
            );
            assert!(
                item.pilihan.iter().all(|choice| !choice.trim().is_empty()),
                "ada pilihan kosong di soal {}",
                item.id
            );
            assert!(
                usize::from(item.kunci) < item.pilihan.len(),
                "indeks kunci di luar pilihan di soal {}",
                item.id
            );
            if let Some(scores) = &item.skor_pilihan {
                assert_eq!(
                    scores.len(),
                    item.pilihan.len(),
                    "jumlah skor pilihan tidak cocok di soal {}",
                    item.id
                );
                assert!(
                    usize::from(item.kunci) < scores.len(),
                    "indeks kunci tanpa skor di soal {}",
                    item.id
                );
            }
        }

        let app_manifest: HashSet<u32> = app_ids
            .iter()
            .map(|id| id.parse::<u32>().expect("ID kurasi harus numerik"))
            .collect();
        assert_eq!(
            app_manifest.len(),
            app_ids.len(),
            "manifest ID aplikasi duplikat"
        );
        assert_eq!(
            app_manifest, ids,
            "bank aplikasi harus sama persis dengan manifest bank aplikasi"
        );

        let curated: HashSet<u32> = curated_ids
            .iter()
            .map(|id| id.parse::<u32>().expect("ID kandidat harus numerik"))
            .collect();
        assert!(
            curated.is_subset(&ids),
            "semua soal yang disetujui pengguna harus dipertahankan di bank aplikasi"
        );
    }

    #[test]
    fn kunci_soal_hitung_yang_dapat_dibuktikan() {
        let bank: Vec<Soal> = serde_json::from_str(include_str!("../data/questions.json"))
            .expect("bank soal harus JSON valid");
        let key_for = |id| {
            bank.iter()
                .find(|item| item.id == id)
                .map(|item| item.kunci)
        };

        // For x < z with y strictly between x and z, x < y.
        assert_eq!(key_for(1763), Some(1));
    }

    #[test]
    fn latihan_memilih_hanya_kategori_dan_submateri_terpilih() {
        let bank: Vec<Soal> = serde_json::from_str(include_str!("../data/questions.json"))
            .expect("bank soal harus JSON valid");
        let selected = pilih_soal_latihan(&bank, Kat::Tiu, &[String::from("Numerik")], 5, 12345);
        assert!(!selected.is_empty());
        assert!(
            selected
                .iter()
                .all(|question| { question.kat == Kat::Tiu && question.sub == "Numerik" })
        );
        assert!(selected.len() <= 5);
    }

    #[test]
    fn latihan_mengambil_hingga_jumlah_tersedia() {
        let bank: Vec<Soal> = serde_json::from_str(include_str!("../data/questions.json"))
            .expect("bank soal harus JSON valid");
        let one_question = vec![
            bank.iter()
                .find(|question| question.kat == Kat::Tiu && question.sub == "Numerik")
                .expect("bank terkurasi harus berisi soal numerik TIU")
                .clone(),
        ];
        let selected = pilih_soal_latihan(
            &one_question,
            Kat::Tiu,
            &[String::from("Numerik")],
            10,
            67890,
        );
        assert_eq!(selected.len(), 1);
    }
}
