# Reedline Pin Workspace

Read this before changing the Reedline pin for Astrohacker Shell.

- Keep Reedline under ignored `forks/reedline/`.
- This workspace is normally a **tip pin only** (no product source patch).
- Do not invent no-op commits or empty `.patch` files; document pin-only state
  in the issue archive README when that is the product input.
- If intentional Astrohacker edits appear, start an issue-scoped patch archive
  and regenerate with `git format-patch`.
- Consumers: `forks/nushell` path dep and `rust/ahsh`.

Current local paths:

- Source: `forks/reedline`
- Archives / notes: `patches/reedline/patches`

## Learn more

- Pin identity and verify steps: [`README.md`](./README.md)
- Shared patch policy: [`../README.md`](../README.md)
- Release series authority: [`../release-manifest.json`](../release-manifest.json)
