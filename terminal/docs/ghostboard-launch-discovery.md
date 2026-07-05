# Ghostboard Launch Discovery

Ghostboard has two local launch modes that must stay distinct while TermSurf is
under active development:

- debug runs from this repository; and
- installed distribution runs, which are tracked separately by Issue 819.

Issue 814 defines the debug contract. The goal is to make it obvious which
binary Ghostboard will spawn and to fail clearly instead of silently using an
old installed Chromium helper.

## Debug App Launch

The debug app binary is:

```bash
ghostboard/macos/build/Debug/Astrohacker Terminal.app/Contents/MacOS/aht
```

The geometry harness launches this binary directly from
`scripts/ghostboard-geometry-matrix.sh`. The app creates its normal terminal
session, listens on a PID-scoped TermSurf socket, and exposes that socket to
child shell commands through `TERMSURF_SOCKET`.

The `web` TUI discovers Ghostboard through `TERMSURF_SOCKET`. A successful debug
launch must show `HelloRequest` in the Ghostboard log before any browser launch
claim is trusted.

## Browser Selection

Ghostboard currently supports these browser selection rules:

| Web command                                   | Browser field received by Ghostboard | Spawn behavior                                                                        |
| --------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------- |
| `web --browser /absolute/path/to/browser URL` | absolute path                        | Spawn exactly that path.                                                              |
| `web URL`                                     | named/default `chromium`             | Debug: resolve through `TERMSURF_ROAMIUM_PATH`; release: installed `ah-chromiumd`.    |
| `web --browser chromium URL`                  | named `chromium`                     | Debug: resolve through `TERMSURF_ROAMIUM_PATH`; release: installed `ah-chromiumd`.    |
| `web --browser webkit URL`                    | named `webkit`                       | Debug: resolve through `TERMSURF_SURFARI_PATH`; release: installed `ah-webkitd`.      |
| `web --browser ladybird URL`                  | named `ladybird`                     | Debug: resolve through `TERMSURF_GIRLBAT_PATH`; release: installed `ah-ladybirdd`.    |
| `web --browser unsupported-name URL`          | unsupported named browser            | Fail as unsupported.                                                                  |

The supported named browsers are currently `chromium`, `webkit`, and the
prototype `ladybird`. Any other relative browser name is unsupported; pass an
absolute path when testing a custom browser executable.

In debug builds, named browser paths are intentionally explicit:

- `TERMSURF_ROAMIUM_PATH` must be set for named/default `chromium`;
- `TERMSURF_SURFARI_PATH` must be set for named `webkit`;
- `TERMSURF_GIRLBAT_PATH` must be set for named `ladybird`;
- each value must be an absolute path;
- debug harnesses set the Chromium helper to `chromium/src/out/Default/ah-chromiumd`;
- debug harnesses set the WebKit helper to the intended `ah-webkitd` path and, when
  needed for debug-only WebKit framework discovery, configure the matching
  runtime environment in the Ghostboard app process;
- debug harnesses set Ladybird to the intended repo-built `ah-ladybirdd` path;
- missing, empty, or relative values fail with a clear
  `SetOverlay: named browser unresolved` log line; and
- Ghostboard must not fall through to `/usr/local/roamium`,
  `/usr/local/bin/roamium`, `/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd`, or
  `/opt/homebrew/opt/astrohacker-terminal-ah-webkitd`, or `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd`
  during debug testing.

In non-debug builds, named browsers first accept their developer override if one
is present, then resolve through installed discovery:

| Browser    | Developer override      | Installed override                | Installed default                                |
| ---------- | ----------------------- | --------------------------------- | ------------------------------------------------ |
| `chromium` | `TERMSURF_ROAMIUM_PATH` | `TERMSURF_INSTALLED_ROAMIUM_PATH` | `/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd/ah-chromiumd`     |
| `webkit`   | `TERMSURF_SURFARI_PATH` | `TERMSURF_INSTALLED_SURFARI_PATH` | `/opt/homebrew/opt/astrohacker-terminal-ah-webkitd/ah-webkitd`         |
| `ladybird` | `TERMSURF_GIRLBAT_PATH` | `TERMSURF_INSTALLED_GIRLBAT_PATH` | `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd/bin/ah-ladybirdd` |

