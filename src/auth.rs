use anyhow::{bail, Result};

/// Resolve the API token using the priority order:
/// 1. `--token` flag (already loaded by clap via env = "KEENTOOLS_API_TOKEN")
/// 2. Config file (~/.config/keentools-cloud/config.toml)
///
/// Note: clap's `env` attribute handles both the flag and the env var,
/// so `token_flag` is `Some` if either `--token` or `KEENTOOLS_API_TOKEN` is set.
pub fn resolve_token(token_flag: Option<&str>) -> Result<String> {
    if let Some(t) = token_flag {
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }

    // Fall back to config file
    if let Ok(cfg) = crate::config::load() {
        if let Some(t) = cfg.auth.token {
            if !t.is_empty() {
                return Ok(t);
            }
        }
    }

    bail!(
        "No API token found.\n\
        Set the KEENTOOLS_API_TOKEN environment variable, use --token <TOKEN>,\n\
        or run: keentools-cloud auth login"
    )
}
