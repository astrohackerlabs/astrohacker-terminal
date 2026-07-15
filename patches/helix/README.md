# Helix Patches

Astrohacker Editor uses a patched Helix fork derived from Velix. The fork
working tree is local-only under `forks/helix`; this directory tracks the
documentation and patch archives needed to reconstruct Astrohacker Editor's
Helix changes without importing Helix history into the company repo.

## Current State (Issue 26071420489654)

- Upstream repository: `https://github.com/helix-editor/helix`
- Upstream base commit: `14d6bc0febed9c692048271a8ae2362ac969c6e0`
- Product branch: `issue-26071420489654-astrohacker-editor-restoration`
- Product HEAD: `4b4301fb1531567f23858d3a322425b17e1f13be`
- Product tree: `6ebaacaf8ca5cec9e056436c16a4e385c4db1261`
- Current live safety-fence branch: `issue-0924-astrohacker-editor`
- Historical Issue 924 branch metadata:
  `issue-26071112000924-astrohacker-editor`
- Local fork working tree: `forks/helix`
- Issue archive: `patches/helix/patches/issue-26071420489654/`
- Current patch:
  `patches/helix/patches/issue-26071420489654/0001-astrohacker-Editor-helix-product-patch-on-tip-issue-.patch`
- Patch SHA-256:
  `765644a0781a3fe18aedb3af3af2e8cf5679ad443ab1ea18285dede524140dab`
- Archive aggregate SHA-256:
  `0098f654abc5c34fbf57b796849f4bbbae574d51eb23d6984b6323ae5cee62f2`
- Verification: **archive replay Pass; not built**
- Executable product name: **`ahed`**
- Prior archive (Issue 26070612000904): base `4ed0899b0b0c3f7dadba550272cb300b871d8fa9`,
  `patches/helix/patches/issue-26070612000904/`

Issue `26071112000924` remains the tag-stored `0.1.17` archive. Every earlier
or later archive and its build/overlay evidence remains historical; none of
those build or boundedness claims is inherited by this archive-only replay.

### Merge-upstream

1. `git ls-remote … refs/heads/master` → tip SHA.
2. Branch `issue-NNNN-astrohacker-editor` from tip; `git am` current archive.
3. If patch explodes (>>~100 product files of unexpected upstream churn), reject
   tip with evidence and pick newest main-reachable bounded base.
4. `scripts/build.sh ahed --release`; `ahed --version`; `ahed --health rust`.
5. `git format-patch -1` into `patches/helix/patches/issue-NNNN/`; update this
   section.

### Historical Issue 26070612000904 base notes

- Rejected candidate base tag: `25.07.1`
- Rejected candidate tag object:
  `ac94841019910ff405f31a8668389a06a169e0e5`
- Rejected candidate tag commit:
  `a05c151bb6e8e9c65ec390b0ae2afe7a5efd619b`
- Velix source repo: `~/dev/velix`
- Velix source commit: `5952cce51968df307937c4b88d80c87d53dc6f62`
- Branch convention (historical): `issue-26070612000904-astrohacker-editor`

## Base Validation

Velix has squashed history, so the base cannot be proven by `git merge-base`.
The selected base is validated by overlaying tracked Velix product files onto a
clean upstream checkout and inspecting the resulting diff.

Candidate results from Issue 26070612000904 Experiment 1:

- Helix `25.07.1` commit `a05c151b`: rejected because the overlay touched 507
  files, including broad upstream source and runtime drift.
- Helix `master` at `39b3f22b`: rejected because the overlay still touched 129
  files, including broad runtime/query drift.
- Helix `4ed0899b`: selected because the overlay touched 40 files and was
  bounded to expected Velix deltas: Vim keymap/profile work, `vlx` identity,
  version/release identity, documentation, completions, default theme, and
  workspace-trust behavior.

Unexpected broad upstream drift means the base commit is wrong and must not be
accepted as a valid patch archive.

## Patch Generation Model

The raw Velix overlay is an input, not the final patch archive. It preserves
the useful fork changes from `~/dev/velix`, but it still contains old Velix
identity, `vlx` executable naming, Helix path defaults, and `HELIX_*` runtime
environment variables.

The accepted Astrohacker Editor archive is generated only after applying the
Astrohacker product edits on top of that overlay:

- public product and executable names become Astrohacker Editor and `ahe`;
- global paths become `astrohacker/editor`;
- workspace-local paths become `.astrohacker/editor`;
- runtime overrides become `ASTROHACKER_EDITOR_RUNTIME` and
  `ASTROHACKER_EDITOR_DEFAULT_RUNTIME`;
- `ASTROHACKER_EDITOR_RUNTIME` is searched before the user config runtime so
  release grammar builds and package health checks can target an explicit
  runtime directory, while user config runtime files still override the
  packaged default runtime;
- obsolete Velix workflow/release material remains excluded.

After those edits, the committed patch below is the authoritative
reconstruction artifact:

```text
patches/helix/patches/issue-26070612000904/0001-astrohacker-editor.patch
```

Future regeneration must reproduce that Astrohacker-edited tree before diffing.
Do not regenerate the archive from the raw Velix overlay alone.

## Overlay Input

