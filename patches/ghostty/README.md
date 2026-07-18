# Ghostty Patches

Ghostty fork work is tracked here as patch archives against the ignored local
clone at `forks/ghostty`.

## Current State (Issue 26071814115751)

- **Upstream policy:** latest commit on **`main`**
- **Upstream base:** `f3c9a2b7262a989ba7e9408d00471fda8f788d16`
- **Product branch:** `issue-26071814115751-ghostty`
- **Product HEAD:** `fc25ec02822f9449914e6a95aeefb5bae2e9b28f`
- **Product tree:** `7f1a24c180d9e935537b08106c0fb093020c8520`
- **Archive:** `patches/ghostty/patches/issue-26071814115751/` (17 patches)
- **Archive aggregate SHA-256:**
  `9467410e92c14a96cb30fb0592f7b2bf839d69551b549e49768e742aa96d45c8`
- **Verification:** **TREE_MATCH Pass**; `scripts/build.sh ahterm --release`
  green with Zig 0.15.2 (Exp 6 implementer)
- **Release authority:** `patches/release-manifest.json` ghostty entry

Build note: tip requires Zig **0.15.2** (`build.zig.zon` minimum). Prefer
`/opt/homebrew/opt/zig@0.15/bin` on PATH when system Zig is 0.16+.

## Prior Active Add-on (Issue 26071813061732)

- Parent product commit: `ee241e83f206288bfa7bd6177a197fcd4b73afd7`
  (prior tip on `issue-26071811041780-welcome-homepage-url`)
- Product branch: `issue-26071813061732-remove-ahapp-poc`
- Product HEAD: `7093f54e7d0e86c558d86dea36cd04b560488d3e`
- Product tree: `5f58f5236712fbc2fd05ba86752fa08c318fe7c4`
- Issue archive: `patches/ghostty/patches/issue-26071813061732/`
- Patches: 0001 app removal, 0002 compile fix, 0003 ignore zig-pkg
- Patch SHA-256:
  - 0001: `850a9d92c2972099b48061b40bd17aa768fd42a74c6d4fe21912d4e40072a1e4`
  - 0002: `cfc2ed8012fca56de057c80746516bcfa04cdeae883310a39ba546c264f087a4`
  - 0003: `0279a6422627c9f4b7701c50ae0062eb373adf3e594d5d68cecef1274b93a3f4`
- Scope: remove TermSurf app host path; fix compile residuals; ignore
  `zig-pkg/` so release_forks clean check passes after local Zig builds.
- Verification: **source + 17-patch release series pin**.

## Prior Add-on (Issue 26071811041780)

- Parent product commit: `ed063b7b49135907b45d32a715bb92d6ba28eb50`
  (prior tip on `issue-26071721129990-shell-xdg-defaults`)
- Product branch: `issue-26071811041780-welcome-homepage-url`
- Product HEAD: `ee241e83f206288bfa7bd6177a197fcd4b73afd7`
- Product tree: `95fa220c4c20cfdf139c6d76775182170aed6d3c`
- Issue archive: `patches/ghostty/patches/issue-26071811041780/`
- Patch: `0001-Default-homepage-to-astrohacker.com-welcome.patch`
- Patch SHA-256:
  `6ec86883ad5afb252690ee8209902f5beb30bcb94a11fdbc453b64125c101b09`
- Scope: product default homepage URL
  `https://astrohacker.com/welcome` (Config, HelloReply fallback, Swift
  bridge) instead of `termsurf.com/welcome`.
- Verification: **source + 14-patch release series pin**; issue closed Pass.

## Prior Add-on (Issue 26071721129990)

- Parent product commit: `1a3ab12fc8619b81d46e61a1be66ef697ae4962e`
  (prior tip on `issue-26071720442142-font-keybind-defaults`)
- Product branch: `issue-26071721129990-shell-xdg-defaults`
- Product HEAD: `ed063b7b49135907b45d32a715bb92d6ba28eb50`
- Product tree: `56354bbfd58ce56f06cfdd5c9175979717acf88e`
- Issue archive: `patches/ghostty/patches/issue-26071721129990/`
- Patch: `0001-Default-shell-to-ahsh-and-XDG_CONFIG_HOME.patch`
- Patch SHA-256:
  `bc54a03efedfd69f89fad9a49a5b047cf26fff5cb2db06f954b9519d04ae62a2`
