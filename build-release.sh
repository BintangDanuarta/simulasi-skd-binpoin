#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
VERSION="${VERSION:-0.1.0-beta.1}"
TARGET="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
APP="simulasi-skd-binpoin-${VERSION}-${TARGET}"
OUT="release"
mkdir -p "$OUT"
cargo build --release --locked --target "$TARGET"
BIN="target/$TARGET/release/simulasi-skd-binpoin"
case "$TARGET" in *windows*) BIN+=".exe";; esac
STAGE="$OUT/$APP"
rm -rf "$STAGE"
mkdir -p "$STAGE/data" "$STAGE/static"
install -m 755 "$BIN" "$STAGE/"
install -m 644 data/questions.json data/materi.json "$STAGE/data/"
[ ! -f data/video.json ] || install -m 644 data/video.json "$STAGE/data/"
install -m 644 static/app.js static/index.html static/style.css static/binpoin-logo.png static/favicon.ico "$STAGE/static/"
printf 'Simulasi SKD by Binpoin — Beta %s\n\n' "$VERSION" > "$STAGE/README.txt"
cat >> "$STAGE/README.txt" <<'TXT'
Jalankan file aplikasi di folder ini. Browser akan terbuka otomatis.
Data dan riwayat latihan tersimpan lokal di profil pengguna dan tidak dikirim.
Keluar dengan menutup jendela terminal aplikasi.
TXT
case "$TARGET" in
  *windows*)
    cat > "$STAGE/Simulasi SKD by Binpoin.bat" <<'BAT'
@echo off
cd /d "%~dp0"
start "" "%~dp0simulasi-skd-binpoin.exe"
BAT
    ARCHIVE="$APP.zip"
    if [[ "$TARGET" == *-windows-msvc ]]; then
      ARCHIVE="simulasi-skd-binpoin-${VERSION}-x86_64-pc-windows-msvc.zip"
    fi
    (cd "$OUT" && zip -qr "$ARCHIVE" "$APP")
    ;;
  *darwin*)
    chmod +x "$STAGE/simulasi-skd-binpoin"
    (cd "$OUT" && tar -czf "$APP.tar.gz" "$APP")
    ;;
  *)
    (cd "$OUT" && tar -czf "$APP.tar.gz" "$APP")
    ;;
esac
sha256sum "$OUT/$APP".*
du -h "$OUT/$APP".*
