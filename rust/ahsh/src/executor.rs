use std::collections::HashMap;
use std::path::PathBuf;

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

/// Parse nonced sentinel body (`__SHANNON_ENV__`, `__SHANNON_CWD=`, `__SHANNON_EXIT=`).
pub fn parse_sentinel_env(contents: &str) -> Option<(HashMap<String, String>, PathBuf, i32)> {
    let mut env = HashMap::new();
    let mut cwd: Option<PathBuf> = None;
    let mut exit_code: Option<i32> = None;

    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix("__SHANNON_ENV__") {
            if let Some((key, value_spec)) = rest.split_once('=') {
                if let Some(b64) = value_spec.strip_prefix("b64:") {
                    match decode_b64_value(b64) {
                        Ok(v) => {
                            env.insert(key.to_string(), v);
                        }
                        Err(_) => continue,
                    }
                }
            }
        } else if let Some(rest) = line.strip_prefix("__SHANNON_CWD=") {
            cwd = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("__SHANNON_EXIT=") {
            exit_code = rest.parse().ok();
        }
    }

    Some((
        env,
        cwd.unwrap_or_else(|| PathBuf::from("/")),
        exit_code.unwrap_or(1),
    ))
}

fn decode_b64_value(b64: &str) -> Result<String, ()> {
    let cleaned: String = b64.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = B64.decode(cleaned.as_bytes()).or_else(|_| {
        // tolerate missing padding
        let mut padded = cleaned.clone();
        while padded.len() % 4 != 0 {
            padded.push('=');
        }
        B64.decode(padded.as_bytes())
    }).map_err(|_| ())?;
    String::from_utf8(bytes).map_err(|_| ())
}

/// Escape a string for single-quoted shell context (bash/zsh).
/// e.g., "it's" becomes `'it'\''s'`
pub fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Encode env value as unwrapped standard base64 (for tests / dump helpers).
pub fn encode_b64_value(s: &str) -> String {
    B64.encode(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_roundtrip_special_values() {
        let cases = [
            ("plain", "hello"),
            ("spaces", "a b c"),
            ("eq", "a=b=c"),
            ("quotes", r#"say "hi""#),
            ("empty", ""),
            ("nl", "line1\nline2"),
            ("trail_nl", "x\n"),
        ];
        let mut body = String::new();
        for (k, v) in cases {
            body.push_str(&format!(
                "__SHANNON_ENV__{k}=b64:{}\n",
                encode_b64_value(v)
            ));
        }
        body.push_str("__SHANNON_CWD=/tmp/demo\n__SHANNON_EXIT=0\n");
        let (env, cwd, code) = parse_sentinel_env(&body).unwrap();
        for (k, v) in cases {
            assert_eq!(env.get(k).map(String::as_str), Some(v), "key {k}");
        }
        assert_eq!(cwd, PathBuf::from("/tmp/demo"));
        assert_eq!(code, 0);
    }

    #[test]
    fn shell_escape_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }
}
