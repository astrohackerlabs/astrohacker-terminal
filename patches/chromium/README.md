# Chromium Patches

Astrohacker Terminal uses Chromium through the Roamium engine. The Chromium
working tree is local-only under `forks/chromium/src`; Chromium tooling lives in
`forks/chromium/depot_tools`. This directory tracks the patch archives and
branch notes that are safe to commit.

## Current State

- **Restored reconstructable baseline:** `150.0.7871.47-issue-26071420489654` / base
  `150.0.7871.47` (`0c3cca15d78645281db2d339b2dc3d6fad4ee90a`)
- Main build target: `libtermsurf_chromium`
- Working tree: `forks/chromium/src`
- Tooling: `forks/chromium/depot_tools`
- Patch archives: `patches/chromium/patches`
- Historical: Issue `26071112000924` remains the exact `0.1.17` release
  archive; later archives remain history. Issue `26071420489654` is the active
  restoration record.

### Issue 26071420489654 / 0.1.17 restoration (current)

| Field | Value |
| --- | --- |
| Target base | `150.0.7871.47` / `0c3cca15d78645281db2d339b2dc3d6fad4ee90a` |
| Policy | Restore the exact shipped `0.1.17` Chromium product tree |
| Product branch | `150.0.7871.47-issue-26071420489654` |
| Product HEAD | `cd36368f70078014b2b6386fae0999b912b86b30` (119 commits on base) |
| Product tree | `8264590e738a8f4b2f0c1f0b4f46a4431347f073` (equal to historical `0.1.17`) |
| Archive | `patches/chromium/patches/issue-26071420489654/` (119 format-patches) |
| Archive aggregate SHA-256 | `b332e1468f309e78459da164b40656aa848b4caa2e2f0e92a3abab0844f04a8b` |
| Reconstruction | **Pass** — 119 stable patch IDs equal; two clean replays produced the expected tree |
| Build status | **Not built** — engine build, resize behavior, binary comparison, and release qualification are deferred |

### Issue 26071112000924 / Electron stable Chromium 150 (`0.1.17` historical)

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
git checkout -b 150.0.7871.47-issue-26071420489654
git am ../../../patches/chromium/patches/issue-26071420489654/*.patch
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
