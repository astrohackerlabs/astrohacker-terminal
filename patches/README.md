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

- `chromium/`
- `webkit/`
- `ladybird/`
- `ghostty/`
- `gecko/`

Ghostty has a current path-aware patch archive under
`patches/ghostty/patches/issue-0013/` generated from the current Astrohacker
Terminal/Ghostboard state. Gecko has an ignored shallow/partial checkout under
`forks/gecko`, but no implementation patch set yet; `patches/gecko/` records
that current checkout state for the future Waterwolf path.
