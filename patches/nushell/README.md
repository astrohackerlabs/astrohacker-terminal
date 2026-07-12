# Nushell Patches

Astrohacker Shell uses a patched Nushell fork derived from Shannon. The fork
working tree is local-only under `forks/nushell`; this directory tracks the
patch archive needed to reconstruct Astrohacker Shell's Nushell changes without
importing Nushell history into the company repo.

## Current State (Issue 26071112000924)

- Upstream repository: `https://github.com/nushell/nushell`
- Upstream base policy: **latest commit on upstream `main`**
- Upstream base commit: `0df4ca222cc713e79b6b1684ad8ccaec584ce4ac`
- Workspace version: `0.114.1`
- Product branch: `issue-26071112000924-nushell`
- Product HEAD: `2b93f789303216bde48de1c0627b3c1f9f2dc679`
- Local fork working tree: `forks/nushell`
- Issue archive: `patches/nushell/patches/issue-26071112000924/`
- Patch:
  `patches/nushell/patches/issue-26071112000924/0001-astrohacker-shell-nushell.patch`
- Branch convention: `issue-26071112000924-nushell`
- Reedline: path-dep `../reedline` at tip `0.49.0` (single crate identity for
  `rust/ahsh`)
- Monorepo consumer: `rust/ahsh` pins all `nu-*` / `shannon-nu-*` at `0.114.1`
  and reedline at `0.49.0` path `forks/reedline`
- Prior archive (Issue 26070612000903): tag `0.113.1` /
  `7b7df4aa68e957cf38b9d8157c35fa7523f44a6d`,
  `patches/nushell/patches/issue-26070612000903/`

## Patch Contents

Bounded Shannon/Astrohacker deltas on tip:

- `shannon-nu-cli` / `shannon-nu-lsp` package naming
- path pin of `reedline` to sibling `forks/reedline`
- `ModeDispatcher` support
- Bash syntax highlighting (`bash_highlight.rs` + tree-sitter deps)
- REPL mode-switch glue
- crate-root re-exports of `NuHighlight` / `Print` from `commands`
- related lockfile updates for the path reedline

## Apply (clean base)

```sh
BASE=0df4ca222cc713e79b6b1684ad8ccaec584ce4ac
# Reedline tip must exist at forks/reedline (path dep)
git -C forks/nushell worktree add /tmp/ahsh-nushell-0924 "$BASE"
git -C /tmp/ahsh-nushell-0924 am \
  "$PWD/patches/nushell/patches/issue-26071112000924/0001-astrohacker-shell-nushell.patch"
```

## Generate

```sh
git -C forks/nushell format-patch -1 HEAD --stdout \
  > patches/nushell/patches/issue-26071112000924/0001-astrohacker-shell-nushell.patch
```

## Build / verify

```sh
# after forks/nushell + forks/reedline are at recorded SHAs
scripts/build.sh ahsh --release
rust/ahsh/target/release/ahsh --version
# Cargo resolution proof (no dual reedline, versions 0.114.1 / 0.49.0)
(cd rust/ahsh && cargo metadata --format-version 1 >/dev/null)
```

## Merge-upstream checklist

1. Discover tip: `git ls-remote https://github.com/nushell/nushell.git refs/heads/main`
2. Also tip Reedline (see `patches/reedline/README.md`).
3. Branch from Nushell tip; `git am` archive; resolve conflicts (especially
   package renames and `lib.rs` module layout after upstream moves).
4. Keep workspace `reedline` as `path = "../reedline"` so `ahsh` shares one
   Completer trait implementation.
5. Bump `rust/ahsh/Cargo.toml` (+ lockfile) to match workspace versions.
6. Build `ahsh`; regenerate archive; update this README.

Do not commit `forks/nushell` or temporary worktrees to the Astrohacker repo.