The `TERMSURF_ROAMIUM_PATH`, `TERMSURF_SURFARI_PATH`, and
`TERMSURF_GIRLBAT_PATH` variables are legacy-named Ghostboard-process developer
overrides.
They are read by the running Astrohacker Terminal app when it spawns browser
engine processes. Setting one of them only in an arbitrary shell that later runs
`web` does not affect an already-running Ghostboard process.

The `TERMSURF_INSTALLED_ROAMIUM_PATH`, `TERMSURF_INSTALLED_SURFARI_PATH`, and
`TERMSURF_INSTALLED_GIRLBAT_PATH` variables are legacy-named release discovery test
overrides. They let a release harness point installed discovery at a temporary
absolute path. Normal Homebrew installs should not need these variables; the
installed defaults above are the expected no-env path.

When Ghostboard spawns the WebKit helper, it preserves an inherited `DYLD_FRAMEWORK_PATH`
if the Ghostboard app process already has one. This keeps debug harnesses in
control of debug WebKit framework discovery. If no value is inherited,
Ghostboard sets the WebKit child process `DYLD_FRAMEWORK_PATH` to the directory
containing the resolved `ah-webkitd` executable. This is not a shell-local `web`
lookup override and users should not set it themselves for normal Homebrew
usage. It lets installed `ah-webkitd` load the WebKit frameworks beside
`/opt/homebrew/opt/astrohacker-terminal-ah-webkitd/ah-webkitd`.

Ghostboard keeps the pane/server/browser key as the requested browser name
(`chromium`) even when it spawns the executable from `TERMSURF_ROAMIUM_PATH`.
That preserves protocol identity: `BrowserReady` reports `browser=chromium`,
while the process spawn log records the resolved executable path.

## Harness Coverage

`scripts/ghostboard-geometry-matrix.sh launch-discovery-contract` validates the
launch contract without opening the GUI:

- the absolute-path command includes `--browser` with the debug Chromium helper path;
- the named/default command omits `--browser`;
- the named/default debug environment uses an absolute Chromium helper path; and
- the invalid-env sentinel is relative.

Runtime coverage is provided by:

- `scripts/ghostboard-geometry-matrix.sh initial-open` for the explicit absolute
  browser path;
- `scripts/ghostboard-geometry-matrix.sh named-roamium-debug-launch` for
  default/named `chromium` resolving through `TERMSURF_ROAMIUM_PATH`; and
- `scripts/ghostboard-geometry-matrix.sh named-roamium-invalid-env` for clear
  failure without creating a pending `default/chromium` server or spawning a
  browser process; and
- `scripts/ghostboard-geometry-matrix.sh installed-roamium-release-launch` for
  release named/default `chromium` resolving through installed discovery without
  `TERMSURF_ROAMIUM_PATH`; and
- `scripts/test-issue-867-release-no-env-browser-discovery.sh` for release named
  `chromium`, `webkit`, and `ladybird` resolving through installed defaults
  without any browser path environment variable.

## Boundary With Issue 819

Issue 814 does not define the final installed distribution path. It defines the
debug contract and prevents accidental installed-binary fallback while the app
is being tested from the repository.

Issue 819 owns packaging identity and normal installed distribution behavior. It
defines the installed Chromium helper location as
`/opt/homebrew/opt/astrohacker-terminal-ah-chromiumd/ah-chromiumd` and the installed WebKit helper
location as `/opt/homebrew/opt/astrohacker-terminal-ah-webkitd/ah-webkitd`. Issue 885 adds the installed
Ladybird prototype location as `/opt/homebrew/opt/astrohacker-terminal-ah-ladybirdd/bin/ah-ladybirdd`,
matching the Homebrew cask and manual install scripts.
