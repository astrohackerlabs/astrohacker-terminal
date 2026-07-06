# XDG Directories

Astrohacker Terminal follows the
[XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
for storing user data.

## Directories

| Variable          | Default          | Terminal path                          | Contents                                      |
| ----------------- | ---------------- | -------------------------------------- | --------------------------------------------- |
| `XDG_CONFIG_HOME` | `~/.config`      | `~/.config/astrohacker/terminal/`      | Astrohacker Terminal configuration and themes |
| `XDG_DATA_HOME`   | `~/.local/share` | `~/.local/share/astrohacker/terminal/` | Browser profile data                          |
| `XDG_STATE_HOME`  | `~/.local/state` | `~/.local/state/astrohacker/terminal/` | Log files, crash reports, SSH cache state     |
| `XDG_CACHE_HOME`  | `~/.cache`       | `~/.cache/astrohacker/terminal/`       | Terminal-owned cache data, if needed          |

The product folder is always `astrohacker/terminal` under the XDG base
directory. The shared `astrohacker` root is reserved for suite-level
Astrohacker settings and future products.

## What goes where

**Config** (`XDG_CONFIG_HOME/astrohacker/terminal/`):

- `config` — Astrohacker Terminal configuration
- `themes/` — user themes

**Data** (`XDG_DATA_HOME/astrohacker/terminal/`):

- `chromium-profiles/<profile>/` — Per-profile Chromium data (cookies,
  localStorage, browsing history, cached assets). One directory per browser
  profile name.
- `webkit-profiles/<profile>/` — Per-profile WebKit data.
- `browser-profiles/<profile>/` — Fallback per-profile browser data for other
  engines.

## Environment variables

If `XDG_DATA_HOME` is set, Astrohacker Terminal uses it. Otherwise it falls back to
`$HOME/.local/share`. The same pattern applies to `XDG_CONFIG_HOME` (used by
Astrohacker Terminal for its config), `XDG_STATE_HOME` (default:
`$HOME/.local/state`), and `XDG_CACHE_HOME` (default: `$HOME/.cache`).

**State** (`XDG_STATE_HOME/astrohacker/terminal/`):

- `chromium-server.log` — Chromium Profile Server log output. Created
  automatically on server startup.
- `crash/` — crash reports.
- `ssh_cache` — SSH terminfo cache state.

**Cache** (`XDG_CACHE_HOME/astrohacker/terminal/`):

- `sentry/` — Sentry crash reporting cache when XDG cache paths are used.
