# Chromium Patches

Astrohacker Terminal uses Chromium through the Roamium engine. The Chromium
working tree is local-only under `forks/chromium/src`; Chromium tooling lives in
`forks/chromium/depot_tools`. This directory tracks the patch archives and
branch notes that are safe to commit.

## Current State

- **Production reconstructable baseline:** `150.0.7871.47-issue-26071112000924` / base
  `150.0.7871.47` (`0c3cca15d78645281db2d339b2dc3d6fad4ee90a`)
- Main build target: `libtermsurf_chromium`
- Working tree: `forks/chromium/src`
- Tooling: `forks/chromium/depot_tools`
- Patch archives: `patches/chromium/patches`
- Historical: `148.0.7778.271-issue-26070612000901` remains in the archive ledger only;
  issue-26071112000924 is current after Issue 26071112000924 Experiment 4 Pass.

### Issue 26071112000924 / Electron stable Chromium 150 (current)

| Field | Value |
| --- | --- |
| Target base | `150.0.7871.47` / `0c3cca15d78645281db2d339b2dc3d6fad4ee90a` |
| Policy | Electron stable Chromium only |
| Product branch | `150.0.7871.47-issue-26071112000924` |
| Product HEAD (local) | `ca9329e85c734d8cb1524a9e27328349a72c94de` (119 commits on base) |
| Archive | `patches/chromium/patches/issue-26071112000924/` (119 format-patches; TREE_MATCH) |
| Build status | **Green** — `libtermsurf_chromium` + `ah-chromiumd --termsurf-warmup` |

### Merge-upstream (Chromium)

1. Discover Electron stable Chromium version (see Issue 26071112000924 Exp 1 pattern).
2. Fetch tag; branch `{version}-issue-NNNN` at the tag commit.
3. `gclient sync` / `runhooks` (prefer `managed: False` for src; avoid full
   unshallow stalls).
4. `git am` current archive; resolve conflicts; keep stack ledger.
5. `gn gen out/Default` then `autoninja -C out/Default libtermsurf_chromium`.
6. Build/smoke `ah-chromiumd`; regenerate format-patch archive; update this
   README.

## Branch Strategy

Chromium issue branches use:

```text
{version}-issue-{N}
{version}-issue-{N}-exp{M}
```

When future Astrohacker issues modify Chromium source, create an issue-specific
branch in `forks/chromium/src`, commit there, regenerate the matching patch
archive under `patches/chromium/patches/`, and record the issue/experiment in
the result.

## Applying Patches

For the current fully archived baseline:

```bash
cd forks/chromium/src
git checkout 0c3cca15d78645281db2d339b2dc3d6fad4ee90a
git checkout -b 150.0.7871.47-issue-26071112000924
git am ../../../patches/chromium/patches/issue-26071112000924/*.patch
```

Historical 901 baseline (pre–Issue 26071112000924):

```bash
cd forks/chromium/src
git checkout 148.0.7778.271
git checkout -b 148.0.7778.271-issue-26070612000901
git am ../../../patches/chromium/patches/issue-26070612000901/*.patch
```

Some historical patch directories after issue 794 are incremental rather than
cumulative. Treat those as branch history records unless a later experiment
regenerates and verifies them as full-stack archives.

## Generating Patches

After committing Chromium changes inside `forks/chromium/src`:

```bash
cd forks/chromium/src
rm -rf ../../../patches/chromium/patches/issue-{N}
git format-patch 0c3cca15d78645281db2d339b2dc3d6fad4ee90a..HEAD \
  -o ../../../patches/chromium/patches/issue-{N}
```

Then commit the patch archive and the issue experiment result in the
Astrohacker repo.

## Verification

```bash
git -C forks/chromium/src status --short
git -C forks/chromium/src rev-parse --abbrev-ref HEAD
git -C forks/chromium/src rev-parse HEAD
git diff --check
```

When Chromium source changed, also build:

```bash
cd forks/chromium/src
export PATH="$PWD/../depot_tools:$PATH"
autoninja -C out/Default libtermsurf_chromium
```
