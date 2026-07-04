# AGENTS.md

Guidance for coding agents working in Astrohacker Terminal support material.

This directory contains Astrohacker-owned Terminal product material that is not
itself a Rust crate or Bun package: docs, assets, scripts, public-source notes,
and legal/project metadata imported from the old Terminal repo.

Fork source does not live here. Large upstream working trees stay under
top-level `forks/`, and patch archives stay under top-level `patches/`.

## Hygiene

- Do not add build logs, screenshots from ad hoc test runs, app bundles,
  dependency directories, or fork working trees.
- Prefer updating scripts to use the new `rust/`, `bun/`, `forks/`, and
  `patches/` paths before relying on them for verification.
- When adding an `AGENTS.md` in a subdirectory, also add a relative
  `CLAUDE.md` symlink to it.
