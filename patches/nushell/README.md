# Nushell Patches

Astrohacker Shell uses a patched Nushell fork derived from Shannon. The fork
working tree is local-only under `forks/nushell`; this directory tracks the
patch archive needed to reconstruct Astrohacker Shell's Nushell changes without
importing Nushell history into the company repo.

## Current State (Issue 26071215508194)

- Upstream repository: `https://github.com/nushell/nushell`
- Upstream base policy: **latest commit on upstream `main`** (pinned at apply time)
- Upstream base commit: `0df4ca222cc713e79b6b1684ad8ccaec584ce4ac`
- Workspace version: `0.114.1`
- Product branch: `issue-26071215508194-nushell` (or local equivalent)
- Product HEAD (after apply): `6490ac68c27a48d88b66d1011fc53ef990de9735`
- Local fork working tree: `forks/nushell`
- Issue archive: `patches/nushell/patches/issue-26071215508194/`
- Patch:
  `patches/nushell/patches/issue-26071215508194/0001-astrohacker-shell-nushell.patch`
- Reedline: path-dep `../reedline` at tip `0.49.0` (single crate identity for
  `rust/ahsh`)
- Monorepo consumer: `rust/ahsh` pins all `nu-*` / `shannon-nu-*` at `0.114.1`
  and reedline at `0.49.0` path `forks/reedline`
- Prior archives:
  - Issue 26071112000924: `patches/nushell/patches/issue-26071112000924/`
  - Issue 26070612000903: `patches/nushell/patches/issue-26070612000903/`

## Patch Contents

Bounded Shannon/Astrohacker deltas on tip:

- `shannon-nu-cli` / `shannon-nu-lsp` package naming
- path pin of `reedline` to sibling `forks/reedline`
- `ModeDispatcher` support
- Bash syntax highlighting (`bash_highlight.rs` + tree-sitter deps)
- REPL mode-switch glue: binary toggle `nu` ↔ `$env.SHANNON_ALT_MODE`
  (`bash`|`zsh`, default `zsh` if unset)
- Highlighter treats `bash` and `zsh` modes with bash highlighter
- crate-root re-exports of `NuHighlight` / `Print` from `commands`
- related lockfile updates for the path reedline

## Apply (clean base)

```sh
BASE=0df4ca222cc713e79b6b1684ad8ccaec584ce4ac
# Reedline tip must exist at forks/reedline (path dep)
git -C forks/nushell fetch origin "$BASE"
git -C forks/nushell checkout -B issue-26071215508194-nushell "$BASE"
git -C forks/nushell am \
  "$PWD/patches/nushell/patches/issue-26071215508194/0001-astrohacker-shell-nushell.patch"
```

## Generate

```sh
git -C forks/nushell format-patch -1 HEAD --stdout \
  > patches/nushell/patches/issue-26071215508194/0001-astrohacker-shell-nushell.patch
```

## Build / verify

```sh
# after forks/nushell + forks/reedline are at recorded SHAs
scripts/build.sh ahsh --release
rust/ahsh/target/release/ahsh --version
(cd rust/ahsh && cargo test --lib)
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
