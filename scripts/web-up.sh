#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ ! -f .env ]]; then
  echo "Copy .env.web.example to .env and set PUBLIC_ORIGIN, ALLOWED_HOSTS and POSTGRES_PASSWORD." >&2
  exit 1
fi

password="$(grep -E '^POSTGRES_PASSWORD=' .env | cut -d= -f2- | tr -d '[:space:]')"
encoded="$(grep -E '^POSTGRES_PASSWORD_URLENCODED=' .env | cut -d= -f2- | tr -d '[:space:]')"
if [[ -z "$password" || -z "$encoded" || "$password" == replace-* || "$encoded" == replace-* ]]; then
  echo "Set both Postgres password values in .env before deploying." >&2
  exit 1
fi
if [[ "$password" != "$encoded" ]]; then
  echo "Use a URL-safe alphanumeric password so POSTGRES_PASSWORD_URLENCODED matches it." >&2
  exit 1
fi

docker compose --env-file .env -f compose.web.yml up -d --build
docker compose --env-file .env -f compose.web.yml ps