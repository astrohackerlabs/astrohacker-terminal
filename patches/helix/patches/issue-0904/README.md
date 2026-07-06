# Issue 0904 Helix Patch Archive

This archive will capture Velix-derived Helix fork changes for the Astrohacker
Editor migration.

- Selected base:
  `4ed0899b0b0c3f7dadba550272cb300b871d8fa9`
- Source: `~/dev/velix` at
  `5952cce51968df307937c4b88d80c87d53dc6f62`
- Branch convention: `issue-0904-astrohacker-editor`
- Current patch: `0001-astrohacker-editor.patch`

Issue 904 Experiment 1 validated the base with an overlay diff preview. The
selected base produced a 40-file product diff bounded to expected Velix deltas.

Issue 904 Experiment 2 generated the full patch archive. The raw Velix overlay
is only an intermediate input. The final patch also includes Astrohacker Editor
naming, `ahe` executable identity, `astrohacker/editor` XDG paths,
`.astrohacker/editor` workspace-local paths, full `ASTROHACKER_EDITOR_*`
runtime environment variables, and local build tooling.

Replay must be verified in a fresh checkout with `git apply --check` and
`git apply`; all local build, version, help, and health checks must run against
the patch-reconstructed checkout rather than the intermediate overlay tree.

Experiment 2 replayed this patch against
`4ed0899b0b0c3f7dadba550272cb300b871d8fa9` and verified the resulting local
binary as `Astrohacker Editor 0.1.12`.

Completion review found that the first generated patch accidentally reverted
the selected base commit's closed-document save crash fix. The accepted patch
preserves that base fix and its regression test; it does not carry reverse
hunks for `helix-term/src/commands/lsp.rs`,
`helix-term/src/commands/typed.rs`, or
`helix-term/tests/test/commands/write.rs`.
