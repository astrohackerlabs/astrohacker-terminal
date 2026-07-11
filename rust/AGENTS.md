# AGENTS.md

Guidance for coding agents working in the Astrohacker Rust workspace.

This directory contains Astrohacker-owned Rust and native support code for
Astrohacker Terminal. Fork working trees live outside this directory under
top-level `forks/`; fork changes are tracked as patches under top-level
`patches/`.

## Commands

Run Rust workspace commands from this directory:

```sh
cargo metadata --no-deps
cargo check --workspace
cargo build --workspace
```

Some crates link against fork build artifacts and may require Chromium, WebKit,
Ladybird, or Ghostty support libraries to be built first. Do not vendor those
fork working trees here.

## Hygiene

- Keep build outputs out of git: `target/`, C/C++ build directories, generated
  app bundles, xcframeworks, logs, and caches are not source.
- Keep path fixes scoped and document any temporary compatibility path in the
  issue experiment that introduces it.
- When a subdirectory needs agent guidance, add an `AGENTS.md` there.
