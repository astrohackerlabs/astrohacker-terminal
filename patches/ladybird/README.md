# Ladybird Patches

Astrohacker Terminal uses Ladybird through the experimental Girlbat engine. The
Ladybird working tree is local-only under `forks/ladybird`. This directory
tracks patch archives and branch notes that are safe to commit.

## Current State

- **Upstream policy:** default branch **`master`** tip (remote HEAD is
  `refs/heads/master`, not `main`).
- **Current base:** `2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd`
- **Current branch:** `2a3bc6a3-issue-26071112000924`
- **Current HEAD:** `616dd0cdc9061fc17f49248fa507770df4df92de`
- **Archive:** `patches/ladybird/patches/issue-26071112000924/` (18 patches; TREE_MATCH)
- Working tree: `forks/ladybird`

Historical flat 0884/0890 files under `patches/ladybird/patches/` remain as
pre-924 imports; issue-26071112000924 is the reconstructable current stack.

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
git fetch origin 2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd
git switch -C 2a3bc6a3-issue-26071112000924 2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd
git am ../../patches/ladybird/patches/issue-26071112000924/*.patch
```

## Generating Patches

```bash
git -C forks/ladybird format-patch \
  2a3bc6a32fdd35cf95536d4e80cb395dc2201fcd..HEAD \
  -o "$PWD/patches/ladybird/patches/issue-26071112000924"
```

## Verification

```bash
git -C forks/ladybird status --short
git -C forks/ladybird rev-parse --abbrev-ref HEAD
git -C forks/ladybird rev-parse HEAD
TERMSURF_LADYBIRD_BACKEND=real scripts/build.sh ladybird --release
```
