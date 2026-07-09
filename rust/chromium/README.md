# Chromium

Chromium-backed browser engine process for Astrohacker Terminal.

This crate builds `ah-chromiumd`. It speaks the shared TermSurf
protobuf/socket protocol and links against the patched Chromium work tracked
through `forks/chromium/` and `patches/chromium/`.

Useful commands from `rust/`:

```sh
cargo check -p chromium
cargo build -p chromium
```

Chromium must be prepared separately before full linking or runtime tests.
