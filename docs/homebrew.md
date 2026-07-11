# Homebrew

Astrohacker ships to macOS through the `astrohackerlabs/astrohacker` Homebrew
tap. There is **one desktop download**: the cask `astrohacker`. It targets
Apple silicon macOS and installs Astrohacker Terminal, Shell, Editor, and
related helpers as one Astrohacker bundle.

## Public command surface

| Command | Role |
| --- | --- |
| `ahterm` | Astrohacker Terminal (app executable + PATH launcher) |
| `ahsh` | Astrohacker Shell |
| `ahed` | Astrohacker Editor |
| `ahweb` | Open URLs / browser panes in Terminal |
| `ahapp` | GTUI in-Terminal app launcher (requires Terminal pane env) |

Reserved (not shipping until the product ships): `ahwallet`.

There is **no** `ah` / `astrohacker` meta CLI dispatcher today.

Engine helpers (implementation; on PATH for packaging/debug):

- `ah-chromiumd`, `ah-webkitd`, `ah-ladybirdd`

Engine **selectors** for `ahweb` remain family names: `chromium`, `webkit`,
`ladybird` (future `gecko`).

`TermSurf` remains the **protocol** name (`termsurf.proto`, `libtermsurf_*`,
`TERMSURF_*` env). It is not the product brand and is not the PATH CLI name
(`ahapp` replaced the old `termsurf` PATH binary).

Historical cask token `astrohacker-terminal` is retired. Users install
`astrohacker` only. The public GitHub source repo name
`astrohackerlabs/astrohacker-terminal` is still the release asset host.

Astrohacker Wallet is planned for a future update of this **same** cask—not a
second formula.

## Install

```bash
brew tap astrohackerlabs/astrohacker
brew trust astrohackerlabs/astrohacker
brew install --cask astrohacker
```

Upgrade:

```bash
brew update
brew upgrade --cask astrohacker
```

## Signing model

Distribution uses ad-hoc codesign in the cask postflight (quarantine clear +
`codesign --sign -`) until Developer ID notarization is in place.
`brew trust` trusts the tap source; it does not notarize the app with Apple.

Normal install/reinstall/uninstall of Astrohacker-owned opt artifacts must not
require `sudo` (helpers are Homebrew `artifact`s).

## Installed layout

- `Astrohacker Terminal.app` → `/Applications/Astrohacker Terminal.app`
  (executable `Contents/MacOS/ahterm`)
- PATH: `ahterm`, `ahweb`, `ahapp`, `ahsh`, `ahed`, engine helpers
- Editor runtime →
  `/opt/homebrew/opt/astrohacker-terminal-editor/runtime/`
- Chromium / WebKit / Ladybird trees →
  `/opt/homebrew/opt/astrohacker-terminal-ah-{chromiumd,webkitd,ladybirdd}/`
- GTUI assets → `/opt/homebrew/opt/astrohacker-terminal-gtui/`

Editor is built with baked default runtime
`/opt/homebrew/opt/astrohacker-terminal-editor/runtime`. Installed
`ahed --health rust` must work without `ASTROHACKER_EDITOR_RUNTIME`.

## Release tarball contract

Asset name: `astrohacker-<version>-aarch64-apple-darwin.tar.gz`

Top-level contents:

- `Astrohacker Terminal.app/` (with `Contents/MacOS/ahterm`)
- `ahweb`, `ahapp`, `ahsh`, `ahed`
- `ahed-runtime/runtime/`
- `gtui/`
- `ah-chromiumd/`, `ah-webkitd/`, `ah-ladybirdd/`

## Release / publish (agents and humans)

Canonical three-repository Homebrew release flow. Packaging scripts live in the
**private** monorepo; they are not synced to the public source tree.

### Topology

| Role | Local default | GitHub |
| --- | --- | --- |
| Private monorepo | this repo | private business monorepo |
| Public Terminal source | `~/dev/astrohacker-terminal` | `astrohackerlabs/astrohacker-terminal` |
| Homebrew tap | `~/dev/homebrew-astrohacker` | `astrohackerlabs/homebrew-astrohacker` |

Cask file: `~/dev/homebrew-astrohacker/Casks/astrohacker.rb`

Env overrides: `ASTROHACKER_TERMINAL_PUBLIC_REPO`,
`ASTROHACKER_TERMINAL_PUBLIC_GITHUB_REPO`,
`ASTROHACKER_TERMINAL_HOMEBREW_TAP_REPO` (legacy `TERMSURF_*` aliases still
accepted by scripts).

### Scripts

| Script | Role |
| --- | --- |
| `scripts/build.sh` | Build components / `all --release` |
| `scripts/release.sh` | Stage tarball; with `PUBLISH=1` tag, upload, update cask sha |
| `scripts/sync-public-source.sh` | Sync allowlisted paths into public checkout |

Flags for `scripts/release.sh <version>`:

- Package only (default if publish unset):
  `ASTROHACKER_TERMINAL_RELEASE_PACKAGE_ONLY=1`
  or simply omit `ASTROHACKER_TERMINAL_RELEASE_PUBLISH=1`
- Publish:
  `ASTROHACKER_TERMINAL_RELEASE_PUBLISH=1`

