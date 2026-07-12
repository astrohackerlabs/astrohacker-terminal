# WebKit Patches

Astrohacker Terminal uses WebKit through the Surfari engine. The WebKit working
tree is local-only under `forks/webkit/src`. This directory tracks the patch
archives and branch notes that are safe to commit.

## Current State

- **Current upstream base:** `f1a2d7ccc011b8da238839e6e66172d50f283e4f`
- **Current branch:** `webkit-f1a2d7cc-issue-26071112000924`
- **Current HEAD:** `60899f4934ec2f66e175cecd44398d834c745b0c`
- **Archive:** `patches/webkit/patches/issue-26071112000924/` (2 patches; TREE_MATCH)
- Shallow checkout: `true`
- Working tree: `forks/webkit/src`
- **Residual:** wrapper `smoke-test` focus observation fails; tracked by
  [Issue 26071112000926](../../issues/0926-webkit-focus-residual/README.md). Daemon
  warmup is green.

Historical issue-26062712000857 archive remains under `patches/webkit/patches/issue-26062712000857/`
for the pre-924 baseline (`d144dd78…`).

## Merge-upstream

1. `git ls-remote https://github.com/WebKit/WebKit.git refs/heads/main`
2. Fetch tip; branch `webkit-{short8}-issue-NNNN` at tip.
3. `git am` current issue archive.
4. `scripts/build.sh webkit-fork --release` then `webkit --release`.
5. Smoke: `ah-webkitd --termsurf-warmup` (+ wrapper smoke when fixed).
6. Regenerate format-patch archive; update this README.

## Applying Patches

```bash
cd forks/webkit/src
git fetch --depth 1 origin f1a2d7ccc011b8da238839e6e66172d50f283e4f
git switch -C webkit-f1a2d7cc-issue-26071112000924 f1a2d7ccc011b8da238839e6e66172d50f283e4f
git am ../../../patches/webkit/patches/issue-26071112000924/*.patch
```

## Generating Patches

```bash
rm -rf patches/webkit/patches/issue-26071112000924
mkdir -p patches/webkit/patches/issue-26071112000924
git -C forks/webkit/src format-patch \
  f1a2d7ccc011b8da238839e6e66172d50f283e4f..HEAD \
  -o "$PWD/patches/webkit/patches/issue-26071112000924"
```

## Verification

```bash
git -C forks/webkit/src status --short
git -C forks/webkit/src rev-parse HEAD
scripts/build.sh webkit-fork --release
scripts/build.sh webkit --release
```
