# Ladybird

Ladybird-backed browser engine prototype for Astrohacker Terminal.

This crate builds the `ah-ladybirdd` engine process. It speaks the shared
TermSurf protobuf/socket protocol used by the terminal frontend and wraps the
Ladybird embedding work under `libtermsurf_ladybird/`.

Useful commands from `rust/`:

```sh
cargo check -p ladybird
cargo build -p ladybird
```

The `libtermsurf_ladybird` name is an internal ABI compatibility name. Do not
rename it without a dedicated compatibility issue.
