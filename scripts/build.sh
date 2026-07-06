#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
COMPANY_DIR="$REPO_DIR"
RUST_DIR="$COMPANY_DIR/rust"
CHROMIUM_SRC="$COMPANY_DIR/forks/chromium/src"
CHROMIUM_OUT="$CHROMIUM_SRC/out/Default"
CHROMIUM_PROTOC="$CHROMIUM_OUT/protoc"
WEBKIT_SRC="$COMPANY_DIR/forks/webkit/src"
SURFARI_LIB_DIR="$RUST_DIR/surfari/libtermsurf_webkit"
GIRLBAT_LIB_DIR="$RUST_DIR/girlbat/libtermsurf_ladybird"
GHOSTTY_DIR="$COMPANY_DIR/forks/ghostty"
HELIX_DIR="$COMPANY_DIR/forks/helix"

RELEASE=false
CLEAN=false
OPEN=false
PRINT_PATHS=false
COMPONENT=""

usage() {
  echo "Usage: $0 <component> [--release] [--clean] [--open]"
  echo "Components: aht, ahsh, ahe, roamium, webtui, gtui, chromium, webkit, surfari-lib, surfari, girlbat-lib, girlbat, all"
}

configuration() {
  if $RELEASE; then
    echo "Release"
  else
    echo "Debug"
  fi
}

for arg in "$@"; do
  case "$arg" in
    --print-paths) PRINT_PATHS=true ;;
    --release) RELEASE=true ;;
    --clean)   CLEAN=true ;;
    --open)    OPEN=true ;;
    -*)
      echo "Unknown flag: $arg"
      usage
      exit 1
      ;;
    *)
      if [ -z "$COMPONENT" ]; then
        COMPONENT="$arg"
      else
        echo "Error: multiple components specified"
        exit 1
      fi
      ;;
  esac
done

if $PRINT_PATHS; then
  printf 'SCRIPT_DIR=%s\n' "$SCRIPT_DIR"
  printf 'REPO_DIR=%s\n' "$REPO_DIR"
  printf 'COMPANY_DIR=%s\n' "$COMPANY_DIR"
  printf 'RUST_DIR=%s\n' "$RUST_DIR"
  printf 'CHROMIUM_SRC=%s\n' "$CHROMIUM_SRC"
  printf 'WEBKIT_SRC=%s\n' "$WEBKIT_SRC"
  printf 'GHOSTTY_DIR=%s\n' "$GHOSTTY_DIR"
  printf 'HELIX_DIR=%s\n' "$HELIX_DIR"
  exit 0
fi

if [ -z "$COMPONENT" ]; then
  usage
  exit 1
fi

# Export PROTOC from Chromium if available (needed by prost_build).
if [ -x "$CHROMIUM_PROTOC" ]; then
  export PROTOC="$CHROMIUM_PROTOC"
fi

build_chromium() {
  if [ ! -d "$CHROMIUM_SRC" ]; then
    echo "==> Skipping Chromium ($CHROMIUM_SRC not found)"
    return
  fi
  export PATH="$COMPANY_DIR/forks/chromium/depot_tools:$PATH"
  cd "$CHROMIUM_SRC"
  if $CLEAN; then
    echo "==> Cleaning Chromium..."
    gn clean out/Default
  fi
  echo "==> Building Chromium..."
  autoninja -C out/Default libtermsurf_chromium
  echo "  Chromium: $CHROMIUM_OUT"
}

build_webtui() {
  cd "$RUST_DIR"
  if $CLEAN; then
    echo "==> Cleaning webtui..."
    cargo clean -p webtui
  fi
  if $RELEASE; then
    echo "==> Building webtui (release)..."
    cargo build --release -p webtui
    echo "  webtui: $RUST_DIR/target/release/web"
  else
    echo "==> Building webtui (debug)..."
    cargo build -p webtui
    echo "  webtui: $RUST_DIR/target/debug/web"
  fi
}

