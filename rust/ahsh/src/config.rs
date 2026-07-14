//! Product config for Astrohacker Shell (`~/.config/astrohacker/shell/config.toml`).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Non-Nu shell that Shift+Tab switches to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltShellKind {
    Bash,
    Zsh,
}

impl AltShellKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AltShellKind::Bash => "bash",
            AltShellKind::Zsh => "zsh",
        }
    }

    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        match s.trim() {
            "bash" => Ok(AltShellKind::Bash),
            "zsh" => Ok(AltShellKind::Zsh),
            other => Err(ConfigError::InvalidAltShell(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellConfig {
    pub alt_shell: AltShellKind,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            alt_shell: AltShellKind::Zsh,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    InvalidAltShell(String),
    WrongType(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(msg) => write!(f, "config I/O error: {msg}"),
            ConfigError::Parse(msg) => write!(f, "config parse error: {msg}"),
            ConfigError::InvalidAltShell(v) => write!(
                f,
                "invalid alt_shell {v:?}: allowed values are \"bash\" and \"zsh\""
            ),
            ConfigError::WrongType(msg) => write!(f, "config type error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Config path: `$XDG_CONFIG_HOME/astrohacker/shell/config.toml` if
/// `XDG_CONFIG_HOME` is set and non-empty, else `$HOME/.config/astrohacker/shell/config.toml`.
pub fn default_config_path() -> PathBuf {
    default_config_path_from_env(|k| env::var(k).ok())
}

pub fn default_config_path_from_env<F>(mut get_env: F) -> PathBuf
where
    F: FnMut(&str) -> Option<String>,
{
    let base = get_env("XDG_CONFIG_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            get_env("HOME")
                .filter(|s| !s.is_empty())
                .map(|h| PathBuf::from(h).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from("/.config"));
    base.join("astrohacker").join("shell").join("config.toml")
}

#[derive(Debug, Deserialize)]
struct RawShellConfig {
    alt_shell: Option<toml::Value>,
}

/// Load product config from `path`. Missing file → default (zsh).
pub fn load_shell_config(path: &Path) -> Result<ShellConfig, ConfigError> {
    match fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ShellConfig::default()),
        Err(e) => Err(ConfigError::Io(format!("{}: {e}", path.display()))),
        Ok(contents) => parse_shell_config_str(&contents),
    }
}

/// Parse TOML body. Empty / omitted `alt_shell` → zsh.
pub fn parse_shell_config_str(contents: &str) -> Result<ShellConfig, ConfigError> {
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        return Ok(ShellConfig::default());
    }

    let raw: RawShellConfig =
        toml::from_str(contents).map_err(|e| ConfigError::Parse(e.to_string()))?;

    match raw.alt_shell {
        None => Ok(ShellConfig::default()),
        Some(toml::Value::String(s)) => Ok(ShellConfig {
            alt_shell: AltShellKind::parse(&s)?,
        }),
        Some(other) => Err(ConfigError::WrongType(format!(
            "alt_shell must be a string, got {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn default_is_zsh() {
        assert_eq!(ShellConfig::default().alt_shell, AltShellKind::Zsh);
    }

    #[test]
    fn path_prefers_xdg() {
        let p = default_config_path_from_env(|k| match k {
            "XDG_CONFIG_HOME" => Some("/xdg".into()),
            "HOME" => Some("/home/user".into()),
            _ => None,
        });
        assert_eq!(
            p,
            PathBuf::from("/xdg/astrohacker/shell/config.toml")
        );
    }

    #[test]
    fn path_falls_back_to_home_config() {
        let p = default_config_path_from_env(|k| match k {
            "XDG_CONFIG_HOME" => Some("".into()),
            "HOME" => Some("/Users/ryan".into()),
            _ => None,
        });
        assert_eq!(
            p,
            PathBuf::from("/Users/ryan/.config/astrohacker/shell/config.toml")
        );
    }

    #[test]
    fn path_never_uses_application_support() {
        let p = default_config_path_from_env(|k| match k {
            "HOME" => Some("/Users/ryan".into()),
            _ => None,
        });
        let s = p.to_string_lossy();
        assert!(!s.contains("Application Support"));
        assert!(s.ends_with("astrohacker/shell/config.toml"));
    }

    #[test]
    fn parse_explicit_bash_and_zsh() {
        assert_eq!(
            parse_shell_config_str("alt_shell = \"bash\"")
                .unwrap()
                .alt_shell,
            AltShellKind::Bash
        );
        assert_eq!(
            parse_shell_config_str("alt_shell = \"zsh\"")
                .unwrap()
                .alt_shell,
            AltShellKind::Zsh
        );
    }

    #[test]
    fn parse_empty_and_omitted_default_zsh() {
        assert_eq!(
            parse_shell_config_str("").unwrap().alt_shell,
            AltShellKind::Zsh
        );
        assert_eq!(
            parse_shell_config_str("# comment only\n").unwrap().alt_shell,
            AltShellKind::Zsh
        );
        // empty table / no key
        assert_eq!(
            parse_shell_config_str("\n").unwrap().alt_shell,
            AltShellKind::Zsh
        );
    }

    #[test]
    fn parse_invalid_and_wrong_type() {
        assert!(matches!(
            parse_shell_config_str("alt_shell = \"fish\""),
            Err(ConfigError::InvalidAltShell(_))
        ));
        assert!(matches!(
            parse_shell_config_str("alt_shell = 1"),
            Err(ConfigError::WrongType(_))
        ));
        assert!(matches!(
            parse_shell_config_str("alt_shell = ["),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn load_missing_file_defaults() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.toml");
        let cfg = load_shell_config(&path).unwrap();
        assert_eq!(cfg.alt_shell, AltShellKind::Zsh);
    }

    #[test]
    fn load_file_bash() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "alt_shell = \"bash\"").unwrap();
        let cfg = load_shell_config(&path).unwrap();
        assert_eq!(cfg.alt_shell, AltShellKind::Bash);
    }

    #[test]
    fn load_unreadable_is_error() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("dir_as_file");
        fs::create_dir(&path).unwrap();
        assert!(matches!(load_shell_config(&path), Err(ConfigError::Io(_))));
    }
}
