# Ghostty Patches

Ghostty fork work is tracked here as patch archives against the ignored local
clone at `forks/ghostty`.

## Current State (Issue 26071112000924)

- Upstream repository: `https://github.com/ghostty-org/ghostty`
- Upstream base policy: **latest commit on upstream `main`**
- Upstream base commit: `53bd14fecfd68c6c0ab64d37b5943247299e2b40`
- Local fork working tree: `forks/ghostty`
- Product branch: `issue-26071112000924-ghostty-upstream`
- Product HEAD (base + product commit):
  `ad9768db5138df928b3c307754e7dae0f7945af9`
- Issue archive: `patches/ghostty/patches/issue-26071112000924/`
- Patch:
  `patches/ghostty/patches/issue-26071112000924/0001-astrohacker-terminal-ghostty.patch`
- Prior archive (historical): `patches/ghostty/patches/issue-26070412000013/`
  on base `2c62d182cec246764ff725096a70b9ef44996f7f`

Executable product name: **`ahterm`** inside
`Astrohacker Terminal.app`.

## Apply (clean base)

```sh
BASE=53bd14fecfd68c6c0ab64d37b5943247299e2b40
git -C forks/ghostty fetch origin
git -C forks/ghostty worktree add /tmp/astrohacker-ghostty-0924 "$BASE"
git -C /tmp/astrohacker-ghostty-0924 am \
  "$PWD/patches/ghostty/patches/issue-26071112000924/0001-astrohacker-terminal-ghostty.patch"
```

## Generate

```sh
git -C forks/ghostty format-patch -1 HEAD --stdout \
  > patches/ghostty/patches/issue-26071112000924/0001-astrohacker-terminal-ghostty.patch
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
