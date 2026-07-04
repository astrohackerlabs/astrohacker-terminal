# Astrohacker Terminal

Astrohacker Terminal is a terminal with a real browser in the pane. Run `web`,
open a URL, and the page appears alongside shells, editors, and other terminal
workflows.

This public repository contains the open source client material synced from the
private Astrohacker monorepo for source releases. It includes:

- `terminal/` — product docs, assets, public build/install helpers, and smoke
  scripts.
- `rust/` — Rust workspace crates for `web`, Roamium, Surfari, Girlbat, GTUI,
  and protocol/native support code.
- `bun/astrohacker-terminal-website/` — public Terminal website source.
- `bun/gtui-app/` — GTUI app assets used by the Terminal package.
- `patches/` — fork patch archives and reconstruction notes for Chromium,
  WebKit, Ladybird, Ghostty, and Gecko.

Large upstream fork checkouts and build outputs are not committed here. Use the
patch records under `patches/` to reconstruct local engine workspaces when
developing browser integrations.

## Install

The Homebrew cask currently targets Apple silicon macOS:

```bash
brew tap astrohackerlabs/astrohacker
brew trust astrohackerlabs/astrohacker
brew install --cask astrohacker-terminal
```

To upgrade:

```bash
brew update
brew upgrade --cask astrohacker-terminal
```

## Build

Development builds require Xcode, Zig, Rust, Bun, Chromium's `depot_tools`, and
the WebKit/Ladybird build tooling described in the patch documentation.

```bash
brew install zig
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://bun.sh/install | bash
```

Prepare local engine workspaces from the recorded patch archives, then build the
client components:

```bash
./terminal/scripts/build.sh chromium
./terminal/scripts/build.sh roamium
./terminal/scripts/build.sh webkit
./terminal/scripts/build.sh surfari
./terminal/scripts/build.sh webtui
./terminal/scripts/build.sh ghostboard
```

For a release-style local build:

```bash
./terminal/scripts/build.sh all --release
```

The app bundle is written to:

```text
forks/ghostty/macos/build/Release/Astrohacker Terminal.app
```

## Run

During development, launch the Ghostty-based frontend from the reconstructed
Ghostty workspace:

```bash
cd forks/ghostty
zig build -Demit-macos-app=false
cd macos
./build.nu --configuration Debug --action build
```

Inside Astrohacker Terminal, run the debug `web` binary and point it at a local
engine build:

```bash
./rust/target/debug/web \
  --browser ./forks/chromium/src/out/Default/roamium \
  https://example.com
```

## License

See `LICENSE`, `NOTICE`, and `TRADEMARKS.md`.
