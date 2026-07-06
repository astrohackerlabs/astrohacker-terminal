# Homebrew

Astrohacker ships to macOS through the `astrohackerlabs/astrohacker` Homebrew
tap. The cask currently targets Apple silicon macOS and installs Astrohacker
Terminal. Astrohacker Shell and Astrohacker Editor will join the same bundle
later.

## Install

```bash
brew tap astrohackerlabs/astrohacker
brew trust astrohackerlabs/astrohacker
brew install --cask astrohacker
```

## Initial 0.1.0 Signing Model

Astrohacker `0.1.0` is distributed through the Astrohacker Homebrew tap
before Developer ID notarization is in place. The cask postflight clears
quarantine attributes from the installed app and runtime artifacts, then applies
ad-hoc signatures to the app bundle, CLI wrappers, and bundled browser runtime
executables and libraries.

`brew trust astrohackerlabs/astrohacker` trusts the tap source used by Homebrew.
It does not notarize the downloaded app with Apple. Treat the initial cask as a
trusted Astrohacker tap install while notarized distribution is still future
work.

To upgrade an existing install:

```bash
brew update
brew upgrade --cask astrohacker
```

## Installed Layout

The cask installs:

- `Astrohacker Terminal.app` to `/Applications/Astrohacker Terminal.app`;
- `web` to the Homebrew binary path;
- `termsurf` to the Homebrew binary path;
- `ah-chromiumd` and Chromium runtime resources to
  `/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd/`;
- `ah-webkitd` and WebKit runtime resources to
  `/opt/homebrew/opt/astrohacker-terminal-ah-webkitd/`;
- `ah-ladybirdd` prototype and Ladybird runtime resources to
  `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd/`;
- GTUI Deno app assets to
  `/opt/homebrew/opt/astrohacker-terminal-gtui/`.

The release tarball contains the same top-level package contract:

- `Astrohacker Terminal.app/`;
- `web`;
- `termsurf`;
- `gtui/`;
- `ah-chromiumd/`;
- `ah-webkitd/`;
- `ah-ladybirdd/`.

## Verification

After install or upgrade, verify that `web` can open a page from inside
Astrohacker Terminal without passing a repo-local browser path or setting
browser path environment variables. That proves the installed app, installed
`web` binary, and installed browser runtime discovery paths are working
together.

For local release validation, the smoke test should record elapsed time and
evidence such as logs or screenshots showing:

- `Astrohacker Terminal.app` launched from the installed app path;
- the TermSurf socket was created;
- `web --browser chromium https://example.com` opened and rendered the page;
- Astrohacker Terminal resolved the installed `ah-chromiumd` path:
  `/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd/ah-chromiumd`;
- `web --browser webkit https://example.com` opened and rendered the page;
- Astrohacker Terminal resolved the installed `ah-webkitd` path:
  `/opt/homebrew/opt/astrohacker-terminal-ah-webkitd/ah-webkitd`;
- `web --browser ladybird http://127.0.0.1:<fixture>/` opened the local fixture
  through the installed Ladybird prototype;
- Astrohacker Terminal resolved the installed `ah-ladybirdd` path:
  `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd/bin/ah-ladybirdd`;
- the installed Ladybird prototype loaded its resources from
  `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd/Resources` and all non-system
  dylib dependencies resolved from
  `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd/lib`;
- no smoke required `TERMSURF_ROAMIUM_PATH`, `TERMSURF_SURFARI_PATH`,
  `TERMSURF_GIRLBAT_PATH`, `TERMSURF_INSTALLED_ROAMIUM_PATH`,
  `TERMSURF_INSTALLED_SURFARI_PATH`, or `TERMSURF_INSTALLED_GIRLBAT_PATH`.

Ladybird is included as a prototype only. Its Homebrew presence proves installed
runtime packaging and gives us a testable Ladybird-backed engine, but it does
not imply browser parity, PDF parity, visual screenshot parity, or production
readiness.
