#!/usr/bin/env bash
set -euo pipefail

SMOKE_ROOT="$(cd "$(dirname "$0")" && pwd)"
LIB_ROOT="$(cd "$SMOKE_ROOT/.." && pwd)"
REPO_ROOT="$(git -C "$LIB_ROOT" rev-parse --show-toplevel)"
FIXTURE="$REPO_ROOT/rust/ah-geckod/libtermsurf_gecko/smoke-test/navigation-fixture/server.py"
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ladybird-navigation-actions.XXXXXX")"
SERVER_PID=""

cleanup() {
  if test -n "$SERVER_PID"; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -rf "$RUN_DIR"
}
trap cleanup EXIT

python3 "$FIXTURE" --port-file "$RUN_DIR/port" >"$RUN_DIR/server.log" 2>&1 &
SERVER_PID=$!
for _ in $(seq 1 100); do
  test -s "$RUN_DIR/port" && break
  sleep 0.05
done
test -s "$RUN_DIR/port" || { cat "$RUN_DIR/server.log" >&2; exit 1; }
PORT="$(cat "$RUN_DIR/port")"

TERMSURF_LADYBIRD_BACKEND=real "$LIB_ROOT/build.sh" --configuration Debug
(
  cd "$REPO_ROOT/rust"
  TERMSURF_LADYBIRD_BACKEND=real \
    TERMSURF_LADYBIRD_SMOKE_BASE_URL="http://127.0.0.1:$PORT" \
    cargo run -p ah-ladybirdd -- --termsurf-navigation-actions-smoke
)
