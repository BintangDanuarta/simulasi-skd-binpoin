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
if [[ "$TARGET" == *darwin* ]]; then
  if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" && -z "${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ]]; then
    echo "Set TAURI_SIGNING_PRIVATE_KEY (or TAURI_SIGNING_PRIVATE_KEY_PATH) to sign updater artifacts." >&2
    exit 1
  fi
  mkdir -p bundle/backend bundle/data bundle/static
  rm -f bundle/backend/simulasi-skd-binpoin
  rm -f bundle/data/questions.json bundle/data/materi.json bundle/data/video.json
  install -m 755 "$BIN" bundle/backend/simulasi-skd-binpoin
  install -m 644 data/questions.json data/materi.json bundle/data/
  if [[ -f data/video.json ]]; then
    install -m 644 data/video.json bundle/data/
  else
    printf '{"TWK":[],"TIU":[],"TKP":[]}' > bundle/data/video.json
  fi
  VERSION="$VERSION" npm run build:desktop-frontend
  rsync -a --delete static/ bundle/static/
  VERSION="$VERSION" npx --yes @tauri-apps/cli build --features desktop,custom-protocol --target "$TARGET" --bundles app,dmg --config "{\"version\":\"$VERSION\"}"
  APP_DIR="target/$TARGET/release/bundle/macos/Simulasi SKD by Binpoin.app"
  DMG="target/$TARGET/release/bundle/dmg/Simulasi SKD by Binpoin_${VERSION}_$( [[ "$TARGET" == x86_64-* ]] && echo x64 || echo aarch64 ).dmg"
  mkdir -p "$OUT"
  ditto "$APP_DIR" "$OUT/Simulasi SKD by Binpoin-${VERSION}-${TARGET}.app"
  cp "$DMG" "$OUT/simulasi-skd-binpoin-${VERSION}-${TARGET}.dmg"
  UPDATE_ARCH="aarch64"
  [[ "$TARGET" == x86_64-* ]] && UPDATE_ARCH="x86_64"
  UPDATE_ARCHIVE="target/$TARGET/release/bundle/macos/Simulasi SKD by Binpoin.app.tar.gz"
  cp "$UPDATE_ARCHIVE" "$OUT/simulasi-skd-binpoin-${VERSION}-${UPDATE_ARCH}.app.tar.gz"
  cp "$UPDATE_ARCHIVE.sig" "$OUT/simulasi-skd-binpoin-${VERSION}-${UPDATE_ARCH}.app.tar.gz.sig"
  ./scripts/create-updater-manifest.sh "$VERSION" "darwin-${UPDATE_ARCH}" "simulasi-skd-binpoin-${VERSION}-${UPDATE_ARCH}.app.tar.gz"
  shasum -a 256 "$OUT/simulasi-skd-binpoin-${VERSION}-${TARGET}.dmg"
  du -h "$OUT/simulasi-skd-binpoin-${VERSION}-${TARGET}.dmg"
  rm -f bundle/backend/simulasi-skd-binpoin bundle/data/questions.json bundle/data/materi.json bundle/data/video.json
  find bundle/static -type f ! -name .keep -delete
  exit 0
fi
STAGE="$OUT/$APP"
rm -rf "$STAGE"
mkdir -p "$STAGE/data" "$STAGE/static"
install -m 755 "$BIN" "$STAGE/"
install -m 644 data/questions.json data/materi.json "$STAGE/data/"
[ ! -f data/video.json ] || install -m 644 data/video.json "$STAGE/data/"
install -m 644 static/app.js static/index.html static/style.css static/binpoin-logo.png static/favicon.ico "$STAGE/static/"
printf 'Simulasi SKD by Binpoin — Beta %s\n\n' "$VERSION" > "$STAGE/README.txt"
cat >> "$STAGE/README.txt" <<'TXT'
MACOS: IMPORTANT — this beta is not signed/notarized by Apple. Gatekeeper may show
“Apple could not verify ... is free of malware”. Only continue if you downloaded
this archive from the official Binpoin GitHub release and verified its SHA-256.

To open on macOS: Control-click the executable, choose Open, then confirm Open.
If blocked, open System Settings > Privacy & Security and choose Open Anyway for
simulasi-skd-binpoin. This exception is for this binary only.

The app serves its local web interface from the static/ folder beside the
executable. Keep the extracted folder together; do not move the executable alone.
The app opens the interface in your browser. Data and history stay on this Mac.
Close the terminal process to exit the app.
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
  *)
    (cd "$OUT" && tar -czf "$APP.tar.gz" "$APP")
    ;;
esac
sha256sum "$OUT/$APP".*
du -h "$OUT/$APP".*
