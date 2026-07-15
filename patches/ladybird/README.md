# Ladybird Patches

Astrohacker Terminal uses Ladybird through the experimental Girlbat engine. The
Ladybird working tree is local-only under `forks/ladybird`. This directory
tracks patch archives and branch notes that are safe to commit.

## Current State

- **Upstream policy:** default branch **`master`** tip (remote HEAD is
  `refs/heads/master`, not `main`).
- **Current base:** `2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd`
- **Current branch:** `2a3bc6a3-issue-26071420489654-restoration`
- **Current HEAD:** `986f63961a10e8b375c7fdc82b4d0983fce4e56a`
- **Current tree:** `118f26862c9d745e38509a3c9e77cc856409925f`
- **Archive:** `patches/ladybird/patches/issue-26071420489654/` (18 patches)
- **Archive aggregate SHA-256:**
  `c1d4db1a665c1b07e69ea3067ed03169ac90003d1355417fcb3d705d4fb3f041`
- **Verification:** **archive replay Pass; not built**
- Working tree: `forks/ladybird`

Every earlier/later archive under `patches/ladybird/patches/` remains a
historical record. Issue `26071112000924` is the tag-stored `0.1.17` stack;
post-`0.1.17` archives are present but are not part of this restoration claim.

## Merge-upstream

1. Discover tip: `git ls-remote --symref
   https://github.com/LadybirdBrowser/ladybird.git HEAD`
2. Fetch tip; branch `{short8}-issue-26071112000924` at tip commit.
3. Apply `patches/ladybird/patches/issue-26071112000924/*.patch` in numeric order
   (`git am`).
4. Build real backend:

   ```bash
   TERMSURF_LADYBIRD_BACKEND=real scripts/build.sh ladybird --release
   ```

5. Smokes (ensure `Ladybird.app/Contents/MacOS` helpers are on `PATH` or
   symlinked beside `ah-ladybirdd`):

   ```bash
   ah-ladybirdd --termsurf-warmup
   ah-ladybirdd --termsurf-abi-negative-smoke
   ah-ladybirdd --termsurf-engine-thread-smoke
   ah-ladybirdd --termsurf-render-surface-smoke
   ah-ladybirdd --termsurf-real-frame-attachment-smoke
   ah-ladybirdd --termsurf-renderer-crash-smoke
   ah-ladybirdd --termsurf-resource-root-smoke
   cargo test -p ladybird
   ```

6. Regenerate archive:

   ```bash
   rm -rf patches/ladybird/patches/issue-26071112000924
   mkdir -p patches/ladybird/patches/issue-26071112000924
   git -C forks/ladybird format-patch {base}..HEAD \
     -o "$PWD/patches/ladybird/patches/issue-26071112000924"
   ```

7. Update this README Current State.

## Applying Patches

```bash
cd forks/ladybird
git worktree add -b 2a3bc6a3-issue-26071420489654-restoration \
  /tmp/astrohacker-ladybird-restoration \
  2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd
git -C /tmp/astrohacker-ladybird-restoration am \
  "$PWD/../../patches/ladybird/patches/issue-26071420489654/"*.patch
```

## Generating Patches

```bash
git -C forks/ladybird format-patch \
  2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd..HEAD \
  -o "$PWD/patches/ladybird/patches/issue-26071420489654"
```

## Verification

```bash
git -C forks/ladybird status --short
git -C forks/ladybird rev-parse --abbrev-ref HEAD
git -C forks/ladybird rev-parse HEAD
TERMSURF_LADYBIRD_BACKEND=real scripts/build.sh ladybird --release
```
