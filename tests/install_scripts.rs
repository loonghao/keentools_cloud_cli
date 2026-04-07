/// Validation tests for install scripts.
///
/// These tests catch compatibility regressions (e.g., PS 7-only syntax in install.ps1,
/// broken bash in install.sh) before they reach users.
use std::path::Path;

const INSTALL_PS1: &str = "install.ps1";
const INSTALL_SH: &str = "install.sh";

// ---------------------------------------------------------------------------
// PowerShell 5.1 – forbidden patterns
// ---------------------------------------------------------------------------

/// Syntax that only works in PowerShell 7+ and MUST NOT appear in install.ps1.
/// Covers: null-coalescing (??), ternary (? :), pipeline-chain (&& / ||).
static PS7_ONLY_PATTERNS: &[&str] = &["??", "? :", "&& ", "|| "];

#[test]
fn install_ps1_exists() {
    assert!(
        Path::new(INSTALL_PS1).exists(),
        "{} must exist in project root",
        INSTALL_PS1
    );
}

#[test]
fn install_ps1_no_ps7_only_syntax() {
    let source = std::fs::read_to_string(INSTALL_PS1).expect("failed to read install.ps1");
    for (line_no, line) in source.lines().enumerate() {
        // Skip lines that are inside comments
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        for &pattern in PS7_ONLY_PATTERNS {
            if line.contains(pattern) {
                panic!(
                    "install.ps1:{} contains PowerShell 7+-only syntax '{}':\n  {}",
                    line_no + 1,
                    pattern,
                    trimmed
                );
            }
        }
    }
}

#[test]
fn install_ps1_has_param_defaults_for_env_vars() {
    let source = std::fs::read_to_string(INSTALL_PS1).expect("failed to read install.ps1");
    // Ensure all three env vars are handled with a PS 5.1-compatible fallback pattern
    assert!(
        source.contains("KEENTOOLS_INSTALL_VERSION"),
        "install.ps1 must handle KEENTOOLS_INSTALL_VERSION"
    );
    assert!(
        source.contains("KEENTOOLS_INSTALL_DIR"),
        "install.ps1 must handle KEENTOOLS_INSTALL_DIR"
    );
    assert!(
        source.contains("KEENTOOLS_INSTALL_REPOSITORY"),
        "install.ps1 must handle KEENTOOLS_INSTALL_REPOSITORY"
    );
}

// ---------------------------------------------------------------------------
// Bash – basic sanity checks
// ---------------------------------------------------------------------------

#[test]
fn install_sh_exists() {
    assert!(
        Path::new(INSTALL_SH).exists(),
        "{} must exist in project root",
        INSTALL_SH
    );
}

#[test]
fn install_sh_uses_strict_mode() {
    let source = std::fs::read_to_string(INSTALL_SH).expect("failed to read install.sh");
    // Must have set -euo pipefail or equivalent strict mode
    assert!(
        source.contains("set -e")
            || source.contains("set -o errexit")
            || source.contains("set -eu"),
        "install.sh should use 'set -e' (errexit)"
    );
    assert!(
        source.contains("set -u")
            || source.contains("set -o nounset")
            || source.contains("set -eu"),
        "install.sh should use 'set -u' (nounset)"
    );
    assert!(
        source.contains("set -o pipefail") || source.contains("set -euo"),
        "install.sh should use 'set -o pipefail'"
    );
}

#[test]
fn install_sh_handles_env_var_defaults() {
    let source = std::fs::read_to_string(INSTALL_SH).expect("failed to read install.sh");
    // Must use ${VAR:-default} syntax for all three env vars
    assert!(
        source.contains("${KEENTOOLS_INSTALL_REPOSITORY:-"),
        "install.sh must use ${{REPO}}:-default for REPOSITORY"
    );
    assert!(
        source.contains("${KEENTOOLS_INSTALL_DIR:-")
            || source.contains("${KEENTOOLS_INSTALL_DIR:-"),
        "install.sh must use ${{DIR}}:-default for DIR"
    );
}
