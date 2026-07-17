# Nushell Patch Workspace

Read this before modifying the Nushell fork for Astrohacker Shell.

- Keep Nushell source under ignored `forks/nushell/`.
- Keep tracked patch archives under `patches/nushell/patches/`.
- Do not commit Nushell source or build outputs to the Astrohacker repo.
- Use issue-specific branches in `forks/nushell`.
- Regenerate patch archives with `git format-patch` after committing Nushell
  branch changes.
- Record the branch, base commit, verification, and patch archive in the active
  issue experiment.
- Reedline is a sibling path pin under `forks/reedline` — see
  `patches/reedline/`.

Current local paths:

- Source: `forks/nushell`
- Patches: `patches/nushell/patches`

## Learn more

- Reconstruction and current archives: [`README.md`](./README.md)
- Shared patch policy: [`../README.md`](../README.md)
- Release series authority: [`../release-manifest.json`](../release-manifest.json)
