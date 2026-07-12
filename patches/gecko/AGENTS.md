# Gecko Patch Workspace

Read this before modifying Gecko/Firefox for Astrohacker Terminal.

- Keep Gecko/Firefox source and build state under ignored `forks/gecko/`.
- Keep tracked patch archives and notes under `patches/gecko/`.
- Do not commit Firefox source, `obj-*` build trees, `mozconfig`, or
  `~/.mozbuild` toolchains to the Astrohacker monorepo.
- Upstream remote (Issue 26071112000932+): `https://github.com/mozilla-firefox/firefox.git`
  default branch **`main`**. Historical `mozilla/gecko-dev` master is frozen
  and must not be used as the product tip.
- Use issue-scoped branches in `forks/gecko`: `{short8}-issue-NNNN`.
- Regenerate patch archives with `git format-patch` only after committing
  intentional Astrohacker changes on a fork branch.
- Prefer full (non-artifact) desktop builds for embedding work:
  `ac_add_options --enable-application=browser` without
  `--enable-artifact-builds`.
- Bootstrap and build with a Python **3.9–3.12** interpreter (pin via
  Homebrew `python@3.12` when the system Python is newer).
- Product names: selector `gecko`, helper `ah-geckod`, C ABI
  `libtermsurf_gecko`. Do not use historical codenames as user-facing names.

Current local paths:

- Source: `forks/gecko`
- Objdir convention: `forks/gecko/obj-astrohacker-ff`
- Toolchains: `$MOZBUILD_STATE_PATH` (default `~/.mozbuild`)
- Patches: `patches/gecko/patches/` (created when product patches exist)
