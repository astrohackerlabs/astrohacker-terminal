# web TUI

Browser chrome for Astrohacker Terminal, rendered inside the terminal pane.
Built with Rust and [ratatui](https://ratatui.rs/).

When the user types `web google.com`, this TUI draws the URL bar, viewport
border, and status bar. It connects to the GUI via Unix socket to send overlay
coordinates and receive mode/URL updates. The actual webpage renders as a GPU
texture overlay — the TUI handles only the chrome around it.

## Build

```bash
cargo build -p web
```

## Run

Inside an Astrohacker Terminal pane:

```bash
cargo run -p web -- https://google.com
```
