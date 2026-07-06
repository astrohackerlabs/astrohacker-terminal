# Ghostty Patches

Ghostty fork work is tracked here as patch archives against the ignored local
clone at `forks/ghostty`.

Current archive:

- Issue archive: `patches/issue-0013/`
- Patch:
  `patches/issue-0013/0001-current-astrohacker-terminal-ghostty.patch`
- Upstream base commit: `2c62d182cec246764ff725096a70b9ef44996f7f`
- Source snapshot: current ignored fork checkout at `forks/ghostty`

This first archive captures the current Astrohacker Terminal/Ghostboard state
only. Historical Ghostty patch reconstruction is intentionally out of scope for
issue 13.

To apply the current patch to a temporary worktree:

```sh
git -C forks/ghostty worktree add /tmp/astrohacker-ghostty-issue-0013 \
  2c62d182cec246764ff725096a70b9ef44996f7f
git -C /tmp/astrohacker-ghostty-issue-0013 apply \
  "$PWD/patches/ghostty/patches/issue-0013/0001-current-astrohacker-terminal-ghostty.patch"
```

To rebuild the patched app in the ignored local checkout:

```sh
cd forks/ghostty
zig build -Demit-macos-app=false
macos/build.nu --scheme Ghostty --configuration Debug --action build
```

The Debug app bundle is
`forks/ghostty/macos/build/Debug/Astrohacker Terminal.app`.

Do not commit `forks/ghostty` or temporary worktrees to the Astrohacker repo.
