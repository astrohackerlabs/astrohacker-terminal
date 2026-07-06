# Nushell Patches

Astrohacker Shell uses a patched Nushell fork derived from Shannon. The fork
working tree is local-only under `forks/nushell`; this directory tracks the
patch archive needed to reconstruct Astrohacker Shell's Nushell changes without
importing Nushell history into the company repo.

## Current State

- Upstream repository: `https://github.com/nushell/nushell`
- Upstream base tag: `0.113.1`
- Upstream base commit: `7b7df4aa68e957cf38b9d8157c35fa7523f44a6d`
- Shannon source repo: `~/dev/shannon`
- Shannon source commit: `3ed63bae4c7993782f1ebcf09266c482a4191c1e`
- Local fork working tree: `forks/nushell`
- Issue archive: `patches/nushell/patches/issue-0903`
- Patch: `patches/issue-0903/0001-shannon-nushell.patch`
- Branch convention: `issue-0903-astrohacker-shell`

## Patch Contents

The issue 0903 patch is bounded to Shannon's expected Nushell deltas:

- package manifest and lockfile changes for Shannon fork crates;
- `shannon-nu-cli` and `shannon-nu-lsp` package naming/versioning;
- `ModeDispatcher` support;
- Bash syntax highlighting support;
- Shift+Tab mode-switch dispatch through the Nushell REPL path;
- related `nu-cli` exports and highlighter glue.

Unexpected broad upstream drift indicates the base commit is wrong and should
not be accepted as a valid patch archive.

## Generating Patches

Generate from a clean Shannon checkout at the recorded source commit:

```sh
rm -rf /tmp/ahsh-nushell-gen
cp -R forks/nushell /tmp/ahsh-nushell-gen
rsync -a --delete \
  --exclude .git \
  --exclude target \
  --exclude logs \
  --exclude .DS_Store \
  "$HOME/dev/shannon/nushell/" /tmp/ahsh-nushell-gen/
git -C /tmp/ahsh-nushell-gen add -A
git -C /tmp/ahsh-nushell-gen diff --cached --binary \
  > patches/nushell/patches/issue-0903/0001-shannon-nushell.patch
```

## Applying Patches

Apply to a clean checkout at the recorded base:

```sh
git -C forks/nushell checkout 7b7df4aa68e957cf38b9d8157c35fa7523f44a6d
git -C forks/nushell switch -c issue-0903-astrohacker-shell
git -C forks/nushell apply \
  ../../patches/nushell/patches/issue-0903/0001-shannon-nushell.patch
```

## Verification

```sh
git -C forks/nushell rev-parse HEAD
git -C forks/nushell apply --check \
  ../../patches/nushell/patches/issue-0903/0001-shannon-nushell.patch
git -C forks/nushell status --short --ignored
git diff --check
```

Do not commit `forks/nushell` or temporary worktrees to the Astrohacker repo.
