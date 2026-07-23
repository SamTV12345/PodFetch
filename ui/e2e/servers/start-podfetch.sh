#!/usr/bin/env bash
# Prepares a clean runtime directory (fresh sqlite DB, built UI as ./static)
# and starts the podfetch binary for the playwright suite.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
RUN_DIR="$ROOT/.e2e-run"
BINARY="$ROOT/target/debug/podfetch"

if [ ! -x "$BINARY" ]; then
    echo "podfetch binary not found at $BINARY — build it first:" >&2
    echo "  cargo build --no-default-features --features sqlite" >&2
    exit 1
fi
if [ ! -f "$ROOT/ui/dist/index.html" ] && [ ! -d "$ROOT/ui/dist/assets" ]; then
    echo "ui/dist not found — build the UI first: cd ui && pnpm run build" >&2
    exit 1
fi

rm -rf "$RUN_DIR"
mkdir -p "$RUN_DIR/static"
cp -R "$ROOT/ui/dist/." "$RUN_DIR/static/"

cd "$RUN_DIR"
# diesel's multi-connection URL parser rejects the `sqlite://<abs-path>` form
# for Windows-style paths (the drive letter trips it up), so on Git Bash /
# MSYS use a bare relative path (we already cd'd into RUN_DIR). Linux/macOS
# keep the original scheme form, so CI is unaffected.
case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) export DATABASE_URL="podcast.db" ;;
    *) export DATABASE_URL="sqlite://$RUN_DIR/podcast.db" ;;
esac
export SERVER_URL="http://127.0.0.1:8000"
export TRANSCRIPTION_API_BASE_URL="http://127.0.0.1:9998"

exec "$BINARY"
