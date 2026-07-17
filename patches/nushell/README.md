# Nushell Patches

Astrohacker Shell uses a patched Nushell fork derived from Shannon. The fork
working tree is local-only under `forks/nushell`; this directory tracks the
patch archive needed to reconstruct Astrohacker Shell's Nushell changes without
importing Nushell history into the company repo.

## Current State (Issue 26071420489654)

- Upstream repository: `https://github.com/nushell/nushell`
- Upstream base commit: `0df4ca222cc713e79b6b1684ad8ccaec584ce4ac`
- Restored workspace version: `0.114.1` (verified in the `0.1.17` tree)
- Product branch: `issue-26071420489654-nushell-restoration`
- Product HEAD: `2dc50e0ee0997e58a1b758942ae95f97be462417`
- Product tree: `bcb597ffc4d0e2ccc304923bd281240356d01c13`
- Local fork working tree: `forks/nushell`
- Issue archive: `patches/nushell/patches/issue-26071420489654/`
- Patch:
  `patches/nushell/patches/issue-26071420489654/0001-astrohacker-Shannon-shell-product-patch-on-nushell-t.patch`
- Patch SHA-256:
  `dd677832038ae82e8c813333ff1f6873afe0821210480e092bca9534160c7462`
- Archive aggregate SHA-256:
  `c84aadd45d36c6c9fa2a9ee76a5996d5ee63da9c5c58f03bcbb33ddd423caff1`
- Reedline: historical path dependency `../reedline` at `0.49.0`; its exact
  `0.1.17` tip restoration is tracked separately.
- Verification: **archive replay Pass; not built**

Every prior/later archive under `patches/nushell/patches/` remains a historical
record. Issue `26071112000924` is the tag-stored `0.1.17` input; later archives
are present but are not part of this restoration claim.

## Patch Contents

Bounded Shannon/Astrohacker deltas on tip:

- `shannon-nu-cli` / `shannon-nu-lsp` package naming
- path pin of `reedline` to sibling `forks/reedline`
- `ModeDispatcher` support
- Bash syntax highlighting (`bash_highlight.rs` + tree-sitter deps; reused for
  traditional **zsh** mode highlighting)
- REPL mode-dispatch hooks and highlighter selection for `nu` / `zsh`
- related lockfile updates for the path reedline
- Issue archives may include a follow-on patch renaming mode cycle from `bash`
  to `zsh` (see `0002-…` under issue-26071420489654 when present)

## Apply (clean base)

```sh
BASE=0df4ca222cc713e79b6b1684ad8ccaec584ce4ac
# Reedline tip must exist at forks/reedline (path dep)
git -C forks/nushell worktree add -b \
  issue-26071420489654-nushell-restoration \
  /tmp/astrohacker-nushell-restoration "$BASE"
git -C /tmp/astrohacker-nushell-restoration am \
  "$PWD/patches/nushell/patches/issue-26071420489654/0001-astrohacker-Shannon-shell-product-patch-on-nushell-t.patch"
```

## Generate

```sh
BASE=0df4ca222cc713e79b6b1684ad8ccaec584ce4ac
git -C forks/nushell format-patch "$BASE"..HEAD \
  -o patches/nushell/patches/issue-26071420489654/
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
