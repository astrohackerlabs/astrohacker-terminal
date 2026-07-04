# Ghostty Issue 0013 Patch Archive

This directory contains the first Astrohacker Terminal patch archive for the
Ghostty fork.

Archive:

- Patch: `0001-current-astrohacker-terminal-ghostty.patch`
- Upstream project: `ghostty-org/ghostty`
- Local ignored fork clone: `forks/ghostty`
- Base commit: `2c62d182cec246764ff725096a70b9ef44996f7f`
- Base summary: `gtk: fix context menu hiding quick-terminal (#12843)`
- Source input: old Terminal `repos/terminal/ghostboard/` snapshot
- Temporary generation worktree: `/tmp/astrohacker-ghostty-issue-0013`
- Generated for Astrohacker issue:
  `issues/0013-astrohacker-terminal-monorepo-migration/`

This is a current-state archive only. It does not reconstruct historical
Ghostty/Ghostboard experiments.

Experiment 9 regenerated the archive after rebuild verification found that the
fork patch still referred to `../render-channel`. In the monorepo layout that
owned code lives at `rust/render-channel`, so the archive now points Ghostty's
build to `../../rust/render-channel`.

Generation method:

```sh
git -C forks/ghostty worktree add /tmp/astrohacker-ghostty-issue-0013 \
  2c62d182cec246764ff725096a70b9ef44996f7f
rsync -a --delete \
  --exclude .git \
  --exclude .git/ \
  --exclude .zig-cache/ \
  --exclude zig-out/ \
  --exclude result \
  --exclude macos/build/ \
  --exclude macos/GhosttyKit.xcframework/ \
  --exclude macos/Ghostty.xcodeproj/project.xcworkspace/xcshareddata/swiftpm/configuration/ \
  --exclude DerivedData/ \
  --exclude node_modules/ \
  --exclude target/ \
  --exclude '*.log' \
  --exclude .DS_Store \
  repos/terminal/ghostboard/ /tmp/astrohacker-ghostty-issue-0013/
git -C /tmp/astrohacker-ghostty-issue-0013 add -A
git -C /tmp/astrohacker-ghostty-issue-0013 diff --cached --binary \
  > patches/ghostty/patches/issue-0013/0001-current-astrohacker-terminal-ghostty.patch
```

The `rsync --delete` step is intentional. It captures upstream files absent
from the current Ghostboard snapshot as deletions, not only modified and added
files.

Excluded paths:

- `.git`
- `.git/`
- `.zig-cache/`
- `zig-out/`
- `result`
- `macos/build/`
- `macos/GhosttyKit.xcframework/`
- `macos/Ghostty.xcodeproj/project.xcworkspace/xcshareddata/swiftpm/configuration/`
- `DerivedData/`
- `node_modules/`
- `target/`
- `*.log`
- `.DS_Store`

Generated diff summary:

```text
212 files changed, 32258 insertions(+), 5667 deletions(-)
```

The Experiment 9 regeneration preserved the same file/change count while
repairing the two render-channel build paths.

To verify application:

```sh
git -C forks/ghostty worktree add /tmp/astrohacker-ghostty-issue-0013-verify \
  2c62d182cec246764ff725096a70b9ef44996f7f
git -C /tmp/astrohacker-ghostty-issue-0013-verify apply --check \
  "$PWD/patches/ghostty/patches/issue-0013/0001-current-astrohacker-terminal-ghostty.patch"
git -C /tmp/astrohacker-ghostty-issue-0013-verify apply \
  "$PWD/patches/ghostty/patches/issue-0013/0001-current-astrohacker-terminal-ghostty.patch"
```
