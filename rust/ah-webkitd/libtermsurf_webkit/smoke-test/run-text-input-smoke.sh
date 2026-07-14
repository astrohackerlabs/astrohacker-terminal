#!/usr/bin/env bash
set -euo pipefail

SMOKE_ROOT="$(cd "$(dirname "$0")" && pwd)"
LIB_ROOT="$(cd "$SMOKE_ROOT/.." && pwd)"
REPO_ROOT="$(cd "$LIB_ROOT/../../.." && pwd)"
CONFIGURATION="${TERMSURF_WEBKIT_CONFIGURATION:-Release}"
WEBKIT_BUILD="$REPO_ROOT/forks/webkit/src/WebKitBuild/$CONFIGURATION"

if ! test -d "$WEBKIT_BUILD/WebKit.framework"; then
  echo "TEXT_SMOKE_FAIL missing_webkit_framework path=$WEBKIT_BUILD/WebKit.framework" >&2
  exit 1
fi

TERMSURF_WEBKIT_CONFIGURATION="$CONFIGURATION" "$LIB_ROOT/build.sh"
FIXTURE="file://$REPO_ROOT/rust/test-content/text-input.html"
DYLD_FRAMEWORK_PATH="$WEBKIT_BUILD${DYLD_FRAMEWORK_PATH:+:$DYLD_FRAMEWORK_PATH}" \
  "$LIB_ROOT/build/text-input-smoke" "$FIXTURE?peer=A" "$FIXTURE?peer=B"
