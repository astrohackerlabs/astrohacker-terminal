use std::collections::HashMap;
use std::path::PathBuf;

use nu_cli::{ModeDispatcher, ModeResult};

use crate::alt_process::AltShellProcess;
use crate::config::AltShellKind;
use crate::shell::ShellState;
use crate::shell_engine::ShellEngine;

pub struct ShannonDispatcher {
    alt: AltShellKind,
    process: AltShellProcess,
}

impl ShannonDispatcher {
    pub fn new(alt: AltShellKind) -> Self {
        let process = AltShellProcess::new(alt);
        ShannonDispatcher { alt, process }
    }

    pub fn alt_kind(&self) -> AltShellKind {
        self.alt
    }

    /// Get the current env vars from the alt shell (after login initialization).
    pub fn env_vars(&mut self) -> HashMap<String, String> {
        self.process.capture_env()
    }
}

impl ModeDispatcher for ShannonDispatcher {
    fn execute(
        &mut self,
        mode: &str,
        command: &str,
        env: HashMap<String, String>,
        cwd: PathBuf,
    ) -> ModeResult {
        let state = ShellState {
            env,
            cwd,
            last_exit_code: 0,
        };
        if mode == self.alt.as_str() {
            self.process.inject_state(&state);
            let result = self.process.execute(command);
            ModeResult {
                env: result.env,
                cwd: result.cwd,
                exit_code: result.last_exit_code,
            }
        } else {
            ModeResult {
                env: state.env,
                cwd: state.cwd,
                exit_code: 127,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::TEST_ENV_LOCK;

    #[test]
    fn dispatcher_kind_matches_config_and_routes() {
        let _guard = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        for kind in [AltShellKind::Bash, AltShellKind::Zsh] {
            let mut d = ShannonDispatcher::new(kind);
            assert_eq!(d.alt_kind(), kind);
            let res = d.execute(
                kind.as_str(),
                "true",
                HashMap::new(),
                PathBuf::from("/"),
            );
            assert_eq!(res.exit_code, 0, "kind {}", kind.as_str());

            let bad = d.execute("nu", "true", HashMap::new(), PathBuf::from("/"));
            assert_eq!(bad.exit_code, 127);

            let other = if kind == AltShellKind::Bash {
                "zsh"
            } else {
                "bash"
            };
            let wrong_alt = d.execute(other, "true", HashMap::new(), PathBuf::from("/"));
            assert_eq!(wrong_alt.exit_code, 127);
        }
    }
}
