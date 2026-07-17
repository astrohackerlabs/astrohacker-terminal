# WebKit Patch Workspace

Read this before modifying WebKit for Astrohacker Terminal.

- Keep WebKit source and build state under ignored `forks/webkit/`.
- Keep tracked patch archives under `patches/webkit/patches/`.
- Do not commit WebKit source or build outputs to the Astrohacker repo.
- Use issue-specific branches in `forks/webkit/src`.
- Regenerate patch archives with `git format-patch` after committing WebKit
  branch changes.
- Record the branch, base commit, verification, and patch archive in the active
  issue experiment.

Current local paths:

- Source: `forks/webkit/src`
- Patches: `patches/webkit/patches`

## Learn more

- Reconstruction and current archives: [`README.md`](./README.md)
- Shared patch policy: [`../README.md`](../README.md)
- Release series authority: [`../release-manifest.json`](../release-manifest.json)
