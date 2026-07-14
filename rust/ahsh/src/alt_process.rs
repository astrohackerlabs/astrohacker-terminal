//! Persistent alt-shell subprocess (bash or zsh) with nonced sentinel protocol.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::config::AltShellKind;
use crate::executor::{self, shell_escape};
use crate::shell::ShellState;
use crate::shell_engine::ShellEngine;

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(1);

// Reserved inside the managed alt-shell process. pre_exec creates these after
// std::process installs the pipes but before login files can redirect fd 1/2.
const PROTECTED_STDERR_FD: i32 = 8;
const PROTECTED_STDOUT_FD: i32 = 9;

pub struct AltShellProcess {
    kind: AltShellKind,
    _child: Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
    stderr_rx: Receiver<String>,
    pending_state: Option<ShellState>,
}

impl AltShellProcess {
    pub fn kind(&self) -> AltShellKind {
        self.kind
    }

    pub fn new(kind: AltShellKind) -> Self {
        let mut cmd = match kind {
            AltShellKind::Bash => {
                let mut c = Command::new("bash");
                c.arg("--login");
                c
            }
            AltShellKind::Zsh => {
                let mut c = Command::new("zsh");
                c.arg("-l");
                c
            }
        };

        #[cfg(unix)]
        // SAFETY: this runs in the child after std::process installs fd 1/2.
        // It uses only async-signal-safe libc calls before exec.
        unsafe {
            cmd.pre_exec(|| {
                for (source, target) in [
                    (libc::STDERR_FILENO, PROTECTED_STDERR_FD),
                    (libc::STDOUT_FILENO, PROTECTED_STDOUT_FD),
                ] {
                    if libc::dup2(source, target) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    let flags = libc::fcntl(target, libc::F_GETFD);
                    if flags == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    if libc::fcntl(target, libc::F_SETFD, flags & !libc::FD_CLOEXEC) == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("failed to spawn {} process: {e}", kind.as_str()));

        let stdin = child.stdin.take().expect("failed to take alt shell stdin");
        let stdout = child
            .stdout
            .take()
            .expect("failed to take alt shell stdout");
        let stderr = child
            .stderr
            .take()
            .expect("failed to take alt shell stderr");

        let (stderr_tx, stderr_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if stderr_tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut bp = AltShellProcess {
            kind,
            _child: child,
            stdin,
            stdout_reader: BufReader::new(stdout),
            stderr_rx,
            pending_state: None,
        };

        bp.run_setup();
        bp
    }

    fn run_setup(&mut self) {
        let nonce = Self::next_nonce();
        let stdout_mark = format!("==AHSH_BOOTSTRAP_STDOUT_{nonce}==");
        let stderr_mark = format!("==AHSH_BOOTSTRAP_STDERR_{nonce}==");

        // Bootstrap is a separate, non-secret phase. No env dump source is
        // sent until startup redirects are restored and tracing is disabled.
        let setup = match self.kind {
            AltShellKind::Bash => {
                // Login profiles already ran. Reapply protocol controls.
                format!(
                    "exec 1>&{PROTECTED_STDOUT_FD} 2>&{PROTECTED_STDERR_FD}\n\
                     set +v +x\n\
                     shopt -s expand_aliases\n\
                     PS1=\n\
                     trap 'true' INT\n\
                     printf '%s\\n' '{stdout_mark}' >&{PROTECTED_STDOUT_FD}\n\
                     printf '%s\\n' '{stderr_mark}' >&{PROTECTED_STDERR_FD}\n"
                )
            }
            AltShellKind::Zsh => {
                // Source .zshrc first (may double-source if zprofile already did — accepted).
                // Then reapply protocol so user rc cannot leave prompts/traps broken.
                format!(
                    r#"
exec 1>&{PROTECTED_STDOUT_FD} 2>&{PROTECTED_STDERR_FD}
[ -f "${{ZDOTDIR:-$HOME}}/.zshrc" ] && . "${{ZDOTDIR:-$HOME}}/.zshrc"
exec 1>&{PROTECTED_STDOUT_FD} 2>&{PROTECTED_STDERR_FD}
unsetopt verbose xtrace
setopt aliases
PROMPT=
RPROMPT=
PS1=
trap 'true' INT
print -r -- '{stdout_mark}' >&{PROTECTED_STDOUT_FD}
print -r -- '{stderr_mark}' >&{PROTECTED_STDERR_FD}
"#
                )
            }
        };

        if self.write_block(&setup) {
            let stdout_done = self.read_stdout_until(&stdout_mark, true);
            let stderr_done = self.read_stderr_until(&stderr_mark, true, &[&stdout_mark]);
            if !stdout_done || !stderr_done {
                eprintln!("ahsh: {} bootstrap did not complete", self.kind.as_str());
            }
        }
    }

    pub fn capture_env(&mut self) -> HashMap<String, String> {
        let state = self.run_command_with_visibility("true", false);
        state.env
    }

    fn build_preamble(&mut self) -> String {
        let state = match self.pending_state.take() {
            Some(s) => s,
            None => return String::new(),
        };

        let mut preamble = String::new();
        preamble.push_str(&format!(
            "cd {}\n",
            shell_escape(&state.cwd.to_string_lossy())
        ));
        for (key, value) in &state.env {
            if !is_safe_env_name(key) {
                continue;
            }
            preamble.push_str(&format!("export {}={}\n", key, shell_escape(value)));
        }
        preamble
    }

    fn dump_env_snippet(&self, nonce: &str) -> String {
        // Emit __SHANNON_ENV__name=b64:... using only shell + `base64` + `tr`
        // (no Python). Decode remains Rust-only on inject.
        // zsh: avoid ${(t)parameters[...]} — it errors on some macOS values with ':';
        // use `typeset -p` + ${(P)name} instead.
        match self.kind {
            AltShellKind::Bash => format!(
                r#"
{{
echo "==SHANNON_SENTINEL_START_{nonce}=="
while IFS= read -r __shannon_name; do
  [[ "$__shannon_name" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || continue
  __shannon_val="${{!__shannon_name}}"
  __shannon_b64=$(printf '%s' "$__shannon_val" | base64 | tr -d '\n\r ')
  printf '__SHANNON_ENV__%s=b64:%s\n' "$__shannon_name" "$__shannon_b64"
done < <(compgen -e 2>/dev/null || true)
echo "__SHANNON_CWD=$(pwd)"
echo "__SHANNON_EXIT=$__shannon_ec"
echo "==SHANNON_SENTINEL_END_{nonce}=="
}} >&{PROTECTED_STDOUT_FD}
"#
            ),
            AltShellKind::Zsh => format!(
                r#"
{{
echo "==SHANNON_SENTINEL_START_{nonce}=="
for __shannon_name in "${{(@k)parameters}}"; do
  case $__shannon_name in
    [A-Za-z_][A-Za-z0-9_]*) ;;
    *) continue ;;
  esac
  __shannon_tp=$(typeset -p -- "$__shannon_name" 2>/dev/null | head -n1)
  case $__shannon_tp in
    export\ *|typeset\ -*x*) ;;
    *) continue ;;
  esac
  __shannon_val="${{(P)__shannon_name}}"
  __shannon_b64=$(printf '%s' "$__shannon_val" | base64 | tr -d '\n\r ')
  print -r -- "__SHANNON_ENV__${{__shannon_name}}=b64:${{__shannon_b64}}"
done
echo "__SHANNON_CWD=$(pwd)"
echo "__SHANNON_EXIT=$__shannon_ec"
echo "==SHANNON_SENTINEL_END_{nonce}=="
}} >&{PROTECTED_STDOUT_FD}
"#
            ),
        }
    }

    fn next_nonce() -> String {
        let n = NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("{n:016x}")
    }

    fn empty_state() -> ShellState {
        ShellState {
            env: HashMap::new(),
            cwd: std::path::PathBuf::from("/"),
            last_exit_code: 1,
        }
    }

    fn write_block(&mut self, block: &str) -> bool {
        if let Err(e) = self.stdin.write_all(block.as_bytes()) {
            eprintln!("ahsh: failed to write to {} stdin: {e}", self.kind.as_str());
            return false;
        }
        if let Err(e) = self.stdin.flush() {
            eprintln!("ahsh: failed to flush {} stdin: {e}", self.kind.as_str());
            return false;
        }
        true
    }

    fn read_stdout_until(&mut self, marker: &str, visible: bool) -> bool {
        let mut line = String::new();
        loop {
            line.clear();
            match self.stdout_reader.read_line(&mut line) {
                Ok(0) => return false,
                Ok(_) => {
                    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                    if trimmed == marker {
                        return true;
                    }
                    if let Some(prefix) = trimmed.strip_suffix(marker) {
                        if visible {
                            print!("{prefix}");
                            let _ = std::io::stdout().flush();
                        }
                        return true;
                    }
                    if visible {
                        print!("{line}");
                        let _ = std::io::stdout().flush();
                    }
                }
                Err(e) => {
                    eprintln!("ahsh: error reading {} stdout: {e}", self.kind.as_str());
                    return false;
                }
            }
        }
    }

    fn read_stderr_until(
        &mut self,
        marker: &str,
        visible: bool,
        suppressed_markers: &[&str],
    ) -> bool {
        while let Ok(line) = self.stderr_rx.recv() {
            let trimmed = line.trim_end_matches('\r');
            if trimmed == marker {
                return true;
            }
            if let Some(prefix) = trimmed.strip_suffix(marker) {
                if visible
                    && !suppressed_markers
                        .iter()
                        .any(|suppressed| prefix.contains(*suppressed))
                {
                    let _ = write!(std::io::stderr(), "{prefix}");
                }
                return true;
            }
            if trimmed.contains(marker)
                || suppressed_markers
                    .iter()
                    .any(|suppressed| trimmed.contains(*suppressed))
            {
                continue;
            }
            if visible {
                let _ = writeln!(std::io::stderr(), "{line}");
            }
        }
        false
    }

    fn command_normalization(&self) -> &'static str {
        match self.kind {
            AltShellKind::Bash => "set +v +x",
            AltShellKind::Zsh => "unsetopt verbose xtrace",
        }
    }

    fn run_command_with_visibility(&mut self, command: &str, visible: bool) -> ShellState {
        let preamble = self.build_preamble();
        let nonce = Self::next_nonce();
        let command_stdout_mark = format!("==AHSH_COMMAND_STDOUT_{nonce}==");
        let command_stderr_mark = format!("==AHSH_COMMAND_STDERR_{nonce}==");
        let dump_stderr_mark = format!("==AHSH_DUMP_STDERR_{nonce}==");
        let start_mark = format!("==SHANNON_SENTINEL_START_{nonce}==");
        let end_mark = format!("==SHANNON_SENTINEL_END_{nonce}==");
        let normalize = self.command_normalization();

        let command_block = format!(
            "exec 1>&{PROTECTED_STDOUT_FD} 2>&{PROTECTED_STDERR_FD}\n\
             {normalize}\n\
             {preamble}{command}\n\
             __shannon_ec=$?\n\
             exec 1>&{PROTECTED_STDOUT_FD} 2>&{PROTECTED_STDERR_FD}\n\
             {normalize}\n\
             printf '%s\\n' '{command_stdout_mark}' >&{PROTECTED_STDOUT_FD}\n\
             printf '%s\\n' '{command_stderr_mark}' >&{PROTECTED_STDERR_FD}\n"
        );

        if !self.write_block(&command_block)
            || !self.read_stdout_until(&command_stdout_mark, visible)
            || !self.read_stderr_until(&command_stderr_mark, visible, &[&command_stdout_mark])
        {
            return Self::empty_state();
        }

        // Env capture is a distinct hidden write after tracing is disabled.
        let dump = self.dump_env_snippet(&nonce);
        let dump_block = format!(
            "{dump}\n\
             printf '%s\\n' '{dump_stderr_mark}' >&{PROTECTED_STDERR_FD}\n"
        );
        if !self.write_block(&dump_block) {
            return Self::empty_state();
        }

        let mut in_sentinel = false;
        let mut saw_end = false;
        let mut sentinel_buf = String::new();
        let mut line = String::new();

        loop {
            line.clear();
            match self.stdout_reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                    if trimmed == end_mark {
                        saw_end = true;
                        break;
                    } else if trimmed == start_mark {
                        in_sentinel = true;
                    } else if in_sentinel {
                        sentinel_buf.push_str(trimmed);
                        sentinel_buf.push('\n');
                    }
                }
                Err(e) => {
                    eprintln!("ahsh: error reading {} stdout: {e}", self.kind.as_str());
                    break;
                }
            }
        }

