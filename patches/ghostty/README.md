# Ghostty Patches

Ghostty fork work is tracked here as patch archives against the ignored local
clone at `forks/ghostty`.

## Current State (Issue 26071420489654)

- Upstream repository: `https://github.com/ghostty-org/ghostty`
- Upstream base commit: `53bd14fecfd68c6c0ab64d37b5943247299e2b40`
- Local fork working tree: `forks/ghostty`
- Product branch: `issue-26071420489654-ghostty-restoration`
- Product HEAD (base + product commit):
  `e380e7211d12c0da2ad7f8a1796d5793e12f11fc`
- Product tree: `362ce2b98d3700ab1a00c231614388d53dff5786`
- Issue archive: `patches/ghostty/patches/issue-26071420489654/`
- Patch:
  `patches/ghostty/patches/issue-26071420489654/0001-astrohacker-Terminal-ghostty-product-patch-on-tip-is.patch`
- Patch SHA-256:
  `e620a06511f57372488dd640459db4700d99cd0a3c5601936b515faada6b9387`
- Archive aggregate SHA-256:
  `1b81bd9875d152221b8d7329883217f590a080f14f828743c0c705bacc4314dc`
- Verification: **archive replay Pass; not built**

## Historical Archives

- Issue `26071112000924`: `patches/ghostty/patches/issue-26071112000924/`
  on base `53bd14fecfd68c6c0ab64d37b5943247299e2b40`, product HEAD
  `ad9768db5138df928b3c307754e7dae0f7945af9`.
- Issue `26070412000013`: `patches/ghostty/patches/issue-26070412000013/`
  on base `2c62d182cec246764ff725096a70b9ef44996f7f`.

Executable product name: **`ahterm`** inside
`Astrohacker Terminal.app`.

## Apply (clean base)

```sh
BASE=53bd14fecfd68c6c0ab64d37b5943247299e2b40
git -C forks/ghostty worktree add /tmp/astrohacker-ghostty-restoration "$BASE"
git -C /tmp/astrohacker-ghostty-restoration am \
  "$PWD/patches/ghostty/patches/issue-26071420489654/0001-astrohacker-Terminal-ghostty-product-patch-on-tip-is.patch"
```

## Generate

```sh
git -C forks/ghostty format-patch -1 HEAD \
  -o patches/ghostty/patches/issue-26071420489654/
```

## Build / verify

```sh
scripts/build.sh ahterm --release
# identity
"./forks/ghostty/macos/build/Release/Astrohacker Terminal.app/Contents/MacOS/ahterm" +version
# host TermSurf browser-resolution unit test
cd forks/ghostty && zig build test \
  -Dtest-filter="termsurf server register matches profile and browser"
```

## Merge-upstream checklist

1. Discover tip: `git ls-remote https://github.com/ghostty-org/ghostty.git refs/heads/main`
2. Fetch; create `issue-NNNN-ghostty-upstream` from the tip SHA.
3. `git am` current archive (or re-apply prior product commit); resolve conflicts.
4. Build `ahterm` Release; run `+version` and TermSurf unit filters.
5. `git format-patch -1` into `patches/ghostty/patches/issue-NNNN/`.
6. Update this README Current State (base SHA, branch, archive path, date).

Do not commit `forks/ghostty` or temporary worktrees to the Astrohacker repo.
