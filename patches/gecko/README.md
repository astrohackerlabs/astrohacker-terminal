# Gecko Patches

No implementation patch set currently exists for Gecko / Waterwolf.

Current state:

- Ignored working tree: `forks/gecko`
- Upstream remote: `https://github.com/mozilla/gecko-dev.git`
- Current checkout commit:
  `5836a062726f715fda621338a17b51aff30d0a8c`
- Checkout type: shallow partial clone
- `git rev-parse --is-shallow-repository`: `true`
- `remote.origin.promisor`: `true`
- `remote.origin.partialclonefilter`: `blob:none`
- Related planned engine: Waterwolf

The local checkout exists so the monorepo fork layout includes all intended
Astrohacker Terminal browser engines without committing upstream history. A
future issue or experiment should create Gecko implementation patches only after
there is a concrete Gecko / Waterwolf implementation path.
