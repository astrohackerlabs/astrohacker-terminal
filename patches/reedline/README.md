# Reedline Patches

Astrohacker Shell currently uses upstream Reedline through Shannon's dependency
graph, but Shannon's `reedline/` subtree has no effective source changes
relative to upstream `v0.48.0`.

The local fork working tree is still expected under `forks/reedline` so future
Astrohacker Shell work can follow the same fork/patch discipline as other large
upstream projects.

## Current State

- Upstream repository: `https://github.com/nushell/reedline`
- Upstream base tag: `v0.48.0`
- Upstream base commit: `c076ee0c97f4e3e113db03957269d68b83e9784a`
- Shannon source repo: `~/dev/shannon`
- Shannon source commit: `3ed63bae4c7993782f1ebcf09266c482a4191c1e`
- Local fork working tree: `forks/reedline`
- Issue archive: `patches/reedline/patches/issue-0903`
- Patch: none. The Shannon subtree matches the recorded upstream base.
- Branch convention: `issue-0903-astrohacker-shell`

## Generating Patches

If future Reedline changes are made, generate them with the same overlay method:

```sh
rm -rf /tmp/ahsh-reedline-gen
cp -R forks/reedline /tmp/ahsh-reedline-gen
rsync -a --delete \
  --exclude .git \
  --exclude target \
  --exclude logs \
  --exclude .DS_Store \
  "$HOME/dev/shannon/reedline/" /tmp/ahsh-reedline-gen/
git -C /tmp/ahsh-reedline-gen add -A
git -C /tmp/ahsh-reedline-gen diff --cached --binary \
  > patches/reedline/patches/issue-0903/0001-shannon-reedline.patch
```

For issue 0903 this produced an empty diff, so no patch file is tracked.

## Verification

```sh
git -C forks/reedline rev-parse HEAD
git -C forks/reedline status --short --ignored
git diff --check
```

Do not commit `forks/reedline` or temporary worktrees to the Astrohacker repo.
