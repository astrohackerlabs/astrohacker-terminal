//! Interactive startup banner for Astrohacker Shell.

use crate::config::AltShellKind;
use std::time::Duration;

/// Mirrors nushell `BannerKind` without depending on nu_protocol in pure tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupBannerKind {
    Full,
    Short,
    None,
}

/// Production banner renderer used by `run_repl`.
///
/// Returns `None` when the banner is disabled (no alt honesty lines).
pub fn render_startup_banner(
    kind: StartupBannerKind,
    alt: AltShellKind,
    version: &str,
    nu_version: &str,
    startup: Duration,
) -> Option<String> {
    match kind {
        StartupBannerKind::None => None,
        StartupBannerKind::Short => {
            let mut out = String::new();
            out.push_str(&format!("Startup Time: {startup:?}\n"));
            out.push_str(&alt_honesty_lines(alt));
            out.push('\n');
            Some(out)
        }
        StartupBannerKind::Full => {
            let alt_name = alt.as_str();
            let mut out = String::new();
            out.push_str(
                "Welcome to Astrohacker Shell, based on the Nu language, where all data is structured!\n",
            );
            out.push_str(&format!("Version: {version} (nushell {nu_version})\n"));
            out.push_str(&format!("Startup Time: {startup:?}\n"));
            out.push_str(&alt_honesty_lines(alt));
            // Keep alt name visible in full body for greppable honesty (also in alt lines).
            let _ = alt_name;
            out.push('\n');
            Some(out)
        }
    }
}

/// Shared lines required for issue honesty (full and short interactive).
pub fn alt_honesty_lines(alt: AltShellKind) -> String {
    let name = alt.as_str();
    format!(
        "Shift+Tab switches between Nu and {name} (the configured alt shell).\n\
         For more information: ahweb astrohacker.com/shell\n"
    )
}

/// Embedded prompt script fragment — must color `zsh` and show `[mode]`.
pub fn default_prompt_script() -> &'static str {
    r#"$env.PROMPT_COMMAND = {||
            let mode = ($env.SHANNON_MODE? | default "nu")
            let color = match $mode {
                "nu" => (ansi green)
                "bash" => (ansi cyan)
                "zsh" => (ansi cyan)
                _ => (ansi green)
            }
            let reset = (ansi reset)
            let dir = if ($env.PWD | str starts-with $env.HOME) {
                $env.PWD | str replace $env.HOME "~"
            } else {
                $env.PWD
            }
            $"($color)[($mode)](ansi reset) ($dir)"
        }"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_silent() {
        assert!(
            render_startup_banner(
                StartupBannerKind::None,
                AltShellKind::Zsh,
                "0.1.0",
                "0.114.1",
                Duration::from_millis(1),
            )
            .is_none()
        );
    }

    #[test]
    fn full_and_short_name_alt_and_docs() {
        for kind in [StartupBannerKind::Full, StartupBannerKind::Short] {
            for alt in [AltShellKind::Bash, AltShellKind::Zsh] {
                let text = render_startup_banner(
                    kind,
                    alt,
                    "0.1.0",
                    "0.114.1",
                    Duration::from_millis(12),
                )
                .expect("banner");
                assert!(text.contains("Shift+Tab"), "{text}");
                assert!(text.contains(alt.as_str()), "{text}");
                assert!(text.contains("ahweb astrohacker.com/shell"), "{text}");
            }
        }
    }

    #[test]
    fn prompt_script_has_zsh_and_mode_brackets() {
        let s = default_prompt_script();
        assert!(s.contains("\"zsh\""));
        assert!(s.contains("[($mode)]") || s.contains("[($mode)"));
        assert!(s.contains("SHANNON_MODE"));
    }
}
