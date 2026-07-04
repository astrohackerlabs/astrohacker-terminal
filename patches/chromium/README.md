# Chromium Patches

Astrohacker Terminal uses Chromium through the Roamium engine. The Chromium
working tree is local-only under `forks/chromium/src`; Chromium tooling lives in
`forks/chromium/depot_tools`. This directory tracks the patch archives and
branch notes that are safe to commit.

## Current State

- Current archived baseline: `148.0.7778.271-issue-860`
- Base version: `148.0.7778.271`
- Main build target: `libtermsurf_chromium`
- Working tree: `forks/chromium/src`
- Tooling: `forks/chromium/depot_tools`
- Patch archives: `patches/chromium/patches`

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
git checkout 148.0.7778.271
git checkout -b 148.0.7778.271-issue-860
git am ../../../patches/chromium/patches/issue-860/*.patch
```

Some historical patch directories after issue 794 are incremental rather than
cumulative. Treat those as branch history records unless a later experiment
regenerates and verifies them as full-stack archives.

## Generating Patches

After committing Chromium changes inside `forks/chromium/src`:

```bash
cd forks/chromium/src
rm -rf ../../../patches/chromium/patches/issue-{N}
git format-patch 148.0.7778.271..HEAD \
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