Publish mode requires **clean** public and tap worktrees. It only rewrites cask
`version` and `sha256`. Commit any binary/postflight content changes on the tap
**before** publish mode.

### Ordered steps

1. **Preflight version** (remote-facing):

   ```sh
   gh release list --repo astrohackerlabs/astrohacker-terminal --limit 5
   git -C ~/dev/astrohacker-terminal ls-remote origin 'refs/heads/main' 'refs/tags/v*'
   git -C ~/dev/homebrew-astrohacker fetch origin
   git -C ~/dev/homebrew-astrohacker show origin/main:Casks/astrohacker.rb | grep -E 'version |sha256 '
   ```

   Choose next version from max(public release, tag, remote cask).

2. **Land product changes** in private monorepo; push tap **content** changes
   (not version/sha) if needed so the tap is clean for publish.

3. **Full release build** (editor is included in `all`; still safe to rebuild
   editor cleanly if version is sticky):

   ```sh
   TERMSURF_VERSION=<version> \
   ASTROHACKER_VERSION=<version> \
   ASTROHACKER_EDITOR_DEFAULT_RUNTIME=/opt/homebrew/opt/astrohacker-terminal-editor/runtime \
     scripts/build.sh all --release

   ASTROHACKER_VERSION=<version> \
   ASTROHACKER_EDITOR_DEFAULT_RUNTIME=/opt/homebrew/opt/astrohacker-terminal-editor/runtime \
     scripts/build.sh ahed --release --clean
   ```

4. **Package-only** (dry-run SHA is not authoritative):

   ```sh
   ASTROHACKER_TERMINAL_RELEASE_PACKAGE_ONLY=1 \
   ASTROHACKER_TERMINAL_RELEASE_PUBLISH=0 \
     scripts/release.sh <version>
   ```

   Inspect `dist/release` and
   `dist/astrohacker-<version>-aarch64-apple-darwin.tar.gz`.

5. **Public source sync** (private monorepo → public checkout), then commit on
   public `main` so the tree is clean:

   ```sh
   scripts/sync-public-source.sh
   # commit in ~/dev/astrohacker-terminal
   ```

6. **Publish**:

   ```sh
   ASTROHACKER_TERMINAL_RELEASE_PUBLISH=1 scripts/release.sh <version>
   ```

   Creates/pushes `v<version>`, GitHub release asset, tap commit `v<version>`
   with authoritative SHA256.

7. **Homebrew validate**:

   ```sh
   ruby -c ~/dev/homebrew-astrohacker/Casks/astrohacker.rb
   brew style --cask astrohacker
   brew audit --cask astrohacker
   brew cat --cask astrohacker
   ```

8. **Installed verify** (public tap):

   ```sh
   brew tap astrohackerlabs/astrohacker
   brew trust astrohackerlabs/astrohacker
   brew reinstall --cask astrohacker
   ahterm +version
   ahsh --version
   ahed --version
   ahed --health rust
   ahweb --help | head
   ```

   Check opt helper paths and
   `/opt/homebrew/var/log/astrohacker/terminal-postflight-warmup.log`.

### Release-gate smokes

| Script | Role |
| --- | --- |
| `scripts/test-issue-869-installed-homebrew-browser-smoke.sh` | installed three-engine browser smoke (**gate**) |
| `scripts/test-issue-882-installed-cold-start.sh` | cold-start + warmup (**gate** when GUI available) |
| `scripts/test-issue-867-release-no-env-browser-discovery.sh` | useful discovery check |
| Older Surfari-named 871/872 harnesses | historical; not current gates until updated |

Example:

```sh
ASTROHACKER_TERMINAL_SMOKE_VERSION=<version> \
  scripts/test-issue-869-installed-homebrew-browser-smoke.sh
```

### Traps

- Dirty tap or public repo aborts publish mode.
- Package-only SHA ≠ publish SHA if anything is rebuilt between steps; cask
  SHA from publish is authoritative.
- Partial publish: inspect tag/asset/tap; rerun same version; do not invent a
  new version just to recover.
- `scripts/build.sh all --release` must build editor (`ahed`); prefer an
  explicit clean editor rebuild when version identity matters.
- Ruby style/audit does not run postflight; reinstall is the postflight gate.
- Do not revive cask token `astrohacker-terminal`.

### Agent checklist

1. Preflight remote version + clean trees + forks present
2. Product/cask content committed (tap clean for publish)
3. `scripts/build.sh all --release` + clean `ahed` if needed
4. Package-only `scripts/release.sh`
5. Inspect staging/tarball names
6. `scripts/sync-public-source.sh` + public commit
7. Publish mode
8. `ruby -c` / `brew style` / `brew audit` / `brew cat`
9. `brew reinstall --cask astrohacker` + CLI/app/opt/warmup checks
10. Gate smokes 869 (+ 882 when possible)

## Installed smoke expectations

After install, from inside Astrohacker Terminal:

- `ahweb --browser chromium https://example.com`
- `ahweb --browser webkit https://example.com`
- `ahweb --browser ladybird http://127.0.0.1:<fixture>/`

Helpers resolve under `/opt/homebrew/opt/astrohacker-terminal-ah-*` without
browser path env overrides.

Ladybird is a prototype packaging surface, not production browser parity.