build_gtui() {
  cd "$RUST_DIR"
  if $CLEAN; then
    echo "==> Cleaning gtui..."
    cargo clean -p gtui
  fi
  if $RELEASE; then
    echo "==> Building gtui (release)..."
    cargo build --release -p gtui
    echo "  gtui: $RUST_DIR/target/release/termsurf"
  else
    echo "==> Building gtui (debug)..."
    cargo build -p gtui
    echo "  gtui: $RUST_DIR/target/debug/termsurf"
  fi
}

build_ahsh() {
  local AHSH_DIR="$RUST_DIR/ahsh"
  if [ ! -d "$COMPANY_DIR/forks/nushell" ]; then
    echo "Missing Nushell fork checkout: $COMPANY_DIR/forks/nushell" >&2
    echo "Reconstruct it from patches/nushell before building ahsh." >&2
    exit 1
  fi
  if [ ! -d "$COMPANY_DIR/forks/reedline" ]; then
    echo "Missing Reedline fork checkout: $COMPANY_DIR/forks/reedline" >&2
    echo "Reconstruct it from patches/reedline before building ahsh." >&2
    exit 1
  fi
  cd "$AHSH_DIR"
  if $CLEAN; then
    echo "==> Cleaning ahsh..."
    cargo clean
  fi
  if $RELEASE; then
    echo "==> Building ahsh (release)..."
    cargo build --release
    echo "  ahsh: $AHSH_DIR/target/release/ahsh"
  else
    echo "==> Building ahsh (debug)..."
    cargo build
    echo "  ahsh: $AHSH_DIR/target/debug/ahsh"
  fi
}

build_ahe() {
  if [ ! -d "$HELIX_DIR" ]; then
    echo "Missing Helix fork checkout: $HELIX_DIR" >&2
    echo "Reconstruct it from patches/helix before building ahe." >&2
    exit 1
  fi
  if [ ! -f "$HELIX_DIR/helix-term/Cargo.toml" ]; then
    echo "Invalid Helix fork checkout: $HELIX_DIR" >&2
    echo "Expected helix-term/Cargo.toml under forks/helix." >&2
    exit 1
  fi
  cd "$HELIX_DIR"
  if $CLEAN; then
    echo "==> Cleaning ahe..."
    cargo clean
  fi
  if $RELEASE; then
    echo "==> Building ahe (release)..."
    HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1 cargo build --release -p helix-term
    echo "  ahe: $HELIX_DIR/target/release/ahe"
  else
    echo "==> Building ahe (debug)..."
    HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1 cargo build -p helix-term
    echo "  ahe: $HELIX_DIR/target/debug/ahe"
  fi
}

build_roamium() {
  cd "$RUST_DIR"
  if [ ! -d "$CHROMIUM_OUT" ]; then
    echo "Missing Chromium output directory: $CHROMIUM_OUT" >&2
    echo "Build Chromium first with: $0 chromium" >&2
    exit 1
  fi
  if $CLEAN; then
    echo "==> Cleaning Roamium..."
    cargo clean -p roamium
  fi
  if $RELEASE; then
    echo "==> Building Roamium (release)..."
    cargo build --release -p roamium
    cp "$RUST_DIR/target/release/ah-chromiumd" "$CHROMIUM_OUT/ah-chromiumd"
  else
    echo "==> Building Roamium (debug)..."
    cargo build -p roamium
    cp "$RUST_DIR/target/debug/ah-chromiumd" "$CHROMIUM_OUT/ah-chromiumd"
  fi
  echo "  Roamium: $CHROMIUM_OUT/ah-chromiumd"
}

