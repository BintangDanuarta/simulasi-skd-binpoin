// ===== Simulasi SKD CPNS — frontend =====
'use strict';

const $ = (s) => document.querySelector(s);
const APP_BASE = location.pathname === '/app' || location.pathname.startsWith('/app/') ? '/app' : '';
const appUrl = (path) => `${APP_BASE}${path}`;

let META = null;
let CURRENT_USER = null;
let ACCOUNT_PROGRESS = null;
let curView = 'home';
let ujian = null;      // {id, mode, mulai, deadline, soal[], jawaban{}, ragu[]}
let curIdx = 0;
let timerInt = null;
let latihanFilter = { kat: 'TWK', sub: null, offset: 0 };
let latihanState = { kat: 'TWK', subs: [], jumlah: 10, soal: [], jawaban: {}, idx: 0, finished: false };
let warningTab = false;

// ---------- helpers ----------
async function api(url, opt = {}) {
  const r = await fetch(appUrl(url), {
    headers: { 'Content-Type': 'application/json' },
    credentials: 'same-origin',
    ...opt,
  });
  if (!r.ok) {
    let msg = 'Terjadi kesalahan server';
    try {
      const j = await r.json();
      if (j.error) msg = j.error;
    } catch (_) {}
    if (r.status === 401 && !url.startsWith('/api/auth/')) {
      CURRENT_USER = null;
      renderAuthGate();
    }
    const error = new Error(msg);
    error.status = r.status;
    throw error;
  }
  return r.json();
}
function post(url, body) {
  return api(url, { method: 'POST', body: JSON.stringify(body || {}) });
}

let authMode = 'login';
let appBooted = false;
let authChecking = true;
let authElementsReady = false;

function renderAuthGate(message = '') {
  const gate = $('#auth-gate');
  if (!gate) return;
  gate.classList.toggle('hidden', Boolean(CURRENT_USER));
  $('#topbar').classList.toggle('hidden', !CURRENT_USER);
  if (CURRENT_USER) return;
  if (!authElementsReady) return;
  $('#auth-title').textContent = authMode === 'login' ? 'Masuk ke akun' : 'Buat akun';
  $('#auth-copy').textContent = authMode === 'login'
    ? 'Simpan progres dan riwayat ujian di akunmu.'
    : 'Daftar untuk menyimpan sesi dan riwayat di semua perangkat.';
  $('#auth-password').autocomplete = authMode === 'login' ? 'current-password' : 'new-password';
  $('#auth-submit').textContent = authMode === 'login' ? 'Masuk' : 'Buat akun';
  $('#auth-toggle').textContent = authMode === 'login' ? 'Belum punya akun? Daftar' : 'Sudah punya akun? Masuk';
  const error = $('#auth-error');
  error.textContent = message;
  error.classList.toggle('hidden', !message);
}

async function initializeAccountApp() {
  try {
    const response = await api('/api/auth/me');
    CURRENT_USER = response.user;
    if (!appBooted) {
      renderAuthGate();
      await syncProgressFromServer();
    }
    $('#account-email').textContent = CURRENT_USER.email;
    if (!appBooted) {
      appBooted = true;
      nav('home');
    }
  } catch (error) {
    CURRENT_USER = null;
    appBooted = false;
    renderAuthGate();
    if (error.status !== 401) {
      $('#auth-error').textContent = error.message || 'Layanan akun belum tersedia. Coba muat ulang.';
      $('#auth-error').classList.remove('hidden');
    }
  } finally {
    authChecking = false;
    document.body.classList.remove('auth-checking');
  }
}

async function submitAuth(event) {
  event.preventDefault();
  const email = $('#auth-email').value.trim();
  const password = $('#auth-password').value;
  $('#auth-submit').disabled = true;
  try {
    const endpoint = authMode === 'login' ? '/api/auth/login' : '/api/auth/register';
    await post(endpoint, { email, password });
    $('#auth-password').value = '';
    authChecking = true;
    document.body.classList.add('auth-checking');
    await initializeAccountApp();
  } catch (error) {
    renderAuthGate(error.message || 'Tidak dapat memproses akun.');
  } finally {
    $('#auth-submit').disabled = false;
  }
}

