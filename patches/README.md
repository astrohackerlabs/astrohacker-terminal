# Fork Patches

This directory tracks Astrohacker patch sets for large upstream projects whose
working trees live outside git under `forks/`.

Policy:

- `forks/` contains local upstream working trees and build state. It is ignored.
- `patches/` contains the tracked documentation and patch archives needed to
  reconstruct Astrohacker fork branches from recorded upstream commits.
- Each fork should record its upstream base, branch naming convention, patch
  generation command, and patch application command.
- Branch names should map to Astrohacker issues and experiments whenever fork
  source changes are made.

Current fork areas:

- `chromium/` — **Issue 26071112000924:** Electron stable Chromium **150.0.7871.47** /
  archive `issue-26071112000924` (authoritative after Exp4 Pass).
- `webkit/` — **Issue 26071112000924:** main tip `f1a2d7cc…` / archive `issue-26071112000924`;
  residual focus smoke → Issue 26071112000926.
- `ladybird/` — **Issue 26071112000924:** master tip `2a3bc6a3…` / archive `issue-26071112000924`
  (18 patches).
- `ghostty/` — **Issue 26071112000924:** host Exp2 archive `issue-26071112000924` (also historical
  `issue-26070412000013`).
- `gecko/` — optional; no product patch set required for 924.
- `nushell/` / `reedline/` / `helix/` — **Issue 26071112000924** host Exp2 tips/archives.

### Merge-upstream (portfolio)

1. Identify targets (Electron stable Chromium; main/master tips for others).
2. Per fork: branch → apply/regenerate patches → build → smoke → update
   per-fork README + issue-scoped archive under `patches/<fork>/patches/`.
3. Integrated: `TERMSURF_LADYBIRD_BACKEND=real scripts/build.sh all --release`.
4. Do not publish Homebrew from upgrade issues (use Issue 26071112000925+).
