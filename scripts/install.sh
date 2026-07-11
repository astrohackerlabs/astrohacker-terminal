#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$REPO_DIR/rust"
BUN_DIR="$REPO_DIR/bun"
CHROMIUM_OUT="$REPO_DIR/forks/chromium/src/out/Default"
source "$SCRIPT_DIR/chromium-resources.sh"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister"
AHT_RELEASE_APP="$REPO_DIR/forks/ghostty/macos/build/Release/Astrohacker Terminal.app"
APPLICATIONS_DIR="${TERMSURF_APPLICATIONS_DIR:-/Applications}"
CHROMIUMD_INSTALL_DIR="${ASTROHACKER_CHROMIUM_INSTALL_DIR:-/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd}"
GTUI_BIN_DIR="${TERMSURF_GTUI_BIN_DIR:-/usr/local/bin}"
GTUI_INSTALL_DIR="${TERMSURF_GTUI_INSTALL_DIR:-/usr/local/share/termsurf/gtui}"

COMPONENT="${1:-}"

if [ -z "$COMPONENT" ]; then
  echo "Usage: $0 <component>"
  echo "Components: ahterm (alias aht), ah-chromiumd, webtui, gtui, all"
  exit 1
fi

case "$COMPONENT" in
  ah-chromiumd | aht | webtui | gtui | all) ;;
  *)
    echo "Unknown component: $COMPONENT"
    echo "Components: ahterm (alias aht), ah-chromiumd, webtui, gtui, all"
    exit 1
    ;;
esac

if [ "$COMPONENT" = "aht" ] && [ ! -x "$AHT_RELEASE_APP/Contents/MacOS/ahterm" ]; then
  echo "Error: Release app not found at $AHT_RELEASE_APP"
  echo "Run: scripts/build.sh aht --release"
  exit 1
fi

needs_root() {
  if [ "$COMPONENT" = "ah-chromiumd" ] && [ "$CHROMIUMD_INSTALL_DIR" != "/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd" ]; then
    mkdir -p "$CHROMIUMD_INSTALL_DIR" || {
      echo "Error: ASTROHACKER_CHROMIUM_INSTALL_DIR is not writable: $CHROMIUMD_INSTALL_DIR"
      exit 1
    }
    [ -w "$CHROMIUMD_INSTALL_DIR" ] && return 1
    echo "Error: ASTROHACKER_CHROMIUM_INSTALL_DIR is not writable: $CHROMIUMD_INSTALL_DIR"
    exit 1
  fi
  if [ "$COMPONENT" = "aht" ] && [ "$APPLICATIONS_DIR" != "/Applications" ]; then
    mkdir -p "$APPLICATIONS_DIR" || {
      echo "Error: TERMSURF_APPLICATIONS_DIR is not writable: $APPLICATIONS_DIR"
      exit 1
    }
    [ -w "$APPLICATIONS_DIR" ] && return 1
    echo "Error: TERMSURF_APPLICATIONS_DIR is not writable: $APPLICATIONS_DIR"
    exit 1
  fi
  if [ "$COMPONENT" = "gtui" ]; then
    mkdir -p "$GTUI_BIN_DIR" "$GTUI_INSTALL_DIR" 2>/dev/null || return 0
    [ -w "$GTUI_BIN_DIR" ] && [ -w "$GTUI_INSTALL_DIR" ] && return 1
    return 0
  fi
  return 0
}

# Re-exec as root so we only prompt for the password once.
if [ "$(id -u)" -ne 0 ] && needs_root; then
  exec sudo env \
    TERMSURF_APPLICATIONS_DIR="$APPLICATIONS_DIR" \
    ASTROHACKER_CHROMIUM_INSTALL_DIR="$CHROMIUMD_INSTALL_DIR" \
    TERMSURF_GTUI_BIN_DIR="$GTUI_BIN_DIR" \
    TERMSURF_GTUI_INSTALL_DIR="$GTUI_INSTALL_DIR" \
    "$0" "$@"
fi

