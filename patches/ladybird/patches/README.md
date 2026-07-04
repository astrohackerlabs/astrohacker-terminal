# Ladybird Patch Archive

This directory stores TermSurf patch archives for Ladybird source changes.

Patch archives should be created only from committed Ladybird source branches.
Use filenames or subdirectories that include the issue number and upstream
baseline so future agents can reconstruct the source state.

Suggested shape:

```text
ladybird/patches/
  issue-0884/
    0001-add-termsurf-ladybird-embedding.patch
```

Do not place full Ladybird source checkouts or build products here.
