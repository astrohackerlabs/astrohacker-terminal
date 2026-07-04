#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="${TERMSURF_SMOKE_VERSION:-1.4.10}"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
START_EPOCH="$(date +%s)"
LOG_DIR="$ROOT/logs/issue-872-exp1-installed-homebrew-scroll"
RUN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/termsurf-issue872-exp1.XXXXXX")"
SITE_DIR="$RUN_DIR/site"
APP="/Applications/Astrohacker Terminal.app"
APP_BIN="$APP/Contents/MacOS/ghostboard"
WEB="/opt/homebrew/bin/web"
SURFARI="/opt/homebrew/opt/astrohacker-terminal-surfari/surfari"
SURFARI_LIB="/opt/homebrew/opt/astrohacker-terminal-surfari/libtermsurf_webkit.dylib"
COMMAND="$RUN_DIR/run-web.sh"
APP_LOG="$LOG_DIR/app-$RUN_ID.log"
WEBTUI_TRACE="$LOG_DIR/webtui-$RUN_ID.log"
SCROLL_TRACE="$LOG_DIR/ghostboard-scroll-$RUN_ID.log"
SURFARI_SCROLL_TRACE="$LOG_DIR/surfari-scroll-$RUN_ID.log"
HARNESS_LOG="$LOG_DIR/harness-$RUN_ID.log"
SCREENSHOT="$LOG_DIR/screenshot-$RUN_ID.png"
PID=""
HTTP_PID=""

mkdir -p "$LOG_DIR" "$SITE_DIR"

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
  if [ -n "${PID:-}" ] && kill -0 "$PID" >/dev/null 2>&1; then
    kill "$PID" >/dev/null 2>&1 || true
    delay 0.5 || true
    kill -9 "$PID" >/dev/null 2>&1 || true
  fi
  if [ -n "${HTTP_PID:-}" ] && kill -0 "$HTTP_PID" >/dev/null 2>&1; then
    kill "$HTTP_PID" >/dev/null 2>&1 || true
  fi
  rm -rf "$RUN_DIR"
}
trap cleanup EXIT

require_executable() {
  [ -x "$1" ] || fail "missing executable: $1"
}

require_unset() {
  local name="$1"
  if [ -n "${!name+x}" ]; then
    fail "$name must be unset for installed Homebrew scroll smoke"
  fi
}

line_count() {
  local file="$1"
  if [ -r "$file" ]; then
    wc -l <"$file" | tr -d ' '
  else
    printf '0\n'
  fi
}

wait_for_file_pattern_after() {
  local file="$1"
  local start_line="$2"
  local pattern="$3"
  local label="$4"
  local attempts="${5:-60}"
  for _ in $(seq 1 "$attempts"); do
    if tail -n +"$((start_line + 1))" "$file" 2>/dev/null | grep -E "$pattern" >/dev/null 2>&1; then
      log "PASS: $label"
      return 0
    fi
    delay 1
  done
  fail "timed out waiting for $label"
}

extract_first_match() {
  local file="$1"
  local pattern="$2"
  grep -E "$pattern" "$file" | head -1 || true
}

extract_window_id() {
  printf '%s\n' "$1" | sed -E 's/.*identity=window_id:([0-9]+).*/\1/'
}

extract_frame_x() {
  printf '%s\n' "$1" | sed -E 's/.*overlay_frame=\{\{([^,]+), [^}]+\}, \{[^}]+\}\}.*/\1/'
}

extract_frame_y() {
  printf '%s\n' "$1" | sed -E 's/.*overlay_frame=\{\{[^,]+, ([^}]+)\}, \{[^}]+\}\}.*/\1/'
}

extract_root_frame_size() {
  printf '%s\n' "$1" | sed -E 's/.*root_frame=\{\{[^}]+\}, \{([^,]+), ([^}]+)\}\}.*/\1x\2/'
}

pair_height() {
  printf '%s\n' "$1" | awk -Fx '{print $2}'
}