install_chromiumd() {
  local CHROMIUMD_SRC="$RUST_DIR/target/release/ah-chromiumd"
  local INSTALL_DIR="$CHROMIUMD_INSTALL_DIR"

  if [ ! -f "$CHROMIUMD_SRC" ]; then
    echo "Error: Release build not found at $CHROMIUMD_SRC"
    echo "Run: scripts/build.sh chromium --release"
    exit 1
  fi

  echo "==> Installing ah-chromiumd to $INSTALL_DIR..."
  mkdir -p "$INSTALL_DIR"
  cp "$CHROMIUMD_SRC" "$INSTALL_DIR/ah-chromiumd"

  copy_chromium_runtime_resources "$CHROMIUM_OUT" "$INSTALL_DIR"

  echo "==> Codesigning ah-chromiumd..."
  codesign --force --sign - "$INSTALL_DIR/ah-chromiumd" || true

  # Clean up old install locations.
  rm -rf /usr/local/chromium
  rm -f /usr/local/bin/chromium
  rm -rf /usr/local/lib/chromium
  rm -rf /opt/homebrew/opt/astrohacker-terminal-chromium

  echo "  Dir: $INSTALL_DIR"
  echo "  Bin: $INSTALL_DIR/ah-chromiumd"
}

install_aht() {
  local APP_SRC="$AHT_RELEASE_APP"
  local APP_DIR="/Applications"
  if [ "$COMPONENT" = "aht" ]; then
    APP_DIR="$APPLICATIONS_DIR"
  fi
  local APP="$APP_DIR/Astrohacker Terminal.app"

  if [ ! -x "$APP_SRC/Contents/MacOS/ahterm" ]; then
    echo "Error: Release app not found at $APP_SRC"
    echo "Run: scripts/build.sh aht --release"
    exit 1
  fi

  echo "==> Installing Astrohacker Terminal to $APP..."
  rm -rf "$APP"
  cp -R "$APP_SRC" "$APP"

  echo "==> Codesigning..."
  codesign --force --deep --sign - "$APP" || true

  if [ -x "$LSREGISTER" ]; then
    "$LSREGISTER" -f -R -trusted "$APP" || true
  fi

  echo "  App: $APP"
}

install_webtui() {
  local WEB="$RUST_DIR/target/release/ahweb"

  if [ ! -f "$WEB" ]; then
    echo "Error: Release build not found at $WEB"
    echo "Run: scripts/build.sh webtui --release"
    exit 1
  fi

  echo "==> Installing webtui to /usr/local/bin/ahweb..."
  cp "$WEB" /usr/local/bin/ahweb
  codesign --force --sign - /usr/local/bin/ahweb || true

  echo "  Bin: /usr/local/bin/ahweb"
}

install_gtui() {
  local TERMSURF_CLI="$RUST_DIR/target/release/ahapp"

  if [ ! -f "$TERMSURF_CLI" ]; then
    echo "Error: Release build not found at $TERMSURF_CLI"
    echo "Run: scripts/build.sh gtui --release"
    exit 1
  fi

  echo "==> Installing Astrohacker ahapp to $GTUI_BIN_DIR/ahapp..."
  mkdir -p "$GTUI_BIN_DIR" "$GTUI_INSTALL_DIR"
  cp "$TERMSURF_CLI" "$GTUI_BIN_DIR/ahapp"
  rm -rf "$GTUI_INSTALL_DIR/app"
  cp -R "$BUN_DIR/gtui-app" "$GTUI_INSTALL_DIR/app"
  codesign --force --sign - "$GTUI_BIN_DIR/ahapp" || true

  echo "  Bin: $GTUI_BIN_DIR/ahapp"
  echo "  App: $GTUI_INSTALL_DIR/app"
}

case "$COMPONENT" in
  ah-chromiumd) install_chromiumd ;;
  aht)          install_aht ;;
  webtui)       install_webtui ;;
  gtui)         install_gtui ;;
  all)
    install_chromiumd
    install_aht
    install_webtui
    install_gtui
    echo ""
    echo "Done (all)."
    ;;
esac