Create the intermediate overlay from a clean Velix checkout at the recorded
source commit. Use a fresh base checkout and `rsync -a --delete` so deletions
are captured:

```sh
rm -rf /tmp/ahe-helix-gen /tmp/ahe-velix-overlay
git clone https://github.com/helix-editor/helix.git /tmp/ahe-helix-gen
git -C /tmp/ahe-helix-gen checkout 4ed0899b0b0c3f7dadba550272cb300b871d8fa9
git -C /tmp/ahe-helix-gen switch -c issue-26070612000904-astrohacker-editor

mkdir -p /tmp/ahe-velix-overlay
git -C "$HOME/dev/velix" archive 5952cce51968df307937c4b88d80c87d53dc6f62 |
  tar -x -C /tmp/ahe-velix-overlay

rm -rf \
  /tmp/ahe-velix-overlay/.claude \
  /tmp/ahe-velix-overlay/.codex \
  /tmp/ahe-velix-overlay/.github \
  /tmp/ahe-velix-overlay/issues \
  /tmp/ahe-velix-overlay/epics \
  /tmp/ahe-velix-overlay/skills \
  /tmp/ahe-velix-overlay/scripts \
  /tmp/ahe-velix-overlay/AGENTS.md \
  /tmp/ahe-velix-overlay/CLAUDE.md \
  /tmp/ahe-velix-overlay/docs/homebrew.md

rsync -a --delete \
  --exclude .git \
  --exclude .github \
  --exclude scripts \
  --exclude issues \
  --exclude epics \
  --exclude skills \
  --exclude .claude \
  --exclude .codex \
  /tmp/ahe-velix-overlay/ /tmp/ahe-helix-gen/

# Apply the Astrohacker Editor source edits described above before diffing.
git -C /tmp/ahe-helix-gen add -A
git -C /tmp/ahe-helix-gen diff --cached --binary \
  > patches/helix/patches/issue-26070612000904/0001-astrohacker-editor.patch
```

The old Velix issue workflow, agent skills, GitHub workflows, and standalone
Homebrew release tooling are excluded from the product patch archive. They are
historical project scaffolding, not the Astrohacker Editor product surface.

## Applying Patches

Apply to a clean checkout at the recorded base. Issue 26070612000904 Experiment 2 verified
that the patch below replays cleanly and reconstructs the local `ahe` build
tree:

```sh
git -C forks/helix checkout 4ed0899b0b0c3f7dadba550272cb300b871d8fa9
git -C forks/helix switch -c issue-26070612000904-astrohacker-editor
git -C forks/helix apply \
  ../../patches/helix/patches/issue-26070612000904/0001-astrohacker-editor.patch
```

## Verification

```sh
git -C forks/helix rev-parse HEAD
git -C forks/helix reset --hard 4ed0899b0b0c3f7dadba550272cb300b871d8fa9
git -C forks/helix apply --check \
  ../../patches/helix/patches/issue-26070612000904/0001-astrohacker-editor.patch
git -C forks/helix apply \
  ../../patches/helix/patches/issue-26070612000904/0001-astrohacker-editor.patch
git -C forks/helix status --short --ignored
git diff --check
```

Before accepting a regenerated patch, inspect the diffstat. It should remain
bounded to the expected Astrohacker Editor product deltas. Do not commit
`forks/helix` or temporary worktrees to the Astrohacker repo.

## Issue 26070612000904 Experiment 2 Evidence

Experiment 2 generated `0001-astrohacker-editor.patch` from the
Astrohacker-edited overlay, then reset `forks/helix` to the selected base and
replayed the committed patch. The replayed tree's staged binary diff matched
the archived patch byte-for-byte.

Recorded evidence:

- Patch generation: `logs/issue-26070612000904-exp2-patch-generation-fixed.log`
- Patch replay: `logs/issue-26070612000904-exp2-patch-replay-fixed.log`
- Release build: `logs/issue-26070612000904-exp2-ahe-release-build-fixed.log`
- Version/help/health checks:
  `logs/issue-26070612000904-exp2-ahe-version-fixed.log`,
  `logs/issue-26070612000904-exp2-ahe-help-fixed.log`,
  `logs/issue-26070612000904-exp2-ahe-health-rust-fixed.log`
- Source searches: `logs/issue-26070612000904-exp2-source-searches-fixed.log`
- Focused tests:
  `logs/issue-26070612000904-exp2-test-helix-loader-workspace-trust-fixed.log`,
  `logs/issue-26070612000904-exp2-test-helix-term-vim-profile-fixed.log`,
  `logs/issue-26070612000904-exp2-test-closed-doc-regression-fixed.log`

The local `ahe` build disables Helix's automatic grammar fetch/build step with
`HELIX_DISABLE_AUTO_GRAMMAR_BUILD=1`. Packaging built grammar artifacts is
deferred to the release experiment.

Retained Helix build/development knobs:

- `HELIX_DISABLE_AUTO_GRAMMAR_BUILD` remains the inherited build-script opt-out
  for automatic grammar compilation.
- `HELIX_NIX_BUILD_REV` remains the inherited Nix build revision input.

These are build/development controls, not runtime path controls. Runtime path
overrides use `ASTROHACKER_EDITOR_RUNTIME` and
`ASTROHACKER_EDITOR_DEFAULT_RUNTIME`.