build_webkit() {
  local CONFIGURATION
  CONFIGURATION="$(configuration)"
  local CONFIG_FLAG="--debug"
  if $RELEASE; then
    CONFIG_FLAG="--release"
  fi
  local WEBKIT_ARCH="${TERMSURF_WEBKIT_ARCH:-arm64}"
  local WEBKIT_SCOPE_FLAG="--only=WebKit"
  if [ "${TERMSURF_WEBKIT_FULL_BUILD:-0}" = "1" ]; then
    WEBKIT_SCOPE_FLAG=""
  fi
  local WEBKIT_BUILD_SETTINGS=(
    "--architecture=$WEBKIT_ARCH"
    "OVERRIDE_ENABLE_MODULE_VERIFIER=NO"
    "ENABLE_WK_LIBRARY_MODULE_VERIFIER=NO"
  )
  if $RELEASE; then
    # Xcode 26.5 can build bmalloc's Swift C++ interop module through both
    # staged and source header paths, which trips duplicate definitions.
    WEBKIT_BUILD_SETTINGS+=("WK_SWIFT_EXPLICIT_MODULES_ALLOW_CXX_INTEROP=NO")
  fi

  if [ ! -d "$WEBKIT_SRC" ]; then
    echo "==> Skipping WebKit ($WEBKIT_SRC not found)"
    return
  fi

  cd "$COMPANY_DIR"
  if $CLEAN; then
    echo "==> Cleaning WebKit ($CONFIGURATION)..."
    "$WEBKIT_SRC/Tools/Scripts/build-webkit" "$CONFIG_FLAG" --clean "${WEBKIT_BUILD_SETTINGS[@]}"
  fi

  echo "==> Building WebKit ($CONFIGURATION, $WEBKIT_ARCH)..."
  if $RELEASE && [ -n "$WEBKIT_SCOPE_FLAG" ]; then
    local WEBKIT_RELEASE_TARGETS=(
      "Everything up to WebKit"
      "WebInspectorUI"
    )
    for target in "${WEBKIT_RELEASE_TARGETS[@]}"; do
      echo "==> Building WebKit prerequisite ($target, $CONFIGURATION, $WEBKIT_ARCH)..."
      "$WEBKIT_SRC/Tools/Scripts/build-webkit" "$CONFIG_FLAG" "--only=$target" "${WEBKIT_BUILD_SETTINGS[@]}"
    done
  elif [ -n "$WEBKIT_SCOPE_FLAG" ]; then
    "$WEBKIT_SRC/Tools/Scripts/build-webkit" "$CONFIG_FLAG" "$WEBKIT_SCOPE_FLAG" "${WEBKIT_BUILD_SETTINGS[@]}"
  else
    "$WEBKIT_SRC/Tools/Scripts/build-webkit" "$CONFIG_FLAG" "${WEBKIT_BUILD_SETTINGS[@]}"
  fi
  echo "  WebKit: $WEBKIT_SRC/WebKitBuild/$CONFIGURATION"
}

build_surfari_lib() {
  local CONFIGURATION
  CONFIGURATION="$(configuration)"

  echo "==> Building libtermsurf_webkit ($CONFIGURATION)..."
  cd "$COMPANY_DIR"
  local args=("--configuration" "$CONFIGURATION")
  if $CLEAN; then
    args+=("--clean")
  fi
  "$SURFARI_LIB_DIR/build.sh" "${args[@]}"
  echo "  libtermsurf_webkit: $SURFARI_LIB_DIR/build/libtermsurf_webkit.dylib"
}

build_surfari() {
  local CONFIGURATION
  CONFIGURATION="$(configuration)"

  build_surfari_lib

  cd "$RUST_DIR"
  if $CLEAN; then
    echo "==> Cleaning Surfari..."
    cargo clean -p surfari
  fi
  if $RELEASE; then
    echo "==> Building Surfari (release)..."
    cargo build --release -p surfari
    echo "  Surfari: $RUST_DIR/target/release/ah-webkitd"
  else
    echo "==> Building Surfari (debug)..."
    cargo build -p surfari
    echo "  Surfari: $RUST_DIR/target/debug/ah-webkitd"
  fi
}

