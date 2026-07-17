# Ghostty Patch Workspace

Read this before modifying Ghostty for Astrohacker Terminal.

- Keep Ghostty source and build state under ignored `forks/ghostty/`.
- Keep tracked patch archives under `patches/ghostty/patches/`.
- Do not commit Ghostty source or build outputs to the Astrohacker repo.
- Use issue-specific branches in `forks/ghostty`.
- Regenerate patch archives with `git format-patch` after committing Ghostty
  branch changes.
- Record the branch, base commit, verification, and patch archive in the active
  issue experiment.

Current local paths:

- Source: `forks/ghostty`
- Patches: `patches/ghostty/patches`

## Learn more

- Reconstruction and current archives: [`README.md`](./README.md)
- Shared patch policy: [`../README.md`](../README.md)
- Release series authority: [`../release-manifest.json`](../release-manifest.json)
