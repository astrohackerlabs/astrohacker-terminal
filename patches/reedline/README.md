# Reedline Patches

Astrohacker Shell uses a **path pin** of upstream Reedline under
`forks/reedline`. There is currently **no product source patch** — only a tip
checkout that Nushell and `rust/ahsh` share via path dependency.

## Current State (Issue 26071112000924)

- Upstream repository: `https://github.com/nushell/reedline`
- Upstream base policy: **latest commit on upstream `main`**
- Upstream base / product HEAD: `028d4b54eb7b9740aa98eec9f9ca3dc0c6c397ce`
- Version: `0.49.0`
- Local fork working tree: `forks/reedline`
- Branch: `issue-26071112000924-reedline` (tip pin only)
- Issue archive: **none** (empty product delta vs tip)
- Consumers:
  - `forks/nushell` workspace `reedline = { path = "../reedline", version = "0.49.0" }`
  - `rust/ahsh` `reedline = { version = "0.49.0", path = "../../forks/reedline", … }`
- Prior pin (Issue 26070612000903): tag `v0.48.0` /
  `c076ee0c97f4e3e113db03957269d68b83e9784a`

## Merge-upstream checklist

1. `git ls-remote https://github.com/nushell/reedline.git refs/heads/main`
2. Checkout tip on `issue-NNNN-reedline` (or detached tip).
3. Confirm `Cargo.toml` version; rebuild `ahsh` with Nushell path pin.
4. If product edits appear, start an issue-scoped patch archive; until then
   document tip-only pin here.

## Verification

```sh
git -C forks/reedline rev-parse HEAD
# expect 028d4b54eb7b9740aa98eec9f9ca3dc0c6c397ce after Issue 26071112000924
grep '^version' forks/reedline/Cargo.toml
scripts/build.sh ahsh --release
```

Do not commit `forks/reedline` or temporary worktrees to the Astrohacker repo.