build_girlbat() {
  build_girlbat_lib

  cd "$RUST_DIR"
  if $CLEAN; then
    echo "==> Cleaning Girlbat..."
    cargo clean -p girlbat
  fi
  if $RELEASE; then
    echo "==> Building Girlbat (release)..."
    cargo build --release -p girlbat
    echo "  Girlbat: $RUST_DIR/target/release/ah-ladybirdd"
  else
    echo "==> Building Girlbat (debug)..."
    cargo build -p girlbat
    echo "  Girlbat: $RUST_DIR/target/debug/ah-ladybirdd"
  fi
}

build_girlbat_lib() {
  local CONFIGURATION
  CONFIGURATION="$(configuration)"

  echo "==> Building libtermsurf_ladybird ($CONFIGURATION)..."
  cd "$COMPANY_DIR"
  local args=("--configuration" "$CONFIGURATION")
  if $CLEAN; then
    args+=("--clean")
  fi
  if $RELEASE && [ -z "${TERMSURF_LADYBIRD_BACKEND:-}" ]; then
    TERMSURF_LADYBIRD_BACKEND=real "$GIRLBAT_LIB_DIR/build.sh" "${args[@]}"
  else
    "$GIRLBAT_LIB_DIR/build.sh" "${args[@]}"
  fi
  echo "  libtermsurf_ladybird: $GIRLBAT_LIB_DIR/build/libtermsurf_ladybird.dylib"
}

build_aht() {
  local CONFIGURATION="Debug"
  local ZIG_OPTIMIZE="Debug"
  if $RELEASE; then
    CONFIGURATION="Release"
    ZIG_OPTIMIZE="ReleaseFast"
  fi

  echo "==> Building AHTKit ($ZIG_OPTIMIZE)..."
  cd "$GHOSTTY_DIR"
  if [ -n "${TERMSURF_VERSION:-}" ]; then
    zig build -Demit-macos-app=false -Doptimize="$ZIG_OPTIMIZE" "-Dversion-string=$TERMSURF_VERSION"
  else
    zig build -Demit-macos-app=false -Doptimize="$ZIG_OPTIMIZE"
  fi

  cd "$GHOSTTY_DIR/macos"
  if $CLEAN; then
    echo "==> Cleaning AHT ($CONFIGURATION)..."
    ./build.nu --configuration "$CONFIGURATION" --action clean
  fi

  echo "==> Building AHT ($CONFIGURATION)..."
  if [ -n "${TERMSURF_VERSION:-}" ]; then
    ./build.nu --configuration "$CONFIGURATION" --action build --version "$TERMSURF_VERSION"
  else
    ./build.nu --configuration "$CONFIGURATION" --action build
  fi
  if $RELEASE; then
    codesign --force --deep --sign - "build/$CONFIGURATION/Astrohacker Terminal.app"
  fi
  echo "  AHT: $GHOSTTY_DIR/macos/build/$CONFIGURATION/Astrohacker Terminal.app"
  echo "  AHT executable: $GHOSTTY_DIR/macos/build/$CONFIGURATION/Astrohacker Terminal.app/Contents/MacOS/aht"
}

case "$COMPONENT" in
  chromium)   build_chromium ;;
  webtui)     build_webtui ;;
  gtui)       build_gtui ;;
  ahsh)       build_ahsh ;;
  ahe)        build_ahe ;;
  roamium)    build_roamium ;;
  webkit)     build_webkit ;;
  surfari-lib) build_surfari_lib ;;
  surfari)    build_surfari ;;
  girlbat-lib) build_girlbat_lib ;;
  girlbat)    build_girlbat ;;
  aht)        build_aht ;;
  all)
    build_chromium
    build_webtui
    build_gtui
    build_ahsh
    build_roamium
    build_webkit
    build_surfari
    build_girlbat
    build_aht
    echo ""
    echo "Done (all)."
    ;;
  *)
    echo "Unknown component: $COMPONENT"
    usage
    exit 1
    ;;
esac