exact_window_bounds() {
  local window_id="$1"
  swift - "$window_id" <<'SWIFT'
import CoreGraphics
import Foundation

let target = Int(CommandLine.arguments[1])!
guard let info = CGWindowListCopyWindowInfo([.optionAll], kCGNullWindowID) as? [[String: Any]] else {
    exit(1)
}

for window in info {
    guard let id = window[kCGWindowNumber as String] as? Int, id == target else { continue }
    let bounds = (window[kCGWindowBounds as String] as? [String: Any]) ?? [:]
    let x = Int((bounds["X"] as? Double) ?? 0)
    let y = Int((bounds["Y"] as? Double) ?? 0)
    let width = Int((bounds["Width"] as? Double) ?? 0)
    let height = Int((bounds["Height"] as? Double) ?? 0)
    print("\(id)\t\(x)\t\(y)\t\(width)\t\(height)")
    exit(0)
}

exit(1)
SWIFT
}

activate_pid() {
  local pid="$1"
  local label="$2"
  local front_pid
  front_pid="$(osascript \
    -e 'tell application "System Events" to set frontmost of first process whose unix id is '"$pid"' to true' \
    -e 'delay 0.25' \
    -e 'tell application "System Events" to unix id of first process whose frontmost is true')"
  if [ "$front_pid" != "$pid" ]; then
    fail "$label frontmost PID mismatch: got=$front_pid expected=$pid"
  fi
  log "PASS: $label frontmost pid=$front_pid"
}

global_point_for_web_point() {
  local win_line="$1"
  local present_line="$2"
  local web_x="$3"
  local web_y="$4"
  local _wid wx wy _ww wh frame_x frame_y root_frame_size root_height content_y_offset
  IFS=$'\t' read -r _wid wx wy _ww wh <<<"$win_line"
  frame_x="$(extract_frame_x "$present_line")"
  frame_y="$(extract_frame_y "$present_line")"
  root_frame_size="$(extract_root_frame_size "$present_line")"
  root_height="$(pair_height "$root_frame_size")"
  content_y_offset="$(awk -v wh="$wh" -v root_h="$root_height" 'BEGIN { print int(wh - root_h) }')"
  awk \
    -v wx="$wx" \
    -v wy="$wy" \
    -v content_y="$content_y_offset" \
    -v frame_x="$frame_x" \
    -v frame_y="$frame_y" \
    -v web_x="$web_x" \
    -v web_y="$web_y" \
    'BEGIN {
      print int(wx + frame_x + web_x + 0.5) "\t" int(wy + content_y + frame_y + web_y + 0.5)
    }'
}

require_unset TERMSURF_ROAMIUM_PATH
require_unset TERMSURF_SURFARI_PATH
require_unset TERMSURF_INSTALLED_ROAMIUM_PATH
require_unset TERMSURF_INSTALLED_SURFARI_PATH
require_unset DYLD_FRAMEWORK_PATH

require_executable "$APP_BIN"
require_executable "$WEB"
require_executable "$SURFARI"
[ -f "$SURFARI_LIB" ] || fail "missing Surfari library: $SURFARI_LIB"

CLI_VERSION="$("$APP_BIN" +version 2>&1 | sed -n '1p')"
[ "$CLI_VERSION" = "Astrohacker Terminal $VERSION" ] || fail "CLI version mismatch: $CLI_VERSION"

HTTP_PORT="$(python3 - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.bind(("127.0.0.1", 0))
    print(s.getsockname()[1])
PY
)"
URL="http://127.0.0.1:${HTTP_PORT}/index.html"

cat >"$SITE_DIR/index.html" <<'HTML'
<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Issue 872 Installed Scroll Fixture</title>
    <style>
      html,
      body {
        margin: 0;
        min-height: 2400px;
        background: white;
        color: #111;
        font: 18px -apple-system, BlinkMacSystemFont, sans-serif;
      }

      header {
        height: 140px;
        padding: 32px;
        background: #e9eef5;
      }

      #nested {
        position: absolute;
        left: 80px;
        top: 260px;
        width: 460px;
        height: 260px;
        overflow: auto;
        border: 2px solid #333;
      }

      #nested-content {
        height: 1400px;
        padding: 24px;
        background: linear-gradient(#f8fafc, #dbeafe);
      }

      #body-marker {
        position: absolute;
        left: 80px;
        top: 900px;
      }
    </style>
  </head>
  <body>
    <header>
      <h1>Issue 872 installed scroll fixture</h1>
      <p>Body and nested scroll regions for Homebrew Surfari.</p>
    </header>
    <section id="nested">
      <div id="nested-content">
        <p>Nested scroll content starts here.</p>
        <p style="margin-top: 900px">Nested scroll content ends here.</p>
      </div>
    </section>
    <p id="body-marker">Body scroll marker.</p>
  </body>
