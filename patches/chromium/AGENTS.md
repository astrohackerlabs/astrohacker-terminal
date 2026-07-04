# Chromium Patch Workspace

Read this before modifying Chromium for Astrohacker Terminal.

- Keep Chromium source and build state under ignored `forks/chromium/`.
- Keep tracked patch archives under `patches/chromium/patches/`.
- Do not commit Chromium source, `depot_tools`, gclient state, or build outputs
  to the Astrohacker repo.
- Use issue-specific branches in `forks/chromium/src`.
- Regenerate patch archives with `git format-patch` after committing Chromium
  branch changes.
- Record the branch, base commit/version, verification, and patch archive in the
  active issue experiment.
- Never run `ninja` directly in Chromium's build output; use `autoninja`.

Current local paths:

- Source: `forks/chromium/src`
- Tools: `forks/chromium/depot_tools`
- Patches: `patches/chromium/patches`
