# Docs

Operational monorepo documentation for Astrohacker Terminal: install/release,
environment naming, XDG paths, legal package sources, Ghostty notes, keybindings,
and public-source templates.

**User-facing product docs** live on the website
([astrohacker.com/docs](https://astrohacker.com/docs) via
`bun/website-rr/app/routes/docs/`). Craft rules:
[`marketing/docs-writing.md`](../marketing/docs-writing.md).

| Path | Role |
| --- | --- |
| [`homebrew.md`](./homebrew.md) | Canonical install + release (Apple silicon cask) |
| [`environment.md`](./environment.md) | Process env taxonomy |
| [`xdg.md`](./xdg.md) | XDG layout for Terminal |
| [`keybindings.md`](./keybindings.md) | Product keybindings |
| [`ghostty.md`](./ghostty.md) | Ghostty relationship notes |
| `astrohacker-terminal-license` / `-notice` / `-trademarks.md` | Legal package sources |
| [`public-source/`](./public-source/) | Templates for public source mirror |

Engine research matrices and historical product essays were removed from this
tree; recover from git history if needed. Prefer **issues/** and **epics/** for
active R&D records.