        let stderr_done =
            self.read_stderr_until(&dump_stderr_mark, false, &[&start_mark, &end_mark]);
        if !in_sentinel || !saw_end || !stderr_done {
            return Self::empty_state();
        }

        let (env, cwd, exit_code) = executor::parse_sentinel_env(&sentinel_buf)
            .unwrap_or_else(|| (HashMap::new(), std::path::PathBuf::from("/"), 1));

        ShellState {
            env,
            cwd,
            last_exit_code: exit_code,
        }
    }

    /// Send a user-visible command, then capture its resulting shell state on
    /// the protected internal channel.
    pub fn run_command(&mut self, command: &str) -> ShellState {
        self.run_command_with_visibility(command, true)
    }
}

impl ShellEngine for AltShellProcess {
    fn inject_state(&mut self, state: &ShellState) {
        self.pending_state = Some(state.clone());
    }

    fn execute(&mut self, command: &str) -> ShellState {
        self.run_command(command)
    }
}

fn is_safe_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::TEST_ENV_LOCK;
    use std::fs;
    use tempfile::TempDir;

    fn with_isolated_home_files<F: FnOnce()>(
        bash_profile: &str,
        zprofile: &str,
        zshrc: &str,
        f: F,
    ) {
        let _guard = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let old_home = std::env::var_os("HOME");
        let old_zdot = std::env::var_os("ZDOTDIR");
        // SAFETY: guarded by ENV_LOCK; restores prior values after `f`.
        unsafe {
            std::env::set_var("HOME", &home);
            std::env::set_var("ZDOTDIR", &home);
        }
        let _ = fs::write(home.join(".bash_profile"), bash_profile);
        let _ = fs::write(home.join(".bashrc"), "");
        let _ = fs::write(home.join(".zprofile"), zprofile);
        let _ = fs::write(home.join(".zshrc"), zshrc);
        f();
        unsafe {
            match old_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match old_zdot {
                Some(v) => std::env::set_var("ZDOTDIR", v),
                None => std::env::remove_var("ZDOTDIR"),
            }
        }
    }

    fn with_isolated_home<F: FnOnce()>(f: F) {
        with_isolated_home_files(
            "",
            "",
            "export AHSH_ZSHRC_MARKER=from_zshrc\nalias ahsh_zsh_alias=true\n",
            f,
        );
    }

    #[test]
    fn bash_echo_exit_env_cwd() {
        with_isolated_home(|| {
            let mut bp = AltShellProcess::new(AltShellKind::Bash);
            assert_eq!(bp.kind(), AltShellKind::Bash);
            let state = bp.run_command("export TEST_VAR='a b=c'; true");
            assert_eq!(state.last_exit_code, 0);
            assert_eq!(state.env.get("TEST_VAR").map(String::as_str), Some("a b=c"));

            let state = bp.run_command("false");
            assert_eq!(state.last_exit_code, 1);

            let dir = TempDir::new().unwrap();
            let path = dir
                .path()
                .canonicalize()
                .unwrap_or_else(|_| dir.path().to_path_buf());
            bp.run_command(&format!("cd {}", shell_escape(&path.to_string_lossy())));
            let state = bp.run_command("true");
            let got = state.cwd.canonicalize().unwrap_or(state.cwd.clone());
            assert_eq!(got, path);
        });
    }

    #[test]
    fn zsh_loads_zshrc_and_roundtrips() {
        with_isolated_home(|| {
            let mut zp = AltShellProcess::new(AltShellKind::Zsh);
            assert_eq!(zp.kind(), AltShellKind::Zsh);
            let env = zp.capture_env();
            assert_eq!(
                env.get("AHSH_ZSHRC_MARKER").map(String::as_str),
                Some("from_zshrc"),
                "zshrc marker missing; env keys sample: {:?}",
                env.keys().take(20).collect::<Vec<_>>()
            );

            let state = zp.run_command("export ZTEST='x=y z'; true");
            assert_eq!(state.last_exit_code, 0);
            assert_eq!(state.env.get("ZTEST").map(String::as_str), Some("x=y z"));

            // Prefer printf for multiline for reliability
            let state = zp.run_command("export MULTILINE=\"$(printf 'a\\nb')\"; true");
            assert_eq!(state.last_exit_code, 0);
            assert_eq!(state.env.get("MULTILINE").map(String::as_str), Some("a\nb"));

            let state = zp.run_command("export EMPTY=; true");
            assert_eq!(state.env.get("EMPTY").map(String::as_str), Some(""));

            let state = zp.run_command("false");
            assert_eq!(state.last_exit_code, 1);
        });
    }

    fn inject_special_values(kind: AltShellKind) {
        with_isolated_home(|| {
            let dir = TempDir::new().unwrap();
            let mut env = HashMap::new();
            env.insert("INJECTED".to_string(), "yes please".to_string());
            env.insert("EQ".to_string(), "a=b".to_string());
            env.insert("QUOTES".to_string(), r#"say "hi""#.to_string());
            env.insert("EMPTY".to_string(), "".to_string());
            env.insert("MULTILINE".to_string(), "a\nb".to_string());
            env.insert("TRAIL_NL".to_string(), "x\n".to_string());
            let state = ShellState {
                env,
                cwd: dir.path().to_path_buf(),
                last_exit_code: 0,
            };
            let mut proc = AltShellProcess::new(kind);
            proc.inject_state(&state);
            let result = proc.run_command("true");
            assert_eq!(
                result.env.get("INJECTED").map(String::as_str),
                Some("yes please")
            );
            assert_eq!(result.env.get("EQ").map(String::as_str), Some("a=b"));
            assert_eq!(
                result.env.get("QUOTES").map(String::as_str),
                Some(r#"say "hi""#)
            );
            assert_eq!(result.env.get("EMPTY").map(String::as_str), Some(""));
            assert_eq!(
                result.env.get("MULTILINE").map(String::as_str),
                Some("a\nb")
            );
            assert_eq!(result.env.get("TRAIL_NL").map(String::as_str), Some("x\n"));
            let expected = dir
                .path()
                .canonicalize()
                .unwrap_or_else(|_| dir.path().to_path_buf());
            let got = result.cwd.canonicalize().unwrap_or(result.cwd.clone());
            assert_eq!(got, expected, "kind {}", kind.as_str());
            // Exit status still works after inject
            let fail = proc.run_command("false");
            assert_eq!(fail.last_exit_code, 1);
        });
    }

    #[test]
    fn inject_state_roundtrip_bash() {
        inject_special_values(AltShellKind::Bash);
    }

    #[test]
    fn inject_state_roundtrip_zsh() {
        inject_special_values(AltShellKind::Zsh);
    }

    #[test]
    fn zsh_alias_from_zshrc_available() {
        with_isolated_home(|| {
            let mut zp = AltShellProcess::new(AltShellKind::Zsh);
            // alias defined in fixture .zshrc; expand via alias invocation
            let state = zp.run_command("alias ahsh_zsh_alias");
            assert_eq!(state.last_exit_code, 0);
        });
    }

    #[test]
    fn static_sentinel_collision_does_not_break_bash() {
        with_isolated_home(|| {
            let mut bp = AltShellProcess::new(AltShellKind::Bash);
            let state = bp.run_command("echo '==SHANNON_SENTINEL_START=='; export COLLISION_OK=1");
            assert_eq!(state.last_exit_code, 0);
            assert_eq!(state.env.get("COLLISION_OK").map(String::as_str), Some("1"));
        });
    }

    #[test]
    fn command_output_without_newline_does_not_swallow_control_marker() {
        with_isolated_home(|| {
            for kind in [AltShellKind::Bash, AltShellKind::Zsh] {
                let mut process = AltShellProcess::new(kind);
                let state = process.run_command("printf no-newline; printf err-no-newline >&2");
                assert_eq!(state.last_exit_code, 0, "kind {}", kind.as_str());
                assert!(!state.env.is_empty(), "kind {}", kind.as_str());
            }
        });
    }

    #[test]
    fn zsh_startup_redirects_cannot_steal_capture_channel() {
        with_isolated_home_files(
            "",
            "exec 1>&2\nexport AHSH_ZPROFILE_REDIRECT=profile\n",
            "exec 3>&1\nexec 1>&2\nexport AHSH_ZSHRC_REDIRECT=rc\n",
            || {
                let mut process = AltShellProcess::new(AltShellKind::Zsh);
                let env = process.capture_env();
                assert_eq!(
                    env.get("AHSH_ZPROFILE_REDIRECT").map(String::as_str),
                    Some("profile")
                );
                assert_eq!(
                    env.get("AHSH_ZSHRC_REDIRECT").map(String::as_str),
                    Some("rc")
                );

                let state = process.run_command("export AHSH_AFTER_REDIRECT=ok; exec 1>&2");
                assert_eq!(state.last_exit_code, 0);
                assert_eq!(
                    state.env.get("AHSH_AFTER_REDIRECT").map(String::as_str),
                    Some("ok")
                );
            },
        );
    }

    #[test]
    fn bash_startup_redirect_cannot_steal_capture_channel() {
        with_isolated_home_files(
            "exec 3>&1\nexec 1>&2\nexport AHSH_BASH_PROFILE_REDIRECT=profile\n",
            "",
            "",
            || {
                let mut process = AltShellProcess::new(AltShellKind::Bash);
                let env = process.capture_env();
                assert_eq!(
                    env.get("AHSH_BASH_PROFILE_REDIRECT").map(String::as_str),
                    Some("profile")
                );

                let state = process.run_command("export AHSH_AFTER_REDIRECT=ok; exec 1>&2");
                assert_eq!(state.last_exit_code, 0);
                assert_eq!(
                    state.env.get("AHSH_AFTER_REDIRECT").map(String::as_str),
                    Some("ok")
                );
            },
        );
    }

    #[test]
    fn zsh_verbose_and_xtrace_are_normalized_before_capture() {
        with_isolated_home_files(
            "",
            "",
            "setopt verbose xtrace\nexport AHSH_TRACE_FIXTURE=enabled\n",
            || {
                let mut process = AltShellProcess::new(AltShellKind::Zsh);
                let env = process.capture_env();
                assert_eq!(
                    env.get("AHSH_TRACE_FIXTURE").map(String::as_str),
                    Some("enabled")
                );
                let state = process.run_command("export AHSH_TRACE_COMMAND=ok; true");
                assert_eq!(state.last_exit_code, 0);
                assert_eq!(
                    state.env.get("AHSH_TRACE_COMMAND").map(String::as_str),
                    Some("ok")
                );
            },
        );
    }
}