- Scope: default shell to packaged ahsh absolute paths with system-shell
  fallback; inject `XDG_CONFIG_HOME=$HOME/.config` when unset.
- Verification: **source + 13-patch release series pin**; operator release
  visual gate open (prior tip).

## Prior Add-on (Issue 26071720442142)

- Parent product commit: `56ff57e016c29c670b09867a1722f1d9854c6c9a`
  (prior tip on `issue-26071720300520-unfocused-opacity-default`)
- Product branch: `issue-26071720442142-font-keybind-defaults`
- Product HEAD: `1a3ab12fc8619b81d46e61a1be66ef697ae4962e`
- Product tree: `3e2ec116aae1667819d13f2744d15785bfdca024`
- Issue archive: `patches/ghostty/patches/issue-26071720442142/`
- Patches: `0001-…product-keybinds.patch`, `0002-…Allocator.Error-set.patch`
- Scope: default `font-family = JetBrainsMono Nerd Font`, `font-size = 12`,
  and Astrohacker split/tab product keybinds on macOS.
- Verification: **source + series pin Pass; issue closed Pass**.

## Prior Add-on (Issue 26071720300520)

- Parent product commit: `2aa4373bd65e685ea29d800a28af809cc30a3848`
  (prior tip on `issue-26071720189508-tokyonight-default`)
- Product branch: `issue-26071720300520-unfocused-opacity-default`
- Product HEAD: `56ff57e016c29c670b09867a1722f1d9854c6c9a`
- Product tree: `91ad8b4dc398298ffb9089a8c44495cf2460d64e`
- Issue archive: `patches/ghostty/patches/issue-26071720300520/`
- Patch: `0001-Default-unfocused-split-opacity-to-1.patch`
- Patch SHA-256:
  `e656a2ac7fc0763fe2c12f09251a5a5e3a6fa2a939243685f3196f9d4f028ece`
- Scope: product default `unfocused-split-opacity = 1` (no inactive-pane
  dimming); borders mark focus.
- Verification: **source + series pin Pass; issue closed Pass**.

## Prior Add-on (Issue 26071720189508)

- Parent product commit: `25004fc64cdc3577bccd58238aacef18397f272b`
  (prior tip on `issue-26071719409451-border-theme-defaults`)
- Product branch: `issue-26071720189508-tokyonight-default`
- Product HEAD: `2aa4373bd65e685ea29d800a28af809cc30a3848`
- Product tree: `8d87c4521aa89d7a4b74e3b399fb7c69cd3b1108`
- Issue archive: `patches/ghostty/patches/issue-26071720189508/`
- Patch: `0001-Default-theme-to-TokyoNight.patch`
- Patch SHA-256:
  `d50a411b7e4ac6fc53cebf7d54e447b88fd714ed6ffbf3e56dc2ed3942e0c81c`
- Scope: product default `theme = TokyoNight` (exact resource name) for light
  and dark when unset.
- Verification: **source + series pin Pass; issue closed Pass**.

## Prior Add-on (Issue 26071719409451)

- Parent product commit: `2cc105acaaf8eb8fa82cb3344067d5b4b2468d68`
  (prior tip on `issue-26071611180778-split-webview-disappearance`)
- Product branch: `issue-26071719409451-border-theme-defaults`
- Product HEAD: `25004fc64cdc3577bccd58238aacef18397f272b`
- Product tree: `aa1192bf00dc4359a35d79aba27ed7897b4494e5`
- Issue archive: `patches/ghostty/patches/issue-26071719409451/`
- Patch:
  `0001-Default-split-borders-to-theme-palette-colors.patch`
- Patch SHA-256:
  `985744ab2a9b3b0abecb7fa586440e235a341f6198dacd1973236b17e52cd007`
