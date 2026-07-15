# Reedline Patches

Astrohacker Shell uses a **path pin** of upstream Reedline under
`forks/reedline`. There is **no product source patch** — only an exact tip pin
that Nushell and `rust/ahsh` share via path dependency. Issue
`26071420489654` records that zero-delta input in a README-only archive rather
than fabricating a no-op commit.

## Current State (Issue 26071420489654)

- Upstream repository: `https://github.com/nushell/reedline`
- Upstream base policy: **latest commit on upstream `main`**
- Upstream base / product HEAD: `028d4b54eb7b9740aa98eec9f9ca3dc0c6c397ce`
- Product tree: `ef10dad013474dc7580126f5263c3323e17f3e1f`
- Version: `0.49.0`
- Local fork working tree: `forks/reedline`
- Restoration branch: `issue-26071420489654-reedline-restoration` (tip pin only)
- Current live safety-fence branch: `issue-0924-reedline`
- Issue archive:
  `patches/reedline/patches/issue-26071420489654/README.md`
- Product commits / patch files: `0` / `0`
- Empty patch-inventory aggregate SHA-256:
  `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- Verification: **pin replay Pass; not built**
- Consumers:
  - `forks/nushell` workspace `reedline = { path = "../reedline", version = "0.49.0" }`
  - `rust/ahsh` `reedline = { version = "0.49.0", path = "../../forks/reedline", … }`
- Prior pin (Issue 26070612000903): tag `v0.48.0` /
  `c076ee0c97f4e3e113db03957269d68b83e9784a`

Issue `26071112000924` remains the historical `0.1.17` pin metadata. Every
prior/later Reedline record remains unchanged; the new archive is the active
restoration record.

## Merge-upstream checklist

1. `git ls-remote https://github.com/nushell/reedline.git refs/heads/main`
2. Checkout tip on `issue-NNNN-reedline` (or detached tip).
3. Confirm `Cargo.toml` version; rebuild `ahsh` with Nushell path pin.
4. If product edits appear, start an issue-scoped patch archive; until then
   document tip-only pin here.

## Verification

```sh
git -C forks/reedline rev-parse \
  issue-26071420489654-reedline-restoration
# expect 028d4b54eb7b9740aa98eec9f9ca3dc0c6c397ce
grep '^version' forks/reedline/Cargo.toml
scripts/build.sh ahsh --release
```

Do not commit `forks/reedline` or temporary worktrees to the Astrohacker repo.
