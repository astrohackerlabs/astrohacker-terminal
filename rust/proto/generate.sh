#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

repo_root="$(cd .. && pwd)"
ghostty_dir="${TERMSURF_GHOSTTY_DIR:-$repo_root/forks/ghostty}"
if [[ ! -d "$ghostty_dir/src/protobuf" ]]; then
  echo "Ghostty protobuf directory not found: $ghostty_dir/src/protobuf" >&2
  exit 1
fi

# Generate C code from the proto schema.
protoc-c --c_out="$ghostty_dir/src/protobuf" --proto_path=proto proto/termsurf.proto

echo "Generated $ghostty_dir/src/protobuf/termsurf.pb-c.{c,h}"
