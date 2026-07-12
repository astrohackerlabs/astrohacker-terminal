#!/usr/bin/env bash
set -euo pipefail

COMPONENT="${1:-}"
APPLICATIONS_DIR="${TERMSURF_APPLICATIONS_DIR:-/Applications}"
CHROMIUMD_INSTALL_DIR="${ASTROHACKER_CHROMIUM_INSTALL_DIR:-/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd}"
GTUI_BIN_DIR="${TERMSURF_GTUI_BIN_DIR:-/usr/local/bin}"
GTUI_INSTALL_DIR="${TERMSURF_GTUI_INSTALL_DIR:-/usr/local/share/termsurf/gtui}"

if [ -z "$COMPONENT" ]; then
  echo "Usage: $0 <component>"
  echo "Components: aht, ah-chromiumd, ahweb, ahapp, all"
  echo "Aliases: webtui→ahweb, gtui→ahapp"
  exit 1
fi

case "$COMPONENT" in
  ah-chromiumd | aht | ahweb | webtui | ahapp | gtui | all) ;;
  *)
    echo "Unknown component: $COMPONENT"
    echo "Components: aht, ah-chromiumd, ahweb, ahapp, all"
    exit 1
    ;;
esac

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

LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister"

uninstall_chromiumd() {
  echo "==> Uninstalling ah-chromiumd..."
  rm -rf "$CHROMIUMD_INSTALL_DIR"
  rm -rf /usr/local/chromium
  rm -f /usr/local/bin/chromium
  rm -rf /usr/local/lib/chromium
  rm -rf /opt/homebrew/opt/astrohacker-terminal-chromium

  echo "  Removed: $CHROMIUMD_INSTALL_DIR"
}

uninstall_aht() {
  local APP_DIR="/Applications"
  if [ "$COMPONENT" = "aht" ]; then
    APP_DIR="$APPLICATIONS_DIR"
  fi
  local APP="$APP_DIR/Astrohacker Terminal.app"

  echo "==> Uninstalling Astrohacker Terminal..."
  rm -rf "$APP"

  echo "  Removed: $APP"
}

uninstall_ahweb() {
  echo "==> Uninstalling ahweb..."
  rm -f /usr/local/bin/ahweb
  rm -f /usr/local/bin/web

  echo "  Removed: /usr/local/bin/ahweb (and legacy /usr/local/bin/web if present)"
}

uninstall_ahapp() {
  echo "==> Uninstalling ahapp..."
  rm -f "$GTUI_BIN_DIR/ahapp"
  rm -f "$GTUI_BIN_DIR/termsurf"
  rm -rf "$GTUI_INSTALL_DIR"

  echo "  Removed: $GTUI_BIN_DIR/ahapp (and legacy termsurf if present)"
  echo "  Removed: $GTUI_INSTALL_DIR"
}

case "$COMPONENT" in
  ah-chromiumd) uninstall_chromiumd ;;
  aht)          uninstall_aht ;;
  ahweb|webtui) uninstall_ahweb ;;
  ahapp|gtui)   uninstall_ahapp ;;
  all)
    uninstall_chromiumd
    uninstall_aht
    uninstall_ahweb
    uninstall_ahapp
    echo ""
    echo "Done (all)."
    ;;
esac
