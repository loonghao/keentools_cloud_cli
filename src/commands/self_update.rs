use anyhow::{bail, Context, Result};
use clap::Args;
use semver::Version;
use serde::Deserialize;
use std::env;
use std::io::Write;
use std::path::PathBuf;

use crate::output::{OutputFormat, Printer};

const REPO: &str = "loonghao/keentools_cloud_cli";
const GITHUB_API: &str = "https://api.github.com";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Args, Debug)]
pub struct SelfUpdateArgs {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,

    /// Install a specific version (e.g. v0.2.0)
    #[arg(long)]
    pub version: Option<String>,

    /// Force reinstall even if already on the latest version
    #[arg(long)]
    pub force: bool,
}

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
    body: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn run(args: SelfUpdateArgs, output: OutputFormat) -> Result<()> {
    let printer = Printer::new(output);

    let client = reqwest::Client::builder()
        .user_agent(format!("keentools-cloud/{}", CURRENT_VERSION))
        .build()
        .context("Failed to build HTTP client")?;

    let release = if let Some(ref v) = args.version {
        let tag = if v.starts_with('v') {
            v.clone()
        } else {
            format!("v{}", v)
        };
        fetch_release_by_tag(&client, &tag).await?
    } else {
        fetch_latest_release(&client).await?
    };

    let latest_tag = release.tag_name.trim_start_matches('v');
    let latest = Version::parse(latest_tag)
        .with_context(|| format!("Invalid version tag: {}", release.tag_name))?;
    let current = Version::parse(CURRENT_VERSION)?;

    if printer.is_json() {
        printer.success(&serde_json::json!({
            "current_version": CURRENT_VERSION,
            "latest_version": latest.to_string(),
            "update_available": latest > current,
            "tag": release.tag_name,
        }));
    } else {
        printer.status_line("Current version", CURRENT_VERSION);
        printer.status_line("Latest version", &latest.to_string());
    }

    if args.check {
        if latest > current {
            if !printer.is_json() {
                printer.message(&format!(
                    "Update available! Run `keentools-cloud self-update` to install v{}",
                    latest
                ));
            }
        } else if !printer.is_json() {
            printer.message("Already on the latest version.");
        }
        return Ok(());
    }

    if latest <= current && !args.force {
        printer.message("Already on the latest version. Use --force to reinstall.");
        return Ok(());
    }

    let target = detect_target();
    let asset = find_asset(&release, &target)
        .with_context(|| format!("No release asset found for target: {}", target))?;

    if !printer.is_json() {
        printer.status_line("Downloading", &asset.name);
    }

    let exe_path = env::current_exe().context("Cannot determine current executable path")?;
    download_and_replace(&client, &asset.browser_download_url, &asset.name, &exe_path).await?;

    printer.message(&format!("Updated to v{}", latest));
    Ok(())
}

async fn fetch_latest_release(client: &reqwest::Client) -> Result<GithubRelease> {
    let url = format!("{}/repos/{}/releases/latest", GITHUB_API, REPO);
    client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Failed to reach GitHub API")?
        .error_for_status()
        .context("GitHub API error (check that the repository has published releases)")?
        .json::<GithubRelease>()
        .await
        .context("Failed to parse GitHub release response")
}

async fn fetch_release_by_tag(client: &reqwest::Client, tag: &str) -> Result<GithubRelease> {
    let url = format!("{}/repos/{}/releases/tags/{}", GITHUB_API, REPO, tag);
    client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .context("Failed to reach GitHub API")?
        .error_for_status()
        .with_context(|| format!("Release {} not found", tag))?
        .json::<GithubRelease>()
        .await
        .context("Failed to parse GitHub release response")
}

fn find_asset<'a>(release: &'a GithubRelease, target: &str) -> Option<&'a GithubAsset> {
    // Look for an asset that contains the target triple and is a .tar.gz or .zip
    release.assets.iter().find(|a| {
        let name = &a.name;
        name.contains(target) && (name.ends_with(".tar.gz") || name.ends_with(".zip"))
    })
}

/// Detect the current platform's Rust target triple (best effort).
fn detect_target() -> String {
    // Embedded at compile time — this is the target the binary was built for.
    env!("TARGET").to_string()
}

async fn download_and_replace(
    client: &reqwest::Client,
    url: &str,
    archive_name: &str,
    exe_path: &PathBuf,
) -> Result<()> {
    // Download the archive to a temp file
    let bytes = client
        .get(url)
        .send()
        .await
        .context("Download failed")?
        .error_for_status()
        .context("Download HTTP error")?
        .bytes()
        .await
        .context("Failed to read download body")?;

    // Extract the binary from the archive
    let binary_bytes = if archive_name.ends_with(".tar.gz") {
        extract_from_tar_gz(&bytes)?
    } else if archive_name.ends_with(".zip") {
        extract_from_zip(&bytes)?
    } else {
        bail!("Unsupported archive format: {}", archive_name)
    };

    // Atomically replace the current executable
    replace_executable(exe_path, &binary_bytes)?;
    Ok(())
}

fn extract_from_tar_gz(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;

    let gz = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(gz);

    for entry in archive.entries().context("Failed to read tar archive")? {
        let mut entry = entry.context("Invalid tar entry")?;
        let path = entry.path().context("Invalid entry path")?;

        // Find the binary — it's the file named "keentools-cloud" (no extension)
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        if filename == "keentools-cloud" || filename == "keentools-cloud.exe" {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .context("Failed to read binary from archive")?;
            return Ok(buf);
        }
    }
    bail!("keentools-cloud binary not found in archive")
}

fn extract_from_zip(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;

    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to open ZIP archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).context("Invalid ZIP entry")?;
        let name = file.name().to_string();
        if name == "keentools-cloud.exe" || name == "keentools-cloud" {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .context("Failed to read binary from ZIP")?;
            return Ok(buf);
        }
    }
    bail!("keentools-cloud binary not found in ZIP archive")
}

fn replace_executable(exe_path: &PathBuf, new_bytes: &[u8]) -> Result<()> {
    // Write to a .tmp file alongside the current binary, then rename
    let tmp_path = exe_path.with_extension("tmp");

    {
        let mut f = std::fs::File::create(&tmp_path)
            .with_context(|| format!("Cannot write to {}", tmp_path.display()))?;
        f.write_all(new_bytes)
            .context("Failed to write new binary")?;
    }

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))
            .context("Failed to set executable permissions")?;
    }

    // On Windows, rename the running binary first (Windows locks running executables)
    #[cfg(windows)]
    {
        let old_path = exe_path.with_extension("old");
        // Remove any previous .old file
        let _ = std::fs::remove_file(&old_path);
        std::fs::rename(exe_path, &old_path)
            .context("Failed to move current binary (is another process running?)")?;
    }

    std::fs::rename(&tmp_path, exe_path)
        .with_context(|| format!("Failed to replace binary at {}", exe_path.display()))?;

    Ok(())
}
