#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="$ROOT/logs"
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/termsurf-girlbat-runtime.XXXXXX")"
APP="${TERMSURF_GHOSTBOARD_APP:-$ROOT/ghostboard/macos/build/Debug/Astrohacker Terminal.app}"
APP_BIN="$APP/Contents/MacOS/aht"
WEB="${TERMSURF_WEB:-$ROOT/target/debug/web}"
GIRLBAT="${TERMSURF_GIRLBAT:-$ROOT/target/debug/ah-ladybirdd}"
APP_LOG="$LOG_DIR/girlbat-ghostboard-runtime-app-${TS}.log"
HARNESS_LOG="$LOG_DIR/girlbat-ghostboard-runtime-harness-${TS}.log"
WEBTUI_TRACE="$LOG_DIR/girlbat-ghostboard-runtime-webtui-${TS}.log"
HTTP_PID=""
PID=""

mkdir -p "$LOG_DIR"

log() {
  printf '%s\n' "$*" | tee -a "$HARNESS_LOG"
}

fail() {
  log "FAIL: $*"
  exit 1
}

delay() {
  osascript -e "delay ${1:-0.5}" >/dev/null
}

cleanup() {
  if [ -n "${HTTP_PID:-}" ] && kill -0 "$HTTP_PID" >/dev/null 2>&1; then
    kill "$HTTP_PID" >/dev/null 2>&1 || true
  fi
  if [ -n "${PID:-}" ] && kill -0 "$PID" >/dev/null 2>&1; then
    kill "$PID" >/dev/null 2>&1 || true
    delay 0.5 || true
    kill -9 "$PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$RUN_DIR"
}
trap cleanup EXIT

require_executable() {
  [ -x "$1" ] || fail "missing executable: $1"
}

wait_for_pattern() {
  local pattern="$1"
  local label="$2"
  local attempts="${3:-60}"
  for _ in $(seq 1 "$attempts"); do
    if grep -E "$pattern" "$APP_LOG" >/dev/null 2>&1; then
      log "PASS: $label"
      return 0
    fi
    delay 1
  done
  fail "timed out waiting for $label pattern=$pattern app_log=$APP_LOG"
}

wait_for_literal() {
  local text="$1"
  local label="$2"
  local attempts="${3:-60}"
  for _ in $(seq 1 "$attempts"); do
    if grep -F "$text" "$APP_LOG" >/dev/null 2>&1; then
      log "PASS: $label"
      return 0
    fi
    delay 1
  done
  fail "timed out waiting for $label text=$text app_log=$APP_LOG"
}

