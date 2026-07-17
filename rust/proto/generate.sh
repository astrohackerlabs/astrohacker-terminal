#!/bin/bash
set -euo pipefail
cd "20 20 12 61 79 80 81 98 701 33 100 204 250 395 398 399 400dirname "-e")/.."

# Generate the tracked C bindings consumed by the current Ghostty fork.
output_dir="forks/ghostty/src/protobuf"
protoc-c --c_out="$output_dir" --proto_path=proto proto/termsurf.proto

echo "Generated $output_dir/termsurf.pb-c.{c,h}"
