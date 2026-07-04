# Ladybird Patches

Astrohacker Terminal uses Ladybird through the experimental Girlbat engine. The
Ladybird working tree is local-only under `forks/ladybird`. This directory
tracks patch archives and branch notes that are safe to commit.

## Current State

- Current local branch: `a80d01fc-issue-0884-visible-runtime-load-finish`
- Current local HEAD: `51ae5b2a51fa45cc8ccd9ac87d87761651e86974`
- Working tree: `forks/ladybird`
- Patch archives: `patches/ladybird/patches`

The imported patch set contains issue 0884 patches for the initial Girlbat /
Ladybird ABI work.

## Applying Patches

Apply the issue 0884 patches from the relevant upstream base recorded in the
patch history:

```bash
cd forks/ladybird
git switch -c a80d01fc-issue-0884-visible-runtime-load-finish {base-commit}
git am ../../patches/ladybird/patches/*.patch
```

## Generating Patches

After committing Ladybird changes inside `forks/ladybird`:

```bash
git -C forks/ladybird format-patch {base-commit}..HEAD \
  -o ../../patches/ladybird/patches
```

Then commit the patch archive and the issue experiment result in the
Astrohacker repo.

## Verification

```bash
git -C forks/ladybird status --short
git -C forks/ladybird rev-parse --abbrev-ref HEAD
git -C forks/ladybird rev-parse HEAD
git diff --check
```