- Scope: default `split-border-width = 2`; unset focused/unfocused border
  colors fall back to theme `palette[6]` / `palette[8]` in the macOS Swift
  config bridge.
- Verification: **source build Pass; issue closed Pass**.

## Prior Add-on (Issue 26071611180778)

- Parent product commit: `328d150826cb17be0f0eaa15fada9549fe2c60a1`
- Product branch: `issue-26071611180778-split-webview-disappearance`
- Product HEAD: `58d5855ccfc1b2d5d788af87d708f8c1b9b15c98`
- Product tree: `c49e204f49636262be90e23c0fd90e5b7c4f0a4e`
- Issue archive: `patches/ghostty/patches/issue-26071611180778/`
- Scope: split-tree/focus diagnostics plus AppKit overlay-lifetime preservation
  across transient window detachment.
- Verification: **focused tests, source build, corrected Chromium product gate,
  and two-patch archive replay Pass**; Experiment 2 result review approved.

## Current State (Issue 26071420489654)

- Upstream repository: `https://github.com/ghostty-org/ghostty`
- Upstream base commit: `53bd14fecfd68c6c0ab64d37b5943247299e2b40`
- Local fork working tree: `forks/ghostty`
- Product branch: `issue-26071420489654-ghostty-restoration`
- Product HEAD (base + product commit):
  `e380e7211d12c0da2ad7f8a1796d5793e12f11fc`
- Product tree: `362ce2b98d3700ab1a00c231614388d53dff5786`
- Issue archive: `patches/ghostty/patches/issue-26071420489654/`
- Patch:
  `patches/ghostty/patches/issue-26071420489654/0001-astrohacker-Terminal-ghostty-product-patch-on-tip-is.patch`
- Patch SHA-256:
  `e620a06511f57372488dd640459db4700d99cd0a3c5601936b515faada6b9387`
- Archive aggregate SHA-256:
  `1b81bd9875d152221b8d7329883217f590a080f14f828743c0c705bacc4314dc`
- Verification: **archive replay Pass; not built**

## Historical Archives

- Issue `26071112000924`: `patches/ghostty/patches/issue-26071112000924/`
  on base `53bd14fecfd68c6c0ab64d37b5943247299e2b40`, product HEAD
  `ad9768db5138df928b3c307754e7dae0f7945af9`.
- Issue `26070412000013`: `patches/ghostty/patches/issue-26070412000013/`
  on base `2c62d182cec246764ff725096a70b9ef44996f7f`.

Executable product name: **`ahterm`** inside
`Astrohacker Terminal.app`.

## Apply (clean base)

```sh
BASE=53bd14fecfd68c6c0ab64d37b5943247299e2b40
git -C forks/ghostty worktree add /tmp/astrohacker-ghostty-restoration "$BASE"
git -C /tmp/astrohacker-ghostty-restoration am \
  "$PWD/patches/ghostty/patches/issue-26071420489654/0001-astrohacker-Terminal-ghostty-product-patch-on-tip-is.patch"
```

## Generate

```sh
git -C forks/ghostty format-patch -1 HEAD \
  -o patches/ghostty/patches/issue-26071420489654/
```

## Build / verify

```sh
scripts/build.sh ahterm --release
# identity
"./forks/ghostty/macos/build/Release/Astrohacker Terminal.app/Contents/MacOS/ahterm" +version
# host TermSurf browser-resolution unit test
cd forks/ghostty && zig build test \
  -Dtest-filter="termsurf server register matches profile and browser"
```

## Merge-upstream checklist

1. Discover tip: `git ls-remote https://github.com/ghostty-org/ghostty.git refs/heads/main`
2. Fetch; create `issue-NNNN-ghostty-upstream` from the tip SHA.
3. `git am` current archive (or re-apply prior product commit); resolve conflicts.
4. Build `ahterm` Release; run `+version` and TermSurf unit filters.
5. `git format-patch -1` into `patches/ghostty/patches/issue-NNNN/`.
6. Update this README Current State (base SHA, branch, archive path, date).

Do not commit `forks/ghostty` or temporary worktrees to the Astrohacker repo.
