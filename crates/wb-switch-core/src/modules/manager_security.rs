//! Manager policy and redaction; no runtime switch may enable unverified updates.
use regex::Regex;
use std::sync::OnceLock;
pub const UPDATES_ENABLED: bool = false;
pub const OPTIONAL_COMPANION_ENABLED: bool = false;

pub fn private_directory(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(std::io::Error::other("私有目录不能为链接"));
    }
    #[cfg(windows)]
    {
        use std::os::windows::{fs::MetadataExt, process::CommandExt};
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(std::io::Error::other("私有目录不能为重解析点"));
        }
        let output = std::process::Command::new("whoami.exe")
            .args(["/user", "/fo", "csv", "/nh"])
            .creation_flags(0x08000000)
            .output()?;
        let output = String::from_utf8_lossy(&output.stdout);
        let sid = output
            .split(',')
            .last()
            .unwrap_or("")
            .trim()
            .trim_matches('"');
        if !sid.starts_with("S-1-")
            || !sid
                .bytes()
                .all(|c| c.is_ascii_digit() || c == b'S' || c == b'-')
        {
            return Err(std::io::Error::other("当前用户身份不可识别"));
        }
        let result = std::process::Command::new("icacls.exe")
            .arg(path)
            .args([
                "/inheritance:r",
                "/grant:r",
                &format!("*{sid}:(OI)(CI)F"),
                "*S-1-5-18:(OI)(CI)F",
            ])
            .creation_flags(0x08000000)
            .output()?;
        if !result.status.success() {
            return Err(std::io::Error::other("私有目录权限设置失败"));
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn redact(text: &str) -> String {
    static KEYS: OnceLock<Regex> = OnceLock::new();
    static AUTH: OnceLock<Regex> = OnceLock::new();
    static JWT: OnceLock<Regex> = OnceLock::new();
    let keys = KEYS.get_or_init(|| Regex::new(r#"(?i)((?:[a-z_]*token|authorization|cookie|password|secret|credential|api[-_]?key)["']?\s*[:=]\s*)("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[^\s,"'&}\]]+)"#).unwrap());
    let auth =
        AUTH.get_or_init(|| Regex::new(r"(?i)\b(Bearer|Basic)\s+[A-Za-z0-9_./+=-]+").unwrap());
    let jwt = JWT.get_or_init(|| {
        Regex::new(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b").unwrap()
    });
    let text = auth.replace_all(text, "${1} [已脱敏]");
    let text = keys.replace_all(&text, "${1}[已脱敏]");
    jwt.replace_all(&text, "[已脱敏]").into_owned()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_named_credentials_bearer_and_jwt_without_losing_business_errors() {
        let input = r#"网络连接失败 {"access_token":"fiction-secret"} ?refreshToken=fiction-refresh&x=1 Bearer fiction-bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJmaWN0aW9uIn0.signature"#;
        let output = redact(input);
        for secret in [
            "fiction-secret",
            "fiction-refresh",
            "fiction-bearer",
            "eyJhbG",
        ] {
            assert!(!output.contains(secret));
        }
        assert!(output.contains("网络连接失败"));
        assert_eq!(redact("会话数据库正被占用"), "会话数据库正被占用");
        assert!(!UPDATES_ENABLED);
    }
    #[test]
    fn redacts_authorization_headers_and_quoted_values_with_spaces() {
        let output = redact(
            r#"Authorization: Bearer fiction-header {"password":"fiction secret with spaces", "token":"escaped\" secret"} secret='another spaced value'"#,
        );
        for secret in [
            "fiction-header",
            "fiction secret",
            "escaped",
            "another spaced",
        ] {
            assert!(!output.contains(secret), "{output}");
        }
    }
}