</html>
HTML

python3 -m http.server "$HTTP_PORT" --bind 127.0.0.1 --directory "$SITE_DIR" >>"$HARNESS_LOG" 2>&1 &
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

cat >"$COMMAND" <<EOF
#!/usr/bin/env bash
set -euo pipefail
export TERMSURF_WEBTUI_STATE_TRACE_FILE="$WEBTUI_TRACE"
exec "$WEB" --browser surfari "$URL"
EOF
chmod +x "$COMMAND"

log "run_id=$RUN_ID"
log "version=$VERSION"
log "started_at_epoch=$START_EPOCH"
log "app_bin=$APP_BIN"
log "web=$WEB"
log "surfari=$SURFARI"
log "surfari_lib=$SURFARI_LIB"
log "url=$URL"
log "app_log=$APP_LOG"
log "webtui_trace=$WEBTUI_TRACE"
log "scroll_trace=$SCROLL_TRACE"
log "surfari_scroll_trace=$SURFARI_SCROLL_TRACE"

env \
  -u TERMSURF_ROAMIUM_PATH \
  -u TERMSURF_SURFARI_PATH \
  -u TERMSURF_INSTALLED_ROAMIUM_PATH \
  -u TERMSURF_INSTALLED_SURFARI_PATH \
  -u DYLD_FRAMEWORK_PATH \
  GHOSTTY_LOG=stderr \
  TERMSURF_GEOMETRY_TRACE=1 \
  TERMSURF_GEOMETRY_SCENARIO="issue872-exp1-installed-scroll" \
  TERMSURF_SCROLL_TRACE=1 \
  TERMSURF_SCROLL_TRACE_FILE="$SCROLL_TRACE" \
  TERMSURF_SURFARI_SCROLL_TRACE=1 \
  TERMSURF_SURFARI_SCROLL_TRACE_FILE="$SURFARI_SCROLL_TRACE" \
  "$APP_BIN" \
  --window-save-state=never \
  --confirm-close-surface=false \
  --initial-command="direct:$COMMAND" >"$APP_LOG" 2>&1 &
PID="$!"
log "pid=$PID"

START_LINE="$(line_count "$APP_LOG")"
wait_for_file_pattern_after "$APP_LOG" "$START_LINE" "SetOverlay: pane_id=.* browser=surfari url=${URL}" "web requested surfari overlay" 90
wait_for_file_pattern_after "$APP_LOG" "$START_LINE" "SetOverlay: named browser resolved browser=surfari installed_path=${SURFARI}" "surfari resolved to installed Homebrew binary" 90
wait_for_file_pattern_after "$APP_LOG" "$START_LINE" "browser spawn runtime env browser=surfari DYLD_FRAMEWORK_PATH=/opt/homebrew/opt/astrohacker-terminal-surfari" "Ghostboard supplied installed Surfari runtime" 90
wait_for_file_pattern_after "$APP_LOG" "$START_LINE" "spawned browser path=${SURFARI} .* browser=surfari " "Ghostboard spawned installed Surfari binary" 90
wait_for_file_pattern_after "$APP_LOG" "$START_LINE" "BrowserReady: pane_id=.* browser=surfari" "Ghostboard emitted surfari BrowserReady" 160
wait_for_file_pattern_after "$APP_LOG" "$START_LINE" "TermSurf geometry layer=appkit event=presented " "AppKit presented overlay" 90

BROWSER_READY_LINE="$(extract_first_match "$APP_LOG" "BrowserReady: pane_id=.* browser=surfari")"
PANE_ID="$(printf '%s\n' "$BROWSER_READY_LINE" | sed -E 's/.*pane_id=([^ ]+) tab_id=.*/\1/')"
BROWSER_TAB_ID="$(printf '%s\n' "$BROWSER_READY_LINE" | sed -E 's/.*tab_id=([0-9]+) socket=.*/\1/')"
case "$PANE_ID" in
  '' | "$BROWSER_READY_LINE") fail "could not extract pane id from BrowserReady: $BROWSER_READY_LINE" ;;
esac
case "$BROWSER_TAB_ID" in
  '' | *[!0-9]*) fail "could not extract tab id from BrowserReady: $BROWSER_READY_LINE" ;;