pick_port() {
  python3 - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

require_executable "$APP_BIN"
require_executable "$WEB"
require_executable "$GIRLBAT"

WEB_ROOT="$RUN_DIR/site"
mkdir -p "$WEB_ROOT"
cat >"$WEB_ROOT/index.html" <<'EOF'
<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Girlbat Runtime Smoke</title>
    <style>
      html,
      body {
        margin: 0;
        min-height: 100vh;
        background: #16324f;
        color: #f6f0d7;
        font: 24px -apple-system, BlinkMacSystemFont, sans-serif;
      }
      main {
        padding: 36px;
      }
    </style>
  </head>
  <body>
    <main>
      <h1>Girlbat Runtime Smoke</h1>
      <p id="beacon">ordinary http page</p>
    </main>
    <script>
      console.log("girlbat-runtime-smoke-console");
    </script>
  </body>
</html>
EOF

PORT="$(pick_port)"
URL="http://127.0.0.1:${PORT}/index.html"
python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$WEB_ROOT" >>"$HARNESS_LOG" 2>&1 &
HTTP_PID="$!"

for _ in $(seq 1 30); do
  if python3 - "$URL" <<'PY' >/dev/null 2>&1
import sys
import urllib.request

with urllib.request.urlopen(sys.argv[1], timeout=1) as response:
    raise SystemExit(0 if response.status == 200 else 1)
PY
  then
    break
  fi
  delay 0.25
done

python3 - "$URL" <<'PY' >/dev/null 2>&1 || fail "HTTP fixture did not become ready"
import sys
import urllib.request

with urllib.request.urlopen(sys.argv[1], timeout=1) as response:
    raise SystemExit(0 if response.status == 200 else 1)
PY

COMMAND="$RUN_DIR/run-web.sh"
CONFIG="$RUN_DIR/config"
XDG_CONFIG_HOME="$RUN_DIR/xdg"
cat >"$COMMAND" <<EOF
#!/usr/bin/env bash
exec "$WEB" --browser girlbat "$URL"
EOF
chmod +x "$COMMAND"

cat >"$CONFIG" <<EOF
window-save-state = never
initial-command = direct:$COMMAND
EOF

log "app=$APP"
log "web=$WEB"
log "girlbat=$GIRLBAT"
log "url=$URL"
log "app_log=$APP_LOG"
log "harness_log=$HARNESS_LOG"
log "webtui_trace=$WEBTUI_TRACE"

XDG_CONFIG_HOME="$XDG_CONFIG_HOME" \
GHOSTTY_CONFIG_PATH="$CONFIG" \
GHOSTTY_LOG=stderr \
TERMSURF_GEOMETRY_TRACE=1 \
TERMSURF_GEOMETRY_SCENARIO=issue-884-girlbat-runtime \
TERMSURF_GIRLBAT_PATH="$GIRLBAT" \
TERMSURF_WEBTUI_STATE_TRACE_FILE="$WEBTUI_TRACE" \
  "$APP_BIN" >"$APP_LOG" 2>&1 &
PID="$!"
log "pid=$PID"

wait_for_pattern "TermSurf message decoded type=HelloRequest" "WebTUI connected to Astrohacker Terminal"
wait_for_pattern "SetOverlay: pane_id=.* profile=default browser=girlbat url=${URL}" "SetOverlay names Girlbat"
wait_for_literal "SetOverlay: named browser resolved browser=girlbat env=TERMSURF_GIRLBAT_PATH path=${GIRLBAT}" "Astrohacker Terminal resolved named Girlbat"
wait_for_pattern "spawned browser path=${GIRLBAT} pid=[0-9]+ profile=default browser=girlbat .*render_surface_service=com\\.termsurf\\.girlbat\\.render\\." "Astrohacker Terminal spawned Girlbat with render side-channel"
wait_for_literal "[Girlbat] render side-channel global connected=true" "Girlbat connected render side-channel"
wait_for_pattern "ServerRegister: profile=default browser=girlbat" "Astrohacker Terminal registered Girlbat"
wait_for_pattern "TabReady: pane_id=.* tab_id=[0-9]+" "Astrohacker Terminal mapped Girlbat TabReady"
wait_for_pattern "BrowserReady: pane_id=.* tab_id=[0-9]+ socket=.* browser=girlbat" "Astrohacker Terminal sent BrowserReady for Girlbat"
wait_for_pattern "\\[Girlbat\\] engine load finished tab_id=[0-9]+ url=${URL}" "Girlbat finished normal HTTP page load" 90
wait_for_pattern "\\[Girlbat\\] engine RenderSurface metadata sent_to=[1-9][0-9]* tab_id=[0-9]+ generation=[0-9]+ attachment_id=[1-9][0-9]*" "Girlbat emitted nonzero RenderSurface metadata"
wait_for_pattern "RenderSurface: tab_id=[0-9]+ pane_id=.* generation=[0-9]+ pixel=[1-9][0-9]*x[1-9][0-9]* .* attachment_id=[1-9][0-9]*" "Astrohacker Terminal received matched Girlbat RenderSurface"
wait_for_pattern "layer=bridge event=present_iosurface_target_found .*attachment_id=[1-9][0-9]*" "Bridge targeted Girlbat IOSurface"
wait_for_pattern "layer=appkit event=presented_iosurface .*context_id=[1-9][0-9]* .*visible=true" "AppKit structurally presented Girlbat IOSurface"
wait_for_pattern "layer=appkit event=presented_iosurface_pixels .*attachment_id=[1-9][0-9]* .*visible=true note=reported-presented-iosurface-pixels" "AppKit reported structural IOSurface presentation pixels"

if grep -E "RendererCrashed|engine render surface export failed|render surface send skipped" "$APP_LOG" >/dev/null 2>&1; then
  fail "unexpected Girlbat crash or render-surface failure; see $APP_LOG"
fi

log "PASS: girlbat Astrohacker Terminal runtime structural presentation smoke"
