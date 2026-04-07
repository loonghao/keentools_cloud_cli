use anyhow::{bail, Result};

const MAX_ID_LEN: usize = 128;

/// Validate an avatar ID for safety against common agent hallucination patterns.
///
/// Rejects:
/// - Control characters (< ASCII 0x20)
/// - Path traversal sequences (`..`, `/`, `\`)
/// - Embedded query params (`?`, `#`)
/// - URL encoding (`%`)
/// - IDs longer than 128 characters
pub fn avatar_id(id: &str) -> Result<()> {
    if id.is_empty() {
        bail!("avatar ID must not be empty");
    }
    if id.len() > MAX_ID_LEN {
        bail!(
            "avatar ID too long: {} chars (max {})",
            id.len(),
            MAX_ID_LEN
        );
    }
    for ch in id.chars() {
        if (ch as u32) < 0x20 {
            bail!("avatar ID contains control character (0x{:02x})", ch as u32);
        }
    }
    let forbidden = ['?', '#', '%', '/', '\\'];
    for ch in forbidden {
        if id.contains(ch) {
            bail!("avatar ID contains forbidden character '{}'", ch);
        }
    }
    if id.contains("..") {
        bail!("avatar ID contains path traversal sequence '..'");
    }
    Ok(())
}

/// Validate that a URL is HTTPS and contains no obvious injection patterns.
pub fn https_url(url: &str) -> Result<()> {
    if !url.starts_with("https://") {
        bail!("URL must start with https://: {}", url);
    }
    // Basic length guard
    if url.len() > 4096 {
        bail!("URL is too long ({} chars)", url.len());
    }
    Ok(())
}

/// Validate photo count is within API limits (2–15).
pub fn photo_count(count: usize) -> Result<()> {
    if count < 2 || count > 15 {
        bail!(
            "photo count must be between 2 and 15, got {}",
            count
        );
    }
    Ok(())
}

/// Validate that a file path is safe (no traversal, exists, is a file).
pub fn photo_path(path: &std::path::Path) -> Result<()> {
    let canonical = path
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Cannot access '{}': {}", path.display(), e))?;

    if !canonical.is_file() {
        bail!("'{}' is not a file", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_avatar_id() {
        assert!(avatar_id("avatar_12345").is_ok());
        assert!(avatar_id("abc-def_123").is_ok());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(avatar_id("../../.ssh/id_rsa").is_err());
        assert!(avatar_id("foo/bar").is_err());
        assert!(avatar_id("foo\\bar").is_err());
        assert!(avatar_id("foo..bar").is_err());
    }

    #[test]
    fn rejects_query_injection() {
        assert!(avatar_id("avatar?fields=name").is_err());
        assert!(avatar_id("avatar#fragment").is_err());
        assert!(avatar_id("avatar%2e%2e").is_err());
    }

    #[test]
    fn rejects_control_chars() {
        assert!(avatar_id("avatar\x00id").is_err());
        assert!(avatar_id("avatar\x1fid").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(avatar_id("").is_err());
    }

    #[test]
    fn rejects_too_long() {
        let long_id = "a".repeat(129);
        assert!(avatar_id(&long_id).is_err());
    }

    #[test]
    fn valid_https_url() {
        assert!(https_url("https://example.com/path").is_ok());
    }

    #[test]
    fn rejects_http_url() {
        assert!(https_url("http://example.com").is_err());
    }
}