esac

PRESENTED_LINE="$(extract_first_match "$APP_LOG" "TermSurf geometry layer=appkit event=presented .*pane_id:${PANE_ID}")"
[ -n "$PRESENTED_LINE" ] || fail "missing AppKit presented line for pane $PANE_ID"
WINDOW_ID="$(extract_window_id "$PRESENTED_LINE")"
WIN_LINE="$(exact_window_bounds "$WINDOW_ID")" || fail "failed to resolve presented window bounds"
log "pane_id=$PANE_ID"
log "browser_tab_id=$BROWSER_TAB_ID"
log "presented_window_bounds=$WIN_LINE"

activate_pid "$PID" "pre-browse Ghostboard activation"
MODE_START="$(line_count "$APP_LOG")"
swift "$ROOT/scripts/ghostty-app/inject.swift" key 36 >>"$HARNESS_LOG" 2>&1
wait_for_file_pattern_after "$APP_LOG" "$MODE_START" "ModeChanged: pane_id=${PANE_ID} browsing=true" "webtui entered Browse mode" 45
activate_pid "$PID" "post-browse Ghostboard activation"

read -r BODY_X BODY_Y <<<"$(global_point_for_web_point "$WIN_LINE" "$PRESENTED_LINE" 650 350)"
read -r NESTED_X NESTED_Y <<<"$(global_point_for_web_point "$WIN_LINE" "$PRESENTED_LINE" 220 390)"
log "points body=${BODY_X},${BODY_Y} nested=${NESTED_X},${NESTED_Y}"

BODY_START="$(line_count "$SCROLL_TRACE")"
SURFARI_BODY_START="$(line_count "$SURFARI_SCROLL_TRACE")"
swift "$ROOT/scripts/ghostty-app/inject.swift" scroll-pixel "$BODY_X" "$BODY_Y" 240 >>"$HARNESS_LOG" 2>&1
wait_for_file_pattern_after "$SCROLL_TRACE" "$BODY_START" "ghostboard-scroll .*pane_id=${PANE_ID} .*hit=true .*forwarded=true" "Surfari body webview scroll hit" 30
wait_for_file_pattern_after "$SURFARI_SCROLL_TRACE" "$SURFARI_BODY_START" "surfari-scroll .*dispatch_mode=window-send-event" "Surfari body scroll used window dispatch" 30
if tail -n +"$((BODY_START + 1))" "$SCROLL_TRACE" 2>/dev/null | grep -E "fallback=terminal" >/dev/null 2>&1; then
  fail "Surfari body webview scroll fell back to terminal"
fi
log "PASS: Surfari body scroll forwarded without terminal fallback"

NESTED_START="$(line_count "$SCROLL_TRACE")"
SURFARI_NESTED_START="$(line_count "$SURFARI_SCROLL_TRACE")"
swift "$ROOT/scripts/ghostty-app/inject.swift" scroll-pixel "$NESTED_X" "$NESTED_Y" 240 >>"$HARNESS_LOG" 2>&1
wait_for_file_pattern_after "$SCROLL_TRACE" "$NESTED_START" "ghostboard-scroll .*pane_id=${PANE_ID} .*hit=true .*forwarded=true" "Surfari nested webview scroll hit" 30
wait_for_file_pattern_after "$SURFARI_SCROLL_TRACE" "$SURFARI_NESTED_START" "surfari-scroll .*dispatch_mode=window-send-event" "Surfari nested scroll used window dispatch" 30
if tail -n +"$((NESTED_START + 1))" "$SCROLL_TRACE" 2>/dev/null | grep -E "fallback=terminal" >/dev/null 2>&1; then
  fail "Surfari nested webview scroll fell back to terminal"
fi
log "PASS: Surfari nested scroll forwarded without terminal fallback"

screencapture -x -o -l"$WINDOW_ID" "$SCREENSHOT"
[ -s "$SCREENSHOT" ] || fail "screenshot not written: $SCREENSHOT"
log "PASS: screenshot=$SCREENSHOT"

FINISH_EPOCH="$(date +%s)"
DURATION="$((FINISH_EPOCH - START_EPOCH))"
log "finished_at_epoch=$FINISH_EPOCH"
log "duration_seconds=$DURATION"
log "PASS: issue 872 installed Homebrew Surfari scroll smoke"