async function logoutAccount() {
  try { await api('/api/auth/logout', { method: 'POST' }); } catch (_) {}
  materiCache = [];
  latihanState = { kat: 'TWK', subs: [], jumlah: 10, soal: [], jawaban: {}, idx: 0, finished: false };
  ujian = null;
  CURRENT_USER = null;
  META = null;
  ACCOUNT_PROGRESS = null;
  appBooted = false;
  renderAuthGate();
}
async function syncProgressFromServer() {
  try { ACCOUNT_PROGRESS = await api('/api/progress'); }
  catch (_) { ACCOUNT_PROGRESS = null; }
}
function progressMap(kind) {
  if (ACCOUNT_PROGRESS) return ACCOUNT_PROGRESS[kind] || {};
  try { return JSON.parse(localStorage.getItem(kind === 'kuis' ? 'kuisDone' : 'materiDone') || '{}'); }
  catch (_) { return {}; }
}
async function markProgress(kind, id) {
  const map = { ...progressMap(kind), [id]: 1 };
  if (ACCOUNT_PROGRESS) {
    ACCOUNT_PROGRESS[kind] = map;
    try {
      const response = await api('/api/progress', { method: 'PUT', body: JSON.stringify({ material_id: id, kind, completed: true }) });
      ACCOUNT_PROGRESS = response;
    } catch (error) { console.warn('Progres gagal tersimpan ke akun:', error); }
  } else {
    localStorage.setItem(kind === 'kuis' ? 'kuisDone' : 'materiDone', JSON.stringify(map));
  }
}
async function setMateriCompleted(id, completed) {
  if (!ACCOUNT_PROGRESS) {
    const map = { ...progressMap('materi') };
    if (completed) map[id] = 1; else delete map[id];
    localStorage.setItem('materiDone', JSON.stringify(map));
    return;
  }
  try {
    ACCOUNT_PROGRESS.materi = { ...ACCOUNT_PROGRESS.materi };
    if (completed) ACCOUNT_PROGRESS.materi[id] = 1; else delete ACCOUNT_PROGRESS.materi[id];
    ACCOUNT_PROGRESS = await api('/api/progress', { method: 'PUT', body: JSON.stringify({ material_id: id, kind: 'materi', completed }) });
  } catch (error) {
    console.warn('Perubahan progres gagal disimpan:', error);
  }
}
async function saveAccountProgress(kind, id) {
  if (!ACCOUNT_PROGRESS) return;
  try {
    ACCOUNT_PROGRESS[kind] = { ...ACCOUNT_PROGRESS[kind], [id]: 1 };
    ACCOUNT_PROGRESS = await api('/api/progress', { method: 'PUT', body: JSON.stringify({ material_id: id, kind, completed: true }) });
  } catch (error) {
    console.warn('Progres kuis gagal disimpan:', error);
  }
}
function esc(s) {
  return String(s ?? '').replace(/[&<>"']/g, (c) => ({
    '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
  }[c]));
}
function fmtWaktu(t) {
  if (t <= 0) return '00:00';
  const h = Math.floor(t / 3600), m = Math.floor((t % 3600) / 60), s = t % 60;
  return h > 0
    ? `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
    : `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

// ---------- navigasi ----------
function showLoadError(targetId, title, error) {
  const target = document.getElementById(targetId);
  if (!target) return;
  target.innerHTML = `<div class="empty-state" role="alert"><b>${esc(title)}</b><p>${esc(error?.message || 'Periksa koneksi ke server lokal lalu coba lagi.')}</p><button class="btn btn-primary" onclick="nav('${curView}')">Coba muat ulang</button></div>`;
}

function nav(view) {
  curView = view;
  document.querySelectorAll('main > section').forEach((s) => s.classList.add('hidden'));
  $('#view-' + view).classList.remove('hidden');
  $('#topbar').classList.toggle('hidden', view === 'ujian');
  document.body.classList.toggle('ujian-aktif', view === 'ujian');
  document.querySelectorAll('#mainnav button').forEach((b) => {
    const active = b.dataset.nav === view || (view === 'hasil' && b.dataset.nav === 'riwayat');
    b.classList.toggle('active', active);
    if (active) b.setAttribute('aria-current', 'page');
    else b.removeAttribute('aria-current');
  });
  if (view !== 'ujian') window.scrollTo({ top: 0, behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth' });
  if (view === 'home') loadHome().catch((e) => showLoadError('hero-stats', 'Beranda belum dapat dimuat.', e));
  if (view === 'latihan') loadLatihanSetup().catch((e) => showLoadError('latihan-setup', 'Latihan soal belum dapat dimuat.', e));
  if (view === 'materi') loadMateri().catch((e) => showLoadError('materi-list', 'Materi belum dapat dimuat.', e));
  if (view === 'riwayat') loadRiwayat().catch((e) => showLoadError('riwayat-list', 'Riwayat belum dapat dimuat.', e));
}

// ---------- latihan soal mandiri ----------
const LATIHAN_KATEGORI = ['TWK', 'TIU', 'TKP'];

function subsLatihan(kat = latihanState.kat) {
  return (META?.subs?.[kat] || []).map(([sub, jumlah]) => ({ sub, jumlah }));
}

async function loadLatihanSetup() {
  if (!META) META = await api('/api/meta');
  const kategori = $('#latihan-kategori');
  kategori.value = latihanState.kat;
  kategori.onchange = () => {
    latihanState.kat = kategori.value;
    latihanState.subs = subsLatihan().map((item) => item.sub);
    renderLatihanSetup();
  };
  if (!latihanState.subs.length) latihanState.subs = subsLatihan().map((item) => item.sub);
  renderLatihanSetup();
}

function renderLatihanSetup() {
  const availableSubs = subsLatihan();
  latihanState.subs = latihanState.subs.filter((sub) => availableSubs.some((item) => item.sub === sub));
  if (!latihanState.subs.length && availableSubs.length) latihanState.subs = availableSubs.map((item) => item.sub);

  const selectedCount = availableSubs
    .filter((item) => latihanState.subs.includes(item.sub))
    .reduce((sum, item) => sum + item.jumlah, 0);
  const chips = $('#latihan-submateri');
  const hint = $('#latihan-count-hint');
  const amount = $('#latihan-jumlah');
  const start = $('#latihan-mulai');
  const allSelected = availableSubs.length > 0 && latihanState.subs.length === availableSubs.length;

  $('#latihan-toggle-all').textContent = allSelected ? 'Hapus pilihan' : 'Pilih semua';
  chips.innerHTML = availableSubs.length
    ? availableSubs.map((item, index) => {
      const selected = latihanState.subs.includes(item.sub);
      return `<button type="button" class="latihan-subchip ${selected ? 'selected' : ''}" data-sub-index="${index}" aria-pressed="${selected}">
        <span class="latihan-check" aria-hidden="true">${selected ? '✓' : '+'}</span>
        <span>${esc(item.sub)}</span><small>${item.jumlah} soal</small>
      </button>`;
    }).join('')
    : '<div class="kuis-empty">Belum ada submateri untuk kategori ini.</div>';
  chips.querySelectorAll('[data-sub-index]').forEach((button) => {
    button.addEventListener('click', () => {
      const sub = availableSubs[Number(button.dataset.subIndex)]?.sub;
      if (!sub) return;
      latihanState.subs = latihanState.subs.includes(sub)
        ? latihanState.subs.filter((selected) => selected !== sub)
        : [...latihanState.subs, sub];
      renderLatihanSetup();
    });
  });

  hint.textContent = `${selectedCount} soal tersedia dari materi yang dipilih.`;
  amount.max = String(Math.max(1, Math.min(selectedCount, 100)));
  amount.value = String(Math.max(1, Math.min(Number(latihanState.jumlah) || 10, selectedCount || 1, 100)));
  latihanState.jumlah = Number(amount.value);
  amount.oninput = () => {
    latihanState.jumlah = Number(amount.value);
    clearLatihanError();
  };
  start.disabled = selectedCount === 0 || latihanState.subs.length === 0;
}

function toggleSemuaSubmateri() {
  const availableSubs = subsLatihan();
  const allSelected = availableSubs.length > 0 && latihanState.subs.length === availableSubs.length;
  latihanState.subs = allSelected ? [] : availableSubs.map((item) => item.sub);
  renderLatihanSetup();
}

function showLatihanError(message) {
  const error = $('#latihan-setup-error');
  error.textContent = message;
  error.classList.remove('hidden');
}

function clearLatihanError() {
  const error = $('#latihan-setup-error');
  error.textContent = '';
  error.classList.add('hidden');
}

async function mulaiLatihan() {
  clearLatihanError();
  const jumlah = Number($('#latihan-jumlah').value);
  if (!latihanState.subs.length) return showLatihanError('Pilih setidaknya satu materi.');
  if (!Number.isInteger(jumlah) || jumlah < 1) return showLatihanError('Masukkan jumlah soal minimal 1.');

  const button = $('#latihan-mulai');
  button.disabled = true;
  button.textContent = 'Menyiapkan soal…';
  try {
    const result = await post('/api/latihan/start', {
      kat: latihanState.kat,
      sub: latihanState.subs,
      jumlah,
    });
    latihanState = {
      kat: result.kat,
      subs: [...latihanState.subs],
      jumlah: result.jumlah,
      soal: result.soal || [],
      jawaban: {},
      idx: 0,
      finished: false,
      started: false,
    };
    $('#latihan-setup').classList.add('hidden');
    $('#latihan-hasil').classList.add('hidden');
    $('#latihan-sesi').classList.remove('hidden');
    renderLatihanQuestion();
  } catch (error) {
    showLatihanError(error.message || 'Soal latihan gagal dimuat.');
  } finally {
    button.disabled = false;
    button.textContent = 'Mulai latihan';
  }
}

function renderLatihanQuestion() {
  const state = latihanState;
  if (state.finished || !state.soal.length) return renderLatihanResults();
  const question = state.soal[state.idx];
  const shouldFocusMenu = !state.started;
  state.started = true;
  const selected = state.jawaban[state.idx];
  const answered = Object.keys(state.jawaban).length;
  const feedback = selected == null ? '' : `
    <div class="lat-feedback ${selected === question.kunci ? 'benar' : 'salah'}" role="status">
      ${selected === question.kunci ? '✓ Jawaban benar' : `✗ Jawaban benar: ${String.fromCharCode(65 + question.kunci)}`}
      ${question.pembahasan ? `<div class="pembahasan">${esc(question.pembahasan)}</div>` : ''}
    </div>`;
  $('#latihan-sesi').innerHTML = `
    <div class="latihan-toolbar">
      <button type="button" class="btn btn-outline btn-sm" data-action="menu">← Kembali ke pengaturan</button>
      <button type="button" class="latihan-exit-link" data-action="cancel">Keluar dari latihan</button>
    </div>
    <div id="latihan-cancel-confirm" class="latihan-cancel-confirm hidden" role="region" aria-label="Konfirmasi keluar latihan"></div>
    <div class="latihan-session-head">
      <div><span class="eyebrow">${esc(state.kat)} · ${esc(question.sub)}</span>
        <h3>Soal ${state.idx + 1} <span class="muted">/ ${state.soal.length}</span></h3></div>
      <span class="progress-text">${answered}/${state.soal.length} dijawab</span>
    </div>
    <div class="progress-bar latihan-progress" role="progressbar" aria-label="Progres latihan" aria-valuenow="${answered}" aria-valuemin="0" aria-valuemax="${state.soal.length}"><div class="progress-fill" style="width:${Math.round(answered / state.soal.length * 100)}%"></div></div>
    <article class="latihan-question card">
      <p class="latihan-question-text">${esc(question.teks)}</p>
      <div class="kuis-opsi">${question.pilihan.map((option, index) => {
        const revealed = selected != null;
        const cls = revealed && index === question.kunci ? 'benar' : (revealed && index === selected ? 'salah' : '');
        return `<button type="button" class="opsi ${selected === index ? 'selected' : ''} ${cls}" data-answer="${index}" ${revealed ? 'disabled' : ''}>
          <span class="letter">${String.fromCharCode(65 + index)}.</span><span>${esc(option)}</span>
        </button>`;
      }).join('')}</div>
      ${feedback}
    </article>
    <div class="latihan-session-actions">
      <button type="button" class="btn btn-outline" data-action="previous" ${state.idx === 0 ? 'disabled' : ''}>Sebelumnya</button>
      <button type="button" class="btn btn-primary" data-action="next" ${selected == null ? 'disabled' : ''}>${state.idx === state.soal.length - 1 ? 'Lihat hasil' : 'Soal berikutnya'}</button>
    </div>`;

  $('#latihan-sesi').querySelector('[data-action="menu"]').addEventListener('click', () => {
    showEditLatihanConfirmation();
  });
  $('#latihan-sesi').querySelector('[data-action="cancel"]').addEventListener('click', showCancelLatihanConfirmation);
  $('#latihan-sesi').querySelector('[data-action="previous"]').addEventListener('click', () => {
    if (state.idx > 0) {
      state.idx -= 1;
      renderLatihanQuestion();
    }
  });
  $('#latihan-sesi').querySelectorAll('[data-answer]').forEach((button) => {
    button.addEventListener('click', () => {
      state.jawaban[state.idx] = Number(button.dataset.answer);
      renderLatihanQuestion();
    });
  });
  $('#latihan-sesi').querySelector('[data-action="next"]').addEventListener('click', () => {
    if (state.jawaban[state.idx] == null) return;
    if (state.idx + 1 < state.soal.length) {
      state.idx += 1;
      renderLatihanQuestion();
    } else {
      state.finished = true;
      renderLatihanResults();
    }
  });
  if (shouldFocusMenu) $('#latihan-sesi').querySelector('[data-action="menu"]').focus();
}

function showEditLatihanConfirmation() {
  const confirmation = $('#latihan-cancel-confirm');
  confirmation.classList.remove('hidden');
  confirmation.innerHTML = `
    <div><b>Kembali ke pengaturan?</b><p>Jawaban sesi ini tidak disimpan jika pengaturan latihan diubah.</p></div>
    <div class="latihan-confirm-actions">
      <button type="button" class="btn btn-outline btn-sm" data-edit-stay>Lanjut mengerjakan</button>
      <button type="button" class="btn btn-primary btn-sm" data-edit-confirm>Ubah pengaturan</button>
    </div>`;
  confirmation.querySelector('[data-edit-stay]').addEventListener('click', () => confirmation.classList.add('hidden'));
  confirmation.querySelector('[data-edit-confirm]').addEventListener('click', () => {
    latihanState = { kat: latihanState.kat, subs: [...latihanState.subs], jumlah: latihanState.jumlah, soal: [], jawaban: {}, idx: 0, finished: false, started: false };
    confirmation.classList.add('hidden');
    $('#latihan-sesi').classList.add('hidden');
    $('#latihan-hasil').classList.add('hidden');
    $('#latihan-setup').classList.remove('hidden');
    renderLatihanSetup();
    window.scrollTo({ top: 0, behavior: 'smooth' });
    $('#latihan-kategori').focus();
  });
  confirmation.querySelector('[data-edit-stay]').focus();
}

function showCancelLatihanConfirmation() {
  const confirmation = $('#latihan-cancel-confirm');
  confirmation.classList.remove('hidden');
  confirmation.innerHTML = `
    <div><b>Akhiri latihan?</b><p>Progres sesi ini tidak disimpan.</p></div>
    <div class="latihan-confirm-actions">
      <button type="button" class="btn btn-outline btn-sm" data-cancel-stay>Lanjut mengerjakan</button>
      <button type="button" class="btn btn-danger-outline btn-sm" data-cancel-confirm>Keluar</button>
    </div>`;
  confirmation.querySelector('[data-cancel-stay]').addEventListener('click', () => confirmation.classList.add('hidden'));
  confirmation.querySelector('[data-cancel-confirm]').addEventListener('click', () => {
    latihanState = { kat: latihanState.kat, subs: [...latihanState.subs], jumlah: latihanState.jumlah, soal: [], jawaban: {}, idx: 0, finished: false, started: false };
    $('#latihan-cancel-confirm').classList.add('hidden');
    $('#latihan-sesi').classList.add('hidden');
    $('#latihan-hasil').classList.add('hidden');
    $('#latihan-setup').classList.remove('hidden');
    renderLatihanSetup();
    window.scrollTo({ top: 0, behavior: 'smooth' });
  });
  confirmation.querySelector('[data-cancel-stay]').focus();
}

function scoreLatihanQuestion(question, answer) {
  if (answer == null) return 0;
  if (Array.isArray(question.skor_pilihan) && question.skor_pilihan[answer] != null) {
    return Number(question.skor_pilihan[answer]);
  }
  if (answer === question.kunci) return 5;
  return question.kat === 'TKP' ? 1 : 0;
}

function renderLatihanResults() {
  const state = latihanState;
  const correct = state.soal.filter((question, index) => state.jawaban[index] === question.kunci).length;
  const score = state.soal.reduce((sum, question, index) => sum + scoreLatihanQuestion(question, state.jawaban[index]), 0);
  $('#latihan-sesi').classList.add('hidden');
  const result = $('#latihan-hasil');
  result.classList.remove('hidden');
  result.innerHTML = `
    <div class="latihan-result-head card">
      <button type="button" class="latihan-exit-link latihan-result-back" data-action="menu">← Kembali ke pengaturan</button>
      <p class="eyebrow">LATIHAN SELESAI · ${esc(state.kat)}</p>
      <h3>${correct} <span class="muted">/ ${state.soal.length} benar</span></h3>
      <p class="card-sub">Skor latihan: ${score} dari ${state.soal.length * 5}</p>
      <button type="button" class="btn btn-primary" data-action="again">Atur latihan baru</button>
    </div>
    <h3 class="latihan-review-title">Kunci dan pembahasan</h3>
    <div class="latihan-review-list">${state.soal.map((question, index) => {
      const answer = state.jawaban[index];
      const right = answer === question.kunci;
      return `<article class="latihan-review card">
        <div class="qhead"><span class="badge">${esc(question.kat)} · ${esc(question.sub)}</span><span class="badge ${right ? 'review-correct' : 'review-wrong'}">${right ? 'Benar' : 'Perlu ditinjau'}</span></div>
        <p class="latihan-question-text">${index + 1}. ${esc(question.teks)}</p>
        <ol class="latihan-review-options" type="A">${question.pilihan.map((option, optionIndex) => {
          const cls = optionIndex === question.kunci ? 'review-key' : (optionIndex === answer ? 'review-selected-wrong' : '');
          return `<li class="${cls}">${esc(option)}${optionIndex === question.kunci ? ' <b>· kunci</b>' : (optionIndex === answer ? ' <b>· jawabanmu</b>' : '')}</li>`;
        }).join('')}</ol>
        ${question.pembahasan ? `<div class="pembahasan">${esc(question.pembahasan)}</div>` : ''}
      </article>`;
    }).join('')}</div>`;
  result.querySelector('[data-action="again"]').addEventListener('click', () => {
    result.classList.add('hidden');
    $('#latihan-setup').classList.remove('hidden');
    renderLatihanSetup();
  });
  result.querySelector('[data-action="menu"]').addEventListener('click', () => {
    result.classList.add('hidden');
    $('#latihan-setup').classList.remove('hidden');
    renderLatihanSetup();
    window.scrollTo({ top: 0, behavior: 'smooth' });
    $('#latihan-kategori').focus();
  });
}

// ---------- beranda ----------
async function loadHome() {
  if (!META) META = await api('/api/meta');
  const pk = META.per_kat || {};
  $('#hero-stats').setAttribute('aria-label', 'Ringkasan bank soal dan materi');
  $('#hero-stats').innerHTML = `
    <span><b>${META.total}</b> soal di bank</span>
    <span><b>${pk.TWK || 0}</b> TWK</span>
    <span><b>${pk.TIU || 0}</b> TIU</span>
    <span><b>${pk.TKP || 0}</b> TKP</span>
    <span><b>${pk.CAMPUR || 0}</b> Campuran</span>
    <span><b>${(META.materi || []).length}</b> materi</span>`;

  // progres belajar per akun (dimuat dari API saat autentikasi)
  const done = getMateriDone();
  const totalMateri = (META.materi || []).length;
  const nDone = (META.materi || []).filter((m) => done[m.id]).length;
  $('#state-pelajari').textContent = `${nDone}/${totalMateri} modul`;
  if (nDone > 0) $('#step-pelajari').classList.add('done');
  else $('#step-pelajari').classList.remove('done');

  const doneKuis = Object.keys(getKuisDone()).length;
  $('#state-latihan').textContent = `${doneKuis} kuis selesai`;
  if (doneKuis > 0) $('#step-latihan').classList.add('done');
  else $('#step-latihan').classList.remove('done');

  try {
    const riw = await api('/api/riwayat');
    $('#state-simulasi').textContent = `${riw.length} ujian`;
    if (riw.length > 0) $('#step-simulasi').classList.add('done');
    else $('#step-simulasi').classList.remove('done');
  } catch (_) {
    $('#state-simulasi').textContent = '0 ujian';
  }

  // resume sesi
  const box = $('#resume-box');
  box.classList.add('hidden');
  try {
    const r = await api('/api/ujian/aktif');
    if (r.ada && r.sesi) {
      const s = r.sesi;
      const jawab = Object.keys(s.jawaban).length;
      const sisa = Math.max(0, s.deadline - Math.floor(Date.now() / 1000));
      box.classList.remove('hidden');
      box.innerHTML = `Ada ujian <b>${esc(s.mode)}</b> tertunda — <b>${jawab}/${s.soal.length}</b> soal terjawab, sisa waktu <b>${fmtWaktu(sisa)}</b>.
        <button class="btn btn-success btn-sm" style="margin-left:12px" onclick="resumeUjian()">Lanjutkan</button>`;
    }
  } catch (_) {}
}

async function mulaiUjian(mode) {
  try {
    const r = await post(`/api/ujian/start?mode=${mode}`);
    mulaiSesi(r.sesi);
  } catch (e) {
    const m = e.message;
    if (m === 'ada_sesi_aktif') {
      showModal('Ada sesi ujian tertunda',
        'Anda masih memiliki ujian yang berjalan. Lanjutkan atau batalkan dulu?',
        [
          ['Batalkan yang lama & mulai baru', async () => {
            closeModal();
            const aktif = await api('/api/ujian/aktif');
            if (aktif.ada) await post(`/api/ujian/${aktif.sesi.id}/batalkan`);
            mulaiUjian(mode);
          }],
          ['Lanjutkan ujian lama', () => { closeModal(); resumeUjian(); }],
          ['Batal', closeModal],
        ]);
    } else {
      showModal('Gagal', esc(m), [['OK', closeModal]]);
    }
  }
}

function mulaiSesi(sesi) {
  ujian = sesi;
  curIdx = 0;
  nav('ujian');
  renderPanel();
  renderSoal();
  startTimer();
  $('#ujian-mode-badge').textContent = ujian.mode;
}

async function resumeUjian() {
  const r = await api('/api/ujian/aktif');
  if (r.ada) mulaiSesi(r.sesi);
}

// ---------- timer ----------
function startTimer() {
  clearInterval(timerInt);
  const tick = () => {
    const sisa = Math.max(0, ujian.deadline - Math.floor(Date.now() / 1000));
    const el = $('#ujian-timer');
    el.textContent = fmtWaktu(sisa);
    el.classList.toggle('warn', sisa < 300);
    if (sisa <= 0) {
      clearInterval(timerInt);
      kumpulkan(true);
    }
  };
  tick();
  timerInt = setInterval(tick, 1000);
}

// ---------- soal ----------
function renderSoal() {
  const s = ujian.soal[curIdx];
  $('#ujian-progress').textContent = `Soal ${curIdx + 1} / ${ujian.soal.length}`;
  $('#soal-no').textContent = curIdx + 1;
  $('#soal-kat').textContent = s.kat;
  $('#soal-sub').textContent = s.sub;
  $('#soal-teks').textContent = s.teks;
  const ragu = ujian.ragu.includes(curIdx + 1);
  const btnRagu = $('#btn-ragu');
  btnRagu.classList.toggle('on', ragu);
  btnRagu.textContent = ragu ? 'Ragu-ragu (ditandai)' : 'Ragu-ragu';
  btnRagu.setAttribute('aria-pressed', ragu ? 'true' : 'false');
  const jawab = ujian.jawaban[curIdx + 1];
  $('#soal-opsi').innerHTML = s.pilihan
    .map(
      (p, i) => `
    <button type="button" class="opsi ${jawab === i ? 'selected' : ''}" onclick="pilihJawaban(${i})" aria-pressed="${jawab === i}">
      <span class="letter">${String.fromCharCode(65 + i)}.</span><span>${esc(p)}</span>
    </button>`
    )
    .join('');
  $('#btn-prev').disabled = curIdx === 0;
  $('#btn-next').textContent =
    curIdx === ujian.soal.length - 1 ? 'Soal Terakhir' : 'Selanjutnya';
  renderPanel();
}

async function pilihJawaban(i) {
  const no = curIdx + 1;
  const lama = ujian.jawaban[no];
  if (lama === i) {
    // klik lagi = hapus pilihan
    await post(`/api/ujian/${ujian.id}/jawab`, { no, pilihan: null });
    delete ujian.jawaban[no];
  } else {
    await post(`/api/ujian/${ujian.id}/jawab`, { no, pilihan: i });
    ujian.jawaban[no] = i;
  }
  renderSoal();
}

async function hapusJawaban() {
  const no = curIdx + 1;
  if (!(no in ujian.jawaban)) return;
  await post(`/api/ujian/${ujian.id}/jawab`, { no, pilihan: null });
  delete ujian.jawaban[no];
  renderSoal();
}

async function toggleRagu() {
  const no = curIdx + 1;
  const ragu = !ujian.ragu.includes(no);
  await post(`/api/ujian/${ujian.id}/ragu`, { no, ragu });
  if (ragu) ujian.ragu.push(no);
  else ujian.ragu = ujian.ragu.filter((x) => x !== no);
  renderSoal();
}

function gotoSoal(i) {
  if (i < 0 || i >= ujian.soal.length) return;
  curIdx = i;
  renderSoal();
}

function renderPanel() {
  const n = ujian.soal.length;
  $('#panel-grid').innerHTML = Array.from({ length: n }, (_, i) => {
    const no = i + 1;
    let cls = '';
    if (no in ujian.jawaban) cls = 'answered';
    if (ujian.ragu.includes(no)) cls = 'marked';
    if (i === curIdx) cls += ' current';
    return `<button class="${cls}" onclick="gotoSoal(${i})">${no}</button>`;
  }).join('');
  const jawab = Object.keys(ujian.jawaban).length;
  $('#panel-stat').innerHTML = `Terjawab: <b>${jawab}</b> · Ragu: <b>${ujian.ragu.length}</b> · Sisa: <b>${n - jawab}</b>`;
}

// ---------- kumpul ----------
function konfirmasiKumpul() {
  const n = ujian.soal.length;
  const jawab = Object.keys(ujian.jawaban).length;
  let belum = '';
  if (jawab < n) {
    const kosong = [];
    for (let i = 1; i <= n; i++) if (!(i in ujian.jawaban)) kosong.push(i);
    belum = `<p style="margin-top:10px;color:#d23434"><b>${n - jawab} soal belum dijawab:</b></p>
      <div class="unanswered-list">${kosong.map((k) => `<span>${k}</span>`).join('')}</div>`;
  }
  showModal(
    'Kumpulkan Jawaban?',
    `<p>Anda menjawab <b>${jawab}</b> dari <b>${n}</b> soal. Setelah dikumpulkan, ujian <b>tidak bisa</b> dilanjutkan.${belum}</p>`,
    [
      ['Ya, kumpulkan', () => { closeModal(); kumpulkan(false); }],
      ['Kembali mengerjakan', closeModal],
    ]
  );
}

async function kumpulkan(auto = false) {
  try {
    const hasil = await post(`/api/ujian/${ujian.id}/kumpul`);
    clearInterval(timerInt);
    ujian = null;
    tampilHasil(hasil, auto);
  } catch (_) {
    alert('Gagal mengumpulkan jawaban.');
  }
}

async function batalkanUjian() {
  showModal('Batalkan Ujian?',
    '<p>Semua jawaban pada ujian ini akan dibuang dan ujian dianggap selesai tanpa nilai.</p>',
    [
      ['Ya, batalkan', async () => {
        closeModal();
        await post(`/api/ujian/${ujian.id}/batalkan`);
        clearInterval(timerInt);
        ujian = null;
        nav('home');
      }],
      ['Tidak', closeModal],
    ]);
}

// ---------- hasil ----------
function tampilHasil(hasil, auto) {
  nav('hasil');
  const status = hasil.lulus ? 'LULUS' : 'BELUM LULUS';
  const cls = hasil.lulus ? 'lulus' : 'gagal';
  const durasi = fmtWaktu(hasil.durasi_pakai);
  $('#hasil-wrap').innerHTML = `
    <div class="hasil-head">
      <div>${esc(hasil.waktu)} · Mode: <b>${esc(hasil.mode)}</b>${auto ? ' · <b>waktu habis (otomatis)</b>' : ''} · Durasi: ${durasi}</div>
      <div class="hasil-status ${cls}">${status}</div>
      <div class="hasil-total">Skor Total<b>${hasil.skor_total}</b></div>
      <div style="margin-top:16px">
        <button class="btn btn-primary" onclick="nav('home')">Beranda</button>
        <button class="btn btn-outline" onclick="tampilHasilDetail()">Lihat Kunci &amp; Pembahasan</button>
      </div>
    </div>
    <div class="bagian-grid">
      ${hasil.bagian.map(b => {
        const lulus = b.lulus ? 'lulus' : 'gagal';
        const pct = b.jumlah ? Math.round((b.skor / (b.jumlah * 5)) * 100) : 0;
        return `<div class="bagian-card">
          <div class="kat">${b.kat} ${b.passing > 0 ? `<span class="pill ${lulus}">${b.lulus ? 'Lulus' : 'Belum'}</span>` : '<span class="pill">—</span>'}</div>
          <div class="skor">${b.skor}</div>
          <div class="bar ${lulus}"><div style="width:${pct}%"></div></div>
          <div class="ket">Benar: ${b.benar}/${b.jumlah} · ${b.passing > 0 ? `Ambang batas: ${b.passing}` : 'Tanpa ambang'}</div>
        </div>`;
      }).join('')}
    </div>
    <div id="hasil-detail" class="hidden"></div>`;
  window._hasil = hasil;
}

function tampilHasilDetail() {
  const h = window._hasil;
  const el = $('#hasil-detail');
  if (!el.classList.contains('hidden')) { el.classList.add('hidden'); return; }
  el.classList.remove('hidden');
  el.innerHTML = `
    <div class="detail-card">
      <h3 style="margin-bottom:6px">Kunci Jawaban &amp; Pembahasan</h3>
      ${h.detail.map(d => {
        const jw = d.jawaban != null ? String.fromCharCode(65 + d.jawaban) : '—';
        const benar = d.benar;
        const skorTxt = d.skor > 0 ? `Skor: <b>${d.skor}</b>` : 'Skor: 0';
        return `<div class="detail-q">
          <div class="qhead">
            <span class="badge" style="background:${benar ? 'var(--success)' : 'var(--danger)'}">${benar ? 'BENAR' : 'SALAH'}</span>
            <b>No. ${d.no}</b> · ${d.kat} · ${esc(d.sub)}
          </div>
          <div class="qteks">${esc(d.teks)}</div>
          <div class="qopsi">
            ${d.pilihan.map((p, i) => {
              let cls = '';
              if (i === d.kunci) cls = 'kunci';
              else if (i === d.jawaban) cls = 'salah';
              const tag = i === d.kunci ? ' · kunci' : (i === d.jawaban ? ' · jawaban Anda' : '');
              return `<span class="${cls}">${String.fromCharCode(65 + i)}. ${esc(p)}${tag}</span>`;
            }).join('')}
          </div>
          <div class="kunci-line">Jawaban Anda: <b>${jw}</b> · Kunci: <b>${String.fromCharCode(65 + d.kunci)}</b> · ${skorTxt}</div>
          ${d.pembahasan ? `<div class="pembahasan">${esc(d.pembahasan)}</div>` : ''}
        </div>`;
      }).join('')}
    </div>`;
  el.scrollIntoView({ behavior: 'smooth' });
}

// ---------- kuis pelajaran ----------
let kuisCache = {};       // { pelajaranId: [soal] }
let kuisJawab = {};       // { pelajaranId: { idxSoal: pilihanIdx } }
let kuisTerkunci = {};    // { pelajaranId: bool }

async function ambilKuis(m) {
  if (kuisCache[m.id]) return kuisCache[m.id];
  const q = new URLSearchParams({ kat: m.kat, limit: '5', offset: '0' });
  if (m.sub) q.set('sub', m.sub);
  const r = await api('/api/bank?' + q.toString());
  kuisCache[m.id] = r.soal || [];
  return kuisCache[m.id];
}

function renderKuis(m) {
  const soal = kuisCache[m.id] || [];
  if (!soal.length) return `<div class="kuis-empty">Belum ada soal latihan untuk pelajaran ini.</div>`;
  const jawaban = kuisJawab[m.id] || {};
  const terkunci = kuisTerkunci[m.id];
  const terjawab = soal.filter((_, i) => jawaban[i] != null).length;
  const progressPercent = soal.length ? Math.round((terjawab / soal.length) * 100) : 0;
  return `
    <div class="kuis" id="kuis-${m.id}">
      <h3>Kuis Pelajaran</h3>
      <p class="kuis-sub">Kerjakan ${soal.length} soal. Jawab semua untuk membuka tombol lanjut.</p>
      <div class="kuis-status" id="kuis-status">
        <span class="progress-text">${terjawab}/${soal.length} soal dijawab</span>
        <div class="progress-bar"><div class="progress-fill" style="width:${progressPercent}%"></div></div>
      </div>
      ${soal.map((s, i) => {
        const dipilih = jawaban[i];
        const tampilkanFb = dipilih != null || terkunci;
        const isCorrect = dipilih === s.kunci;
        return `
        <div class="kuis-item" data-idx="${i}">
          <div class="kuis-q"><b>${i + 1}.</b> ${esc(s.teks)}</div>
          <div class="kuis-opsi">
            ${s.pilihan.map((p, pi) => {
              let cls = '';
              if (tampilkanFb) {
                if (pi === s.kunci) cls = 'benar';
                else if (pi === dipilih) cls = 'salah';
              }
              return `<button type="button" class="opsi ${dipilih === pi ? 'selected' : ''} ${cls}" onclick="jawabKuis('${m.id}', ${i}, ${pi})" aria-pressed="${dipilih === pi}" ${terkunci ? 'disabled' : ''}>
                <span class="letter">${String.fromCharCode(65 + pi)}.</span><span>${esc(p)}</span>
              </button>`;
            }).join('')}
          </div>
          ${tampilkanFb ? `
            <div class="kuis-fb ${isCorrect ? 'benar' : 'salah'}">
              ${isCorrect ? '✓ Benar' : `✗ Kunci: ${String.fromCharCode(65 + s.kunci)}`}
              ${s.pembahasan ? `<div class="pembahasan">${esc(s.pembahasan)}</div>` : ''}
            </div>` : ''}
        </div>`;
      }).join('')}
    </div>`;
}

function jawabKuis(idMateri, idxSoal, pilihan) {
  if (kuisTerkunci[idMateri]) return;
  kuisJawab[idMateri] = kuisJawab[idMateri] || {};
  kuisJawab[idMateri][idxSoal] = pilihan;
  const m = materiCache.find((x) => x.id === idMateri);
  if (!m) return;
  // Re-render hanya bagian kuis
  const el = document.getElementById('kuis-' + idMateri);
  if (el) {
    const wrap = el.parentElement;
    const tmp = document.createElement('div');
    tmp.innerHTML = renderKuis(m);
    wrap.replaceChild(tmp.firstElementChild, el);
  }
  updateKuisState(m);
}

function updateKuisState(m) {
  if (!m) return;
  const soal = kuisCache[m.id] || [];
  const jawaban = kuisJawab[m.id] || {};
  const terkunci = getKuisDone()[m.id];
  const terjawab = soal.filter((_, i) => jawaban[i] != null).length;
  if (ACCOUNT_PROGRESS && soal.length > 0 && terjawab === soal.length && !terkunci) {
    setKuisDone(m.id, true);
    kuisTerkunci[m.id] = true;
    void saveAccountProgress('kuis', m.id);
  }
  const status = document.getElementById('kuis-status');
  if (status) {
    const percent = soal.length ? Math.round((terjawab / soal.length) * 100) : 0;
    status.innerHTML = terkunci
      ? '<span class="progress-text">Kuis selesai</span><div class="progress-bar" role="progressbar" aria-label="Progres kuis" aria-valuenow="100" aria-valuemin="0" aria-valuemax="100"><div class="progress-fill" style="width:100%"></div></div>'
      : `<span class="progress-text">${terjawab}/${soal.length} soal dijawab</span><div class="progress-bar" role="progressbar" aria-label="Progres kuis" aria-valuenow="${percent}" aria-valuemin="0" aria-valuemax="100"><div class="progress-fill" style="width:${percent}%"></div></div>`;
  }
  const btnNext = document.querySelector('#materi-content .cs-actions .btn-primary');
  if (btnNext) {
    if (!btnNext.dataset.last) btnNext.dataset.last = btnNext.textContent;
    if (terkunci || terjawab >= soal.length) {
      btnNext.disabled = false;
      btnNext.textContent = btnNext.dataset.last;
    } else {
      btnNext.disabled = true;
      btnNext.textContent = `Jawab ${soal.length - terjawab} soal lagi untuk melanjutkan`;
    }
  }
  // Update tombol Tandai Selesai
  const btnTandai = document.getElementById('btn-tandai-selesai');
  if (btnTandai) {
    const done = getMateriDone()[m.id];
    if (done) {
      btnTandai.disabled = false;
      btnTandai.textContent = 'Tandai belum selesai';
    } else {
      btnTandai.disabled = !(terkunci || terjawab >= soal.length);
      btnTandai.textContent = btnTandai.disabled ? 'Selesaikan kuis dulu' : 'Tandai selesai';
    }
  }
}

// ---------- materi ----------
let materiCache = [];
let materiTab = 'ALL';

function getMateriDone() {
  return progressMap('materi');
}
function setMateriDone(id, val) {
  if (!val) return;
  void markProgress('materi', id);
}
function getKuisDone() {
  return progressMap('kuis');
}
function setKuisDone(id, val) {
  if (!val) return;
  void markProgress('kuis', id);
}
async function loadMateri() {
  if (!materiCache.length) materiCache = await api('/api/materi');
  materiTab = 'ALL';
  materiBukaId = null;
  renderMateriTabs();
  $('#materi-content').classList.add('hidden');
  $('#materi-list').classList.remove('hidden');
  $('#materi-progress').classList.remove('hidden');
  renderMateriList();
}

function pilihTabMateri(tab) {
  materiTab = tab;
  materiBukaId = null;
  renderMateriTabs();
  $('#materi-content').classList.add('hidden');
  $('#materi-list').classList.remove('hidden');
  $('#materi-progress').classList.remove('hidden');
  if (tab === 'ALL' || tab === 'TWK' || tab === 'TIU' || tab === 'TKP') {
    renderMateriList();
  } else if (tab === 'VIDEO') {
    renderVideo();
  } else if (tab === 'SUMBER') {
    renderSumber();
  }
}

function renderMateriTabs() {
  document.querySelectorAll('.materi-tab').forEach((b) => {
    b.classList.toggle('active', b.dataset.mtab === materiTab);
  });
}

function renderMateriList() {
  const c = $('#materi-content');
  c.classList.add('hidden');
  const done = getMateriDone();
  const categories = ['TWK', 'TIU', 'TKP'];
  const visibleCategories = materiTab === 'ALL' ? categories : categories.filter((category) => category === materiTab);
  const countDone = materiCache.filter((m) => done[m.id]).length;
  $('#materi-progress').innerHTML = `<span>Progres belajar: <b>${countDone} dari ${materiCache.length}</b> materi selesai. Pilih topik untuk melihat pelajarannya.</span>`;

  $('#materi-list').innerHTML = `<div class="subject-list">
    ${visibleCategories.map((kat) => {
      const items = materiCache.filter((m) => m.kat === kat);
      const completed = items.filter((m) => done[m.id]).length;
      const byTopic = new Map();
      items.forEach((m) => {
        const topic = m.sub || m.modul || kat;
        if (!byTopic.has(topic)) byTopic.set(topic, []);
        byTopic.get(topic).push(m);
      });
      return `<section class="subject-card" aria-labelledby="subject-${kat}">
        <div class="subject-heading">
          <div><span class="subject-kicker">Kelompok materi</span><h3 id="subject-${kat}">${kat}</h3></div>
          <span class="subject-progress">${items.length} materi · ${completed} selesai</span>
        </div>
        ${Array.from(byTopic, ([topic, lessons]) => `<div class="subject-topic">
          <h4>${esc(topic)} <span>${lessons.length} ${lessons.length === 1 ? 'materi' : 'materi'}</span></h4>
          ${lessons.map((m) => {
            const index = items.findIndex((item) => item.id === m.id);
            return `<button type="button" class="lesson-row ${done[m.id] ? 'done' : ''}" onclick="bukaMateri('${m.id}')">
              <span class="lesson-check" aria-hidden="true">${done[m.id] ? '✓' : String(index + 1).padStart(2, '0')}</span>
              <span class="lesson-copy"><b>${esc(m.judul)}</b><small>${esc(m.ringkas || topic)}</small></span>
              <span class="lesson-state">${done[m.id] ? 'Selesai' : 'Buka materi'}</span>
            </button>`;
          }).join('')}</div>
        </div>`).join('')}
      </section>`;
    }).join('')}
  </div>
  <div class="resource-shortcuts"><span>Pelengkap belajar</span><button class="btn btn-outline btn-sm" onclick="pilihTabMateri('VIDEO')">Video pembelajaran</button><button class="btn btn-outline btn-sm" onclick="pilihTabMateri('SUMBER')">Sumber resmi</button></div>`;
}

let materiBukaId = null;

async function bukaMateri(id) {
  materiBukaId = id;
  const m = await api('/api/materi/' + id);
  const done = getMateriDone();
  const group = materiCache.filter((x) => x.kat === m.kat);
  const idx = group.findIndex((x) => x.id === id);

  const prev = group[idx - 1];
  const next = group[idx + 1];

  const doneCount = group.filter((x) => done[x.id]).length;
  const pct = group.length ? Math.round((doneCount / group.length) * 100) : 0;
  const terkunci = getKuisDone()[id] || false;

  const soal = await ambilKuis(m);
  kuisJawab[id] = kuisJawab[id] || {};
  kuisTerkunci[id] = terkunci;

  $('#materi-content').innerHTML = `
    <div class="coursera">
      <aside class="coursera-sidebar">
        <div class="cs-title">${esc(m.kat)}</div>
        <div class="cs-progress"><div class="cs-progress-bar"><div style="width:${pct}%"></div></div>
        <span>${doneCount} dari ${group.length} pelajaran selesai</span></div>
        <div class="cs-list">
          ${group.map((x, i) => {
            const isRequired = !done[x.id];
            return `
            <button type="button" class="cs-item ${x.id === id ? 'active' : ''}"
                    onclick="bukaMateri('${x.id}')"
                    aria-current="${x.id === id ? 'page' : 'false'}"
                    title="${done[x.id] ? 'Pelajaran selesai' : 'Buka pelajaran'}">
              <span class="cs-check ${done[x.id] ? 'done' : isRequired ? 'required' : ''}">${done[x.id] ? '✓' : isRequired ? '●' : ''}</span>
              <span class="cs-item-body"><b>${i + 1}. ${esc(x.judul)}</b><small>${esc(x.ringkas || '')}</small></span>
              ${done[x.id] ? `<span class="cs-required-badge done">Selesai</span>` : ''}
            </button>`;
          }).join('')}
        </div>
        <div class="cs-links">
          <button class="btn btn-outline btn-block" onclick="pilihTabMateri('VIDEO')">Video Pembelajaran</button>
          <button class="btn btn-outline btn-block" onclick="pilihTabMateri('SUMBER')">Sumber Resmi</button>
        </div>
      </aside>
      <div class="coursera-main">
        <div class="cs-breadcrumb">${esc(m.kat)} · ${esc(m.modul || '')} · ${idx + 1}/${group.length}</div>
        <h2>${esc(m.judul)}</h2>
        <p class="cs-sub">${esc(m.ringkas || '')}</p>
        <div class="materi-content-inner">${m.html}</div>
        ${m.video && m.video.length ? `
          <div class="cs-video">
            <h3>Video Pelengkap</h3>
            <div class="video-grid">
              ${m.video.map((it) => `
                <div class="video-card">
                  <a class="video-thumb" href="https://www.youtube.com/watch?v=${it.id}" target="_blank" rel="noopener" aria-label="Tonton ${esc(it.judul)} di YouTube">
                    <img src="https://i.ytimg.com/vi/${it.id}/hqdefault.jpg" alt="Thumbnail video ${esc(it.judul)}" loading="lazy">
                    <span class="play"></span>
                  </a>
                  <div class="vbody">
                    <div class="vtopik">${esc(it.topik || m.kat)}</div>
                    <div class="vjudul">${esc(it.judul)}</div>
                    <a class="vlink" href="https://www.youtube.com/watch?v=${it.id}" target="_blank" rel="noopener">Tonton di YouTube</a>
                  </div>
                </div>`).join('')}
            </div>
          </div>` : ''}
        <div class="kuis-wrap">
          ${renderKuis(m)}
        </div>
        <div class="cs-actions">
          <button class="btn ${done[id] ? 'btn-outline' : 'btn-success'}" id="btn-tandai-selesai" onclick="toggleMateriDone('${id}')">${done[id] ? 'Tandai belum selesai' : 'Tandai selesai'}</button>
          <button class="btn btn-outline" onclick="${prev ? `bukaMateri('${prev.id}')` : 'return false;'}" ${prev ? '' : 'disabled'}>Sebelumnya</button>
          <button class="btn btn-primary" onclick="${next ? `bukaMateri('${next.id}')` : `pilihTabMateri('ALL')`}">${next ? 'Selanjutnya' : 'Kembali ke daftar'}</button>
        </div>
      </div>
    </div>`;

  $('#materi-list').classList.add('hidden');
  $('#materi-progress').classList.add('hidden');
  $('#materi-content').classList.remove('hidden');
  $('#materi-content').scrollIntoView();
  updateKuisState(m);
}

function toggleMateriDone(id) {
  const done = getMateriDone();
  const terkunci = getKuisDone()[id];
  const soal = kuisCache[id] || [];
  const jawaban = kuisJawab[id] || {};
  const terjawab = soal.filter((_, i) => jawaban[i] != null).length;

  if (!done[id]) {
    // ingin tandai selesai
    if (!terkunci && terjawab < soal.length) {
      const progressPercent = Math.round((terjawab / soal.length) * 100);
      showModal('Kuis Belum Lengkap',
        `<p>Kuis pelajaran ini belum dijawab semua.</p>
         <p>Anda telah menjawab ${terjawab}/${soal.length} soal (${progressPercent}%).</p>
         <p>Kerjakan semua soal untuk mengunci pelajaran dan melanjutkan ke materi berikutnya.</p>`,
        [['OK']]);
      return;
    }
  }

  setMateriDone(id, !done[id]);
  void setMateriCompleted(id, !done[id]);
  if (!done[id]) {
    setKuisDone(id, true);
  } else {
    setKuisDone(id, false);
  }
  bukaMateri(id);
}

function tutupMateri() {
  $('#materi-content').classList.add('hidden');
  $('#materi-progress').classList.remove('hidden');
  $('#materi-list').classList.remove('hidden');
  renderMateriList();
}

async function renderVideo() {
  $('#materi-content').classList.add('hidden');
  $('#materi-list').classList.remove('hidden');
  $('#materi-progress').innerHTML =
    `<span>Video pelengkap dikelompokkan per materi — semua video diverifikasi dan terbuka di YouTube.</span>`;
  try {
    const v = await api('/api/video');
    const all = { TWK: [], TIU: [], TKP: [] };
    for (const k of ['TWK', 'TIU', 'TKP']) all[k] = v[k] || [];
    const items = all.TWK.concat(all.TIU, all.TKP);
    const katLabel = (it) => {
      for (const k of ['TWK', 'TIU', 'TKP']) {
        if ((v[k] || []).some((x) => x.id === it.id)) return k;
      }
      return '';
    };
    $('#materi-list').innerHTML = `
      <div class="video-grid">
        ${items.map((it) => `
          <div class="video-card">
            <a class="video-thumb" href="https://www.youtube.com/watch?v=${it.id}" target="_blank" rel="noopener" aria-label="Tonton ${esc(it.judul)} di YouTube">
              <img src="https://i.ytimg.com/vi/${it.id}/hqdefault.jpg" alt="Thumbnail video ${esc(it.judul)}" loading="lazy">
              <span class="play"></span>
            </a>
            <div class="vbody">
              <div class="vtopik">${esc(katLabel(it) || it.topik || 'CPNS')}</div>
              <div class="vjudul">${esc(it.judul)}</div>
              <a class="vlink" href="https://www.youtube.com/watch?v=${it.id}" target="_blank" rel="noopener">Tonton di YouTube</a>
            </div>
          </div>`).join('')}
      </div>`;
    if (!items.length) $('#materi-list').innerHTML =
      `<div class="empty-state">Belum ada video untuk kategori ini.</div>`;
  } catch (_) {
    $('#materi-list').innerHTML =
      `<div class="empty-state">Video gagal dimuat. Periksa koneksi, lalu buka kembali tab Video Pembelajaran.</div>`;
  }
}

function renderSumber() {
  $('#materi-content').classList.add('hidden');
  $('#materi-list').classList.remove('hidden');
  $('#materi-progress').innerHTML =
    `<span>Sumber resmi dan referensi belajar yang dapat kamu buka langsung.</span>`;
  const sumber = [
    ['Portal CAT BKN', 'https://cat.bkn.go.id/', 'Simulasi CAT resmi BKN untuk latihan ujian.'],
    ['SSCASN BKN', 'https://sscasn.bkn.go.id/', 'Portal resmi pendaftaran dan pengumuman CASN.'],
    ['BKN RI', 'https://www.bkn.go.id/', 'Pengumuman resmi, layanan, dan regulasi kepegawaian.'],
    ['JDIH KemenPAN-RB', 'https://jdih.menpan.go.id/', 'Regulasi resmi seleksi, termasuk ketentuan passing grade.'],
    ['JDIH BPK (Peraturan)', 'https://peraturan.bpk.go.id/', 'Database peraturan perundang-undangan.'],
  ];
  $('#materi-list').innerHTML = `
    <div class="sumber-list">
      ${sumber.map(([nama, url, desc]) => `
        <a class="sumber-card" href="${url}" target="_blank" rel="noopener">
          <b>${nama}</b>
          <span>${desc}</span>
        </a>`).join('')}
    </div>`;
}

// ---------- riwayat ----------
async function loadRiwayat() {
  const r = await api('/api/riwayat');
  if (!r.length) {
    $('#riwayat-list').innerHTML =
      `<div class="empty-state"><b>Belum ada riwayat ujian.</b> Setelah kamu mengumpulkan simulasi, hasilnya akan tampil di sini.<br>
        <button class="btn btn-primary" onclick="mulaiUjian('SKD')">Mulai simulasi SKD pertama</button></div>`;
    return;
  }
  $('#riwayat-list').innerHTML = r.map((h) => {
    const cls = h.lulus ? 'lulus' : 'gagal';
    const bagian = h.bagian
      .map((b) => `${b.kat} <b style="color:${b.lulus ? 'var(--success)' : 'var(--danger)'}">${b.skor}</b>`)
      .join(' · ');
    return `<div class="rw-item">
      <div><div class="rw-mode">${esc(h.mode)}</div>
      <div class="rw-waktu">${esc(h.waktu)} · ${fmtWaktu(h.durasi_pakai)}</div></div>
      <div style="color:var(--muted);font-size:13px">${bagian} · Ambang: TWK 65 · TIU 80 · TKP 166</div>
      <div class="rw-skor"><span class="pill ${cls}">${h.lulus ? 'LULUS' : 'BELUM'}</span><br><b>${h.skor_total}</b></div>
      <button class="btn btn-sm btn-outline" onclick="tampilHasilRiwayat(${JSON.stringify(h).replace(/"/g, '&quot;')})">Detail</button>
    </div>`;
  }).join('');
}
function tampilHasilRiwayat(h) {
  window._hasil = h;
  nav('hasil');
  $('#hasil-wrap').innerHTML = '';
  tampilHasil(h, false);
}

// ---------- modal ----------
function showModal(title, bodyHtml, buttons) {
  window._modalReturnFocus = document.activeElement;
  $('#modal-body').innerHTML = `<h3 id="modal-title">${title}</h3>${bodyHtml}`;
  $('#modal-btns').innerHTML = buttons
    .map(([t, fn], i) => {
      const cls = i === 0 ? 'btn btn-primary' : 'btn btn-outline';
      const id = 'mbtn' + i;
      window['_mfn' + i] = fn;
      return `<button class="${cls}" id="${id}">${t}</button>`;
    })
    .join('');
  $('#modal').classList.remove('hidden');
  buttons.forEach((_, i) => {
    document.getElementById('mbtn' + i).onclick = window['_mfn' + i];
  });
  document.getElementById('mbtn0')?.focus();
}
function closeModal() {
  $('#modal').classList.add('hidden');
  window._modalReturnFocus?.focus?.();
}
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape' && !$('#modal').classList.contains('hidden')) closeModal();
});

// ---------- fullscreen & anti-curang ----------
function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen?.();
  } else {
    document.exitFullscreen?.();
  }
}
document.addEventListener('visibilitychange', () => {
  if (curView === 'ujian' && document.hidden && !warningTab) {
    warningTab = true;
    // catat peringatan di server? cukup peringatan lokal
  }
});
document.addEventListener('keydown', (e) => {
  if (curView !== 'ujian' || !ujian) return;
  if (e.key === 'ArrowLeft') gotoSoal(curIdx - 1);
  if (e.key === 'ArrowRight') gotoSoal(curIdx + 1);
  if (/^[a-z]$/i.test(e.key)) {
    const i = e.key.toUpperCase().charCodeAt(0) - 65;
    if (i < ujian.soal[curIdx].pilihan.length) pilihJawaban(i);
  }
});

// ---------- init ----------
document.addEventListener('DOMContentLoaded', () => {
  authElementsReady = true;
  document.body.classList.add('auth-checking');
  $('#auth-form').addEventListener('submit', submitAuth);
  $('#auth-toggle').addEventListener('click', () => {
    authMode = authMode === 'login' ? 'register' : 'login';
    renderAuthGate();
  });
  $('#auth-logout').addEventListener('click', logoutAccount);
  document.querySelectorAll('#mainnav button').forEach((b) => {
    b.addEventListener('click', () => nav(b.dataset.nav));
  });
  initializeAccountApp();
});
