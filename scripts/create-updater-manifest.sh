#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?release version required}"
TARGET="${2:?updater target required}"
OWNER="BintangDanuarta"
REPO="simulasi-skd-binpoin"
PLATFORM="$TARGET"
ASSET="${3:?signed updater artifact filename required}"
SIGNATURE_FILE="release/${ASSET}.sig"
FEED="release/latest-${PLATFORM}.json"
URL="https://github.com/${OWNER}/${REPO}/releases/download/v${VERSION}/${ASSET}"

SIGNATURE="$(cat "release/${ASSET}.sig")"
jq -n \
  --arg version "$VERSION" \
  --arg notes "Simulasi SKD by Binpoin ${VERSION}" \
  --arg url "$URL" \
  --arg signature "$SIGNATURE" \
  --arg platform "$PLATFORM" \
  '{version:$version, notes:$notes, pub_date:(now|todate), platforms:{($platform):{url:$url,signature:$signature}}}' \
  > "$FEED"

test -s "$SIGNATURE_FILE"