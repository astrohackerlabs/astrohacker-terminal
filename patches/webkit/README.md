# WebKit Patches

Astrohacker Terminal uses WebKit through the Surfari engine. The WebKit working
tree is local-only under `forks/webkit/src`. This directory tracks the patch
archives and branch notes that are safe to commit.

## Current State

- Current upstream base: `d144dd782ee6ba6fe20cd04b9c8d3e492f3c4254`
- Current branch: `webkit-d144dd78-issue-857`
- Shallow checkout: `true`
- Working tree: `forks/webkit/src`
- Patch archives: `patches/webkit/patches`

## Branch Strategy

WebKit branches encode the upstream base and issue:

```text
webkit-{short-upstream-commit}-issue-{N}
webkit-{short-upstream-commit}-issue-{N}-exp{M}
```

## Applying Patches

```bash
git -C forks/webkit/src fetch --depth 1 origin {base-commit}
git -C forks/webkit/src switch -C webkit-{short-base}-issue-{N} {base-commit}
git -C forks/webkit/src am ../../../patches/webkit/patches/issue-{N}/*.patch
```

## Generating Patches

After committing WebKit changes inside `forks/webkit/src`:

```bash
rm -rf patches/webkit/patches/issue-{N}
mkdir -p patches/webkit/patches/issue-{N}
git -C forks/webkit/src format-patch {base-commit}..HEAD \
  -o ../../../patches/webkit/patches/issue-{N}
```

Then commit the patch archive and the issue experiment result in the
Astrohacker repo.

## Verification

```bash
git -C forks/webkit/src status --short
git -C forks/webkit/src rev-parse --abbrev-ref HEAD
git -C forks/webkit/src rev-parse HEAD
git -C forks/webkit/src rev-parse --is-shallow-repository
git diff --check
```

When WebKit source changed, build through the migrated Terminal build helper
after it has been adapted to the new monorepo layout.
