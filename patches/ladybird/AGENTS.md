# Ladybird Patch Workspace

Read this before modifying Ladybird for Astrohacker Terminal.

- Keep Ladybird source and build state under ignored `forks/ladybird`.
- Keep tracked patch archives under `patches/ladybird/patches`.
- Do not commit Ladybird source or build outputs to the Astrohacker repo.
- Use issue-specific branches in `forks/ladybird`.
- Regenerate patch archives with `git format-patch` after committing Ladybird
  branch changes.
- Record the branch, base commit, verification, and patch archive in the active
  issue experiment.

Current local paths:

- Source: `forks/ladybird`
- Patches: `patches/ladybird/patches`

## Learn more

- Reconstruction and current archives: [`README.md`](./README.md)
- Shared patch policy: [`../README.md`](../README.md)
- Release series authority: [`../release-manifest.json`](../release-manifest.json)
