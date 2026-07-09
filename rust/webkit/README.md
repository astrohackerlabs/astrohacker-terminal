# WebKit

WebKit-backed browser engine process for Astrohacker Terminal.

This crate builds `ah-webkitd`. It speaks the shared TermSurf protobuf/socket
protocol and uses the macOS WebKit ABI wrapper under `libtermsurf_webkit/`.

Useful commands from `rust/`:

```sh
cargo check -p webkit
cargo build -p webkit
```

The `libtermsurf_webkit` name is an internal ABI compatibility name. Do not
rename it without a dedicated compatibility issue.
