# WebKit Patches

Astrohacker Terminal uses WebKit through the Surfari engine. The WebKit working
tree is local-only under `forks/webkit/src`. This directory tracks the patch
archives and branch notes that are safe to commit.

## Current State

- **Current upstream base:** `f1a2d7ccc011b8da238839e6e66172d50f283e4f`
- **Current branch:** `webkit-f1a2d7cc-issue-26071420489654-restoration`
- **Current HEAD:** `7b7ebf8e3d973c378a11d961c710fd17aeb63069`
- **Current tree:** `1597177f69e292eaff2d284fbac1e7b75fa7e67d`
- **Archive:** `patches/webkit/patches/issue-26071420489654/` (2 patches)
- **Archive aggregate SHA-256:**
  `18b87541ff683a295b05d8be7919b847e213e804ce037b0f21f28a524960835d`
- **Verification:** **archive replay Pass; not built**
- Shallow checkout: `true`
- Working tree: `forks/webkit/src`

All prior archives remain under `patches/webkit/patches/` as historical
records. In particular, Issue `26071112000924` is the tag-stored `0.1.17`
archive and Issue `26062712000857` is the pre-924 baseline (`d144dd78…`).
Later post-`0.1.17` archives remain present but are not part of this restoration
claim.

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
git worktree add -b webkit-f1a2d7cc-issue-26071420489654-restoration \
  /tmp/astrohacker-webkit-restoration \
  f1a2d7ccc011b8da238839e6e66172d50f283e4f
git -C /tmp/astrohacker-webkit-restoration am \
  "$PWD/../../../patches/webkit/patches/issue-26071420489654/"*.patch
```

## Generating Patches

```bash
mkdir -p patches/webkit/patches/issue-26071420489654
git -C forks/webkit/src format-patch \
  f1a2d7ccc011b8da238839e6e66172d50f283e4f..HEAD \
  -o "$PWD/patches/webkit/patches/issue-26071420489654"
```

## Verification

```bash
git -C forks/webkit/src status --short
git -C forks/webkit/src rev-parse HEAD
scripts/build.sh webkit-fork --release
scripts/build.sh webkit --release
```
