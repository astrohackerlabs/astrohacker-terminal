# Fork Patches

Read this before any work under `patches/` or ignored `forks/`.

- Shared policy and merge-upstream portfolio notes: [`README.md`](./README.md).
- **Release authority:** [`release-manifest.json`](./release-manifest.json). Do
  not invent the cumulative shipped series from “latest” issue folder names.
- Per-fork reconstruction detail (bases, archives, apply/generate/verify): each
  fork’s `README.md`.
- Local hygiene and fork-specific hazards: each fork’s `AGENTS.md` when present.

Forks with patch (or pin) workspaces:

| Fork | Notes |
| --- | --- |
| `chromium/` | Engine; large cumulative archives |
| `webkit/` | Engine |
| `ladybird/` | Engine |
| `ghostty/` | Host terminal (`ahterm`) |
| `gecko/` | Optional engine; not in Homebrew ship set |
| `nushell/` | Shell product fork |
| `reedline/` | Tip pin only (no product patch unless that changes) |

Working trees stay under ignored `forks/`; only archives and docs live here.
