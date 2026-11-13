//! Simulasi SKD CPNS — aplikasi web latihan CPNS berbasis Rust (axum).
//! Bank soal: data/questions.json | Materi: data/materi.json | Riwayat: data/riwayat.json
use axum::{
    Json, Router,
    extract::{ConnectInfo, Extension, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::{OsRng, RngCore}};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path as FsPath;
use std::net::{IpAddr, SocketAddr, TcpListener as StdTcpListener};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::services::ServeDir;
use uuid::Uuid;

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

#[derive(Clone, Serialize, Deserialize)]
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
    #[serde(default = "default_attempt_no")]
    pub attempt_no: i32,
}

fn default_attempt_no()->i32 {1}

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

#[derive(Clone, Debug)]
struct AuthUser {
    id: Uuid,
    email: String,
}

#[derive(Clone)]
pub struct WebConfig {
    secure_cookies: bool,
    session_ttl_seconds: i64,
    login_attempts: Arc<Mutex<HashMap<String, (u32, u64)>>>,
    registration_attempts: Arc<Mutex<HashMap<String, (u32, u64)>>>,
}

#[derive(Clone)]
pub struct AppState {
    pub soal: Vec<Soal>,
    pub materi: Vec<Materi>,
    pub video: serde_json::Value,
    pub ujian: Arc<Mutex<HashMap<String, UjianSession>>>,
    pub db: Option<PgPool>,
    web_config: WebConfig,
    pub legacy_riwayat_path: PathBuf,
    pub legacy_riwayat: Arc<Mutex<Vec<HasilUjian>>>,
    cors_origins: Vec<HeaderValue>,
    trusted_proxy_ips: Vec<IpAddr>,
    allowed_hosts: Vec<String>,
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

async fn api_meta(State(st): State<AppState>, Extension(_user): Extension<AuthUser>) -> Result<Json<serde_json::Value>,StatusCode> {
    let (soal,materi)=if let Some(pool)=&st.db {
        let soal:Vec<Soal>=serde_json::from_value(global_content(pool,"questions").await?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        let materi:Vec<Materi>=serde_json::from_value(global_content(pool,"materials").await?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        (soal,materi)
    } else {(st.soal.clone(),st.materi.clone())};
    let mut per_kat: HashMap<&'static str, usize> = HashMap::new();
    let mut per_sub: HashMap<(Kat, String), usize> = HashMap::new();
    for s in &soal {
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
    Ok(Json(serde_json::json!({
        "total": soal.len(),
        "per_kat": per_kat,
        "subs": subs,
        "materi": materi.iter().map(|m| serde_json::json!({"id": m.id, "kat": m.kat.as_str(), "judul": m.judul, "modul": m.modul, "ringkas": m.ringkas})).collect::<Vec<_>>(),
    })))
}

async fn api_materi(State(st): State<AppState>, Extension(_user): Extension<AuthUser>) -> Result<Json<Vec<Materi>>,StatusCode> {
    if let Some(pool)=&st.db {return Ok(Json(serde_json::from_value(global_content(pool,"materials").await?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?));}
    Ok(Json(st.materi.clone()))
}

async fn api_materi_satu(
    State(st): State<AppState>,
    Extension(_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Materi>, StatusCode> {
    let materials=if let Some(pool)=&st.db {serde_json::from_value(global_content(pool,"materials").await?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?} else {st.materi.clone()};
    materials
        .iter()
        .find(|m| m.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn api_video(State(st): State<AppState>, Extension(_user): Extension<AuthUser>) -> Result<Json<serde_json::Value>,StatusCode> {
    if let Some(pool)=&st.db {return Ok(Json(global_content(pool,"video").await?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?));}
    Ok(Json(st.video.clone()))
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
    Extension(_user): Extension<AuthUser>,
    axum::extract::Query(q): axum::extract::Query<BankQuery>,
) -> Result<Json<serde_json::Value>,StatusCode> {
    let bank=if let Some(pool)=&st.db {serde_json::from_value(global_content(pool,"questions").await?.ok_or(StatusCode::SERVICE_UNAVAILABLE)?).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?} else {st.soal.clone()};
    let kat = q.kat.as_deref().and_then(Kat::from_str);
    let sub = q.sub.clone();
    let mut list: Vec<&Soal> = bank
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
    Ok(Json(serde_json::json!({"total": total, "offset": offset, "soal": page})))
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
    Extension(user): Extension<AuthUser>,
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
        .wrapping_add(available_count as u64)
        .wrapping_add(user.id.as_u128() as u64);
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

fn postgres<'a>(st: &'a AppState) -> Result<&'a PgPool, (StatusCode, Json<serde_json::Value>)> {
    st.db.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error":"Fitur akun hanya tersedia pada deployment web."})),
        )
    })
}

async fn global_content(pool:&PgPool,id:&'static str)->Result<Option<serde_json::Value>,StatusCode>{
    sqlx::query_scalar::<_,serde_json::Value>("SELECT body FROM global_content WHERE id=$1")
        .bind(id).fetch_optional(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
}

fn json_error(status: StatusCode, message: &'static str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({"error": message})))
}

async fn load_user_exam(st: &AppState, user_id: Uuid, id: &str) -> Result<UjianSession, StatusCode> {
    if let Some(pool) = &st.db {
        let exam_id = Uuid::parse_str(id).map_err(|_| StatusCode::NOT_FOUND)?;
        let row = sqlx::query_as::<_, (String, i64, i64, bool, serde_json::Value, serde_json::Value, serde_json::Value, Option<serde_json::Value>, i32)>(
            "SELECT mode, started_at, deadline, finished, questions, answers, unsure, result, attempt_no FROM exams WHERE id=$1 AND user_id=$2 AND deadline>extract(epoch from now())::bigint"
        ).bind(exam_id).bind(user_id).fetch_optional(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
        let questions: Vec<Soal> = serde_json::from_value(row.4).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        let answers_raw: HashMap<String,u8> = serde_json::from_value(row.5).unwrap_or_default();
        let jawaban=answers_raw.into_iter().filter_map(|(k,v)|k.parse::<usize>().ok().map(|k|(k,v))).collect();
        let unsure_raw: Vec<usize> = serde_json::from_value(row.6).unwrap_or_default();
        let result=row.7.map(serde_json::from_value).transpose().map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(UjianSession { id:id.to_owned(), mode:row.0, mulai:row.1.max(0) as u64, deadline:row.2.max(0) as u64, selesai:row.3, soal:questions, jawaban, ragu:unsure_raw.into_iter().collect(), hasil:result, attempt_no:row.8.max(1) });
    }
    st.ujian.lock().map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&format!("{user_id}:active")).cloned().filter(|s|s.id==id).ok_or(StatusCode::NOT_FOUND)
}

async fn persist_exam_update(st:&AppState,user_id:Uuid,session:&UjianSession,expected_attempt:i32)->Result<(),StatusCode>{
    let Some(pool)=&st.db else {return persist_user_exam(st,user_id,session).await};
    let id=Uuid::parse_str(&session.id).map_err(|_|StatusCode::NOT_FOUND)?;
    let answers:HashMap<String,u8>=session.jawaban.iter().map(|(k,v)|(k.to_string(),*v)).collect();
    let result=sqlx::query("UPDATE exams SET finished=$3,answers=$4,unsure=$5,result=$6 WHERE id=$1 AND user_id=$2 AND attempt_no=$7 AND finished=false AND deadline>extract(epoch from now())::bigint")
      .bind(id).bind(user_id).bind(session.selesai).bind(serde_json::to_value(answers).unwrap_or_default()).bind(serde_json::to_value(&session.ragu).unwrap_or_default()).bind(serde_json::to_value(&session.hasil).unwrap_or(serde_json::Value::Null)).bind(expected_attempt)
      .execute(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected()!=1 {return Err(StatusCode::CONFLICT)}
    Ok(())
}

async fn persist_user_exam(st:&AppState,user_id:Uuid,s:&UjianSession)->Result<(),StatusCode>{
    if let Some(pool)=&st.db {
        let id=Uuid::parse_str(&s.id).map_err(|_|StatusCode::NOT_FOUND)?;
        let answers:HashMap<String,u8>=s.jawaban.iter().map(|(k,v)|(k.to_string(),*v)).collect();
        sqlx::query("UPDATE exams SET finished=$3,answers=$4,unsure=$5,result=$6 WHERE id=$1 AND user_id=$2")
            .bind(id).bind(user_id).bind(s.selesai).bind(serde_json::to_value(answers).unwrap_or_default()).bind(serde_json::to_value(&s.ragu).unwrap_or_default()).bind(serde_json::to_value(&s.hasil).unwrap_or(serde_json::Value::Null)).execute(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        st.ujian.lock().map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?.insert(format!("{user_id}:active"),s.clone());
    }
    Ok(())
}

async fn require_user(
    State(st): State<AppState>,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    let Some(pool) = st.db.as_ref() else {
        request.extensions_mut().insert(AuthUser { id: Uuid::nil(), email: "desktop@local".into() });
        return next.run(request).await;
    };

    let token = request.headers().get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| cookies.split(';').find_map(|cookie| {
            let (name, value) = cookie.trim().split_once('=')?;
            (name == "binpoin_session").then_some(value.to_owned())
        }));
    let Some(token) = token else {
        return json_error(StatusCode::UNAUTHORIZED, "Silakan login terlebih dahulu.").into_response();
    };
    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    let client_ip=request.headers().get("x-forwarded-for").and_then(|v|v.to_str().ok()).and_then(|v|v.split(',').next()).map(str::trim).unwrap_or("").to_owned();
    let user = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT u.id, u.email FROM sessions s JOIN users u ON u.id = s.user_id WHERE s.token_hash = $1 AND s.expires_at > now() AND s.client_ip=$2"
    ).bind(token_hash).bind(client_ip).fetch_optional(pool).await;
    match user {
        Ok(Some((id, email))) => {
            request.extensions_mut().insert(AuthUser { id, email });
            next.run(request).await
        }
        Ok(None) => json_error(StatusCode::UNAUTHORIZED, "Sesi login sudah habis. Silakan login lagi.").into_response(),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Tidak dapat memeriksa sesi.").into_response(),
    }
}

async fn validate_host(
    State(allowed): State<Vec<String>>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    if allowed.is_empty() { return next.run(request).await; }
    let valid=request.headers().get(header::HOST).and_then(|h|h.to_str().ok()).map(|host| {
        let hostname=host.split(':').next().unwrap_or("").to_ascii_lowercase();
        allowed.iter().any(|candidate| candidate.eq_ignore_ascii_case(&hostname))
    }).unwrap_or(false);
    if valid { next.run(request).await } else { StatusCode::MISDIRECTED_REQUEST.into_response() }
}

#[derive(Deserialize)]
struct Credentials { email: String, password: String }

fn normalized_email(input: &str) -> Option<String> {
    let email = input.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || !domain.contains('.') || email.len() > 254 { return None; }
    Some(email)
}

fn login_rate_limited(st:&AppState,key:String)->bool {
    let current=now();
    let mut attempts=st.web_config.login_attempts.lock().unwrap();
    attempts.retain(|_,(_,since)| current.saturating_sub(*since)<900);
    attempts.get(&key).is_some_and(|(count,since)|*count>=10 && current.saturating_sub(*since)<900)
}

fn record_login_failure(st:&AppState,key:String) {
    let current=now();
    let mut attempts=st.web_config.login_attempts.lock().unwrap();
    let entry=attempts.entry(key).or_insert((0,current));
    if current.saturating_sub(entry.1)>=900 {*entry=(0,current);}
    entry.0=entry.0.saturating_add(1);
}

fn registration_rate_limited(st:&AppState,key:String)->bool {
    let current=now();
    let mut attempts=st.web_config.registration_attempts.lock().unwrap();
    attempts.retain(|_,(_,since)| current.saturating_sub(*since)<3600);
    if let Some((count,since))=attempts.get(&key) {
        if *count>=5 && current.saturating_sub(*since)<3600 { return true; }
    }
    let entry=attempts.entry(key).or_insert((0,current));
    if current.saturating_sub(entry.1)>=3600 {*entry=(0,current);}
    entry.0=entry.0.saturating_add(1);
    false
}

async fn verify_password_async(password:String,encoded_hash:String)->Result<bool,()> {
    tokio::task::spawn_blocking(move||{
        let parsed=PasswordHash::new(&encoded_hash).map_err(|_|())?;
        Ok::<bool,()>(Argon2::default().verify_password(password.as_bytes(),&parsed).is_ok())
    }).await.map_err(|_|())?
}

fn hash_password_sync(password:&str)->Result<String,argon2::password_hash::Error>{
    let salt=SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(password.as_bytes(),&salt).map(|hash|hash.to_string())
}

fn token_hash(token:&str)->String{hex::encode(Sha256::digest(token.as_bytes()))}

fn new_session_token()->String{
    use base64::{Engine as _,engine::general_purpose::URL_SAFE_NO_PAD};
    let mut token=[0u8;32];
    OsRng.fill_bytes(&mut token);
    URL_SAFE_NO_PAD.encode(token)
}

async fn dummy_password_verify(password:String) {
    const DUMMY:&str="$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$6d9NQZ53/8xv2zEyb+2aWs7BoGaqT3HuJfcJdPn5tCU";
    let _=verify_password_async(password,DUMMY.to_owned()).await;
}

async fn api_register(
    State(st): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(credentials): Json<Credentials>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let pool = postgres(&st)?;
    let client_ip=peer_ip(&headers,peer,&st).to_string();
    if registration_rate_limited(&st, client_ip.clone()) { return Err(json_error(StatusCode::TOO_MANY_REQUESTS,"Terlalu banyak permintaan pendaftaran.")); }
    if !valid_csrf_origin(&headers, &st) { return Err(json_error(StatusCode::FORBIDDEN, "Origin request tidak valid.")); }
    let email = normalized_email(&credentials.email).ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Format email tidak valid."))?;
    if !(12..=256).contains(&credentials.password.len()) {
        return Err(json_error(StatusCode::BAD_REQUEST, "Password harus berisi 12 sampai 256 karakter."));
    }
    let password_input=credentials.password;
    let password_hash=tokio::task::spawn_blocking(move||hash_password_sync(&password_input))
        .await.map_err(|_|json_error(StatusCode::INTERNAL_SERVER_ERROR,"Gagal membuat akun."))?
        .map_err(|_|json_error(StatusCode::INTERNAL_SERVER_ERROR,"Gagal membuat akun."))?;
    let user = sqlx::query_as::<_, (Uuid, String)>(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) RETURNING id, email"
    ).bind(Uuid::new_v4()).bind(email).bind(password_hash).fetch_one(pool).await;
    let (user_id, email) = match user {
        Ok(row) => row,
        Err(sqlx::Error::Database(error)) if error.constraint() == Some("users_email_key") =>
            return Err(json_error(StatusCode::CONFLICT, "Email sudah terdaftar.")),
        Err(_) => return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Gagal membuat akun.")),
    };
    create_login_session(pool,&st.web_config,user_id,email,client_ip).await
}

async fn api_login(
    State(st): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(credentials): Json<Credentials>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let pool = postgres(&st)?;
    if !valid_csrf_origin(&headers, &st) { return Err(json_error(StatusCode::FORBIDDEN, "Origin request tidak valid.")); }
    let client_ip=peer_ip(&headers,peer,&st).to_string();
    let email = normalized_email(&credentials.email).ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Format email tidak valid."))?;
    if login_rate_limited(&st, format!("{client_ip}:{email}")) { return Err(json_error(StatusCode::TOO_MANY_REQUESTS, "Terlalu banyak percobaan login untuk akun ini. Tunggu 15 menit.")); }
    let row = sqlx::query_as::<_, (Uuid, String, String)>("SELECT id, email, password_hash FROM users WHERE email = $1")
        .bind(&email).fetch_optional(pool).await.map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Tidak dapat login."))?;
    let Some((user_id, email, password_hash)) = row else {
        dummy_password_verify(credentials.password.clone()).await;
        record_login_failure(&st, format!("{client_ip}:{email}"));
        return Err(json_error(StatusCode::UNAUTHORIZED, "Email atau password salah."));
    };
    verify_password_async(credentials.password.clone(), password_hash).await
        .map_err(|_|json_error(StatusCode::INTERNAL_SERVER_ERROR,"Data akun tidak valid."))?
        .then_some(()).ok_or_else(||{record_login_failure(&st,format!("{client_ip}:{email}"));json_error(StatusCode::UNAUTHORIZED,"Email atau password salah.")})?;
    st.web_config.login_attempts.lock().unwrap().remove(&format!("{client_ip}:{email}"));
    create_login_session(pool, &st.web_config, user_id, email, client_ip).await
}

async fn api_login_page() -> Html<&'static str> { Html(include_str!("../static/login.html")) }

async fn api_register_page() -> Html<&'static str> { Html(include_str!("../static/register.html")) }

async fn create_login_session(
    pool: &PgPool,
    config: &WebConfig,
    user_id: Uuid,
    email: String,
    client_ip: String,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let token = new_session_token();
    let token_hash = token_hash(&token);
    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at, client_ip) VALUES ($1, $2, now() + ($3 * interval '1 second'), $4)")
        .bind(token_hash).bind(user_id).bind(config.session_ttl_seconds).bind(client_ip)
        .execute(pool).await.map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Gagal membuat sesi login."))?;
    let secure = if config.secure_cookies { "; Secure" } else { "" };
    let cookie = format!("binpoin_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}", config.session_ttl_seconds, secure);
    let mut response = Json(serde_json::json!({"user":{"id":user_id,"email":email}})).into_response();
    response.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(response)
}

fn valid_csrf_origin(headers:&HeaderMap,st:&AppState)->bool {
    if st.db.is_none() { return true; }
    let Some(origin)=headers.get(header::ORIGIN).and_then(|v|v.to_str().ok()) else { return false };
    let Ok(public)=std::env::var("PUBLIC_ORIGIN") else { return false };
    origin.trim_end_matches('/')==public.trim_end_matches('/')
}

fn peer_ip(headers:&HeaderMap, peer:SocketAddr, st:&AppState)->IpAddr {
    if st.trusted_proxy_ips.contains(&peer.ip()) {
        if let Some(ip)=headers.get("x-forwarded-for").and_then(|v|v.to_str().ok()).and_then(|v|v.split(',').next()).and_then(|v|v.trim().parse().ok()) { return ip; }
    }
    peer.ip()
}

async fn api_logout(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    if !valid_csrf_origin(&headers, &st) { return Err(json_error(StatusCode::FORBIDDEN, "Origin request tidak valid.")); }
    if let Some(pool) = &st.db {
        if let Some(token) = headers.get(header::COOKIE).and_then(|v|v.to_str().ok()).and_then(|v|v.split(';').find_map(|c| {let (k,v)=c.trim().split_once('=')?;(k=="binpoin_session").then_some(v)})) {
            let hash=hex::encode(Sha256::digest(token.as_bytes()));
            sqlx::query("DELETE FROM sessions WHERE token_hash=$1").bind(hash).execute(pool).await.map_err(|_|json_error(StatusCode::INTERNAL_SERVER_ERROR,"Gagal logout."))?;
        }
    }
    let secure=if st.web_config.secure_cookies {"; Secure"} else {""};
    let cookie=format!("binpoin_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure}");
    let mut response=Json(serde_json::json!({"ok":true})).into_response();
    response.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    Ok(response)
}

async fn api_me(State(st):State<AppState>,headers:HeaderMap) -> Result<Json<serde_json::Value>,StatusCode> {
    let Some(pool)=st.db.as_ref() else { return Ok(Json(serde_json::json!({"user":{"id":"desktop-local","email":"desktop@local"}}))) };
    let token=headers.get(header::COOKIE).and_then(|v|v.to_str().ok()).and_then(|cookies|cookies.split(';').find_map(|cookie|{let (name,value)=cookie.trim().split_once('=')?;(name=="binpoin_session").then_some(value.to_owned())})).ok_or(StatusCode::UNAUTHORIZED)?;
    let token_hash=token_hash(&token);
    let user=sqlx::query_as::<_,(Uuid,String)>("SELECT u.id,u.email FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now()")
        .bind(token_hash).fetch_optional(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(serde_json::json!({"user":{"id":user.0,"email":user.1}})))
}

async fn api_exam_history_detail(State(st):State<AppState>,Extension(user):Extension<AuthUser>,Path(exam_id):Path<String>) -> Result<Json<serde_json::Value>,StatusCode> {
    let Some(pool)=&st.db else {return Err(StatusCode::NOT_FOUND)};
    let id=Uuid::parse_str(&exam_id).map_err(|_|StatusCode::NOT_FOUND)?;
    let row=sqlx::query_as::<_,(i32,serde_json::Value)>("SELECT attempt_no,result FROM exam_attempts WHERE exam_id=$1 AND EXISTS(SELECT 1 FROM exams WHERE id=$1 AND user_id=$2) ORDER BY attempt_no DESC LIMIT 1")
        .bind(id).bind(user.id).fetch_optional(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::json!({"attempt":row.0,"result":row.1})))
}

#[derive(Deserialize)]
struct ProgressUpdate { material_id: String, kind: String, completed: bool }

async fn api_progress_get(State(st):State<AppState>,Extension(user):Extension<AuthUser>) -> Result<Json<serde_json::Value>,StatusCode> {
    if let Some(pool)=&st.db {
        let rows=sqlx::query_as::<_,(String,bool,bool)>("SELECT material_id,quiz_completed,material_completed FROM study_progress WHERE user_id=$1").bind(user.id).fetch_all(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        let quiz:serde_json::Map<String,serde_json::Value>=rows.iter().filter(|r|r.1).map(|r|(r.0.clone(),serde_json::json!(true))).collect();
        let materi:serde_json::Map<String,serde_json::Value>=rows.iter().filter(|r|r.2).map(|r|(r.0.clone(),serde_json::json!(true))).collect();
        return Ok(Json(serde_json::json!({"kuis":quiz,"materi":materi})));
    }
    Err(StatusCode::SERVICE_UNAVAILABLE)
}

async fn api_progress_put(State(st):State<AppState>,Extension(user):Extension<AuthUser>,headers:HeaderMap,Json(update):Json<ProgressUpdate>) -> Result<Json<serde_json::Value>,StatusCode> {
    if !valid_csrf_origin(&headers,&st) {return Err(StatusCode::FORBIDDEN)}
    if update.material_id.is_empty() || update.material_id.len()>64 {return Err(StatusCode::BAD_REQUEST)}
    let Some(pool)=&st.db else {return Err(StatusCode::SERVICE_UNAVAILABLE)};
    if !matches!(update.kind.as_str(), "materi" | "kuis") { return Err(StatusCode::BAD_REQUEST); }
    let column=if update.kind=="kuis" { "quiz_completed" } else { "material_completed" };
    let query=format!("INSERT INTO study_progress(user_id,material_id,{column}) VALUES($1,$2,$3) ON CONFLICT(user_id,material_id) DO UPDATE SET {column}=EXCLUDED.{column},updated_at=now()");
    sqlx::query(&query).bind(user.id).bind(&update.material_id).bind(update.completed).execute(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    api_progress_get(State(st), Extension(user)).await
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
    Extension(user): Extension<AuthUser>,
    axum::extract::Query(q): axum::extract::Query<StartQuery>,
) -> Response {
    let mode = q.mode.unwrap_or_else(|| "SKD".into());
    let mode = mode.to_uppercase();
    let now_ts = now();
    if let Some(pool)=&st.db {
        let active=sqlx::query_scalar::<_,Uuid>("SELECT id FROM exams WHERE user_id=$1 AND finished=false AND deadline>extract(epoch from now())::bigint ORDER BY created_at DESC LIMIT 1")
            .bind(user.id).fetch_optional(pool).await;
        match active {
            Ok(Some(active_id))=>match load_user_exam(&st,user.id,&active_id.to_string()).await {
                Ok(session)=>return (StatusCode::CONFLICT,Json(serde_json::json!({"error":"ada_sesi_aktif","sesi":to_info(&session)}))).into_response(),
                Err(_)=>return json_error(StatusCode::INTERNAL_SERVER_ERROR,"Tidak dapat memuat sesi aktif.").into_response(),
            },
            Ok(None)=>{},
            Err(_)=>return json_error(StatusCode::INTERNAL_SERVER_ERROR,"Tidak dapat memeriksa sesi aktif.").into_response(),
        }
    } else {
        let mut map = st.ujian.lock().unwrap();
        map.retain(|_, s| s.selesai == false || now_ts - s.deadline < 86400);
        if let Some(aktif) = map.get(&format!("{}:{}", user.id, "active")) {
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
    let id = if st.db.is_some() { Uuid::new_v4().to_string() } else { format!("U{}", now_ts % 1_000_000) };
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
        attempt_no: 1,
    };
    if let Some(pool) = &st.db {
        let exam_id = Uuid::parse_str(&id).unwrap();
        let questions_json = serde_json::to_value(&sesi.soal).unwrap_or_default();
        let inserted = sqlx::query("INSERT INTO exams(id,user_id,mode,started_at,deadline,finished,questions,answers,unsure) VALUES($1,$2,$3,$4,$5,false,$6,'{}'::jsonb,'[]'::jsonb)")
            .bind(exam_id).bind(user.id).bind(&sesi.mode).bind(i64::try_from(sesi.mulai).unwrap_or_default()).bind(i64::try_from(sesi.deadline).unwrap_or_default()).bind(questions_json).execute(pool).await;
        if let Err(error)=inserted {
            if let sqlx::Error::Database(db_error)=&error {
                if db_error.constraint()==Some("exams_one_active_per_user_idx") {
                    if let Ok(Some(active_id))=sqlx::query_scalar::<_,Uuid>("SELECT id FROM exams WHERE user_id=$1 AND finished=false AND deadline>extract(epoch from now())::bigint ORDER BY created_at DESC LIMIT 1").bind(user.id).fetch_optional(pool).await {
                        if let Ok(active_session)=load_user_exam(&st,user.id,&active_id.to_string()).await {
                            return (StatusCode::CONFLICT,Json(serde_json::json!({"error":"ada_sesi_aktif","sesi":to_info(&active_session)}))).into_response();
                        }
                    }
                }
            }
            return json_error(StatusCode::INTERNAL_SERVER_ERROR, "Tidak dapat menyimpan sesi ujian.").into_response();
        }
        return Json(serde_json::json!({"sesi": to_info(&sesi)})).into_response();
    }
    let info = to_info(&sesi);
    st.ujian.lock().unwrap().insert(format!("{}:{}", user.id, "active"), sesi);
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

async fn api_ujian_aktif(State(st): State<AppState>, Extension(user): Extension<AuthUser>) -> Response {
    if let Some(pool)=&st.db {
        let row=sqlx::query_as::<_,(Uuid,String,i64,i64,bool,serde_json::Value,serde_json::Value,serde_json::Value,Option<serde_json::Value>)>("SELECT id,mode,started_at,deadline,finished,questions,answers,unsure,result FROM exams WHERE user_id=$1 AND finished=false AND deadline>extract(epoch from now())::bigint ORDER BY created_at DESC LIMIT 1")
            .bind(user.id).fetch_optional(pool).await;
        let row=match row {Ok(v)=>v,Err(_)=>return json_error(StatusCode::INTERNAL_SERVER_ERROR,"Tidak dapat memuat sesi ujian.").into_response()};
        let Some((id,mode,mulai,deadline,selesai,questions,answers,unsure,_result))=row else {return Json(serde_json::json!({"ada":false})).into_response()};
        let Ok(soal):Result<Vec<Soal>,_>=serde_json::from_value(questions) else {return json_error(StatusCode::INTERNAL_SERVER_ERROR,"Data sesi ujian tidak valid.").into_response()};
        let jawaban_map:HashMap<String,u8>=serde_json::from_value(answers).unwrap_or_default();
        let jawaban=jawaban_map.into_iter().filter_map(|(k,v)|k.parse::<usize>().ok().map(|k|(k,v))).collect();
        let ragu_vec:Vec<usize>=serde_json::from_value(unsure).unwrap_or_default();
        let session=UjianSession{id:id.to_string(),mode,mulai:mulai.max(0) as u64,deadline:deadline.max(0) as u64,selesai,soal,jawaban,ragu:ragu_vec.into_iter().collect(),hasil:None,attempt_no:1};
        return Json(serde_json::json!({"ada":true,"sesi":to_info(&session)})).into_response();
    }
    let map=st.ujian.lock().unwrap();
    if let Some(s)=map.get(&format!("{}:active",user.id)).filter(|s|!s.selesai){Json(serde_json::json!({"ada":true,"sesi":to_info(s)})).into_response()}else{Json(serde_json::json!({"ada":false})).into_response()}
}

#[derive(Deserialize)]
struct JawabBody {
    no: usize,
    pilihan: Option<u8>,
}

async fn api_ujian_jawab(
    State(st): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(b): Json<JawabBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut s=load_user_exam(&st,user.id,&id).await?;
    let expected_attempt=s.attempt_no;
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
    persist_exam_update(&st,user.id,&s,expected_attempt).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct RaguBody {
    no: usize,
    ragu: bool,
}

async fn api_ujian_ragu(
    State(st): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(b): Json<RaguBody>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut s=load_user_exam(&st,user.id,&id).await?;
    let expected_attempt=s.attempt_no;
    if s.selesai {
        return Err(StatusCode::CONFLICT);
    }
    if b.ragu {
        s.ragu.insert(b.no);
    } else {
        s.ragu.remove(&b.no);
    }
    persist_exam_update(&st,user.id,&s,expected_attempt).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn api_ujian_kumpul(
    State(st): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<HasilUjian>, StatusCode> {
    if let Some(pool)=&st.db {
        let exam_id=Uuid::parse_str(&id).map_err(|_|StatusCode::NOT_FOUND)?;
        let mut tx=pool.begin().await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        let row=sqlx::query_as::<_,(String,i64,i64,bool,serde_json::Value,serde_json::Value,serde_json::Value,Option<serde_json::Value>,i32)>("SELECT mode,started_at,deadline,finished,questions,answers,unsure,result,attempt_no FROM exams WHERE id=$1 AND user_id=$2 AND deadline>extract(epoch from now())::bigint FOR UPDATE")
            .bind(exam_id).bind(user.id).fetch_optional(&mut *tx).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
        let questions:Vec<Soal>=serde_json::from_value(row.4).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        let answer_map:HashMap<String,u8>=serde_json::from_value(row.5).unwrap_or_default();
        let answers=answer_map.into_iter().filter_map(|(k,v)|k.parse().ok().map(|k|(k,v))).collect();
        let unsure:Vec<usize>=serde_json::from_value(row.6).unwrap_or_default();
        let mut session=UjianSession{id:id.clone(),mode:row.0,mulai:row.1.max(0) as u64,deadline:row.2.max(0) as u64,selesai:row.3,soal:questions,jawaban:answers,ragu:unsure.into_iter().collect(),hasil:row.7.map(serde_json::from_value).transpose().map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?,attempt_no:row.8.max(1)};
        if session.selesai {
            let result=session.hasil.clone().ok_or(StatusCode::NOT_FOUND)?;
            tx.commit().await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(result));
        }
        let mut result=nilai_ujian(&session);
        result.durasi_pakai=now().saturating_sub(session.mulai);
        session.selesai=true;
        session.hasil=Some(result.clone());
        sqlx::query("UPDATE exams SET finished=true,result=$3 WHERE id=$1 AND user_id=$2 AND attempt_no=$4 AND finished=false")
            .bind(exam_id).bind(user.id).bind(serde_json::to_value(&result).unwrap_or_default()).bind(session.attempt_no).execute(&mut *tx).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query("INSERT INTO exam_attempts(exam_id,attempt_no,result) VALUES($1,$2,$3)")
            .bind(exam_id).bind(session.attempt_no).bind(serde_json::to_value(&result).unwrap_or_default()).execute(&mut *tx).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        tx.commit().await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(result));
    }
    let mut session=load_user_exam(&st,user.id,&id).await?;
    if session.selesai { return session.hasil.clone().map(Json).ok_or(StatusCode::NOT_FOUND); }
    let mut hasil=nilai_ujian(&session);
    hasil.durasi_pakai=now().saturating_sub(session.mulai);
    session.selesai=true;
    session.hasil=Some(hasil.clone());
    persist_user_exam(&st,user.id,&session).await?;
    if let Some(pool)=&st.db {
        sqlx::query("INSERT INTO exam_history (user_id, exam_id, result) VALUES ($1,$2,$3) ON CONFLICT (exam_id) DO NOTHING")
            .bind(user.id).bind(Uuid::parse_str(&id).map_err(|_|StatusCode::NOT_FOUND)?).bind(serde_json::to_value(&hasil).unwrap_or_default()).execute(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        let mut rw=st.legacy_riwayat.lock().unwrap(); rw.insert(0,hasil.clone()); rw.truncate(100);
        let _=fs::write(&st.legacy_riwayat_path,serde_json::to_string_pretty(&*rw).unwrap_or_default());
    }
    Ok(Json(hasil))
}

async fn api_ujian_batalkan(
    State(st): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let s=load_user_exam(&st,user.id,&id).await?;
    persist_exam_update(&st,user.id,&UjianSession{selesai:true,..s.clone()},s.attempt_no).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn api_riwayat(State(st): State<AppState>,Extension(user):Extension<AuthUser>) -> Result<Json<Vec<HasilUjian>>,StatusCode> {
    if let Some(pool)=&st.db {
        let rows=sqlx::query_scalar::<_,serde_json::Value>("SELECT result FROM exam_history WHERE user_id=$1 ORDER BY created_at DESC LIMIT 100").bind(user.id).fetch_all(pool).await.map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)?;
        let history=rows.into_iter().filter_map(|row|serde_json::from_value(row).ok()).collect();
        return Ok(Json(history));
    }
    Ok(Json(st.legacy_riwayat.lock().unwrap().clone()))
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

async fn app_state_from_env() -> Result<AppState, String> {
    let soal=muat_soal(); let materi=muat_materi(); let video=muat_video();
    let database_url=std::env::var("DATABASE_URL").ok();
    let db=if let Some(url)=database_url {
        let pool=PgPoolOptions::new().max_connections(20).connect(&url).await.map_err(|e|format!("database connection failed: {e}"))?;
        let migration_path=std::env::var("MIGRATIONS_DIR").unwrap_or_else(|_|"migrations".into());
        sqlx::migrate::Migrator::new(FsPath::new(&migration_path)).await.map_err(|e|format!("database migration discovery failed: {e}"))?.run(&pool).await.map_err(|e|format!("database migration failed: {e}"))?;
        sync_global_content(&pool,&soal,&materi,&video).await?;
        Some(pool)
    } else { None };
    let legacy_riwayat_path=data_dir().join("riwayat.json");
    let legacy_riwayat=if db.is_none() {
        if let Some(parent)=legacy_riwayat_path.parent(){fs::create_dir_all(parent).map_err(|e|format!("gagal membuat data dir: {e}"))?;}
        muat_riwayat(&legacy_riwayat_path)
    } else { Vec::new() };
    let web_mode=db.is_some();
    let cors_origins=std::env::var("CORS_ORIGINS").unwrap_or_default().split(',').filter_map(|origin|HeaderValue::from_str(origin.trim()).ok()).collect();
    let trusted_proxy_ips=std::env::var("TRUSTED_PROXY_IPS").unwrap_or_else(|_|"127.0.0.1,::1".to_owned()).split(',').filter_map(|ip|ip.trim().parse::<IpAddr>().ok()).collect();
    let allowed_hosts=std::env::var("ALLOWED_HOSTS").unwrap_or_default().split(',').map(str::trim).filter(|host|!host.is_empty()).map(str::to_owned).collect();
    Ok(AppState{
        soal,materi,video,ujian:Arc::new(Mutex::new(HashMap::new())),db,
        web_config:WebConfig{secure_cookies:std::env::var("COOKIE_SECURE").map(|v|v!="false").unwrap_or(web_mode),session_ttl_seconds:30*24*60*60,login_attempts:Arc::new(Mutex::new(HashMap::new())),registration_attempts:Arc::new(Mutex::new(HashMap::new()))},
        legacy_riwayat_path,legacy_riwayat:Arc::new(Mutex::new(legacy_riwayat)),
        cors_origins,
        trusted_proxy_ips,
        allowed_hosts,
    })
}

fn app_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CPNS_APP_DIR") {
        return PathBuf::from(dir);
    }
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn data_file(name: &str) -> PathBuf {
    let user = data_dir().join(name);
    if user.exists() {
        user
    } else {
        app_dir().join("data").join(name)
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

fn content_fingerprint(questions:&[Soal],lessons:&[Materi],video:&serde_json::Value)->String {
    let mut hasher=Sha256::new();
    hasher.update(serde_json::to_vec(questions).unwrap_or_default());
    hasher.update(serde_json::to_vec(lessons).unwrap_or_default());
    hasher.update(serde_json::to_vec(video).unwrap_or_default());
    hex::encode(hasher.finalize())
}

async fn sync_global_content(pool:&PgPool,questions:&[Soal],lessons:&[Materi],video:&serde_json::Value)->Result<(),String> {
    let fingerprint=content_fingerprint(questions,lessons,video);
    let existing=sqlx::query_scalar::<_,String>("SELECT value FROM app_metadata WHERE key='content_fingerprint'").fetch_optional(pool).await.map_err(|e|format!("cannot read content fingerprint: {e}"))?;
    if existing.as_deref()==Some(fingerprint.as_str()) {return Ok(())}
    let mut tx=pool.begin().await.map_err(|e|format!("cannot begin content sync: {e}"))?;
    sqlx::query("INSERT INTO global_content(id,kind,body) VALUES($1,'questions',$2) ON CONFLICT(id) DO UPDATE SET body=EXCLUDED.body,updated_at=now()")
        .bind("questions").bind(serde_json::to_value(questions).map_err(|e|e.to_string())?).execute(&mut *tx).await.map_err(|e|format!("cannot sync questions: {e}"))?;
    sqlx::query("INSERT INTO global_content(id,kind,body) VALUES($1,'materials',$2) ON CONFLICT(id) DO UPDATE SET body=EXCLUDED.body,updated_at=now()")
        .bind("materials").bind(serde_json::to_value(lessons).map_err(|e|e.to_string())?).execute(&mut *tx).await.map_err(|e|format!("cannot sync lessons: {e}"))?;
    sqlx::query("INSERT INTO global_content(id,kind,body) VALUES($1,'video',$2) ON CONFLICT(id) DO UPDATE SET body=EXCLUDED.body,updated_at=now()")
        .bind("video").bind(video).execute(&mut *tx).await.map_err(|e|format!("cannot sync videos: {e}"))?;
    sqlx::query("INSERT INTO app_metadata(key,value) VALUES('content_fingerprint',$1) ON CONFLICT(key) DO UPDATE SET value=EXCLUDED.value,updated_at=now()")
        .bind(fingerprint).execute(&mut *tx).await.map_err(|e|format!("cannot update content fingerprint: {e}"))?;
    tx.commit().await.map_err(|e|format!("cannot commit global content sync: {e}"))?;
    Ok(())
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
    let state=match app_state_from_env().await {Ok(s)=>s,Err(e)=>{eprintln!("[fatal] {e}");std::process::exit(1)}};
    println!(
        "[cpns-simulasi] bank: {} soal | materi: {} | riwayat: {}",
        state.soal.len(), state.materi.len(), state.legacy_riwayat.lock().unwrap().len()
    );
    let public_data = Router::new()
        .route("/api/meta", get(api_meta))
        .route("/api/materi", get(api_materi))
        .route("/api/materi/{id}", get(api_materi_satu))
        .route("/api/video", get(api_video))
        .route("/api/bank", get(api_bank));
    let protected = Router::new()
        .route("/api/progress", get(api_progress_get).put(api_progress_put))
        .route("/api/latihan/start", post(api_latihan_start))
        .route("/api/ujian/start", post(api_ujian_start))
        .route("/api/ujian/aktif", get(api_ujian_aktif))
        .route("/api/ujian/{id}/jawab", post(api_ujian_jawab))
        .route("/api/ujian/{id}/ragu", post(api_ujian_ragu))
        .route("/api/ujian/{id}/kumpul", post(api_ujian_kumpul))
        .route("/api/ujian/{id}/batalkan", post(api_ujian_batalkan))
        .route("/api/riwayat/{exam_id}", get(api_exam_history_detail))
        .route("/api/riwayat", get(api_riwayat))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_user));
    let database_enabled = state.db.is_some();
    let cors_origins = state.cors_origins.clone();
    let allowed_hosts=state.allowed_hosts.clone();
    let mut app = Router::new()
        .route("/", get(hal_utama))
        .route("/login", get(api_login_page))
        .route("/register", get(api_register_page))
        .route("/api/auth/me", get(api_me))
        .route("/api/auth/register", post(api_register))
        .route("/api/auth/login", post(api_login))
        .route("/api/auth/logout", post(api_logout))
        .merge(public_data)
        .merge(protected)
        .fallback_service(ServeDir::new(app_dir().join("static")))
        .with_state(state);
    let app_base_path = std::env::var("APP_BASE_PATH").unwrap_or_default();
    let mut app = if app_base_path.trim_end_matches('/') == "/app" {
        Router::new()
            .route(
                "/app/",
                get(hal_utama),
            )
            .nest("/app", app)
    } else {
        app
    };
    if database_enabled {
        app=app.layer(tower_http::limit::RequestBodyLimitLayer::new(128*1024));
        if !allowed_hosts.is_empty() {
            app=app.layer(middleware::from_fn_with_state(allowed_hosts.clone(),validate_host));
        }
    }
    let app = if database_enabled {
        app.layer(tower_http::cors::CorsLayer::new().allow_origin(cors_origins).allow_methods([axum::http::Method::GET,axum::http::Method::POST,axum::http::Method::PUT]).allow_headers([header::CONTENT_TYPE, header::COOKIE]).allow_credentials(true))
    } else { app };

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
    if std::env::var_os("CPNS_DESKTOP_SHELL").is_none() {
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
    }
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.expect("server berhenti");
}

async fn hal_utama() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

#[cfg(test)]
mod tests {
    use super::{Kat, Soal, app_dir, data_file, pilih_soal_latihan, skor_soal};
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
    fn aset_runtime_diambil_dari_direktori_executable() {
        let executable_dir = app_dir();
        assert!(executable_dir.is_absolute());
        assert_eq!(
            data_file("questions.json"),
            executable_dir.join("data/questions.json")
        );
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

    #[test]
    fn subkategori_tiu_mencocokkan_konten_soal() {
        let bank: Vec<Soal> = serde_json::from_str(include_str!("../data/questions.json"))
            .expect("bank soal harus JSON valid");
        let by_id = |id| bank.iter().find(|question| question.id == id).unwrap();

        for id in [484, 6890, 6891, 7111, 7508, 7606, 7707] {
            assert_eq!(by_id(id).kat, Kat::Tiu, "soal {id} harus TIU");
            assert_eq!(by_id(id).sub, "Numerik", "soal {id} harus Numerik");
        }
        for id in [2087, 2090, 2108, 2112] {
            assert_eq!(by_id(id).kat, Kat::Tiu, "soal {id} harus TIU");
            assert_eq!(by_id(id).sub, "Analitis", "soal {id} harus Analitis");
        }
        for id in [642, 646, 659, 672] {
            assert_eq!(by_id(id).kat, Kat::Tkp, "soal {id} harus TKP");
            assert!(by_id(id).skor_pilihan.is_some(), "soal {id} harus mempertahankan skor TKP");
        }
        assert_eq!(by_id(521).sub, "Deret Angka");
        assert_eq!(by_id(5683).sub, "Analogi");
    }

    #[test]
    fn subkategori_materi_tiu_mengambil_bank_soal_yang_sesuai() {
        let bank: Vec<Soal> = serde_json::from_str(include_str!("../data/questions.json"))
            .expect("bank soal harus JSON valid");
        let materi: Vec<super::Materi> = serde_json::from_str(include_str!("../data/materi.json"))
            .expect("materi harus JSON valid");

        for lesson in materi.iter().filter(|lesson| {
            lesson.kat == Kat::Tiu && ["Numerik", "Deret Angka", "Analogi", "Analitis", "Logika"].contains(&lesson.sub.as_str())
        }) {
            let matching = bank.iter().filter(|question| question.kat == lesson.kat && question.sub == lesson.sub).count();
            assert!(matching > 0, "Materi {} tidak memiliki soal kuis {}/{}", lesson.id, lesson.kat.as_str(), lesson.sub);
        }

        let numerical = bank.iter().filter(|question| question.kat == Kat::Tiu && question.sub == "Numerik").count();
        assert!(numerical >= 50, "Bank TIU Numerik masih terlalu kecil setelah koreksi tag: {numerical}");
        let mislabeled_tkp = bank.iter().filter(|question| question.skor_pilihan.is_some() && question.kat != Kat::Tkp && question.kat != Kat::Campur).count();
        assert_eq!(mislabeled_tkp, 0, "Soal berskor TKP kategori mandiri harus ditandai TKP");

        for question in bank.iter().filter(|question| question.kat == Kat::Tiu && question.sub == "Numerik") {
            assert!(
                !question.teks.starts_with("Premis Umum") && !question.teks.starts_with("PU :") && !question.teks.starts_with("PU:"),
                "Soal TIU Numerik {} masih berupa premis/logika", question.id
            );
        }
    }
}
