pub mod alt_process;
pub mod banner;
pub mod config;
pub mod dispatcher;
pub mod executor;
pub mod shell;
pub mod shell_engine;

// Backward-compatible name for older references.
pub mod bash_process {
    pub use crate::alt_process::AltShellProcess as BashProcess;
}
